//! Zugriffsschutz, Routing und Antworten – alles pur, die Todos kommen über [`TodoBackend`].

use nunc_core::model::Todo;
use nunc_core::todo_query::{SortDirection, TodoFilter, TodoSort, query_todos};
use nunc_core::todotxt_file::export_todotxt;
use serde_json::{Value, json};

use crate::http::{Method, Request, Response};
use crate::query::parse_query;

/// Größe des Anfragekörpers, ab der die Anfrage abgelehnt wird (wie beim Datei-Import).
pub const MAX_BODY_BYTES: usize = 5 * 1024 * 1024;

/// Version der Schnittstelle in `GET /v1/health`.
const API_VERSION: u32 = 1;

/// Ergebnis von [`TodoBackend::import`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportOutcome {
    pub added: Vec<Todo>,
    pub duplicates: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApiError {
    #[error("Todo {0} nicht gefunden")]
    NotFound(i32),
    /// Unerwarteter Fehler; die Einzelheiten gehören ins Protokoll der App, nicht in die Antwort.
    #[error("{0}")]
    Internal(String),
}

/// Was die Schnittstelle von der App braucht. In der App ein Adapter über `AppState`, in Tests ein Fake.
pub trait TodoBackend: Send + Sync {
    /// Alle aktiven Todos in manueller Reihenfolge.
    fn list(&self) -> Result<Vec<Todo>, ApiError>;
    /// Legt die neuen Zeilen eines todo.txt-Textes an (siehe `nunc_core::todotxt_file::plan_import`).
    fn import(&self, text: &str) -> Result<ImportOutcome, ApiError>;
    fn set_completed(&self, id: i32, completed: bool) -> Result<Todo, ApiError>;
}

/// Einstellungen des laufenden Servers.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Erwartetes Token (`Authorization: Bearer <token>`).
    pub token: String,
    /// Port, auf dem der Server tatsächlich lauscht (der `Host`-Header muss ihn nennen).
    pub port: u16,
}

/// Vergleich ohne vorzeitigen Abbruch, damit die Antwortzeit nichts über das Token verrät.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut diff = a.len() ^ b.len();
    for i in 0..a.len().max(b.len()) {
        diff |= usize::from(a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0));
    }
    diff == 0
}

fn host_allowed(host: &str, port: u16) -> bool {
    let host = host.to_ascii_lowercase();
    host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
}

fn token_matches(header: Option<&str>, token: &str) -> bool {
    let Some((scheme, value)) = header.and_then(|h| h.split_once(' ')) else {
        return false;
    };
    scheme.eq_ignore_ascii_case("bearer") && constant_time_eq(value.as_bytes(), token.as_bytes())
}

fn bad_request(message: &str) -> Response {
    Response::error(400, "invalid_input", message)
}

fn not_allowed(allow: &str) -> Response {
    Response::error(
        405,
        "method_not_allowed",
        "Method not allowed for this path",
    )
    .with_header("allow", allow)
}

fn backend_error(error: ApiError) -> Response {
    match error {
        ApiError::NotFound(id) => {
            Response::error(404, "not_found", &format!("Todo {id} not found"))
        }
        ApiError::Internal(_) => Response::error(500, "internal", "Internal error"),
    }
}

/// Beantwortet eine Anfrage. Reihenfolge der Prüfungen: Host → Origin → Token → Pfad/Methode → Inhalt.
pub fn route(request: &Request, config: &ApiConfig, backend: &dyn TodoBackend) -> Response {
    // Gegen DNS-Rebinding: nur die Loopback-Namen des eigenen Ports
    if !request
        .header("host")
        .is_some_and(|host| host_allowed(host, config.port))
    {
        return Response::error(403, "forbidden_host", "Host not allowed");
    }
    // Gegen Aufrufe aus dem Browser: Programme senden keinen Origin-Header
    if request.header("origin").is_some() {
        return Response::error(403, "forbidden_origin", "Browser requests are not allowed");
    }
    if !token_matches(request.header("authorization"), &config.token) {
        return Response::error(401, "unauthorized", "Missing or wrong token")
            .with_header("www-authenticate", "Bearer");
    }
    if request.body.len() > MAX_BODY_BYTES {
        return Response::error(413, "payload_too_large", "Request body too large");
    }

    let segments: Vec<&str> = request.path.trim_start_matches('/').split('/').collect();
    match segments.as_slice() {
        ["v1", "health"] => match request.method {
            Method::Get => Response::json(
                200,
                &json!({ "app": "nunc", "version": env!("CARGO_PKG_VERSION"), "apiVersion": API_VERSION }),
            ),
            _ => not_allowed("GET"),
        },
        ["v1", "todos"] => match request.method {
            Method::Get => list_todos(request, backend),
            Method::Post => post_todos(request, backend),
            _ => not_allowed("GET, POST"),
        },
        ["v1", "todos", id] => match request.method {
            Method::Get => with_id(id, |id| get_todo(id, backend)),
            Method::Patch => with_id(id, |id| patch_todo(id, request, backend)),
            _ => not_allowed("GET, PATCH"),
        },
        _ => Response::error(404, "not_found", "Unknown path"),
    }
}

fn with_id(text: &str, run: impl FnOnce(i32) -> Response) -> Response {
    match text.parse::<i32>() {
        Ok(id) => run(id),
        Err(_) => bad_request("The task id must be a number"),
    }
}

