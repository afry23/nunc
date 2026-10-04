use std::path::Path;

use rusqlite::Connection;

use crate::{StoreError, migrate};

/// Öffnet (oder erstellt) die Datenbank unter `path`, setzt WAL und Foreign Keys und führt
/// ausstehende Migrationen aus. Fehlende Elternverzeichnisse werden angelegt.
pub fn open(path: &Path) -> Result<Connection, StoreError> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    prepare(Connection::open(path)?)
}

/// In-Memory-Datenbank mit allen Migrationen (Tests).
pub fn open_in_memory() -> Result<Connection, StoreError> {
    prepare(Connection::open_in_memory()?)
}

fn prepare(mut conn: Connection) -> Result<Connection, StoreError> {
    // wie die Go-App: `PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;`
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrate(&mut conn)?;
    Ok(conn)
}
