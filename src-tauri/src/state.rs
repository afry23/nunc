//! Zustand der App (`manage()`) und Start.
//!
//! Start (siehe [`AppState::open`]): Legacy-Datenbank der Go-App übernehmen → Datenbank öffnen und migrieren →
//! Gerätekennung → Timer laden. Danach vom Aufrufer: [`TimerService::recover`] und das Auto-Backup im
//! Hintergrund ([`AppState::startup_backup`]).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use nunc_core::clock::Clock;
use nunc_core::model::Settings;
use nunc_platform::{EventSink, Notifier, SecretStore, to_local_date};
use nunc_store::{
    BackupResult, LegacyImport, backup_if_needed, device_id, import_legacy_db, open,
    resolve_backup_dir, restore_backup,
};
use nunc_sync::{SyncCoordinator, Trigger};

use crate::db::Db;
use crate::error::AppError;
use crate::events::{self, AppReload};
use crate::sync_service;
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

/// `NUNC_TEST_FAST_TIMER=<Sekunden>`: feste Pomodoro-Dauer für die Oberflächentests (nur Debug-Builds).
fn fast_timer_from_env() -> Option<u32> {
    std::env::var("NUNC_TEST_FAST_TIMER")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .filter(|seconds| *seconds > 0)
}

/// Plattformabhängige Bausteine (echt in der App, Fakes in Tests).
pub struct Platform {
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
    pub notifier: Arc<dyn Notifier>,
    pub secrets: Arc<dyn SecretStore>,
    /// Tokio-Laufzeit für Hintergrundaufgaben (Sync).
    pub runtime: tokio::runtime::Handle,
}

type ShutdownHook = Box<dyn Fn() + Send>;

