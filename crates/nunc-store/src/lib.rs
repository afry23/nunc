//! Persistenz: SQLite (rusqlite), Migrationen, Backup

mod backup;
mod connection;
mod error;
mod migrations;
mod restore;
mod session_repo;
mod settings_repo;
mod time;
mod timer_state;
mod todo_repo;

pub use backup::{
    BackupResult, backup_dir_status, backup_if_needed, create_emergency_backup,
    list_backup_entries, resolve_backup_dir, rotate_backups, run_backup, scan_backup_dir,
    set_backup_dir,
};
pub use connection::{open, open_in_memory};
pub use error::StoreError;
pub use migrations::{MIGRATIONS, Migration, migrate};
pub use restore::{restore_backup, validate_backup_file};
pub use session_repo::{commit_finish, list_sessions};
pub use settings_repo::{load_settings, save_settings};
pub use timer_state::{load_timer, save_timer};
pub use todo_repo::{
    add_todo, delete_todo, increment_pomodoro_count, list_all_todos, list_todos, reorder_todos,
    set_todo_completed, update_todo,
};
