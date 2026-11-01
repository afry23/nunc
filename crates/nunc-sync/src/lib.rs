//! Sync-Client: WebDAV (ADR 0008).

mod config;
mod coordinator;
mod engine;
mod error;
mod http;
pub(crate) mod webdav;

pub use config::{
    SECRET_ACCOUNT, SECRET_SERVICE, WebDavCredentials, has_webdav_password, load_credentials,
    save_webdav,
};
pub use coordinator::{
    RunFuture, RunOutcome, SyncCoordinator, SyncObserver, SyncRunner, Timing, Trigger,
};
pub use engine::{SyncSetup, SyncStore, WebDavRunner, sync_webdav};
pub use error::SyncError;
pub use http::http_client;
pub use webdav::{Precondition, Remote, SYNC_FILE_NAME, WebDavClient};
