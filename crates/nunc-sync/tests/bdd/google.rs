//! Ein kleines „Google“ für die Drive-Szenarien (wiremock): Token-Endpunkt, Drive-Dateien mit Versionen, das
//! Konto und ein Protokoll aller Anfragen. Dazu ein „Browser“, der die Anmeldung selbstständig beantwortet.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use nunc_platform::{OpenError, UrlOpener};
use reqwest::Url;
use wiremock::{MockServer, Request, Respond, ResponseTemplate};

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub parent: String,
    pub folder: bool,
    pub content: Vec<u8>,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct Logged {
    pub method: String,
    pub path: String,
    pub query: String,
    pub bearer: Option<String>,
}

#[derive(Debug, Default)]
pub struct GoogleState {
    pub email: Option<String>,
    pub about_fails: bool,
    pub items: Vec<Item>,
    next_id: u32,
    /// Access-Tokens, die der API-Teil akzeptiert.
    pub valid: BTreeSet<String>,
    token_counter: u32,
    pub token_requests: Vec<BTreeMap<String, String>>,
    /// Antwort des Token-Endpunkts auf den Code-Austausch (Fehlercode) bzw. auf das Erneuern.
    pub reject_code: Option<String>,
    pub reject_refresh: bool,
    /// Lebensdauer der ausgegebenen Tokens in Sekunden.
    pub expires_in: i64,
    pub fail_all: Option<u16>,
    /// Inhalt, den ein anderes Gerät beim nächsten Prüfen der Änderungszeit schreibt.
    pub interfere: Option<Vec<u8>>,
    pub log: Vec<Logged>,
}

impl GoogleState {
    fn new_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}-{}", self.next_id)
    }

    pub fn add_folder(&mut self, name: &str) -> String {
        let id = self.new_id("folder");
        self.items.push(Item {
            id: id.clone(),
            name: name.to_owned(),
            parent: "root".to_owned(),
            folder: true,
            content: Vec::new(),
            version: 1,
        });
        id
    }

    pub fn add_file(&mut self, name: &str, parent: &str, content: Vec<u8>) -> String {
        let id = self.new_id("file");
        self.items.push(Item {
            id: id.clone(),
            name: name.to_owned(),
            parent: parent.to_owned(),
            folder: false,
            content,
            version: 1,
        });
        id
    }

    pub fn folder(&self, name: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.folder && i.name == name)
    }

    pub fn file_in(&self, folder: &str, name: &str) -> Option<&Item> {
        let parent = self.folder(folder)?.id.clone();
        self.items
            .iter()
            .find(|i| !i.folder && i.name == name && i.parent == parent)
    }
}

pub struct Google {
    pub server: MockServer,
    pub state: Arc<Mutex<GoogleState>>,
}

impl std::fmt::Debug for Google {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Google")
    }
}

impl Google {
    pub async fn start(email: Option<&str>) -> Self {
        let server = MockServer::start().await;
        let state = Arc::new(Mutex::new(GoogleState {
            email: email.map(str::to_owned),
            expires_in: 3600,
            ..GoogleState::default()
        }));
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(Responder(state.clone()))
            .mount(&server)
            .await;
        Self { server, state }
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut GoogleState) -> T) -> T {
        f(&mut self.state.lock().unwrap())
    }
}

struct Responder(Arc<Mutex<GoogleState>>);

fn json(status: u16, value: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(value)
}

fn query_of(request: &Request) -> BTreeMap<String, String> {
    request
        .url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}

/// Der Wert zu `key='…'` in einer Drive-Suchanfrage (mit `\'` als Maskierung).
fn quoted(q: &str, key: &str) -> Option<String> {
    let start = q.find(&format!("{key}='"))? + key.len() + 2;
    let mut value = String::new();
    let mut chars = q[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => value.extend(chars.next()),
            '\'' => return Some(value),
            other => value.push(other),
        }
    }
    None
}

fn form_of(body: &[u8]) -> BTreeMap<String, String> {
    Url::parse(&format!("http://x/?{}", String::from_utf8_lossy(body)))
        .map(|url| {
            url.query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect()
        })
        .unwrap_or_default()
}

