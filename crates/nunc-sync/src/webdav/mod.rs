//! WebDAV-Client für `nunc-sync.v2.json` (ADR 0016; die alte `nunc-sync.json` wird nur einmal gelesen).
//!
//! Anfragen: `PROPFIND` (Ordner prüfen, Verbindungstest), `MKCOL` (Ordner anlegen), `GET`/`PUT` (Datei).
//! Beim Hochladen schützen `If-Match` (Datei vom Server geladen) bzw. `If-None-Match: *` (Datei gab es noch
//! nicht) vor verlorenen Änderungen; Server ohne ETag werden ohne Bedingung beschrieben.

mod digest;

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::Mutex;

use nunc_core::model::{ConnectionTestResult, SyncFile, WebDavAuth, WebDavConfig};
use nunc_core::sync_document::SyncDocument;
use reqwest::{Method, Response, StatusCode, Url, header};

use crate::SyncError;
use digest::Challenge;

/// Name der Datei auf dem Server.
pub const SYNC_FILE_NAME: &str = "nunc-sync.v2.json";
/// Name der alten Datei (Format vor ADR 0016), nur für den einmaligen Import.
pub const LEGACY_FILE_NAME: &str = "nunc-sync.json";

const PROPFIND_BODY: &str = r#"<?xml version="1.0" encoding="utf-8"?><d:propfind xmlns:d="DAV:"><d:prop><d:resourcetype/></d:prop></d:propfind>"#;

/// Bedingung für den Upload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Precondition {
    /// Die Datei hatte diesen ETag, als wir sie geladen haben.
    Match(String),
    /// Die Datei gab es nicht; sie darf auch jetzt nicht existieren.
    NoneExists,
    /// Server ohne ETag.
    None,
}

/// Die Datei vom Server samt ETag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub file: SyncDocument,
    pub etag: Option<String>,
}

pub struct WebDavClient {
    http: reqwest::Client,
    config: WebDavConfig,
    password: String,
    base: Url,
    /// Zuletzt gemeldete Digest-Herausforderung und der Anfragezähler (`nc`) dazu.
    digest: Mutex<Option<(Challenge, u32)>>,
}