pub struct AppState {
    pub db: Arc<Db>,
    pub paths: AppPaths,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
    pub timer: Arc<TimerService>,
    /// Tresor für Tokens und Passwörter (Sync).
    pub secrets: Arc<dyn SecretStore>,
    pub sync: Arc<SyncCoordinator>,
    pub runtime: tokio::runtime::Handle,
    shutdown_hooks: Mutex<Vec<ShutdownHook>>,
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
        let sync = sync_service::coordinator(
            db.clone(),
            platform.secrets.clone(),
            platform.clock.clone(),
            platform.events.clone(),
            platform.runtime.clone(),
        )?;
        let session_sync = sync.clone();
        let timer = Arc::new(
            TimerService::new(
                db.clone(),
                platform.clock.clone(),
                platform.events.clone(),
                platform.notifier,
                device_id,
            )?
            .with_session_hook(Arc::new(move || {
                session_sync.trigger(Trigger::SessionFinished)
            }))
            .with_fast_pomodoro(fast_timer_from_env()),
        );
        Ok(Self {
            db,
            paths,
            clock: platform.clock,
            events: platform.events,
            timer,
            secrets: platform.secrets,
            sync,
            runtime: platform.runtime,
            shutdown_hooks: Mutex::new(Vec::new()),
        })
    }

    /// Ein Todo wurde angelegt, geändert, abgehakt oder gelöscht: der Sync-Anbieter gleicht ab.
    pub fn todos_changed(&self) {
        self.sync.trigger(Trigger::TodoChanged);
    }

    /// Eine Session wurde nachgetragen, bearbeitet oder gelöscht: der Sync-Anbieter gleicht ab.
    pub fn sessions_changed(&self) {
        self.sync.trigger(Trigger::TodoChanged);
    }

    /// Stellt den automatischen Sync nach den Einstellungen ein (am Desktop; mobil wirkungslos).
    pub fn apply_sync_interval(&self, settings: &Settings) {
        self.sync.set_interval(settings.sync_interval());
    }

    /// Meldet `hook` für das Beenden der App an (z. B. der Sync, der beim Schließen noch einmal läuft, M6-07).
    /// Die Hooks laufen in der Reihenfolge der Anmeldung.
    pub fn on_shutdown(&self, hook: impl Fn() + Send + 'static) {
        if let Ok(mut hooks) = self.shutdown_hooks.lock() {
            hooks.push(Box::new(hook));
        }
    }

    /// Führt die angemeldeten Hooks aus (`RunEvent::Exit`). Der Timer bleibt unangetastet: Seine geplante
    /// Benachrichtigung muss das Beenden überleben.
    pub fn shutdown(&self) {
        if let Ok(hooks) = self.shutdown_hooks.lock() {
            hooks.iter().for_each(|hook| hook());
        }
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

/// Laufzeit für Tests, die einen [`AppState`] bauen. Niemand treibt sie an: gestartete Hintergrundaufgaben
/// (Sync-Auslöser) laufen dort nicht, was die Tests deterministisch hält.
#[cfg(test)]
pub fn test_runtime() -> tokio::runtime::Handle {
    use std::sync::LazyLock;
    static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("Test-Laufzeit")
    });
    RUNTIME.handle().clone()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use chrono::{TimeZone, Utc};
    use nunc_core::model::TimerMode;
    use nunc_platform::{ManualClock, MemorySecretStore, RecordingEventSink, RecordingNotifier};
    use nunc_store::{add_todo, list_todos, run_backup};
    use tempfile::TempDir;

    use super::*;

    struct Fixture {
        dir: TempDir,
        clock: Arc<ManualClock>,
        events: Arc<RecordingEventSink>,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                dir: tempfile::tempdir().unwrap(),
                clock: Arc::new(ManualClock::new(
                    Utc.with_ymd_and_hms(2026, 10, 4, 9, 0, 0).unwrap(),
                )),
                events: Arc::new(RecordingEventSink::default()),
            }
        }

        fn paths(&self) -> AppPaths {
            AppPaths {
                db: self.dir.path().join("app").join("nunc.db"),
                default_backup_dir: self.dir.path().join("app").join("backups"),
                legacy_db: Some(self.dir.path().join("legacy").join("nunc").join("nunc.db")),
            }
        }

        fn open(&self) -> AppState {
            AppState::open(
                self.paths(),
                Platform {
                    clock: self.clock.clone(),
                    events: self.events.clone(),
                    notifier: Arc::new(RecordingNotifier::new(false)),
                    secrets: Arc::new(MemorySecretStore::default()),
                    runtime: test_runtime(),
                },
            )
            .unwrap()
        }
    }

    fn todo_texts(state: &AppState) -> Vec<String> {
        state
            .db
            .with(|conn| list_todos(conn))
            .unwrap()
            .into_iter()
            .map(|todo| todo.raw_text)
            .collect()
    }

    #[test]
    fn a_fresh_start_creates_the_database_and_keeps_the_device_id() {
        let fixture = Fixture::new();
        let first = fixture.open();
        assert!(fixture.paths().db.is_file());
        let id = first
            .db
            .with(|conn| device_id(conn, || "other".into()))
            .unwrap();
        assert!(!id.is_empty());
        drop(first);

        let second = fixture.open();
        let again = second
            .db
            .with(|conn| device_id(conn, || "other".into()))
            .unwrap();
        assert_eq!(again, id);
    }

    #[test]
    fn the_first_start_takes_over_the_go_database_and_leaves_the_original() {
        let fixture = Fixture::new();
        let legacy = fixture.paths().legacy_db.unwrap();
        {
            let conn = open(&legacy).unwrap();
            add_todo(&conn, "(A) aus der Go-App", "s-1", fixture.clock.now()).unwrap();
        }
        let state = fixture.open();
        assert_eq!(todo_texts(&state), ["(A) aus der Go-App"]);
        assert!(legacy.is_file());

        // beim zweiten Start gewinnt die neue Datenbank
        state
            .db
            .with(|conn| add_todo(conn, "neu", "s-2", fixture.clock.now()))
            .unwrap();
        drop(state);
        assert_eq!(todo_texts(&fixture.open()).len(), 2);
    }

    #[test]
    fn the_startup_backup_runs_once_per_day() {
        let fixture = Fixture::new();
        let state = fixture.open();
        let first = state.startup_backup().unwrap().expect("Backup erstellt");
        assert!(first.path.starts_with(&fixture.paths().default_backup_dir));
        assert!(first.path.is_file());
        assert!(state.startup_backup().unwrap().is_none());

        fixture.clock.advance(chrono::Duration::days(1));
        assert!(state.startup_backup().unwrap().is_some());
    }

    #[test]
    fn restore_swaps_the_database_reloads_the_timer_and_announces_it() {
        let fixture = Fixture::new();
        let state = fixture.open();
        state
            .db
            .with(|conn| add_todo(conn, "vor dem Backup", "s-1", fixture.clock.now()))
            .unwrap();
        let backup = state
            .db
            .with(|conn| run_backup(conn, &fixture.paths().default_backup_dir, state.today()))
            .unwrap();

        state
            .db
            .with(|conn| add_todo(conn, "nach dem Backup", "s-2", fixture.clock.now()))
            .unwrap();
        state.timer.start(TimerMode::Pomodoro, None).unwrap();
        let before = fixture.events.names().len();

        state.restore(&backup.path).unwrap();

        assert_eq!(todo_texts(&state), ["vor dem Backup"]);
        // das Backup enthielt keinen laufenden Timer
        assert_eq!(
            state.timer.snapshot().unwrap().state,
            nunc_core::model::TimerState::Idle
        );
        assert_eq!(fixture.events.names()[before], "app:reload");
    }

    #[test]
    fn a_broken_backup_leaves_everything_untouched() {
        let fixture = Fixture::new();
        let state = fixture.open();
        state
            .db
            .with(|conn| add_todo(conn, "bleibt", "s-1", fixture.clock.now()))
            .unwrap();
        let broken = fixture.dir.path().join("kaputt.db");
        std::fs::write(&broken, b"das ist keine Datenbank").unwrap();

        let error = state.restore(&broken).unwrap_err();
        assert_eq!(error.code, "invalid_backup");
        assert_eq!(todo_texts(&state), ["bleibt"]);
        assert!(!fixture.events.names().contains(&"app:reload".to_owned()));
    }

    #[test]
    fn shutdown_runs_the_registered_hooks_in_order() {
        let fixture = Fixture::new();
        let state = fixture.open();
        let calls = Arc::new(AtomicUsize::new(0));
        let first = calls.clone();
        state.on_shutdown(move || {
            assert_eq!(first.fetch_add(1, Ordering::SeqCst), 0);
        });
        let second = calls.clone();
        state.on_shutdown(move || {
            assert_eq!(second.fetch_add(1, Ordering::SeqCst), 1);
        });
        state.shutdown();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
