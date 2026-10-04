//! Der Server: lauscht nur auf `127.0.0.1`, gibt jede Anfrage an [`route`] weiter und schließt die Verbindung
//! danach. Zeitgrenzen und eine Obergrenze gleichzeitiger Verbindungen halten schlecht erzogene Clients in Schach.

use std::convert::Infallible;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::{TokioIo, TokioTimer};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

use crate::http::{Method, Request, Response};
use crate::router::{ApiConfig, MAX_BODY_BYTES, TodoBackend, route};

/// So viele Ports ab dem gewünschten werden probiert, wenn er belegt ist.
pub const PORT_ATTEMPTS: u16 = 10;
/// Gleichzeitige Verbindungen; weitere werden sofort geschlossen.
const MAX_CONNECTIONS: usize = 32;
const HEADER_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Ein laufender Server. [`ApiServer::stop`] schließt den Port.
#[derive(Debug)]
pub struct ApiServer {
    port: u16,
    task: JoinHandle<()>,
}

impl ApiServer {
    /// Port, auf dem der Server tatsächlich lauscht.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Adresse des Servers (immer `127.0.0.1`).
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::from((Ipv4Addr::LOCALHOST, self.port))
    }

    /// Beendet den Server; danach ist der Port wieder frei.
    pub async fn stop(self) {
        self.task.abort();
        // wartet, bis der Listener wirklich freigegeben ist
        let _ = self.task.await;
    }
}

/// Bindet den ersten freien Port ab `preferred_port` (`0` = beliebiger freier Port) und startet den Server.
pub async fn start(
    preferred_port: u16,
    token: String,
    backend: Arc<dyn TodoBackend>,
) -> io::Result<ApiServer> {
    let (listener, port) = bind(preferred_port).await?;
    let config = Arc::new(ApiConfig { token, port });
    let limit = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let task = tokio::spawn(async move {
        loop {
            let stream = match listener.accept().await {
                Ok((stream, _)) => stream,
                Err(error) => {
                    log::warn!("API: Verbindung nicht angenommen: {error}");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            };
            let Ok(permit) = limit.clone().try_acquire_owned() else {
                continue; // zu viele Verbindungen: dieser Client wird fallen gelassen
            };
            let (config, backend) = (config.clone(), backend.clone());
            tokio::spawn(async move {
                let service =
                    service_fn(move |request| handle(request, config.clone(), backend.clone()));
                let connection = http1::Builder::new()
                    .timer(TokioTimer::new())
                    .header_read_timeout(HEADER_TIMEOUT)
                    .keep_alive(false)
                    .serve_connection(TokioIo::new(stream), service);
                if let Ok(Err(error)) = tokio::time::timeout(CONNECTION_TIMEOUT, connection).await {
                    log::debug!("API: Verbindung beendet: {error}");
                }
                drop(permit);
            });
        }
    });
    Ok(ApiServer { port, task })
}

async fn bind(preferred_port: u16) -> io::Result<(TcpListener, u16)> {
    let attempts = if preferred_port == 0 {
        1
    } else {
        PORT_ATTEMPTS
    };
    let mut last_error = None;
    for offset in 0..attempts {
        let Some(port) = preferred_port.checked_add(offset) else {
            break;
        };
        match TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => {
                let actual = listener.local_addr()?.port();
                return Ok((listener, actual));
            }
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| io::Error::other("kein Port frei")))
}

type HyperResponse = hyper::Response<Full<Bytes>>;

async fn handle(
    request: hyper::Request<Incoming>,
    config: Arc<ApiConfig>,
    backend: Arc<dyn TodoBackend>,
) -> Result<HyperResponse, Infallible> {
    let (parts, body) = request.into_parts();
    let declared: Option<usize> = parts
        .headers
        .get(hyper::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok());
    if declared.is_some_and(|length| length > MAX_BODY_BYTES) {
        return Ok(to_hyper(too_large()));
    }
    let Ok(collected) = Limited::new(body, MAX_BODY_BYTES).collect().await else {
        return Ok(to_hyper(too_large()));
    };
    let request = Request {
        method: Method::parse(parts.method.as_str()),
        path: parts.uri.path().to_owned(),
        query: parts.uri.query().unwrap_or_default().to_owned(),
        headers: parts
            .headers
            .iter()
            .filter_map(|(name, value)| {
                Some((name.as_str().to_owned(), value.to_str().ok()?.to_owned()))
            })
            .collect(),
        body: collected.to_bytes().to_vec(),
    };
    // die Datenbank liegt hinter einer blockierenden Sperre
    let response = tokio::task::spawn_blocking(move || route(&request, &config, &*backend))
        .await
        .unwrap_or_else(|_| Response::error(500, "internal", "Internal error"));
    Ok(to_hyper(response))
}

fn too_large() -> Response {
    Response::error(413, "payload_too_large", "Request body too large")
}

fn to_hyper(response: Response) -> HyperResponse {
    let mut builder = hyper::Response::builder().status(response.status);
    for (name, value) in &response.headers {
        builder = builder.header(name, value);
    }
    builder
        .body(Full::new(Bytes::from(response.body)))
        .unwrap_or_else(|_| hyper::Response::new(Full::new(Bytes::new())))
}