/// Zerlegt `multipart/related` in die Bodies der Teile.
fn multipart_parts(request: &Request) -> Vec<Vec<u8>> {
    let content_type = request
        .headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let boundary = content_type.split("boundary=").nth(1).unwrap_or_default();
    let delimiter = format!("--{boundary}");
    let text = &request.body;
    let mut parts = Vec::new();
    let mut rest: &[u8] = text;
    let find =
        |haystack: &[u8], needle: &[u8]| haystack.windows(needle.len()).position(|w| w == needle);
    while let Some(at) = find(rest, delimiter.as_bytes()) {
        let after = &rest[at + delimiter.len()..];
        if after.starts_with(b"--") {
            break;
        }
        let part = after.strip_prefix(b"\r\n").unwrap_or(after);
        let end = find(part, delimiter.as_bytes()).unwrap_or(part.len());
        let section = &part[..end];
        if let Some(split) = find(section, b"\r\n\r\n") {
            let body = &section[split + 4..];
            parts.push(body.strip_suffix(b"\r\n").unwrap_or(body).to_vec());
        }
        rest = &part[end..];
    }
    parts
}

impl Respond for Responder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let mut state = self.0.lock().unwrap();
        let method = request.method.to_string();
        let path = request.url.path().to_owned();
        let bearer = request
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::to_owned);
        let response = answer(&mut state, request, &method, &path, bearer.as_deref());
        state.log.push(Logged {
            method,
            path,
            query: request.url.query().unwrap_or_default().to_owned(),
            bearer,
        });
        response
    }
}

fn answer(
    state: &mut GoogleState,
    request: &Request,
    method: &str,
    path: &str,
    bearer: Option<&str>,
) -> ResponseTemplate {
    if path == "/token" {
        return token(state, request);
    }
    if let Some(status) = state.fail_all {
        return ResponseTemplate::new(status);
    }
    if !bearer.is_some_and(|token| state.valid.contains(token)) {
        return json(401, serde_json::json!({ "error": { "code": 401 } }));
    }
    let query = query_of(request);
    match (method, path) {
        ("GET", "/drive/v3/about") => {
            if state.about_fails {
                return ResponseTemplate::new(500);
            }
            json(
                200,
                serde_json::json!({ "user": { "emailAddress": state.email } }),
            )
        }
        ("GET", "/drive/v3/files") => {
            let q = query.get("q").cloned().unwrap_or_default();
            let name = quoted(&q, "name").unwrap_or_default();
            let found: Vec<_> = if q.contains("mimeType='application/vnd.google-apps.folder'") {
                state
                    .items
                    .iter()
                    .filter(|i| i.folder && i.name == name && q.contains("'root' in parents"))
                    .collect()
            } else {
                let parent = q
                    .split("' in parents")
                    .next()
                    .and_then(|head| head.rsplit('\'').next())
                    .unwrap_or_default()
                    .to_owned();
                state
                    .items
                    .iter()
                    .filter(|i| !i.folder && i.name == name && i.parent == parent)
                    .collect()
            };
            let files: Vec<_> = found
                .iter()
                .map(|i| serde_json::json!({ "id": i.id, "modifiedTime": format!("t{}", i.version) }))
                .collect();
            json(200, serde_json::json!({ "files": files }))
        }
        ("POST", "/drive/v3/files") => {
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap_or_default();
            let name = body["name"].as_str().unwrap_or_default().to_owned();
            let id = state.add_folder(&name);
            json(200, serde_json::json!({ "id": id }))
        }
        ("GET", p) if p.starts_with("/drive/v3/files/") => {
            let id = p.trim_start_matches("/drive/v3/files/").to_owned();
            if query
                .get("fields")
                .is_some_and(|f| f.contains("modifiedTime"))
                && let Some(content) = state.interfere.take()
                && let Some(item) = state.items.iter_mut().find(|i| i.id == id)
            {
                item.content = content;
                item.version += 1;
            }
            let Some(item) = state.items.iter().find(|i| i.id == id) else {
                return ResponseTemplate::new(404);
            };
            if query.get("alt").map(String::as_str) == Some("media") {
                ResponseTemplate::new(200).set_body_bytes(item.content.clone())
            } else {
                json(
                    200,
                    serde_json::json!({ "id": item.id, "modifiedTime": format!("t{}", item.version) }),
                )
            }
        }
        ("POST", "/upload/drive/v3/files") => {
            let parts = multipart_parts(request);
            let meta: serde_json::Value = parts
                .first()
                .and_then(|p| serde_json::from_slice(p).ok())
                .unwrap_or_default();
            let name = meta["name"].as_str().unwrap_or_default().to_owned();
            let parent = meta["parents"][0].as_str().unwrap_or("root").to_owned();
            let content = parts.get(1).cloned().unwrap_or_default();
            let id = state.add_file(&name, &parent, content);
            json(200, serde_json::json!({ "id": id }))
        }
        ("PATCH", p) if p.starts_with("/upload/drive/v3/files/") => {
            let id = p.trim_start_matches("/upload/drive/v3/files/").to_owned();
            match state.items.iter_mut().find(|i| i.id == id) {
                Some(item) => {
                    item.content = request.body.clone();
                    item.version += 1;
                    json(200, serde_json::json!({ "id": id }))
                }
                None => ResponseTemplate::new(404),
            }
        }
        _ => ResponseTemplate::new(404),
    }
}

