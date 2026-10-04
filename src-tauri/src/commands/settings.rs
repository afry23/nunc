use nunc_core::model::Settings;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, AppError> {
    state.db.with(|conn| nunc_store::load_settings(conn))
}

/// Speichert die Einstellungen nach Prüfung der Wertebereiche. `lastBackupDate` gehört dem Backup-Dienst und
/// `backupDir` dem Ordner-Dialog (`select_backup_dir`): beide bleiben wie gespeichert; ein leeres `backupDir`
/// setzt den Standardordner wieder ein.
#[tauri::command]
#[specta::specta]
pub fn save_settings(state: State<'_, AppState>, mut settings: Settings) -> Result<(), AppError> {
    settings.validate()?;
    state.db.with(|conn| {
        let stored = nunc_store::load_settings(conn)?;
        settings.last_backup_date = stored.last_backup_date;
        if !settings.backup_dir.is_empty() {
            settings.backup_dir = stored.backup_dir;
        }
        nunc_store::save_settings(conn, &settings)
    })
}
