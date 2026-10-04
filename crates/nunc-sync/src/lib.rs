//! Sync-Clients: WebDAV (M6), später Google Drive und Todoist (M7).

mod config;
mod engine;
mod error;
mod webdav;

pub use config::{
    SECRET_ACCOUNT, SECRET_SERVICE, WebDavCredentials, has_webdav_password, load_credentials,
    save_webdav,
};
pub use engine::{SyncStore, sync_webdav};
pub use error::SyncError;
pub use webdav::{Precondition, Remote, SYNC_FILE_NAME, WebDavClient};
