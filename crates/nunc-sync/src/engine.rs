//! Ein Sync-Lauf gegen WebDAV: Server-Stand laden, mit dem lokalen zusammenführen, Änderungen einspielen,
//! Ergebnis hochladen (ADR 0008, 0016).

use nunc_core::clock::Clock;
use nunc_core::crdt::SyncState;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use nunc_core::model::{SyncFile, SyncProvider, SyncResult, WebDavConfig};
use nunc_core::sync_document::{SCHEMA_VERSION, SyncDocument};
use nunc_platform::SecretStore;
use nunc_store::{Applied, StoreError};

use crate::SyncError;
use crate::config::{SECRET_ACCOUNT, SECRET_SERVICE};
use crate::coordinator::{RunFuture, SyncRunner};
use crate::webdav::{Download, Precondition, WebDavClient};
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
/// ist, und den zusammengeführten Stand; `result` zählt, was in der Datenbank ankam.
///
/// Bei einem neuen Anlauf sind die Änderungen schon eingespielt und zählen nicht doppelt.
pub(crate) fn merge_remote(
    store: &dyn SyncStore,
    clock: &dyn Clock,
    remote: Option<&SyncState>,
    result: &mut SyncResult,
) -> Result<(SyncDocument, SyncState), SyncError> {
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
    Ok((SyncDocument::from_state(&merged, now), merged))
}

/// Was ein Lauf vom Server weiß, um den nächsten abzukürzen (ADR 0030): Adresse und ETag der Sync-Datei sowie
/// ein Fingerabdruck des lokalen Stands, mit dem beide übereinstimmten.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Known {
    location: String,
    etag: String,
    fingerprint: u64,
}

/// Gedächtnis zwischen zwei Läufen (nur im Speicher). Leer = der nächste Lauf lädt die Datei ganz.
#[derive(Debug, Default)]
pub struct SyncCache {
    known: Mutex<Option<Known>>,
}

impl SyncCache {
    fn get(&self) -> Option<Known> {
        self.known.lock().ok().and_then(|known| known.clone())
    }

    fn set(&self, value: Option<Known>) {
        if let Ok(mut known) = self.known.lock() {
            *known = value;
        }
    }
}

/// Fingerabdruck des lokalen Stands, so wie er in die Datei käme (ohne abgelaufene Tombstones).
fn fingerprint(store: &dyn SyncStore, clock: &dyn Clock) -> Result<u64, SyncError> {
    let state = store
        .local_state()?
        .collect_garbage(clock.now().timestamp_millis());
    let json =
        serde_json::to_vec(&state).map_err(|error| SyncError::InvalidFile(error.to_string()))?;
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    Ok(hasher.finish())
}

/// Führt einen Sync-Lauf aus und liefert, was vom Server übernommen wurde. Kürzt ab: Hat sich seit dem letzten Lauf weder lokal noch auf dem Server etwas geändert
/// (ETag unverändert, `304`), wird nichts übertragen. Ohne Änderung wird auch nicht hochgeladen.
///
/// Sicherheit: Die Abkürzung gilt nur bei **unverändertem lokalem Stand**. Sonst (auch nach einem Restore) lädt der
/// Lauf die Datei ganz und führt zusammen, damit nie ein älterer lokaler Stand die Datei überschreibt.
pub async fn sync_webdav_cached(
    client: &WebDavClient,
    store: &dyn SyncStore,
    clock: &dyn Clock,
    cache: &SyncCache,
) -> Result<SyncResult, SyncError> {
    client.ensure_folder().await?;
    let location = client.file_location();
    let mut result = SyncResult::default();
    for _ in 0..MAX_ATTEMPTS {
        let known_etag = match cache.get() {
            Some(known)
                if known.location == location
                    && known.fingerprint == fingerprint(store, clock)? =>
            {
                Some(known.etag)
            }
            _ => None,
        };
        let remote = match client.download_if_changed(known_etag.as_deref()).await? {
            Download::NotModified => return Ok(result),
            Download::Missing => None,
            Download::Changed(remote) => Some(remote),
        };
        let legacy = match &remote {
            Some(_) => None,
            None => client.download_legacy().await?,
        };
        let state = remote_state(remote.as_ref().map(|remote| &remote.file), legacy.as_ref())?;
        let (document, merged) = merge_remote(store, clock, state.as_ref(), &mut result)?;
        // Nichts zu schreiben: Die Datei hat schon dieses Format und genau diesen Stand
        let up_to_date = remote.as_ref().is_some_and(|remote| {
            remote.file.schema_version == SCHEMA_VERSION && state.as_ref() == Some(&merged)
        });
        let etag = if up_to_date {
            remote.as_ref().and_then(|remote| remote.etag.clone())
        } else {
            let precondition = match &remote {
                Some(remote) => remote
                    .etag
                    .clone()
                    .map_or(Precondition::None, Precondition::Match),
                None => Precondition::NoneExists,
            };
            match client.upload(&document, &precondition).await {
                Ok(etag) => etag,
                Err(SyncError::Conflict) => continue,
                Err(error) => return Err(error),
            }
        };
        cache.set(match etag {
            Some(etag) => Some(Known {
                location,
                etag,
                fingerprint: fingerprint(store, clock)?,
            }),
            None => None,
        });
        return Ok(result);
    }
    Err(SyncError::Conflict)
}

/// Der Sync-Lauf gegen WebDAV für den [`SyncCoordinator`](crate::SyncCoordinator): prüft, ob überhaupt
/// WebDAV gewählt ist, holt das Passwort aus dem Tresor und führt [`sync_webdav_cached`] aus.
pub struct WebDavRunner {
    store: Arc<dyn SyncStore>,
    secrets: Arc<dyn SecretStore>,
    clock: Arc<dyn Clock>,
    cache: Arc<SyncCache>,
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
            cache: Arc::default(),
        }
    }
}

impl SyncRunner for WebDavRunner {
    fn run(&self) -> RunFuture {
        let store = self.store.clone();
        let secrets = self.secrets.clone();
        let clock = self.clock.clone();
        let cache = self.cache.clone();
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
            let result = sync_webdav_cached(&client, &*store, &*clock, &cache).await?;
            let finished = clock.now();
            store.record_sync(SyncProvider::Webdav, finished)?;
            Ok(Some((
                result,
                finished.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )))
        })
    }
}
