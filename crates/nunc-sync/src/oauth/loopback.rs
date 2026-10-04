//! Der lokale Server, an den der Browser nach der Anmeldung weiterleitet (RFC 8252, Abschnitt 7.3).

use std::time::Duration;

use reqwest::Url;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::SyncError;

/// Antwortseite im Browser.
#[derive(Debug, Clone)]
pub struct Page {
    pub title: &'static str,
    pub message: &'static str,
}

impl Page {
    fn success_html(&self) -> String {
        format!(
            "<!DOCTYPE html><html lang=\"de\"><head><meta charset=\"utf-8\"><title>nunc – Verbunden</title>\
             <style>body{{font-family:system-ui,sans-serif;display:flex;align-items:center;justify-content:center;\
             height:100vh;margin:0;background:#131313;color:#e5e2e1}}.card{{text-align:center;padding:2rem}}\
             h1{{color:#4de082}}p{{color:#c6c6cd}}</style></head><body><div class=\"card\"><h1>&#10003; {}</h1>\
             <p>{}</p><p>Du kannst dieses Fenster jetzt schließen.</p></div></body></html>",
            self.title, self.message
        )
    }
}

pub struct Loopback {
    listener: TcpListener,
    port: u16,
    path: &'static str,
}

/// Wie lange ein einzelner Browser-Zugriff höchstens dauern darf, bis die Anfrage vollständig ist.
const READ_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_REQUEST: usize = 16_384;

impl Loopback {
    /// Bindet an den ersten freien Port der Liste (`0` = beliebiger freier Port).
    pub async fn bind(ports: &[u16], path: &'static str) -> Result<Self, SyncError> {
        for port in ports {
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", *port)).await {
                let port = listener
                    .local_addr()
                    .map_err(|error| SyncError::OAuth(error.to_string()))?
                    .port();
                return Ok(Self {
                    listener,
                    port,
                    path,
                });
            }
        }
        Err(SyncError::OAuth(
            "Der lokale Anmelde-Server konnte nicht gestartet werden (Port belegt).".to_owned(),
        ))
    }

    pub fn redirect_uri(&self) -> String {
        format!("http://127.0.0.1:{}{}", self.port, self.path)
    }

    /// Wartet auf den Rückruf des Browsers. Anfragen mit falschem Pfad oder falschem `state` werden
    /// abgewiesen, ohne die Anmeldung abzubrechen. Gibt den Code zurück.
    pub async fn wait_for_code(
        self,
        state: &str,
        timeout: Duration,
        page: &Page,
        service: &'static str,
    ) -> Result<String, SyncError> {
        let accept = async {
            loop {
                let Ok((mut stream, _)) = self.listener.accept().await else {
                    continue;
                };
                let Some(target) = read_target(&mut stream).await else {
                    continue;
                };
                match self.evaluate(&target, state, service) {
                    Verdict::Ignore(status) => respond(&mut stream, status, "Nicht gefunden").await,
                    Verdict::Reject(status, message) => respond(&mut stream, status, message).await,
                    Verdict::Code(code) => {
                        respond(&mut stream, 200, &page.success_html()).await;
                        return Ok(code);
                    }
                    Verdict::Denied(error) => {
                        respond(&mut stream, 400, "Anmeldung abgebrochen").await;
                        return Err(error);
                    }
                }
            }
        };
        tokio::time::timeout(timeout, accept)
            .await
            .unwrap_or_else(|_| {
                Err(SyncError::OAuth(format!(
                    "Zeitüberschreitung beim Verbinden mit {service} (nach {} s)",
                    timeout.as_secs()
                )))
            })
    }

    fn evaluate(&self, target: &str, state: &str, service: &'static str) -> Verdict {
        let Ok(url) = Url::parse(&format!("http://127.0.0.1{target}")) else {
            return Verdict::Ignore(400);
        };
        if url.path() != self.path {
            return Verdict::Ignore(404);
        }
        let param = |name: &str| {
            url.query_pairs()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.into_owned())
        };
        if param("state").as_deref() != Some(state) {
            return Verdict::Reject(400, "Ungültiger State-Parameter");
        }
        if let Some(error) = param("error") {
            let detail = param("error_description").unwrap_or(error);
            return Verdict::Denied(SyncError::OAuth(format!(
                "Die Anmeldung bei {service} wurde abgebrochen: {detail}"
            )));
        }
        match param("code").filter(|code| !code.is_empty()) {
            Some(code) => Verdict::Code(code),
            None => Verdict::Reject(400, "Kein Autorisierungscode erhalten"),
        }
    }
}

