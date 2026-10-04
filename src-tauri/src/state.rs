//! Zustand der App (`manage()`) und Start.
//!
//! Start (siehe [`AppState::open`]): Legacy-Datenbank der Go-App übernehmen → Datenbank öffnen und migrieren →
//! Gerätekennung → Timer laden. Danach vom Aufrufer: [`TimerService::recover`] und das Auto-Backup im
//! Hintergrund ([`AppState::startup_backup`]).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::NaiveDate;
use nunc_core::clock::Clock;
use nunc_platform::{EventSink, Notifier, SecretStore, to_local_date};
use nunc_store::{
    BackupResult, LegacyImport, backup_if_needed, device_id, import_legacy_db, open,
    resolve_backup_dir, restore_backup,
};

use crate::db::Db;
use crate::error::AppError;
use crate::events::{self, AppReload};
use crate::timer_service::TimerService;

/// Dateien und Ordner der App.
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// `nunc.db` im App-Datenordner.
    pub db: PathBuf,
    /// Backup-Ordner, solange keiner eingestellt ist.
    pub default_backup_dir: PathBuf,
    /// Datenbank der Go-App (`<Benutzer-Konfigurationsordner>/nunc/nunc.db`); `None` mobil.
    pub legacy_db: Option<PathBuf>,
}

/// Plattformabhängige Bausteine (echt in der App, Fakes in Tests).
pub struct Platform {
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
    pub notifier: Arc<dyn Notifier>,
    pub secrets: Arc<dyn SecretStore>,
}

pub struct AppState {
    pub db: Arc<Db>,
    pub paths: AppPaths,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
    pub timer: Arc<TimerService>,
    /// Tresor für Tokens und Passwörter (Sync, ab M6).
    #[expect(dead_code, reason = "wird mit dem WebDAV-Sync in M6 gelesen")]
    pub secrets: Arc<dyn SecretStore>,
}

impl AppState {
    pub fn open(paths: AppPaths, platform: Platform) -> Result<Self, AppError> {
        if let Some(legacy) = &paths.legacy_db {
            match import_legacy_db(legacy, &paths.db)? {
                LegacyImport::Imported => log::info!("Datenbank der Go-App übernommen"),
                LegacyImport::AlreadyPresent | LegacyImport::NoLegacyDatabase => {}
            }
        }
        let db = Arc::new(Db::new(open(&paths.db)?));
        let device_id = db.with(|conn| device_id(conn, || uuid::Uuid::new_v4().to_string()))?;
        let timer = Arc::new(TimerService::new(
            db.clone(),
            platform.clock.clone(),
            platform.events.clone(),
            platform.notifier,
            device_id,
        )?);
        Ok(Self {
            db,
            paths,
            clock: platform.clock,
            events: platform.events,
            timer,
            secrets: platform.secrets,
        })
    }

    /// Heutiger Kalendertag in der Zeitzone des Geräts.
    pub fn today(&self) -> NaiveDate {
        to_local_date(self.clock.now())
    }

    /// Eingestellter oder Standard-Backup-Ordner.
    pub fn backup_dir(&self) -> Result<PathBuf, AppError> {
        self.db
            .with(|conn| resolve_backup_dir(conn, &self.paths.default_backup_dir))
    }

    /// Tägliches Backup beim Start (nur wenn heute noch keines lief). Läuft im Hintergrund; ein Fehler darf
    /// den Start nicht verhindern und wird nur protokolliert.
    pub fn startup_backup(&self) -> Result<Option<BackupResult>, AppError> {
        let dir = self.backup_dir()?;
        let today = self.today();
        self.db.with(|conn| backup_if_needed(conn, &dir, today))
    }

    /// Ersetzt die Datenbank durch das Backup `source` (Notfall-Backup, Validierung, atomarer Austausch,
    /// Rollback bei Fehler – siehe `nunc-store`), lädt den Timer neu und meldet `app:reload`.
    pub fn restore(&self, source: &Path) -> Result<(), AppError> {
        let dir = self.backup_dir()?;
        let today = self.today();
        self.timer.exclusive(|| {
            self.db
                .with(|conn| restore_backup(conn, &self.paths.db, source, &dir, today))
        })?;
        events::emit(&*self.events, &AppReload);
        Ok(())
    }
}
