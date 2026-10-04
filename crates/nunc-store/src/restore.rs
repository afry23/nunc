//! Restore aus einer Backup-Datei (Go: `app.go RestoreFromBackup`, `ValidateNuncBackupFile`).
//!
//! Ablauf: Datei prüfen → Notfall-Backup des aktuellen Standes → Kopie neben der Datenbank vorbereiten und
//! migrieren (so werden auch ältere Go-Backups auf den aktuellen Stand gebracht) → Verbindung schließen →
//! per `rename` austauschen → neu öffnen. Bis zum Austausch bleibt die Originaldatenbank unberührt; scheitert
//! der Austausch, wird sie wieder geöffnet.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use rusqlite::{Connection, OpenFlags};

use crate::{MIGRATIONS, StoreError, create_emergency_backup, open};

const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";

fn invalid(reason: impl Into<String>) -> StoreError {
    StoreError::InvalidBackup(reason.into())
}

/// Prüft, dass `path` eine nunc-Datenbank ist: vorhanden, SQLite-Kopfzeile, Integritätsprüfung, Basis-Migration
/// angewendet und keine Schema-Version, die diese App nicht kennt.
pub fn validate_backup_file(path: &Path) -> Result<(), StoreError> {
    let mut file =
        fs::File::open(path).map_err(|e| invalid(format!("Datei nicht lesbar ({e})")))?;
    let mut header = [0_u8; 16];
    file.read_exact(&mut header)
        .map_err(|_| invalid("keine SQLite-Datenbank"))?;
    if &header != SQLITE_MAGIC {
        return Err(invalid("keine SQLite-Datenbank"));
    }
    drop(file);

    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| invalid("Datenbank nicht lesbar"))?;
    let quick: String = conn
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|_| invalid("Datenbank beschädigt"))?;
    if quick != "ok" {
        return Err(invalid("Datenbank beschädigt"));
    }
    let versions: Vec<i64> = conn
        .prepare("SELECT version FROM schema_migrations")
        .and_then(|mut stmt| stmt.query_map([], |row| row.get(0))?.collect())
        .map_err(|_| invalid("keine nunc-Datenbank (Schema-Tabelle nicht gefunden)"))?;
    if !versions.contains(&1) {
        return Err(invalid("keine nunc-Datenbank (Basis-Migration fehlt)"));
    }
    let known = MIGRATIONS.last().map_or(0, |m| i64::from(m.version));
    if let Some(newest) = versions.iter().copied().max().filter(|v| *v > known) {
        return Err(invalid(format!(
            "Datenbank stammt aus einer neueren Version (Schema {newest}, bekannt bis {known})"
        )));
    }
    Ok(())
}

fn sidecar(db_path: &Path, suffix: &str) -> PathBuf {
    let mut name = db_path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Ersetzt die Datenbank unter `db_path` durch `source`. `conn` ist die offene Verbindung auf `db_path` und
/// zeigt danach auf die wiederhergestellte Datenbank. Das Notfall-Backup landet in `backup_dir`.
///
/// Bei jedem Fehler vor dem Austausch bleibt die Originaldatenbank unverändert und `conn` nutzbar.
pub fn restore_backup(
    conn: &mut Connection,
    db_path: &Path,
    source: &Path,
    backup_dir: &Path,
    today: NaiveDate,
) -> Result<(), StoreError> {
    validate_backup_file(source)?;
    create_emergency_backup(conn, backup_dir, today)?;

    // Kopie neben der Datenbank (gleiches Dateisystem für das atomare `rename`), migriert und sauber geschlossen
    let staged = sidecar(db_path, ".restore");
    let prepare = || -> Result<(), StoreError> {
        fs::copy(source, &staged)?;
        drop(open(&staged)?);
        Ok(())
    };
    if let Err(error) = prepare() {
        discard(&staged);
        return Err(error);
    }

    // Datei freigeben (schließt WAL), dann tauschen
    drop(std::mem::replace(conn, Connection::open_in_memory()?));
    discard(&sidecar(db_path, "-wal"));
    discard(&sidecar(db_path, "-shm"));
    if let Err(error) = fs::rename(&staged, db_path) {
        discard(&staged);
        *conn = open(db_path)?;
        return Err(error.into());
    }
    *conn = open(db_path)?;
    Ok(())
}

fn discard(path: &Path) {
    let _ = fs::remove_file(path);
}
