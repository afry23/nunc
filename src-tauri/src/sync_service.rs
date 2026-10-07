//! Verdrahtung der Synchronisierung: Datenbankzugriff für den Sync (`DbSyncStore`), Ereignisse an die
//! Oberfläche (`SyncEvents`) und der [`SyncCoordinator`]. Die Regeln stecken in `nunc-sync`.

use std::sync::{Arc, OnceLock, Weak};

use chrono::NaiveDate;

use chrono::{DateTime, Utc};
use nunc_core::clock::Clock;
use nunc_core::crdt::SyncState;
use nunc_core::model::{SyncProvider, SyncResult, SyncStatus};
use nunc_core::todoist_mapping::{LocalTodo, TodoistMapping};
use nunc_platform::{EventSink, SecretStore};
use nunc_store::{Applied, StoreError};
use nunc_store::{TodoistApplied, TodoistWrite};
use nunc_sync::{
    ChainRunner, DriveConfig, DriveRunner, DriveService, SyncCoordinator, SyncObserver, SyncSetup,
    SyncStore, Timing, TodoistConfig, TodoistRunner, TodoistService, TodoistStore, Trigger,
    WebDavRunner,
};
use tokio::runtime::Handle;

use crate::credentials;
use crate::db::Db;
use crate::events::{
    self, SyncComplete, SyncStart, SyncStatusChanged, TodoistSyncComplete, TodoistSyncStatus,
};

/// Die App-Datenbank aus Sicht des Syncs. Jeder Zugriff sperrt die Verbindung nur kurz; zwischen den
/// Netzwerkzugriffen eines Laufs bleibt sie frei.
pub struct DbSyncStore {
    db: Arc<Db>,
}

impl DbSyncStore {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    fn with<T>(
        &self,
        f: impl FnOnce(&mut nunc_store::Connection) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        self.db.with_store(f)
    }
}

