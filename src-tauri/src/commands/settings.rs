use nunc_core::model::Settings;
use tauri::{AppHandle, Runtime, State};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, AppError> {
    state.db.with(|conn| nunc_store::load_settings(conn))
}

/// Speichert die Einstellungen nach Prüfung der Wertebereiche. `lastBackupDate` gehört dem Backup-Dienst,
/// `syncProvider` `set_sync_provider` und `backupDir` dem Ordner-Dialog (`select_backup_dir`): sie bleiben wie
/// gespeichert; ein leeres `backupDir`
/// setzt den Standardordner wieder ein.
#[tauri::command]
#[specta::specta]
pub fn save_settings<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    mut settings: Settings,
) -> Result<(), AppError> {
    settings.validate()?;
    state.db.with(|conn| {
        let stored = nunc_store::load_settings(conn)?;
        settings.last_backup_date = stored.last_backup_date;
        settings.sync_provider = stored.sync_provider;
        if !settings.backup_dir.is_empty() {
            settings.backup_dir = stored.backup_dir;
        }
        nunc_store::save_settings(conn, &settings)
    })?;
    state.apply_sync_interval(&settings);
    #[cfg(desktop)]
    {
        use tauri::Manager;
        crate::tray::set_visible(&app, settings.system_tray_enabled);
        // Befehle ohne Laufzeitkontext (Hauptthread): hier darf blockiert werden, bis der Server umgestellt ist
        if let Some(api) = app.try_state::<crate::api_service::ApiService>() {
            tauri::async_runtime::block_on(api.apply(settings.api_enabled, settings.api_port));
        }
    }
    #[cfg(mobile)]
    let _ = app;
    Ok(())
}
