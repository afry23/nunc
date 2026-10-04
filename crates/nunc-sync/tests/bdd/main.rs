//! BDD-Runner für die `@sync`-Szenarien von `nunc-sync`: WebDAV-Client und Sync-Lauf gegen einen kleinen
//! Server mit Zustand (wiremock, siehe `dav.rs`) und echte SQLite-Datenbanken.
//!
//! Aufruf: `cargo test -p nunc-sync --test bdd` · Szenarien mit `@wip` werden übersprungen.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, TimeZone, Utc};
use cucumber::World;
use nunc_core::model::{ConnectionTestResult, SyncFile, SyncProvider, SyncResult, WebDavConfig};
use nunc_platform::{ManualClock, MemorySecretStore};
use nunc_store::{Connection, StoreError};
use nunc_sync::{SyncError, SyncStore};
use tempfile::TempDir;

mod coord;
mod dav;
mod google;
mod google_steps;
mod steps;

pub use dav::Dav;

/// Die Datenbank für den Sync: Verbindung hinter einem Mutex wie in der App.
pub struct TestStore {
    pub conn: Mutex<Connection>,
    pub device_id: String,
    counter: Mutex<u32>,
}

impl TestStore {
    pub fn new(conn: Connection) -> Self {
        let device_id = nunc_store::device_id(&conn, || "device-local".to_owned()).unwrap();
        Self {
            conn: Mutex::new(conn),
            device_id,
            counter: Mutex::new(0),
        }
    }

    pub fn next_id(&self, prefix: &str) -> String {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        format!("{prefix}-{counter}")
    }
}

impl SyncStore for TestStore {
    fn local_file(&self, now: DateTime<Utc>) -> Result<SyncFile, StoreError> {
        let conn = self.conn.lock().unwrap();
        nunc_store::build_sync_file(&conn, &self.device_id, now, || self.next_id("gen"))
    }

    fn setup(&self) -> Result<nunc_sync::SyncSetup, StoreError> {
        let conn = self.conn.lock().unwrap();
        let settings = nunc_store::load_settings(&conn)?;
        Ok(nunc_sync::SyncSetup {
            provider: settings.sync_provider,
            enabled: settings.sync_enabled,
            webdav: nunc_store::load_webdav_config(&conn)?,
        })
    }

    fn record_sync(&self, provider: SyncProvider, at: DateTime<Utc>) -> Result<(), StoreError> {
        nunc_store::set_last_sync_of(&self.conn.lock().unwrap(), provider, at)
    }

    fn record_drive_folder(&self, id: &str) -> Result<(), StoreError> {
        nunc_store::set_drive_folder_id(&self.conn.lock().unwrap(), id)
    }

    fn apply_remote(
        &self,
        todos: &[nunc_core::model::SyncTodo],
        sessions: &[nunc_core::model::SyncSession],
    ) -> Result<nunc_store::Applied, StoreError> {
        nunc_store::apply_remote(&mut self.conn.lock().unwrap(), todos, sessions)
    }
}

#[derive(World)]
#[world(init = Self::new)]
pub struct SyncWorld {
    pub dir: TempDir,
    pub store: Arc<TestStore>,
    pub secrets: MemorySecretStore,
    pub clock: Arc<ManualClock>,
    pub dav: Option<Dav>,
    /// Adresse eines Servers, der nicht läuft.
    pub dead_url: Option<String>,
    pub remote_path: String,
    /// Sync-Datei auf dem Server vor dem Lauf (für „unverändert“).
    pub server_file_before: Option<Vec<u8>>,
    pub outcome: Option<Result<SyncResult, SyncError>>,
    pub test_result: Option<ConnectionTestResult>,
    pub config: Option<WebDavConfig>,
    pub coord: coord::State,
    pub google: google_steps::State,
}

impl std::fmt::Debug for SyncWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SyncWorld")
    }
}

impl SyncWorld {
    fn new() -> Result<Self, std::convert::Infallible> {
        Ok(Self {
            dir: tempfile::tempdir().expect("Temp-Ordner"),
            store: Arc::new(TestStore::new(nunc_store::open_in_memory().unwrap())),
            secrets: MemorySecretStore::default(),
            clock: Arc::new(ManualClock::new(
                Utc.with_ymd_and_hms(2026, 10, 4, 9, 0, 0).unwrap(),
            )),
            dav: None,
            dead_url: None,
            remote_path: "/nunc/".to_owned(),
            server_file_before: None,
            outcome: None,
            test_result: None,
            config: None,
            coord: coord::State::default(),
            google: google_steps::State::default(),
        })
    }

    pub fn dav(&self) -> &Dav {
        self.dav.as_ref().expect("kein WebDAV-Server gestartet")
    }

    pub fn fixture(name: &str) -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures")).join(name)
    }
}

const FEATURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../features");

/// Nur `@sync`-Szenarien (`@integration`), `@wip` wird übersprungen.
fn selected(
    feature: &cucumber::gherkin::Feature,
    _rule: Option<&cucumber::gherkin::Rule>,
    scenario: &cucumber::gherkin::Scenario,
) -> bool {
    let has = |tag: &str| feature.tags.iter().chain(&scenario.tags).any(|t| t == tag);
    has("integration") && has("sync") && !has("wip")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    SyncWorld::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit(FEATURES, selected)
        .await;
}