impl SyncStore for DbSyncStore {
    fn local_state(&self) -> Result<SyncState, StoreError> {
        self.with(|conn| nunc_store::local_state(conn, || uuid::Uuid::new_v4().to_string()))
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

    fn apply_state(&self, merged: &SyncState) -> Result<Applied, StoreError> {
        self.with(|conn| nunc_store::apply_state(conn, merged))
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

/// Was `AppState` für den Sync braucht.
pub struct SyncServices {
    pub coordinator: Arc<SyncCoordinator>,
    pub drive: Arc<DriveService>,
    pub store: Arc<DbSyncStore>,
    pub todoist: Arc<SyncCoordinator>,
    pub todoist_service: Arc<TodoistService>,
}

/// Die App-Datenbank aus Sicht des Todoist-Syncs.
pub struct DbTodoistStore {
    db: Arc<Db>,
}

impl DbTodoistStore {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

impl TodoistStore for DbTodoistStore {
    fn local_todos(&self) -> Result<Vec<LocalTodo>, StoreError> {
        self.db
            .with_store(|conn| nunc_store::list_local_todos(conn))
    }

    fn mappings(&self) -> Result<Vec<TodoistMapping>, StoreError> {
        self.db
            .with_store(|conn| nunc_store::list_todoist_mappings(conn))
    }

    fn add_mapping(
        &self,
        todo_id: i32,
        task_id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        self.db
            .with_store(|conn| nunc_store::add_todoist_mapping(conn, todo_id, task_id, at))
    }

    fn remove_mappings(&self, task_ids: &[String]) -> Result<(), StoreError> {
        self.db
            .with_store(|conn| nunc_store::remove_todoist_mappings(conn, task_ids))
    }

    fn apply(
        &self,
        writes: &[TodoistWrite],
        today: NaiveDate,
        now: DateTime<Utc>,
    ) -> Result<TodoistApplied, StoreError> {
        self.db
            .with_store(|conn| nunc_store::apply_todoist_writes(conn, writes, today, now))
    }

    fn record_sync(&self, at: DateTime<Utc>) -> Result<(), StoreError> {
        self.db
            .with_store(|conn| nunc_store::set_todoist_last_sync_at(conn, at))
    }

    fn new_sync_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// Meldet Todoist-Ereignisse an die Oberfläche. Hat ein Lauf Todos verändert, sind die auch für den
/// Sync-Anbieter neu: `primary` (gesetzt, sobald der Koordinator existiert) synchronisiert dann mit.
pub struct TodoistEvents {
    sink: Arc<dyn EventSink>,
    primary: OnceLock<Weak<SyncCoordinator>>,
}

impl TodoistEvents {
    pub fn new(sink: Arc<dyn EventSink>) -> Self {
        Self {
            sink,
            primary: OnceLock::new(),
        }
    }

    pub fn follow(&self, primary: &Arc<SyncCoordinator>) {
        let _ = self.primary.set(Arc::downgrade(primary));
    }
}

impl SyncObserver for TodoistEvents {
    fn started(&self) {}

    fn status(&self, status: &SyncStatus) {
        events::emit(&*self.sink, &TodoistSyncStatus(status.clone()));
    }

    fn completed(&self, result: &SyncResult) {
        events::emit(&*self.sink, &TodoistSyncComplete(*result));
        let changed = result.imported_todos + result.conflicted_todos > 0;
        if let (true, Some(primary)) = (changed, self.primary.get().and_then(Weak::upgrade)) {
            primary.trigger(Trigger::TodoChanged);
        }
    }
}

/// Todoist reagiert nur auf Start, Todo-Änderungen und das Verbinden – und nur am Desktop.
fn todoist_accepts(trigger: Trigger) -> bool {
    cfg!(desktop)
        && matches!(
            trigger,
            Trigger::Start | Trigger::TodoChanged | Trigger::Connected
        )
}

/// Baut die Dienste für den Sync: den Koordinator mit den Läufern für WebDAV und Google Drive und die
/// Drive-Verbindung (für Anmelden und Abmelden). `last_sync_at` kommt aus der Datenbank, damit die
/// Statusanzeige auch vor dem ersten Lauf dieser Sitzung stimmt.
pub fn coordinator(
    db: Arc<Db>,
    secrets: Arc<dyn SecretStore>,
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventSink>,
    runtime: Handle,
) -> Result<SyncServices, crate::error::AppError> {
    let last_sync_at = db.with(|conn| {
        let provider = nunc_store::load_settings(conn)?.sync_provider;
        nunc_store::last_sync_of(conn, provider)
    })?;
    let todoist_last_sync = db.with(|conn| nunc_store::todoist_last_sync_at(conn))?;
    let todoist_store = Arc::new(DbTodoistStore::new(db.clone()));
    let store = Arc::new(DbSyncStore::new(db));
    let google = credentials::google();
    let drive = Arc::new(DriveService::new(
        DriveConfig::google(&google.client_id, &google.client_secret),
        secrets.clone(),
        clock.clone(),
    )?);
    let runner = Arc::new(ChainRunner(vec![
        Arc::new(WebDavRunner::new(
            store.clone(),
            secrets.clone(),
            clock.clone(),
        )),
        Arc::new(DriveRunner::new(
            store.clone(),
            drive.clone(),
            clock.clone(),
        )),
    ]));
    let coordinator = SyncCoordinator::new(
        runner,
        Arc::new(SyncEvents(events.clone())),
        runtime.clone(),
        cfg!(mobile),
        Timing::default(),
        last_sync_at,
    );

    let todoist_credentials = credentials::todoist();
    let todoist_service = Arc::new(TodoistService::new(
        TodoistConfig::todoist(
            &todoist_credentials.client_id,
            &todoist_credentials.client_secret,
        ),
        secrets,
        clock.clone(),
    )?);
    let todoist_events = Arc::new(TodoistEvents::new(events));
    todoist_events.follow(&coordinator);
    let todoist = SyncCoordinator::filtered(
        Arc::new(TodoistRunner::new(
            todoist_store,
            todoist_service.clone(),
            clock,
        )),
        todoist_events,
        runtime,
        cfg!(mobile),
        Timing::default(),
        todoist_last_sync,
        todoist_accepts,
    );
    Ok(SyncServices {
        coordinator,
        drive,
        store,
        todoist,
        todoist_service,
    })
}
