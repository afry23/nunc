//! Minimale, von hyper unabhängige Anfrage- und Antwortform (macht [`route`](crate::route) rein testbar).

use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Patch,
    Other(String),
}

impl Method {
    pub fn parse(text: &str) -> Self {
        match text {
            "GET" => Self::Get,
            "POST" => Self::Post,
            "PATCH" => Self::Patch,
            other => Self::Other(other.to_owned()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    /// Pfad ohne Query (`/v1/todos`).
    pub path: String,
    /// Rohe Query ohne `?` (`completed=false&project=work`).
    pub query: String,
    /// Kopfzeilen mit kleingeschriebenem Namen.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: u16, content_type: &str, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: vec![("content-type".to_owned(), content_type.to_owned())],
            body,
        }
    }

    pub fn json(status: u16, value: &serde_json::Value) -> Self {
        Self::new(status, "application/json", value.to_string().into_bytes())
    }

    pub fn text(status: u16, body: String) -> Self {
        Self::new(status, "text/plain; charset=utf-8", body.into_bytes())
    }

    /// Fehlerantwort `{ code, message }` wie bei den App-Commands.
    pub fn error(status: u16, code: &str, message: &str) -> Self {
        Self::json(status, &json!({ "code": code, "message": message }))
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}