fn token(state: &mut GoogleState, request: &Request) -> ResponseTemplate {
    let form = form_of(&request.body);
    state.token_requests.push(form.clone());
    let grant = form.get("grant_type").map(String::as_str);
    let rejection = match grant {
        Some("authorization_code") => state.reject_code.clone(),
        Some("refresh_token") => state.reject_refresh.then(|| "invalid_grant".to_owned()),
        _ => Some("unsupported_grant_type".to_owned()),
    };
    if let Some(error) = rejection {
        return json(400, serde_json::json!({ "error": error }));
    }
    state.token_counter += 1;
    let access = format!("access-{}", state.token_counter);
    state.valid.insert(access.clone());
    json(
        200,
        serde_json::json!({
            "access_token": access,
            "refresh_token": format!("refresh-{}", state.token_counter),
            "expires_in": state.expires_in,
            "token_type": "Bearer",
        }),
    )
}

/// Was der „Browser“ nach dem Öffnen der Anmeldeseite tut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Behavior {
    #[default]
    Approve,
    Deny,
    /// Reagiert nicht (Nutzer schließt das Fenster).
    Ignore,
}

/// Öffnet nichts, sondern beantwortet die Anmeldung wie ein Browser: ruft die Rückruf-Adresse auf.
pub struct AutoBrowser {
    handle: tokio::runtime::Handle,
    pub behavior: Mutex<Behavior>,
    pub urls: Mutex<Vec<String>>,
}

impl AutoBrowser {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        Self {
            handle,
            behavior: Mutex::new(Behavior::Approve),
            urls: Mutex::new(Vec::new()),
        }
    }

    pub fn opened(&self) -> Vec<Url> {
        self.urls
            .lock()
            .unwrap()
            .iter()
            .map(|url| Url::parse(url).unwrap())
            .collect()
    }
}

impl UrlOpener for AutoBrowser {
    fn open(&self, url: &str) -> Result<(), OpenError> {
        self.urls.lock().unwrap().push(url.to_owned());
        let behavior = *self.behavior.lock().unwrap();
        let parsed = Url::parse(url).map_err(|e| OpenError(e.to_string()))?;
        let param = |name: &str| {
            parsed
                .query_pairs()
                .find(|(key, _)| key == name)
                .map(|(_, v)| v.into_owned())
                .unwrap_or_default()
        };
        let redirect = param("redirect_uri");
        let state = param("state");
        let target = match behavior {
            Behavior::Approve => format!("{redirect}?code=auth-code&state={state}"),
            Behavior::Deny => format!("{redirect}?error=access_denied&state={state}"),
            Behavior::Ignore => return Ok(()),
        };
        self.handle.spawn(async move {
            let _ = reqwest::get(target).await;
        });
        Ok(())
    }
}
