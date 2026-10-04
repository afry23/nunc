//! Ein Sync-Lauf gegen WebDAV: Server-Stand laden, mit dem lokalen zusammenführen, Änderungen einspielen,
//! Ergebnis hochladen (Go: `WebDAVSyncService.doSync`).

use chrono::{DateTime, Utc};
use nunc_core::clock::Clock;
use nunc_core::model::{SyncFile, SyncResult, SyncSession, SyncTodo};
use nunc_core::sync_merge::{merge_sessions, merge_todos};
use nunc_store::{Applied, StoreError};

use crate::SyncError;
use crate::webdav::{Precondition, WebDavClient};

/// Wie oft ein Lauf neu ansetzt, wenn jemand gleichzeitig die Datei auf dem Server ändert.
const MAX_ATTEMPTS: usize = 3;

/// Die lokale Datenbank aus Sicht des Syncs. Die Methoden sind synchron und halten Sperren nur kurz, nie über
/// einen Netzwerkzugriff hinweg.
pub trait SyncStore: Send + Sync {
    /// Lokaler Stand (vergibt fehlende `sync_id`s).
    fn local_file(&self, now: DateTime<Utc>) -> Result<SyncFile, StoreError>;
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
