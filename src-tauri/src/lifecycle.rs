//! Start und Laufzeit der App: Zustand aufbauen, Timer wiederherstellen, Auto-Backup im Hintergrund,
//! Sekundentakt für die Anzeige, Rückkehr aus dem Hintergrund.

use std::sync::Arc;
use std::time::Duration;

use nunc_platform::{KeyringSecretStore, SecretStore, SystemClock, UnavailableSecretStore};
use tauri::{App, AppHandle, Manager, RunEvent, Runtime};

use crate::adapters::{TauriEventSink, TauriNotifier};
use crate::error::AppError;
use crate::state::{AppPaths, AppState, Platform};

/// Baut den [`AppState`] und startet die Hintergrundaufgaben.
pub fn setup<R: Runtime>(app: &App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    let state = AppState::open(app_paths(&handle)?, platform(&handle))?;
    app.manage(state);

    let state = app.state::<AppState>();
    if let Err(error) = state.timer.recover() {
        log::error!("Timer-Wiederherstellung fehlgeschlagen: {error}");
    }
    spawn_startup_backup(&handle);
    spawn_ticker(&handle);
    Ok(())
}

/// Rückkehr aus dem Hintergrund (Mobil): Stand aus dem Zeitstempel neu berechnen, Anzeige aktualisieren.
pub fn on_run_event<R: Runtime>(app: &AppHandle<R>, event: &RunEvent) {
    if matches!(event, RunEvent::Resumed)
        && let Some(state) = app.try_state::<AppState>()
        && let Err(error) = state.timer.recover()
    {
        log::error!("Timer nach dem Hintergrund nicht aktualisiert: {error}");
    }
}

fn app_paths<R: Runtime>(handle: &AppHandle<R>) -> Result<AppPaths, AppError> {
    let path = handle.path();
    let config_dir = path
        .app_config_dir()
        .map_err(|error| AppError::new("io", format!("Datenordner nicht ermittelbar: {error}")))?;
    // Die Go-App gibt es nur auf dem Desktop; mobil gibt es nichts zu übernehmen.
    #[cfg(desktop)]
    let legacy_db = path
        .config_dir()
        .ok()
        .map(|dir| nunc_store::legacy_db_path(&dir));
    #[cfg(mobile)]
    let legacy_db = None;
    Ok(AppPaths {
        db: config_dir.join("nunc.db"),
        default_backup_dir: config_dir.join("backups"),
        legacy_db,
    })
}

fn platform<R: Runtime>(handle: &AppHandle<R>) -> Platform {
    let secrets: Arc<dyn SecretStore> = match KeyringSecretStore::init() {
        Ok(store) => Arc::new(store),
        Err(error) => {
            // z. B. Linux ohne Secret-Service: die App läuft, nur Konten lassen sich nicht verbinden
            log::warn!("{error}");
            Arc::new(UnavailableSecretStore::new(error.to_string()))
        }
    };
    Platform {
        clock: Arc::new(SystemClock),
        events: Arc::new(TauriEventSink(handle.clone())),
        notifier: Arc::new(TauriNotifier(handle.clone())),
        secrets,
    }
}

/// Tägliches Backup, ohne den Start zu verzögern.
fn spawn_startup_backup<R: Runtime>(handle: &AppHandle<R>) {
    let handle = handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AppState>();
        match state.startup_backup() {
            Ok(Some(result)) => log::info!("Backup erstellt: {}", result.path.display()),
            Ok(None) => {}
            Err(error) => log::error!("Backup beim Start fehlgeschlagen: {error}"),
        }
    });
}

/// Sekundentakt für `timer:tick`. Nur Anzeige: die Zeit kommt immer aus dem Zeitstempel (ADR 0004).
fn spawn_ticker<R: Runtime>(handle: &AppHandle<R>) {
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let state = handle.state::<AppState>();
            if let Err(error) = state.timer.tick() {
                log::error!("Timer-Tick fehlgeschlagen: {error}");
            }
        }
    });
}
