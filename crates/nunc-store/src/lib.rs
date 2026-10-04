//! Persistenz: SQLite (rusqlite), Migrationen, Backup

mod connection;
mod error;
mod migrations;
mod timer_state;

pub use connection::{open, open_in_memory};
pub use error::StoreError;
pub use migrations::{MIGRATIONS, Migration, migrate};
pub use timer_state::{load_timer, save_timer};