fn filter_from_query(query: &str) -> Result<TodoFilter, Response> {
    let mut filter = TodoFilter::default();
    for (name, value) in parse_query(query) {
        match name.as_str() {
            "completed" => {
                filter.completed = Some(match value.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(bad_request("completed must be true or false")),
                });
            }
            "project" => filter.projects.push(prefixed(&value, '+')),
            "context" => filter.contexts.push(prefixed(&value, '@')),
            "priority" => {
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(letter @ 'A'..='Z'), None) => filter.priorities.push(letter),
                    _ => return Err(bad_request("priority must be one letter from A to Z")),
                }
            }
            "q" => filter.text = value,
            _ => {}
        }
    }
    Ok(filter)
}

/// `work` → `+work`; steht das Präfix schon da, bleibt der Wert.
fn prefixed(value: &str, prefix: char) -> String {
    if value.starts_with(prefix) {
        value.to_owned()
    } else {
        format!("{prefix}{value}")
    }
}

fn wants_json(request: &Request) -> bool {
    request
        .header("accept")
        .is_some_and(|accept| accept.to_ascii_lowercase().contains("application/json"))
}

fn list_todos(request: &Request, backend: &dyn TodoBackend) -> Response {
    let filter = match filter_from_query(&request.query) {
        Ok(filter) => filter,
        Err(response) => return response,
    };
    let todos = match backend.list() {
        Ok(todos) => query_todos(&todos, &filter, TodoSort::Manual, SortDirection::Ascending),
        Err(error) => return backend_error(error),
    };
    if wants_json(request) {
        Response::json(200, &json!(todos))
    } else {
        Response::text(200, export_todotxt(&todos, true))
    }
}

fn get_todo(id: i32, backend: &dyn TodoBackend) -> Response {
    match backend.list() {
        Ok(todos) => match todos.into_iter().find(|todo| todo.id == id) {
            Some(todo) => Response::json(200, &json!(todo)),
            None => backend_error(ApiError::NotFound(id)),
        },
        Err(error) => backend_error(error),
    }
}

fn is_json(request: &Request) -> bool {
    request
        .header("content-type")
        .is_some_and(|ty| ty.to_ascii_lowercase().starts_with("application/json"))
}

fn json_body(request: &Request) -> Result<Value, Response> {
    serde_json::from_slice(&request.body).map_err(|_| bad_request("The body is not valid JSON"))
}

fn post_todos(request: &Request, backend: &dyn TodoBackend) -> Response {
    let text = if is_json(request) {
        match json_body(request) {
            Ok(value) => match value.get("rawText").and_then(Value::as_str) {
                Some(text) => text.to_owned(),
                None => return bad_request("rawText must be a string"),
            },
            Err(response) => return response,
        }
    } else {
        match String::from_utf8(request.body.clone()) {
            Ok(text) => text,
            Err(_) => return bad_request("The body is not UTF-8 text"),
        }
    };
    match backend.import(&text) {
        Ok(outcome) => {
            let status = if outcome.added.is_empty() { 200 } else { 201 };
            Response::json(
                status,
                &json!({
                    "added": outcome.added.len(),
                    "duplicates": outcome.duplicates,
                    "todos": outcome.added,
                }),
            )
        }
        Err(error) => backend_error(error),
    }
}

fn patch_todo(id: i32, request: &Request, backend: &dyn TodoBackend) -> Response {
    let value = match json_body(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(completed) = value.get("completed").and_then(Value::as_bool) else {
        return bad_request("completed must be true or false");
    };
    match backend.set_completed(id, completed) {
        Ok(todo) => Response::json(200, &json!(todo)),
        Err(error) => backend_error(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Empty;

    impl TodoBackend for Empty {
        fn list(&self) -> Result<Vec<Todo>, ApiError> {
            Ok(Vec::new())
        }
        fn import(&self, _text: &str) -> Result<ImportOutcome, ApiError> {
            Ok(ImportOutcome::default())
        }
        fn set_completed(&self, id: i32, _completed: bool) -> Result<Todo, ApiError> {
            Err(ApiError::NotFound(id))
        }
    }

    fn post(body_len: usize) -> Response {
        let request = Request {
            method: Method::Post,
            path: "/v1/todos".to_owned(),
            query: String::new(),
            headers: vec![
                ("host".to_owned(), "127.0.0.1:1".to_owned()),
                ("authorization".to_owned(), "Bearer t".to_owned()),
            ],
            body: vec![b'a'; body_len],
        };
        route(
            &request,
            &ApiConfig {
                token: "t".to_owned(),
                port: 1,
            },
            &Empty,
        )
    }

    #[test]
    fn the_body_limit_is_five_mebibytes_and_the_limit_itself_is_allowed() {
        assert_eq!(MAX_BODY_BYTES, 5_242_880);
        assert_eq!(post(MAX_BODY_BYTES).status, 200);
        assert_eq!(post(MAX_BODY_BYTES + 1).status, 413);
    }

    #[test]
    fn constant_time_eq_compares_content_and_length() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(!constant_time_eq(b"ab", b"abc"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn prefixes_are_added_once() {
        assert_eq!(prefixed("work", '+'), "+work");
        assert_eq!(prefixed("+work", '+'), "+work");
        assert_eq!(prefixed("@desk", '@'), "@desk");
    }
}
