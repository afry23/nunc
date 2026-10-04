//! Persistenz: SQLite (rusqlite), Migrationen, Backup

mod connection;
mod error;
mod migrations;

pub use connection::{open, open_in_memory};
pub use error::StoreError;
pub use migrations::{MIGRATIONS, Migration, migrate};
