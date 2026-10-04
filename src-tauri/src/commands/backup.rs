use std::path::PathBuf;

use nunc_core::backup_policy::{BackupEntry, BackupStatus};
use nunc_store::BackupResult;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::AppError;
use crate::state::AppState;

/// Ergebnis einer manuellen Sicherung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub path: String,
    pub size_bytes: u64,
    /// Durch die Rotation entfernte Backups.
    pub deleted: Vec<String>,
    /// Die Sicherung gelang, das Aufräumen alter Backups nicht.
    pub rotation_error: Option<String>,
}

impl From<BackupResult> for BackupInfo {
    fn from(result: BackupResult) -> Self {
        Self {
            path: result.path.to_string_lossy().into_owned(),
            size_bytes: result.size_bytes,
            deleted: result.deleted,
            rotation_error: result.rotation_error,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_backup_status(state: State<'_, AppState>) -> Result<BackupStatus, AppError> {
    let dir = state.backup_dir()?;
    Ok(nunc_store::backup_dir_status(&dir, state.clock.now())?)
}

/// Sichert jetzt (auch wenn heute schon gesichert wurde) und rotiert alte Backups.
#[tauri::command]
#[specta::specta]
pub fn trigger_backup(state: State<'_, AppState>) -> Result<BackupInfo, AppError> {
    let dir = state.backup_dir()?;
    let today = state.today();
    let result = state
        .db
        .with(|conn| nunc_store::run_backup(conn, &dir, today))?;
    Ok(result.into())
}

/// Vorhandene Backups, neueste zuerst.
#[tauri::command]
#[specta::specta]
pub fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupEntry>, AppError> {
    let dir = state.backup_dir()?;
    Ok(nunc_store::list_backup_entries(&dir, state.today())?)
}

/// Stellt die Datenbank aus einer Backup-Datei wieder her und sendet danach `app:reload`.
#[tauri::command]
#[specta::specta]
pub fn restore_from_backup(
    state: State<'_, AppState>,
    source_path: String,
) -> Result<(), AppError> {
    state.restore(&PathBuf::from(source_path))
}

/// Öffnet den Ordner-Dialog und speichert den gewählten Backup-Ordner (nur Desktop). `None` bei Abbruch.
#[tauri::command]
#[specta::specta]
pub async fn select_backup_dir<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<Option<String>, AppError> {
    #[cfg(desktop)]
    {
        let Some(folder) = app
            .dialog()
            .file()
            .set_title("Backup-Ordner wählen")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let dir = folder
            .into_path()
            .map_err(|error| AppError::validation(error.to_string()))?;
        state
            .db
            .with(|conn| nunc_store::set_backup_dir(conn, &dir))?;
        Ok(Some(dir.to_string_lossy().into_owned()))
    }
    #[cfg(mobile)]
    {
        let _ = (app, state);
        Err(AppError::unsupported(
            "Der Backup-Ordner ist auf Mobilgeräten fest vorgegeben.",
        ))
    }
}

/// Öffnet den Datei-Dialog für eine Backup-Datei (`.db`). `None` bei Abbruch.
#[tauri::command]
#[specta::specta]
pub async fn select_restore_file<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<String>, AppError> {
    let Some(file) = app
        .dialog()
        .file()
        .set_title("Backup-Datei wählen")
        .add_filter("Datenbank", &["db"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|_| {
        AppError::unsupported("Dieser Dateiort wird erst mit dem Mobil-Import (M6-03) unterstützt.")
    })?;
    Ok(Some(path.to_string_lossy().into_owned()))
}