impl WebDavClient {
    pub fn new(config: WebDavConfig, password: String) -> Result<Self, SyncError> {
        let base = Url::parse(config.url.trim())
            .ok()
            .filter(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
            .ok_or_else(|| SyncError::InvalidUrl(config.url.clone()))?;
        let http = crate::oauth::http_client()?;
        Ok(Self {
            http,
            config,
            password,
            base,
            digest: Mutex::new(None),
        })
    }

    /// Segmente des Remote-Pfads ohne leere Teile: `/a//b/` → `["a", "b"]`.
    fn segments(&self) -> Vec<&str> {
        self.config
            .remote_path
            .split('/')
            .filter(|segment| !segment.trim().is_empty())
            .collect()
    }

    /// URL des Ordners mit den ersten `depth` Segmenten (immer mit `/` am Ende).
    fn folder_url(&self, depth: usize) -> String {
        let mut url = self.base.as_str().trim_end_matches('/').to_owned();
        url.push('/');
        for segment in self.segments().into_iter().take(depth) {
            url.push_str(&encode_segment(segment));
            url.push('/');
        }
        url
    }

    fn file_url(&self) -> String {
        format!("{}{SYNC_FILE_NAME}", self.folder_url(usize::MAX))
    }

    fn legacy_file_url(&self) -> String {
        format!("{}{LEGACY_FILE_NAME}", self.folder_url(usize::MAX))
    }

    async fn send(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, String)],
        body: Option<Vec<u8>>,
    ) -> Result<Response, SyncError> {
        let first = self.authorization(method, url);
        let response = self
            .send_once(method, url, headers, body.clone(), first)
            .await?;
        if self.config.auth_method != WebDavAuth::Digest
            || response.status() != StatusCode::UNAUTHORIZED
        {
            return Ok(response);
        }
        let challenge = response
            .headers()
            .get_all(header::WWW_AUTHENTICATE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .find_map(digest::parse_challenge);
        let Some(challenge) = challenge else {
            return Ok(response);
        };
        *self.digest.lock().map_err(poisoned)? = Some((challenge, 0));
        let second = self.authorization(method, url);
        self.send_once(method, url, headers, body, second).await
    }

    async fn send_once(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, String)],
        body: Option<Vec<u8>>,
        authorization: Option<String>,
    ) -> Result<Response, SyncError> {
        let method = Method::from_bytes(method.as_bytes())
            .map_err(|error| SyncError::Transport(error.to_string()))?;
        let mut request = self.http.request(method, url);
        for (name, value) in headers {
            request = request.header(*name, value);
        }
        request = match (self.config.auth_method, authorization) {
            (WebDavAuth::Basic, _) => {
                request.basic_auth(&self.config.username, Some(&self.password))
            }
            (WebDavAuth::Digest, Some(value)) => request.header(header::AUTHORIZATION, value),
            (WebDavAuth::Digest, None) => request,
        };
        if let Some(body) = body {
            request = request.body(body);
        }
        request
            .send()
            .await
            .map_err(|error| SyncError::Transport(describe_transport_error(&error)))
    }

    /// `Authorization` für Digest aus der gemerkten Herausforderung (zählt `nc` hoch), sonst `None`.
    fn authorization(&self, method: &str, url: &str) -> Option<String> {
        if self.config.auth_method != WebDavAuth::Digest {
            return None;
        }
        let mut guard = self.digest.lock().ok()?;
        let (challenge, count) = guard.as_mut()?;
        *count += 1;
        let target = Url::parse(url).ok()?;
        let uri = match target.query() {
            Some(query) => format!("{}?{query}", target.path()),
            None => target.path().to_owned(),
        };
        Some(digest::authorization(
            challenge,
            &self.config.username,
            &self.password,
            method,
            &uri,
            *count,
            &client_nonce(),
        ))
    }

    /// Gemeinsame Abbildung der Statuscodes, die für jede Anfrage gleich sind.
    fn check(&self, response: &Response, action: &'static str) -> Result<(), SyncError> {
        match response.status() {
            StatusCode::UNAUTHORIZED => {
                if self.config.auth_method == WebDavAuth::Basic
                    && response
                        .headers()
                        .get_all(header::WWW_AUTHENTICATE)
                        .iter()
                        .filter_map(|value| value.to_str().ok())
                        .any(|value| digest::parse_challenge(value).is_some())
                    && !response
                        .headers()
                        .get_all(header::WWW_AUTHENTICATE)
                        .iter()
                        .filter_map(|value| value.to_str().ok())
                        .any(|value| value.trim_start().to_lowercase().starts_with("basic"))
                {
                    return Err(SyncError::WrongAuthMethod("Digest", "Basic"));
                }
                Err(SyncError::Unauthorized)
            }
            StatusCode::FORBIDDEN => Err(SyncError::Forbidden),
            status => Err(SyncError::Status {
                status: status.as_u16(),
                action,
            }),
        }
    }

    /// Prüft Erreichbarkeit und Zugangsdaten mit einem `PROPFIND` auf den Remote-Ordner. Ein fehlender Ordner
    /// ist kein Fehler: er wird beim ersten Sync angelegt.
    pub async fn test_connection(&self) -> ConnectionTestResult {
        match self.propfind_folder(usize::MAX).await {
            Ok(_) => ConnectionTestResult {
                success: true,
                message: self.base.host_str().unwrap_or_default().to_owned(),
            },
            Err(error) => ConnectionTestResult {
                success: false,
                message: error.to_string(),
            },
        }
    }

    /// `true`, wenn der Ordner existiert, `false` bei 404; alles andere ist ein Fehler.
    async fn propfind_folder(&self, depth: usize) -> Result<bool, SyncError> {
        let response = self
            .send(
                "PROPFIND",
                &self.folder_url(depth),
                &[
                    ("Depth", "0".to_owned()),
                    ("Content-Type", "application/xml".to_owned()),
                ],
                Some(PROPFIND_BODY.as_bytes().to_vec()),
            )
            .await?;
        match response.status() {
            StatusCode::NOT_FOUND => Ok(false),
            status if status.is_success() => Ok(true),
            _ => self.check(&response, "Prüfen des Ordners").map(|()| true),
        }
    }

    /// Legt den Remote-Ordner an, falls er fehlt (`MKCOL` je Pfadsegment, wie `MkdirAll` in Go).
    pub async fn ensure_folder(&self) -> Result<(), SyncError> {
        if self.propfind_folder(usize::MAX).await? {
            return Ok(());
        }
        for depth in 1..=self.segments().len() {
            let response = self
                .send("MKCOL", &self.folder_url(depth), &[], None)
                .await?;
            let status = response.status();
            // 405: gibt es schon
            if !(status.is_success() || status == StatusCode::METHOD_NOT_ALLOWED) {
                return self.check(&response, "Anlegen des Ordners (MKCOL)");
            }
        }
        Ok(())
    }

    /// Lädt eine Datei samt ETag; `None`, wenn es sie nicht gibt.
    async fn fetch(&self, url: &str) -> Result<Option<(Vec<u8>, Option<String>)>, SyncError> {
        let response = self.send("GET", url, &[], None).await?;
        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !status.is_success() {
            return self
                .check(&response, "Herunterladen der Sync-Datei")
                .map(|()| None);
        }
        let etag = response
            .headers()
            .get(header::ETAG)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = response
            .bytes()
            .await
            .map_err(|error| SyncError::Transport(describe_transport_error(&error)))?;
        Ok(Some((bytes.to_vec(), etag)))
    }

    /// Lädt die Sync-Datei; `None`, wenn es sie noch nicht gibt (erster Sync).
    pub async fn download(&self) -> Result<Option<Remote>, SyncError> {
        let Some((bytes, etag)) = self.fetch(&self.file_url()).await? else {
            return Ok(None);
        };
        let file = serde_json::from_slice(&bytes)
            .map_err(|error| SyncError::InvalidFile(error.to_string()))?;
        Ok(Some(Remote { file, etag }))
    }

    /// Lädt die alte Sync-Datei `nunc-sync.json` (Format vor ADR 0016) für den einmaligen Import.
    pub async fn download_legacy(&self) -> Result<Option<SyncFile>, SyncError> {
        let Some((bytes, _)) = self.fetch(&self.legacy_file_url()).await? else {
            return Ok(None);
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| SyncError::InvalidFile(error.to_string()))
    }

    /// Schreibt die Sync-Datei (eingerückt, damit sie sich lesen und vergleichen lässt).
    pub async fn upload(
        &self,
        file: &SyncDocument,
        precondition: &Precondition,
    ) -> Result<(), SyncError> {
        let body = serde_json::to_vec_pretty(file)
            .map_err(|error| SyncError::InvalidFile(error.to_string()))?;
        let mut headers = vec![("Content-Type", "application/json".to_owned())];
        match precondition {
            Precondition::Match(etag) => headers.push(("If-Match", etag.clone())),
            Precondition::NoneExists => headers.push(("If-None-Match", "*".to_owned())),
            Precondition::None => {}
        }
        let response = self
            .send("PUT", &self.file_url(), &headers, Some(body))
            .await?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        if status == StatusCode::PRECONDITION_FAILED {
            return Err(SyncError::Conflict);
        }
        self.check(&response, "Hochladen der Sync-Datei")
    }
}

