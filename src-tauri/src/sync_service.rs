//! Verdrahtung der Synchronisierung: Datenbankzugriff für den Sync (`DbSyncStore`), Ereignisse an die
//! Oberfläche (`SyncEvents`) und der [`SyncCoordinator`]. Die Regeln stecken in `nunc-sync`.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use nunc_core::clock::Clock;
use nunc_core::model::{SyncFile, SyncProvider, SyncResult, SyncSession, SyncStatus, SyncTodo};
use nunc_platform::{EventSink, SecretStore};
use nunc_store::{Applied, StoreError};
use nunc_sync::{SyncCoordinator, SyncObserver, SyncSetup, SyncStore, Timing, WebDavRunner};
use tokio::runtime::Handle;

use crate::db::Db;
use crate::events::{self, SyncComplete, SyncStart, SyncStatusChanged};

/// Die App-Datenbank aus Sicht des Syncs. Jeder Zugriff sperrt die Verbindung nur kurz; zwischen den
/// Netzwerkzugriffen eines Laufs bleibt sie frei.
pub struct DbSyncStore {
    db: Arc<Db>,
    device_id: String,
}

impl DbSyncStore {
    pub fn new(db: Arc<Db>, device_id: String) -> Self {
        Self { db, device_id }
    }

    fn with<T>(
        &self,
        f: impl FnOnce(&mut nunc_store::Connection) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        self.db.with_store(f)
    }
}

impl SyncStore for DbSyncStore {
    fn local_file(&self, now: DateTime<Utc>) -> Result<SyncFile, StoreError> {
        self.with(|conn| {
            nunc_store::build_sync_file(conn, &self.device_id, now, || {
                uuid::Uuid::new_v4().to_string()
            })
        })
    }

    fn setup(&self) -> Result<SyncSetup, StoreError> {
        self.with(|conn| {
            let settings = nunc_store::load_settings(conn)?;
            Ok(SyncSetup {
                provider: settings.sync_provider,
                enabled: settings.sync_enabled,
                webdav: nunc_store::load_webdav_config(conn)?,
            })
        })
    }

    fn record_sync(&self, provider: SyncProvider, at: DateTime<Utc>) -> Result<(), StoreError> {
        self.with(|conn| nunc_store::set_last_sync_of(conn, provider, at))
    }

    fn record_drive_folder(&self, id: &str) -> Result<(), StoreError> {
        self.with(|conn| nunc_store::set_drive_folder_id(conn, id))
    }

    fn apply_remote(
        &self,
        todos: &[SyncTodo],
        sessions: &[SyncSession],
    ) -> Result<Applied, StoreError> {
        self.with(|conn| nunc_store::apply_remote(conn, todos, sessions))
    }
}

/// Meldet Sync-Ereignisse an die Oberfläche (`sync:start`, `sync:status`, `sync:complete`).
pub struct SyncEvents(pub Arc<dyn EventSink>);

impl SyncObserver for SyncEvents {
    fn started(&self) {
        events::emit(&*self.0, &SyncStart);
    }

    fn status(&self, status: &SyncStatus) {
        events::emit(&*self.0, &SyncStatusChanged(status.clone()));
    }

    fn completed(&self, result: &SyncResult) {
        events::emit(&*self.0, &SyncComplete(*result));
    }
}

/// Baut den Koordinator für WebDAV. `last_sync_at` kommt aus der Datenbank, damit die Statusanzeige auch vor
/// dem ersten Lauf dieser Sitzung stimmt.
pub fn coordinator(
    db: Arc<Db>,
    device_id: String,
    secrets: Arc<dyn SecretStore>,
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventSink>,
    runtime: Handle,
) -> Result<Arc<SyncCoordinator>, crate::error::AppError> {
    let last_sync_at = db.with(|conn| nunc_store::last_sync_at(conn))?;
    let store = Arc::new(DbSyncStore::new(db, device_id));
    let runner = Arc::new(WebDavRunner::new(store, secrets, clock));
    Ok(SyncCoordinator::new(
        runner,
        Arc::new(SyncEvents(events)),
        runtime,
        cfg!(mobile),
        Timing::default(),
        last_sync_at,
    ))
}
