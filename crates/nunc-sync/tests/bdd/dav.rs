//! Ein kleiner WebDAV-Server mit Zustand (wiremock) für die `@sync`-Szenarien: Ordner, eine Sync-Datei mit
//! ETag, Basic-/Digest-Anmeldung und ein Protokoll aller Anfragen.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use md5::{Digest, Md5};
use wiremock::{MockServer, Request, Respond, ResponseTemplate};

pub const FILE_NAME: &str = "nunc-sync.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Auth {
    None,
    Basic { user: String, password: String },
    Digest { user: String, password: String },
}

#[derive(Debug, Clone)]
pub struct Logged {
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub status: u16,
}

#[derive(Debug)]
pub struct DavState {
    pub folders: BTreeSet<String>,
    /// Dateien nach Pfad: Inhalt und Versionszähler (ergibt den ETag `"vN"`).
    pub files: BTreeMap<String, (Vec<u8>, u32)>,
    pub auth: Auth,
    pub etags: bool,
    /// Antwortet auf jede Anfrage mit diesem Status.
    pub fail_all: Option<u16>,
    /// Inhalte, die ein anderes Gerät vor dem nächsten Upload schreibt (nacheinander verbraucht).
    pub interfere: Vec<Vec<u8>>,
    /// Statt eines Vorrats: vor jedem Upload ändert jemand die Datei.
    pub interfere_always: bool,
    pub log: Vec<Logged>,
    pub digest_accepted: bool,
    /// PROPFIND meldet vorhandene Ordner als fehlend (ein anderes Gerät legt sie gerade an).
    pub propfind_blind: bool,
}

impl Default for DavState {
    fn default() -> Self {
        Self {
            folders: BTreeSet::new(),
            files: BTreeMap::new(),
            auth: Auth::None,
            etags: true,
            fail_all: None,
            interfere: Vec::new(),
            interfere_always: false,
            log: Vec::new(),
            digest_accepted: false,
            propfind_blind: false,
        }
    }
}

impl DavState {
    pub fn etag(&self, path: &str) -> Option<String> {
        self.files.get(path).map(|(_, v)| format!("\"v{v}\""))
    }

    pub fn put(&mut self, path: &str, body: Vec<u8>) {
        let version = self.files.get(path).map_or(1, |(_, v)| v + 1);
        self.files.insert(path.to_owned(), (body, version));
    }
}

pub struct Dav {
    pub server: MockServer,
    pub state: Arc<Mutex<DavState>>,
}

impl std::fmt::Debug for Dav {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dav")
    }
}

impl Dav {
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        let state = Arc::new(Mutex::new(DavState::default()));
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(Responder(state.clone()))
            .mount(&server)
            .await;
        Self { server, state }
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut DavState) -> T) -> T {
        f(&mut self.state.lock().unwrap())
    }
}

struct Responder(Arc<Mutex<DavState>>);