fn poisoned<T>(_: std::sync::PoisonError<T>) -> SyncError {
    SyncError::Transport("interner Zustand gesperrt".to_owned())
}

/// Zufälliger Client-Nonce für Digest (16 Hex-Zeichen).
fn client_nonce() -> String {
    format!("{:016x}", RandomState::new().build_hasher().finish())
}

/// Prozent-Kodierung eines Pfadsegments: nur unreservierte Zeichen bleiben stehen.
fn encode_segment(segment: &str) -> String {
    segment
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'@' => {
                (byte as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// Lesbarer Text zu einem Netzwerkfehler (Go: `friendlyWebDAVError`).
pub(crate) fn describe_transport_error(error: &reqwest::Error) -> String {
    let mut chain = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        chain.push_str(": ");
        chain.push_str(&cause.to_string());
        source = cause.source();
    }
    classify_transport_error(error.is_timeout(), error.is_connect(), &chain)
}

fn classify_transport_error(timeout: bool, connect: bool, chain: &str) -> String {
    let text = chain.to_lowercase();
    let any = |needles: &[&str]| needles.iter().any(|needle| text.contains(needle));
    if timeout || any(&["timed out", "deadline"]) {
        "Zeitüberschreitung – Server antwortet nicht".to_owned()
    } else if any(&[
        "dns error",
        "failed to lookup",
        "no such host",
        "name or service not known",
        "nodename nor servname",
    ]) {
        "Host nicht gefunden – URL prüfen".to_owned()
    } else if any(&["unknownissuer", "unknown issuer"]) {
        "SSL-Zertifikatsfehler – Aussteller unbekannt (Zwischenzertifikat fehlt oder Firmen-Proxy)"
            .to_owned()
    } else if any(&["notvalidforname", "not valid for name"]) {
        "SSL-Zertifikatsfehler – Zertifikat passt nicht zum Hostnamen".to_owned()
    } else if any(&["expired"]) {
        "SSL-Zertifikatsfehler – Zertifikat abgelaufen".to_owned()
    } else if any(&["certificate", "tls", "ssl", "handshake"]) {
        "SSL-Zertifikatsfehler".to_owned()
    } else if any(&["connection refused", "actively refused"]) {
        "Verbindung abgelehnt – Server nicht erreichbar".to_owned()
    } else if connect {
        "Server nicht erreichbar".to_owned()
    } else {
        chain.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client(url: &str, path: &str) -> WebDavClient {
        WebDavClient::new(
            WebDavConfig {
                url: url.into(),
                remote_path: path.into(),
                username: "u".into(),
                ..WebDavConfig::default()
            },
            "p".into(),
        )
        .unwrap()
    }

    #[test]
    fn urls_are_joined_without_double_slashes() {
        for (url, path) in [
            ("https://h.org/dav", "/nunc/"),
            ("https://h.org/dav/", "nunc"),
            ("https://h.org/dav", "//nunc//"),
        ] {
            let c = client(url, path);
            assert_eq!(c.folder_url(usize::MAX), "https://h.org/dav/nunc/");
            assert_eq!(c.file_url(), "https://h.org/dav/nunc/nunc-sync.v2.json");
        }
        assert_eq!(
            client("https://h.org", "/").file_url(),
            "https://h.org/nunc-sync.v2.json"
        );
        let nested = client("https://h.org/dav", "/a b/c/");
        assert_eq!(nested.folder_url(1), "https://h.org/dav/a%20b/");
        assert_eq!(nested.folder_url(2), "https://h.org/dav/a%20b/c/");
        assert_eq!(nested.folder_url(0), "https://h.org/dav/");
    }

    #[test]
    fn invalid_urls_are_rejected() {
        for url in ["", "h.org", "ftp://h.org", "https://"] {
            let result = WebDavClient::new(
                WebDavConfig {
                    url: url.into(),
                    ..WebDavConfig::default()
                },
                String::new(),
            );
            assert!(matches!(result, Err(SyncError::InvalidUrl(_))), "{url}");
        }
    }

    #[test]
    fn transport_errors_become_readable_messages() {
        let cases = [
            (
                true,
                false,
                "irgendwas",
                "Zeitüberschreitung – Server antwortet nicht",
            ),
            (
                false,
                false,
                "operation timed out",
                "Zeitüberschreitung – Server antwortet nicht",
            ),
            (
                false,
                true,
                "dns error: failed to lookup address",
                "Host nicht gefunden – URL prüfen",
            ),
            (
                false,
                true,
                "invalid peer certificate: UnknownIssuer",
                "SSL-Zertifikatsfehler – Aussteller unbekannt (Zwischenzertifikat fehlt oder Firmen-Proxy)",
            ),
            (
                false,
                true,
                "invalid peer certificate: NotValidForName",
                "SSL-Zertifikatsfehler – Zertifikat passt nicht zum Hostnamen",
            ),
            (
                false,
                true,
                "invalid peer certificate: Expired",
                "SSL-Zertifikatsfehler – Zertifikat abgelaufen",
            ),
            (false, true, "tls handshake eof", "SSL-Zertifikatsfehler"),
            (
                false,
                true,
                "tcp connect error: Connection refused (os error 61)",
                "Verbindung abgelehnt – Server nicht erreichbar",
            ),
            (false, true, "unreachable", "Server nicht erreichbar"),
            (false, false, "etwas anderes", "etwas anderes"),
        ];
        for (timeout, connect, chain, expected) in cases {
            assert_eq!(classify_transport_error(timeout, connect, chain), expected);
        }
    }

    #[test]
    fn segments_keep_only_unreserved_characters() {
        assert_eq!(encode_segment("nunc-1_a.b~c@d"), "nunc-1_a.b~c@d");
        assert_eq!(encode_segment("ä b/c"), "%C3%A4%20b%2Fc");
    }

    #[test]
    fn digest_requests_count_up_with_the_same_challenge() {
        let c = WebDavClient::new(
            WebDavConfig {
                url: "https://h.org".into(),
                auth_method: WebDavAuth::Digest,
                username: "u".into(),
                remote_path: "/".into(),
            },
            "p".into(),
        )
        .unwrap();
        assert!(c.authorization("GET", "https://h.org/x").is_none());
        let challenge =
            digest::parse_challenge(r#"Digest realm="r", nonce="n", qop="auth""#).unwrap();
        *c.digest.lock().unwrap() = Some((challenge, 0));
        let first = c.authorization("GET", "https://h.org/x?a=1").unwrap();
        let second = c.authorization("GET", "https://h.org/x").unwrap();
        assert!(first.contains("nc=00000001") && first.contains("uri=\"/x?a=1\""));
        assert!(second.contains("nc=00000002"));
    }

    #[test]
    fn client_nonces_differ() {
        assert_ne!(client_nonce(), client_nonce());
        assert_eq!(client_nonce().len(), 16);
    }
}
