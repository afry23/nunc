//! Ein Sync-Lauf gegen WebDAV: Server-Stand laden, mit dem lokalen zusammenführen, Änderungen einspielen,
//! Ergebnis hochladen (Go: `WebDAVSyncService.doSync`).

use chrono::{DateTime, Utc};
use nunc_core::clock::Clock;
use std::sync::Arc;

use nunc_core::model::{SyncFile, SyncProvider, SyncResult, SyncSession, SyncTodo, WebDavConfig};
use nunc_core::sync_merge::{merge_sessions, merge_todos};
use nunc_platform::SecretStore;
use nunc_store::{Applied, StoreError};

use crate::SyncError;
use crate::config::{SECRET_ACCOUNT, SECRET_SERVICE};
use crate::coordinator::{RunFuture, SyncRunner};
use crate::webdav::{Precondition, WebDavClient};

/// Wie oft ein Lauf neu ansetzt, wenn jemand gleichzeitig die Datei auf dem Server ändert.
const MAX_ATTEMPTS: usize = 3;

/// Was der Sync von den Einstellungen wissen muss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncSetup {
    pub provider: SyncProvider,
    /// `false` = Sync pausiert (Zugangsdaten bleiben).
    pub enabled: bool,
    pub webdav: WebDavConfig,
}

/// Die lokale Datenbank aus Sicht des Syncs. Die Methoden sind synchron und halten Sperren nur kurz, nie über
/// einen Netzwerkzugriff hinweg.
pub trait SyncStore: Send + Sync {
    /// Lokaler Stand (vergibt fehlende `sync_id`s).
    fn local_file(&self, now: DateTime<Utc>) -> Result<SyncFile, StoreError>;
    /// Gewählter Anbieter und WebDAV-Einstellungen.
    fn setup(&self) -> Result<SyncSetup, StoreError>;
    /// Merkt sich den Zeitpunkt des letzten erfolgreichen Syncs.
    fn record_sync(&self, at: DateTime<Utc>) -> Result<(), StoreError>;
    /// Spielt Änderungen vom Server atomar ein.
    fn apply_remote(
        &self,
        todos: &[SyncTodo],
        sessions: &[SyncSession],
    ) -> Result<Applied, StoreError>;
}

/// Führt einen Sync-Lauf aus. Liefert, was vom Server übernommen wurde.
pub async fn sync_webdav(
    client: &WebDavClient,
    store: &dyn SyncStore,
    clock: &dyn Clock,
) -> Result<SyncResult, SyncError> {
    client.ensure_folder().await?;
    let mut result = SyncResult::default();
    for _ in 0..MAX_ATTEMPTS {
        let remote = client.download().await?;
        let local = store.local_file(clock.now())?;
        let (remote_todos, remote_sessions) = remote
            .as_ref()
            .map(|remote| {
                (
                    remote.file.todos.as_slice(),
                    remote.file.sessions.as_slice(),
                )
            })
            .unwrap_or_default();

        let todos = merge_todos(&local.todos, remote_todos);
        let sessions = merge_sessions(&local.sessions, remote_sessions);
        let changed: Vec<SyncTodo> = todos
            .added_by_remote
            .iter()
            .chain(&todos.conflicted_by_remote)
            .cloned()
            .collect();
        // Gezählt wird, was wirklich in der Datenbank ankam: Der Server kennt auch Sessions, die lokal vorhanden,
        // aber nicht exportierbar sind (abgebrochene), und bereits gelöschte Todos legt der Sync nicht an.
        // Bei einem neuen Anlauf sind die Änderungen schon eingespielt und zählen nicht doppelt.
        let applied = store.apply_remote(&changed, &sessions.new_from_remote)?;
        result.imported_todos += applied.todos_added;
        result.conflicted_todos += applied.todos_replaced;
        result.imported_sessions += applied.sessions_added;

        let merged = SyncFile {
            schema_version: local.schema_version,
            synced_at: clock.now(),
            device_id: local.device_id,
            todos: todos.merged,
            sessions: sessions.all,
        };
        let precondition = match &remote {
            Some(remote) => remote
                .etag
                .clone()
                .map_or(Precondition::None, Precondition::Match),
            None => Precondition::NoneExists,
        };
        match client.upload(&merged, &precondition).await {
            Ok(()) => return Ok(result),
            Err(SyncError::Conflict) => {}
            Err(error) => return Err(error),
        }
    }
    Err(SyncError::Conflict)
}

/// Der Sync-Lauf gegen WebDAV für den [`SyncCoordinator`](crate::SyncCoordinator): prüft, ob überhaupt
/// WebDAV gewählt ist, holt das Passwort aus dem Tresor und führt [`sync_webdav`] aus.
pub struct WebDavRunner {
    store: Arc<dyn SyncStore>,
    secrets: Arc<dyn SecretStore>,
    clock: Arc<dyn Clock>,
}

impl WebDavRunner {
    pub fn new(
        store: Arc<dyn SyncStore>,
        secrets: Arc<dyn SecretStore>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            store,
            secrets,
            clock,
        }
    }
}

impl SyncRunner for WebDavRunner {
    fn run(&self) -> RunFuture {
        let store = self.store.clone();
        let secrets = self.secrets.clone();
        let clock = self.clock.clone();
        Box::pin(async move {
            let setup = store.setup()?;
            if setup.provider != SyncProvider::Webdav || !setup.enabled {
                return Ok(None);
            }
            let password = secrets.get(SECRET_SERVICE, SECRET_ACCOUNT)?;
            let Some(password) = password.filter(|_| setup.webdav.is_complete()) else {
                return Err(SyncError::NotConfigured);
            };
            let client = WebDavClient::new(setup.webdav, password)?;
            let result = sync_webdav(&client, &*store, &*clock).await?;
            let finished = clock.now();
            store.record_sync(finished)?;
            Ok(Some((
                result,
                finished.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )))
        })
    }
}
