//! Tägliche Backups per `VACUUM INTO` (Go: `services/backup.go`). Die Regeln (wann, Namen, Rotation, Status,
//! Kategorien) stecken in `nunc_core::backup_policy`; hier liegen Dateizugriff und Datenbank.
//!
//! Abweichung von Go: Das Backup wird zuerst als `….db.tmp` geschrieben und dann umbenannt, damit nach einem
//! Absturz keine halbe Datei als gültiges Backup erscheint.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use nunc_core::backup_policy::{
    BackupEntry, BackupFile, BackupStatus, MAX_BACKUPS, backup_file_name, backup_status,
    emergency_backup_file_name, files_to_delete, list_backups, needs_backup,
};
use rusqlite::Connection;

use crate::settings_repo::set_setting;
use crate::{StoreError, load_settings};

/// Ergebnis eines Backups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    /// Durch die Rotation entfernte Dateien.
    pub deleted: Vec<String>,
    /// Die Rotation ist fehlgeschlagen; das Backup selbst ist gültig (wie in Go nicht fatal).
    pub rotation_error: Option<String>,
}

fn dir_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::BackupDir(error.to_string())
}

fn create_dir(dir: &Path) -> Result<(), StoreError> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir).map_err(dir_error)
}

/// Schreibt einen konsistenten Schnappschuss der Datenbank nach `dest` (atomar über eine `.tmp`-Datei,
/// eine vorhandene Datei wird ersetzt).
fn snapshot(conn: &Connection, dest: &Path) -> Result<(), StoreError> {
    let mut tmp = dest.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    // `VACUUM INTO` verweigert ein vorhandenes Ziel
    if let Err(error) = fs::remove_file(&tmp)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        return Err(dir_error(error));
    }
    let tmp_text = tmp
        .to_str()
        .ok_or_else(|| dir_error("Pfad ist kein gültiger Text"))?;
    if let Err(error) = conn.execute("VACUUM INTO ?1", [tmp_text]) {
        let _ = fs::remove_file(&tmp);
        return Err(error.into());
    }
    fs::rename(&tmp, dest).map_err(|error| {
        let _ = fs::remove_file(&tmp);
        dir_error(error)
    })
}

/// Ordnerinhalt als Dateiliste für die Regeln. Ein fehlender Ordner ist leer; Unterordner und
/// nicht lesbare Einträge werden übergangen.
pub fn scan_backup_dir(dir: &Path) -> Result<Vec<BackupFile>, StoreError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(dir_error(error)),
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if meta.is_file() {
            files.push(BackupFile {
                name,
                path: entry.path().to_string_lossy().into_owned(),
                size_bytes: meta.len(),
                modified_at: modified.into(),
            });
        }
    }
    Ok(files)
}

/// Löscht die ältesten Backups, bis höchstens `max` übrig sind. Liefert die entfernten Namen.
pub fn rotate_backups(dir: &Path, max: usize) -> Result<Vec<String>, StoreError> {
    let doomed = files_to_delete(&scan_backup_dir(dir)?, max);
    for name in &doomed {
        fs::remove_file(dir.join(name)).map_err(dir_error)?;
    }
    Ok(doomed)
}

/// Legt das Backup für `today` in `dir` an (ein Backup desselben Tages wird ersetzt), rotiert auf
/// [`MAX_BACKUPS`] und vermerkt das Datum in `last_backup_date`. Für „Jetzt sichern“.
pub fn run_backup(
    conn: &Connection,
    dir: &Path,
    today: NaiveDate,
) -> Result<BackupResult, StoreError> {
    create_dir(dir)?;
    let path = dir.join(backup_file_name(today));
    snapshot(conn, &path)?;
    let size_bytes = fs::metadata(&path).map_err(dir_error)?.len();
    set_setting(
        conn,
        "last_backup_date",
        &today.format("%Y-%m-%d").to_string(),
    )?;
    let (deleted, rotation_error) = match rotate_backups(dir, MAX_BACKUPS) {
        Ok(deleted) => (deleted, None),
        Err(error) => (Vec::new(), Some(error.to_string())),
    };
    Ok(BackupResult {
        path,
        size_bytes,
        deleted,
        rotation_error,
    })
}

/// Backup beim Start: nur wenn `last_backup_date` nicht heute ist. `None` = nichts zu tun.
pub fn backup_if_needed(
    conn: &Connection,
    dir: &Path,
    today: NaiveDate,
) -> Result<Option<BackupResult>, StoreError> {
    if !needs_backup(&load_settings(conn)?.last_backup_date, today) {
        return Ok(None);
    }
    run_backup(conn, dir, today).map(Some)
}

/// Sicherung des aktuellen Standes vor einem Restore (`before-restore-YYYY-MM-DD.db`, wird am selben Tag
/// überschrieben, zählt nicht für die Rotation).
pub fn create_emergency_backup(
    conn: &Connection,
    dir: &Path,
    today: NaiveDate,
) -> Result<PathBuf, StoreError> {
    create_dir(dir)?;
    let path = dir.join(emergency_backup_file_name(today));
    snapshot(conn, &path)?;
    Ok(path)
}

/// Zustand des Backup-Ordners zum Zeitpunkt `now`.
pub fn backup_dir_status(dir: &Path, now: DateTime<Utc>) -> Result<BackupStatus, StoreError> {
    Ok(backup_status(&scan_backup_dir(dir)?, now))
}

/// Backups für die Liste, neueste zuerst.
pub fn list_backup_entries(dir: &Path, today: NaiveDate) -> Result<Vec<BackupEntry>, StoreError> {
    Ok(list_backups(&scan_backup_dir(dir)?, today))
}

/// Der eingestellte Backup-Ordner, sonst `default_dir` (vom Betriebssystem abgeleitet, wählt die App).
pub fn resolve_backup_dir(conn: &Connection, default_dir: &Path) -> Result<PathBuf, StoreError> {
    let configured = load_settings(conn)?.backup_dir;
    Ok(if configured.is_empty() {
        default_dir.to_path_buf()
    } else {
        PathBuf::from(configured)
    })
}

/// Übernimmt einen neuen Backup-Ordner nur, wenn er angelegt und beschrieben werden kann; sonst bleibt der
/// bisherige Pfad aktiv.
pub fn set_backup_dir(conn: &Connection, dir: &Path) -> Result<(), StoreError> {
    create_dir(dir)?;
    let probe = dir.join(".nunc-write-test");
    fs::write(&probe, b"").map_err(dir_error)?;
    fs::remove_file(&probe).map_err(dir_error)?;
    let text = dir
        .to_str()
        .ok_or_else(|| dir_error("Pfad ist kein gültiger Text"))?;
    set_setting(conn, "backup_dir", text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_folder_is_empty_but_other_errors_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            scan_backup_dir(&dir.path().join("none"))
                .unwrap()
                .is_empty()
        );
        let file = dir.path().join("file");
        fs::write(&file, b"x").unwrap();
        assert!(matches!(
            scan_backup_dir(&file),
            Err(StoreError::BackupDir(_))
        ));
    }
}
