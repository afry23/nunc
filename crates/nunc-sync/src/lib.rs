//! Sync-Clients: WebDAV (M6), später Google Drive und Todoist (M7).

mod config;
mod coordinator;
mod drive;
mod engine;
mod error;
pub mod oauth;
mod todoist;
pub(crate) mod webdav;

pub use config::{
    SECRET_ACCOUNT, SECRET_SERVICE, WebDavCredentials, has_webdav_password, load_credentials,
    save_webdav,
};
pub use coordinator::{
    RunFuture, RunOutcome, SyncCoordinator, SyncObserver, SyncRunner, Timing, Trigger,
};
pub use drive::{
    CLIENT_ACCOUNT, ChainRunner, DriveClient, DriveConfig, DriveFile, DriveRunner, DriveService,
    GoogleClient, sync_drive,
};
pub use engine::{SyncSetup, SyncStore, WebDavRunner, sync_webdav};
pub use error::SyncError;
pub use todoist::{
    Account, Named, TaskPayload, TodoistClient, TodoistConfig, TodoistRunner, TodoistService,
    TodoistStore, sync_todoist,
};
pub use webdav::{Precondition, Remote, SYNC_FILE_NAME, WebDavClient};
