/// Fehler der Persistenzschicht.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Datenverzeichnis nicht anlegbar: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite-Fehler: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Gespeicherter Zustand nicht lesbar: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Migration {version} fehlgeschlagen: {source}")]
    Migration {
        version: u32,
        #[source]
        source: rusqlite::Error,
    },
}