fn header(request: &Request, name: &str) -> Option<String> {
    request
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn md5_hex(text: &str) -> String {
    Md5::digest(text.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// `key="value"`-Paare eines Digest-Headers (die Testwerte enthalten keine Kommas in Anführungszeichen).
fn digest_fields(value: &str) -> BTreeMap<String, String> {
    value
        .trim_start_matches("Digest")
        .split(',')
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            Some((
                key.trim().to_owned(),
                value.trim().trim_matches('"').to_owned(),
            ))
        })
        .collect()
}

const REALM: &str = "nunc-test";
const NONCE: &str = "dcd98b7102dd2f0e8b11d0f600bfb0c093";

fn authorized(state: &mut DavState, request: &Request) -> bool {
    let sent = header(request, "authorization");
    match state.auth.clone() {
        Auth::None => true,
        Auth::Basic { user, password } => {
            let expected = format!("{user}:{password}");
            sent.as_deref()
                .and_then(|value| value.strip_prefix("Basic "))
                .is_some_and(|encoded| decode_base64(encoded) == expected)
        }
        Auth::Digest { user, password } => {
            let Some(value) = sent.filter(|v| v.starts_with("Digest ")) else {
                return false;
            };
            let f = digest_fields(&value);
            let get = |k: &str| f.get(k).cloned().unwrap_or_default();
            let uri = get("uri");
            let ha1 = md5_hex(&format!("{user}:{REALM}:{password}"));
            let ha2 = md5_hex(&format!("{}:{uri}", request.method));
            let expected = md5_hex(&format!(
                "{ha1}:{NONCE}:{}:{}:auth:{ha2}",
                get("nc"),
                get("cnonce")
            ));
            let ok = get("username") == user
                && get("nonce") == NONCE
                && get("qop") == "auth"
                && get("response") == expected;
            state.digest_accepted |= ok;
            ok
        }
    }
}

/// Minimaler Base64-Decoder für `Authorization: Basic …`.
fn decode_base64(text: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = Vec::new();
    for byte in text.bytes().filter(|b| *b != b'=') {
        let Some(value) = ALPHABET.iter().position(|a| *a == byte) else {
            continue;
        };
        bits = (bits << 6) | value as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push(((bits >> count) & 0xff) as u8);
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn parent_of(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(index) => trimmed[..=index].to_owned(),
        None => "/".to_owned(),
    }
}

struct Reply {
    status: u16,
    headers: Vec<(&'static str, String)>,
    body: Vec<u8>,
}

impl Reply {
    fn new(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    fn header(mut self, name: &'static str, value: String) -> Self {
        self.headers.push((name, value));
        self
    }
}

impl Respond for Responder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let mut state = self.0.lock().unwrap();
        let path = request.url.path().to_owned();
        let method = request.method.to_string();
        let reply = handle(&mut state, request, &method, &path);
        let headers = request
            .headers
            .iter()
            .filter_map(|(k, v)| Some((k.as_str().to_owned(), v.to_str().ok()?.to_owned())))
            .collect();
        state.log.push(Logged {
            method,
            path,
            headers,
            status: reply.status,
        });
        let mut response = ResponseTemplate::new(reply.status);
        for (name, value) in &reply.headers {
            response = response.insert_header(*name, value.as_str());
        }
        if reply.body.is_empty() {
            response
        } else {
            response.set_body_bytes(reply.body)
        }
    }
}

fn handle(state: &mut DavState, request: &Request, method: &str, path: &str) -> Reply {
    if !authorized(state, request) {
        let challenge = match state.auth {
            Auth::Digest { .. } => format!(
                "Digest realm=\"{REALM}\", qop=\"auth\", nonce=\"{NONCE}\", opaque=\"op\", algorithm=MD5"
            ),
            _ => format!("Basic realm=\"{REALM}\""),
        };
        return Reply::new(401).header("WWW-Authenticate", challenge);
    }
    if let Some(status) = state.fail_all {
        return Reply::new(status);
    }
    match method {
        "PROPFIND" => Reply::new(
            if (state.folders.contains(path) && !state.propfind_blind) || path == "/" {
                207
            } else {
                404
            },
        ),
        "MKCOL" => {
            if state.folders.contains(path) {
                Reply::new(405)
            } else if parent_of(path) != "/" && !state.folders.contains(&parent_of(path)) {
                Reply::new(409)
            } else {
                state.folders.insert(path.to_owned());
                Reply::new(201)
            }
        }
        "GET" => match state.files.get(path) {
            Some((body, _)) => {
                let mut reply = Reply::new(200);
                reply.body = body.clone();
                match state.etag(path) {
                    Some(etag) if state.etags => reply.header("ETag", etag),
                    _ => reply,
                }
            }
            None => Reply::new(404),
        },
        "PUT" => put(state, request, path),
        _ => Reply::new(501),
    }
}

fn put(state: &mut DavState, request: &Request, path: &str) -> Reply {
    if !state.folders.contains(&parent_of(path)) && parent_of(path) != "/" {
        return Reply::new(409);
    }
    // ein anderes Gerät war schneller
    let other = if state.interfere_always {
        state.interfere.first().cloned()
    } else if state.interfere.is_empty() {
        None
    } else {
        Some(state.interfere.remove(0))
    };
    if let Some(body) = other {
        state.put(path, body);
    }
    let existing = state.etag(path);
    if let Some(wanted) = header(request, "if-match")
        && existing.as_deref() != Some(wanted.as_str())
    {
        return Reply::new(412);
    }
    if header(request, "if-none-match").as_deref() == Some("*") && existing.is_some() {
        return Reply::new(412);
    }
    let created = existing.is_none();
    state.put(path, request.body.clone());
    Reply::new(if created { 201 } else { 204 })
}
