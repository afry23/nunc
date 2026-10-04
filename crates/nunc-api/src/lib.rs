//! Lokale HTTP-Schnittstelle von nunc (ADR 0013): eine andere App auf demselben Rechner tauscht todo.txt-Zeilen
//! aus und fragt den Status der Aufgaben ab.
//!
//! - [`route`] ist **pur**: Zugriffsschutz (Host, Origin, Token), Routing, Formate und Statuscodes. Die Todos
//!   kommen über das Trait [`TodoBackend`].
//! - [`server`] ist der dünne Rand: ein Tokio-Server auf `127.0.0.1`, der Anfragen an [`route`] weitergibt.

mod http;
mod query;
mod router;
pub mod server;
pub mod token;

pub use http::{Method, Request, Response};
pub use router::{ApiConfig, ApiError, ImportOutcome, MAX_BODY_BYTES, TodoBackend, route};
