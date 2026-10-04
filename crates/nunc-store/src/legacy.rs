//! Übernahme der Datenbank der Go-App beim ersten Start (ADR 0001, M2-07).
//!
//! Die Go-App legt `nunc.db` unter `<Benutzer-Konfigurationsordner>/nunc/` ab, Tauri leitet seinen Datenordner
//! vom Identifier ab (`com.afry.nunc`). Beim ersten Start wird die alte Datenbank **kopiert**; das Original
//! (samt `-wal`) wird nur gelesen und bleibt unverändert. Der Backup-Ordner (`backup_dir`) und alle übrigen
//! Einstellungen stehen in der Datenbank und kommen damit automatisch mit. Alte Backups im Standardordner der
//! Go-App werden nicht verschoben. Secrets der Go-App (Keychain) werden nicht übernommen (ADR 0005).

use std::fs;
use std::path::{Path, PathBuf};

use crate::{StoreError, open, validate_backup_file};

/// Was beim Übernehmen passiert ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyImport {
    /// Die Datenbank wurde kopiert.
    Imported,
    /// Am Zielpfad gibt es schon eine Datenbank; nichts getan.
    AlreadyPresent,
    /// Die Go-App hat keine Datenbank hinterlassen; nichts getan.
    NoLegacyDatabase,
}

/// Pfad der Datenbank der Go-App unter dem Benutzer-Konfigurationsordner
/// (macOS `~/Library/Application Support`, Linux `~/.config`, Windows `%AppData%`).
pub fn legacy_db_path(user_config_dir: &Path) -> PathBuf {
    user_config_dir.join("nunc").join("nunc.db")
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Kopiert `legacy` nach `target`, wenn dort noch keine Datenbank liegt. Die Kopie wird (inklusive noch nicht
/// zurückgeschriebener WAL-Daten) neben dem Ziel vorbereitet, geprüft, migriert und erst dann atomar an den Zielpfad
/// gelegt; bei einem Fehler bleibt `target` unberührt. Ist die alte Datei keine nunc-Datenbank, liefert die
/// Funktion einen Fehler und die App startet mit einer neuen.
pub fn import_legacy_db(legacy: &Path, target: &Path) -> Result<LegacyImport, StoreError> {
    if target.exists() {
        return Ok(LegacyImport::AlreadyPresent);
    }
    if !legacy.is_file() {
        return Ok(LegacyImport::NoLegacyDatabase);
    }
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir)?;
    }
    let staged = sibling(target, ".import");
    let staged_wal = sibling(&staged, "-wal");
    let staged_shm = sibling(&staged, "-shm");
    let cleanup = || {
        for path in [&staged, &staged_wal, &staged_shm] {
            let _ = fs::remove_file(path);
        }
    };

    let result = (|| -> Result<(), StoreError> {
        fs::copy(legacy, &staged)?;
        let legacy_wal = sibling(legacy, "-wal");
        if legacy_wal.is_file() {
            fs::copy(&legacy_wal, &staged_wal)?;
        }
        // Prüfen und öffnen nur an der Kopie: schon das Öffnen einer WAL-Datenbank legt `-shm` an
        validate_backup_file(&staged)?;
        // öffnen: WAL einspielen, migrieren; Schließen schreibt alles in die Hauptdatei zurück
        drop(open(&staged)?);
        let _ = fs::remove_file(&staged_wal);
        let _ = fs::remove_file(&staged_shm);
        fs::rename(&staged, target)?;
        Ok(())
    })();
    if let Err(error) = result {
        cleanup();
        return Err(error);
    }
    Ok(LegacyImport::Imported)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn fixture() -> PathBuf {
        PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/go-v5.db"
        ))
    }

    #[test]
    fn legacy_path_is_below_the_nunc_folder() {
        assert_eq!(
            legacy_db_path(Path::new("/home/u/.config")),
            Path::new("/home/u/.config/nunc/nunc.db")
        );
    }

    #[test]
    fn an_existing_target_wins_and_a_missing_legacy_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("new/nunc.db");
        assert_eq!(
            import_legacy_db(&dir.path().join("none.db"), &target).unwrap(),
            LegacyImport::NoLegacyDatabase
        );
        assert!(!target.exists());
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, b"mine").unwrap();
        assert_eq!(
            import_legacy_db(&fixture(), &target).unwrap(),
            LegacyImport::AlreadyPresent
        );
        assert_eq!(fs::read(&target).unwrap(), b"mine");
    }

    #[test]
    fn changes_still_in_the_write_ahead_log_are_imported() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("old/nunc.db");
        let conn = open(&legacy).unwrap();
        conn.execute_batch("PRAGMA wal_autocheckpoint = 0").unwrap();
        crate::add_todo(&conn, "only in the wal", "s1", chrono::Utc::now()).unwrap();
        assert!(sibling(&legacy, "-wal").metadata().unwrap().len() > 0);

        // Zustand wie nach einem Absturz: Hauptdatei + WAL, ohne dass die Verbindung sauber geschlossen wurde
        let snapshot = dir.path().join("snapshot/nunc.db");
        fs::create_dir_all(snapshot.parent().unwrap()).unwrap();
        fs::copy(&legacy, &snapshot).unwrap();
        fs::copy(sibling(&legacy, "-wal"), sibling(&snapshot, "-wal")).unwrap();
        drop(conn);

        let target = dir.path().join("new/nunc.db");
        assert_eq!(
            import_legacy_db(&snapshot, &target).unwrap(),
            LegacyImport::Imported
        );
        let imported = Connection::open(&target).unwrap();
        let todos = crate::list_todos(&imported).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].raw_text, "only in the wal");
    }

    #[test]
    fn an_invalid_legacy_file_fails_and_leaves_nothing_behind() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("nunc.db");
        fs::write(&legacy, b"not sqlite").unwrap();
        let target = dir.path().join("new/nunc.db");
        assert!(matches!(
            import_legacy_db(&legacy, &target),
            Err(StoreError::InvalidBackup(_))
        ));
        assert!(!target.exists());
        assert_eq!(fs::read(&legacy).unwrap(), b"not sqlite");
    }

    #[test]
    fn no_staging_files_remain_after_success() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("nunc.db");
        import_legacy_db(&fixture(), &target).unwrap();
        let mut names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        assert_eq!(names, ["nunc.db"]);
    }
}