enum Verdict {
    Ignore(u16),
    Reject(u16, &'static str),
    Code(String),
    Denied(SyncError),
}

/// Liest die Anfragezeile (`GET /pfad?query HTTP/1.1`) und liefert das Ziel.
async fn read_target(stream: &mut TcpStream) -> Option<String> {
    let mut buffer = Vec::new();
    let read = async {
        let mut chunk = [0u8; 1024];
        while buffer.len() < MAX_REQUEST && !buffer.windows(2).any(|w| w == b"\r\n") {
            match stream.read(&mut chunk).await {
                Ok(0) | Err(_) => break,
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            }
        }
    };
    tokio::time::timeout(READ_TIMEOUT, read).await.ok()?;
    let head = String::from_utf8_lossy(&buffer);
    let line = head.lines().next()?;
    let mut parts = line.split_whitespace();
    (parts.next()? == "GET").then(|| parts.next().map(str::to_owned))?
}

async fn respond(stream: &mut TcpStream, status: u16, body: &str) {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Bad Request",
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn get(port: u16, target: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        stream
            .write_all(format!("GET {target} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
            .await
            .unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).await.unwrap();
        answer
    }

    const PAGE: Page = Page {
        title: "Verbunden",
        message: "Fertig.",
    };

    #[tokio::test]
    async fn the_second_port_is_used_when_the_first_is_taken() {
        let taken = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let busy = taken.local_addr().unwrap().port();
        let loopback = Loopback::bind(&[busy, 0], "/cb").await.unwrap();
        assert_ne!(loopback.port, busy);
        assert_eq!(
            loopback.redirect_uri(),
            format!("http://127.0.0.1:{}/cb", loopback.port)
        );
        assert!(Loopback::bind(&[busy], "/cb").await.is_err());
    }

    #[tokio::test]
    async fn stray_requests_do_not_end_the_wait() {
        let loopback = Loopback::bind(&[0], "/cb").await.unwrap();
        let port = loopback.port;
        let waiting = tokio::spawn(async move {
            loopback
                .wait_for_code("st", Duration::from_secs(5), &PAGE, "Test")
                .await
        });
        assert!(get(port, "/favicon.ico").await.contains("404"));
        assert!(get(port, "/cb?code=x&state=falsch").await.contains("400"));
        assert!(get(port, "/cb?state=st").await.contains("400"));
        let ok = get(port, "/cb?code=abc&state=st").await;
        assert!(ok.contains("200") && ok.contains("Verbunden"));
        assert_eq!(waiting.await.unwrap().unwrap(), "abc");
    }

    #[tokio::test]
    async fn a_denied_authorization_names_the_reason() {
        let loopback = Loopback::bind(&[0], "/cb").await.unwrap();
        let port = loopback.port;
        let waiting = tokio::spawn(async move {
            loopback
                .wait_for_code("st", Duration::from_secs(5), &PAGE, "Test")
                .await
        });
        get(
            port,
            "/cb?state=st&error=access_denied&error_description=Nein",
        )
        .await;
        let error = waiting.await.unwrap().unwrap_err().to_string();
        assert!(
            error.contains("abgebrochen") && error.contains("Nein"),
            "{error}"
        );
    }

    #[tokio::test]
    async fn a_request_line_that_arrives_in_pieces_is_read_completely() {
        let loopback = Loopback::bind(&[0], "/cb").await.unwrap();
        let port = loopback.port;
        let waiting = tokio::spawn(async move {
            loopback
                .wait_for_code("st", Duration::from_secs(5), &PAGE, "Test")
                .await
        });
        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        stream.write_all(b"GET /c").await.unwrap();
        stream.flush().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        stream
            .write_all(b"b?code=teil&state=st HTTP/1.1\r\nHost: x\r\n\r\n")
            .await
            .unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).await.unwrap();
        assert!(answer.starts_with("HTTP/1.1 200 OK"), "{answer}");
        assert_eq!(waiting.await.unwrap().unwrap(), "teil");
    }

    #[tokio::test]
    async fn answers_carry_the_matching_status_text() {
        let loopback = Loopback::bind(&[0], "/cb").await.unwrap();
        let port = loopback.port;
        let waiting = tokio::spawn(async move {
            loopback
                .wait_for_code("st", Duration::from_secs(5), &PAGE, "Test")
                .await
        });
        assert!(get(port, "/x").await.starts_with("HTTP/1.1 404 Not Found"));
        assert!(
            get(port, "/cb?state=st")
                .await
                .starts_with("HTTP/1.1 400 Bad Request")
        );
        get(port, "/cb?code=c&state=st").await;
        waiting.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn the_wait_ends_after_the_timeout() {
        let loopback = Loopback::bind(&[0], "/cb").await.unwrap();
        let error = loopback
            .wait_for_code("st", Duration::from_millis(50), &PAGE, "Test")
            .await
            .unwrap_err();
        assert!(error.to_string().contains("Zeitüberschreitung"));
    }
}
