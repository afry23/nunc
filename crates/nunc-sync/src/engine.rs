//! Ein Sync-Lauf gegen WebDAV: Server-Stand laden, mit dem lokalen zusammenführen, Änderungen einspielen,
//! Ergebnis hochladen (ADR 0008, 0016).

use nunc_core::clock::Clock;
use nunc_core::crdt::SyncState;
use std::sync::Arc;

use nunc_core::model::{SyncFile, SyncProvider, SyncResult, WebDavConfig};
use nunc_core::sync_document::SyncDocument;
use nunc_platform::SecretStore;
use nunc_store::{Applied, StoreError};

use crate::SyncError;
use crate::config::{SECRET_ACCOUNT, SECRET_SERVICE};
use crate::coordinator::{RunFuture, SyncRunner};
use crate::webdav::{Precondition, WebDavClient};
use chrono::{DateTime, Utc};

/// Wie oft ein Lauf neu ansetzt, wenn jemand gleichzeitig die Datei auf dem Server ändert.
pub(crate) const MAX_ATTEMPTS: usize = 3;

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
    /// Lokaler Stand als CRDT-Zustand (vergibt fehlende `sync_id`s).
    fn local_state(&self) -> Result<SyncState, StoreError>;
    /// Gewählter Anbieter und WebDAV-Einstellungen.
    fn setup(&self) -> Result<SyncSetup, StoreError>;
    /// Merkt sich den Zeitpunkt des letzten erfolgreichen Syncs mit dem Anbieter.
    fn record_sync(&self, provider: SyncProvider, at: DateTime<Utc>) -> Result<(), StoreError>;
    /// Schreibt den zusammengeführten Stand atomar in die Datenbank.
    fn apply_state(&self, merged: &SyncState) -> Result<Applied, StoreError>;
}

/// Der Stand vom Server: die v2-Datei oder, wenn es nur die alte `nunc-sync.json` gibt, deren Inhalt (einmaliger
/// Import, die alte Datei bleibt unangetastet).
pub(crate) fn remote_state(
    current: Option<&SyncDocument>,
    legacy: Option<&SyncFile>,
) -> Result<Option<SyncState>, SyncError> {
    match (current, legacy) {
        (Some(document), _) => document
            .clone()
            .into_state()
            .map(Some)
            .map_err(|error| SyncError::InvalidFile(error.to_string())),
        (None, Some(legacy)) => Ok(Some(SyncState::from_legacy(legacy))),
        (None, None) => Ok(None),
    }
}

/// Führt den Stand vom Server mit dem lokalen zusammen und spielt ihn ein. Liefert die Datei, die hochzuladen
/// ist; `result` zählt, was in der Datenbank ankam.
///
/// Bei einem neuen Anlauf sind die Änderungen schon eingespielt und zählen nicht doppelt.
pub(crate) fn merge_remote(
    store: &dyn SyncStore,
    clock: &dyn Clock,
    remote: Option<&SyncState>,
    result: &mut SyncResult,
) -> Result<SyncDocument, SyncError> {
    let local = store.local_state()?;
    let now = clock.now();
    let now_ms = now.timestamp_millis();
    let merged = match remote {
        Some(remote) => local.merge_remote(remote, now_ms),
        None => local,
    }
    // alte Tombstones verschwinden aus Datei und Abgleich; die Zeilen in der Datenbank bleiben (Statistik)
    .collect_garbage(now_ms);
    let applied = store.apply_state(&merged)?;
    result.imported_todos += applied.todos_added;
    result.conflicted_todos += applied.todos_replaced;
    result.imported_sessions += applied.sessions_added;
    Ok(SyncDocument::from_state(&merged, now))
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
        let legacy = match &remote {
            Some(_) => None,
            None => client.download_legacy().await?,
        };
        let state = remote_state(remote.as_ref().map(|remote| &remote.file), legacy.as_ref())?;
        let merged = merge_remote(store, clock, state.as_ref(), &mut result)?;
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
            store.record_sync(SyncProvider::Webdav, finished)?;
            Ok(Some((
                result,
                finished.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )))
        })
    }
}
