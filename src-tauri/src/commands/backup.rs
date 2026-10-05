use std::path::{Path, PathBuf};

use nunc_core::backup_policy::{BackupEntry, BackupStatus, parse_backup_date};
use nunc_store::BackupResult;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_fs::FsExt;

use super::stats::write_file;

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

/// Stellt die Datenbank aus einer Backup-Datei wieder her und sendet danach `app:reload`. Die lokale Schnittstelle
/// folgt sofort den Einstellungen aus dem Backup.
#[tauri::command]
#[specta::specta]
pub fn restore_from_backup<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    source_path: String,
) -> Result<(), AppError> {
    state.restore(&PathBuf::from(source_path))?;
    // Der Restore ist gelungen; ein Lesefehler hier darf ihn nicht als gescheitert melden
    match state.db.with(|conn| nunc_store::load_settings(conn)) {
        Ok(settings) => state.apply_sync_interval(&settings),
        Err(error) => log::error!("Sync-Intervall nach dem Restore nicht gesetzt: {error}"),
    }
    #[cfg(desktop)]
    if let Some(api) = app.try_state::<crate::api_service::ApiService>() {
        // Der Restore ist gelungen; ein Lesefehler hier darf ihn nicht als gescheitert melden
        match state.db.with(|conn| nunc_store::load_settings(conn)) {
            // Befehle ohne Laufzeitkontext (Hauptthread): hier darf blockiert werden, wie in `save_settings`
            Ok(settings) => {
                tauri::async_runtime::block_on(api.apply(settings.api_enabled, settings.api_port));
            }
            Err(error) => log::error!("Schnittstelle nach dem Restore nicht umgestellt: {error}"),
        }
    }
    #[cfg(mobile)]
    let _ = app;
    Ok(())
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

/// Dateiendungen im Dialog „Aus Datei wiederherstellen“. Mobil gibt es keinen Filter: iOS und Android kennen
/// `.db` nicht als Dateityp und würden Dateien ausgrauen; geprüft wird ohnehin beim Wiederherstellen.
#[cfg(desktop)]
const RESTORE_EXTENSIONS: &[&str] = &["db"];

/// Unterordner im Cache-Ordner der App für importierte Backups.
const IMPORT_DIR: &str = "import";

/// Öffnet den Datei-Dialog für eine Backup-Datei. `None` bei Abbruch.
///
/// Auf Mobilgeräten liefert der Picker keinen Dateipfad (Android: `content://`-URI, iOS: Dateien außerhalb der
/// Sandbox). Die Datei wird dann in den Cache-Ordner der App kopiert und dieser Pfad zurückgegeben.
#[tauri::command]
#[specta::specta]
pub async fn select_restore_file<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<String>, AppError> {
    let dialog = app.dialog().file().set_title("Backup-Datei wählen");
    #[cfg(desktop)]
    let dialog = dialog.add_filter("Datenbank", RESTORE_EXTENSIONS);
    let Some(file) = dialog.blocking_pick_file() else {
        return Ok(None);
    };
    let path = match file {
        FilePath::Path(path) => path,
        other => {
            let name = display_name(&other);
            let bytes = app.fs().read(other)?;
            let dir = app
                .path()
                .app_cache_dir()
                .map_err(|error| AppError::new("io", format!("Cache-Ordner fehlt: {error}")))?
                .join(IMPORT_DIR);
            stage_import(&dir, &name, &bytes)?
        }
    };
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Dateiname einer Auswahl (für die Anzeige im Bestätigungsdialog); ohne Namen `import.db`.
fn display_name(file: &FilePath) -> String {
    percent_decode(&file.to_string())
        .rsplit(['/', '\\', ':'])
        .next()
        .filter(|name| !name.is_empty())
        .map_or_else(|| "import.db".to_owned(), str::to_owned)
}

/// `%XX`-Folgen in URIs auflösen (Android kodiert den Pfad im `content://`-URI); Ungültiges bleibt stehen.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let decoded = (bytes[index] == b'%')
            .then(|| bytes.get(index + 1..index + 3))
            .flatten()
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match decoded {
            Some(byte) => {
                out.push(byte);
                index += 3;
            }
            None => {
                out.push(bytes[index]);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Legt `bytes` als `name` in `dir` ab und räumt vorher frühere Importe weg (nur ein Import ist nötig).
fn stage_import(dir: &Path, name: &str, bytes: &[u8]) -> Result<PathBuf, AppError> {
    // nur der letzte Pfadteil, damit ein Name wie `../x` den Ordner nicht verlässt
    let name = Path::new(name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("import.db");
    if dir.is_dir() {
        std::fs::remove_dir_all(dir)?;
    }
    std::fs::create_dir_all(dir)?;
    let target = dir.join(name);
    std::fs::write(&target, bytes)?;
    Ok(target)
}

/// Ob `source` ein Backup im Backup-Ordner ist (nur diese Dateien dürfen exportiert werden).
fn is_backup_in(dir: &Path, source: &Path) -> bool {
    source.parent() == Some(dir)
        && source
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(parse_backup_date)
            .is_some()
}

/// Kopiert ein Backup an einen vom Nutzer gewählten Ort (Datei-Dialog bzw. „Speichern unter“ auf Mobilgeräten).
/// `None`, wenn der Dialog abgebrochen wurde.
#[tauri::command]
#[specta::specta]
pub async fn export_backup<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    source_path: String,
) -> Result<Option<String>, AppError> {
    let source = PathBuf::from(&source_path);
    if !is_backup_in(&state.backup_dir()?, &source) {
        return Err(AppError::validation(
            "Nur Backups aus dem Backup-Ordner lassen sich exportieren.",
        ));
    }
    let bytes = std::fs::read(&source)?;
    let name = source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let Some(target) = app
        .dialog()
        .file()
        .set_title("Backup exportieren")
        .set_file_name(name)
        .blocking_save_file()
    else {
        return Ok(None);
    };
    write_file(&app, target.clone(), &bytes)?;
    Ok(Some(target.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_backups_in_the_backup_folder_can_be_exported() {
        let dir = Path::new("/data/backups");
        assert!(is_backup_in(dir, &dir.join("nunc-backup-2026-10-04.db")));
        assert!(!is_backup_in(
            dir,
            &dir.join("before-restore-2026-10-04.db")
        ));
        assert!(!is_backup_in(dir, &dir.join("nunc.db")));
        assert!(!is_backup_in(
            dir,
            Path::new("/etc/nunc-backup-2026-10-04.db")
        ));
        assert!(!is_backup_in(
            dir,
            &dir.join("../backups/sub/nunc-backup-2026-10-04.db")
        ));
    }

    #[test]
    fn an_import_replaces_the_previous_one_and_stays_in_its_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("import");
        let first = stage_import(&dir, "alt.db", b"1").unwrap();
        let second = stage_import(&dir, "../../neu.db", b"22").unwrap();
        assert_eq!(second, dir.join("neu.db"));
        assert!(!first.exists());
        assert_eq!(std::fs::read(second).unwrap(), b"22");
    }

    #[test]
    fn the_name_of_a_picked_location_is_its_last_segment() {
        let url = |text: &str| FilePath::Url(text.parse().unwrap());
        assert_eq!(
            display_name(&url(
                "content://com.android.providers.downloads.documents/document/raw%3A%2Fold.db"
            )),
            "old.db"
        );
        assert_eq!(percent_decode("a%20b%zz%4"), "a b%zz%4");
        assert_eq!(
            display_name(&url("file:///private/var/Downloads/backup.db")),
            "backup.db"
        );
        assert_eq!(display_name(&url("content://host/")), "import.db");
    }

    #[cfg(desktop)]
    #[test]
    fn the_restore_dialog_offers_only_db_files() {
        assert_eq!(RESTORE_EXTENSIONS, ["db"]);
    }
}
