//! Persistenz: SQLite (rusqlite), Migrationen, Backup

mod backup;
mod connection;
mod crdt_repo;
mod error;
mod legacy;
mod migrations;
mod restore;
mod session_repo;
mod session_sync;
mod settings_repo;
mod sync_repo;
mod time;
mod timer_state;
mod todo_repo;

pub use backup::{
    BackupResult, backup_dir_status, backup_if_needed, create_emergency_backup,
    list_backup_entries, resolve_backup_dir, rotate_backups, run_backup, scan_backup_dir,
    set_backup_dir,
};
pub use connection::{open, open_in_memory};
pub use crdt_repo::{apply_state, local_state};
pub use error::StoreError;
pub use legacy::{LegacyImport, import_legacy_db, legacy_db_path};
pub use migrations::{MIGRATIONS, Migration, migrate};
pub use restore::{restore_backup, validate_backup_file};
pub use rusqlite::Connection;
pub use session_repo::{
    ManualSession, SessionDetails, SessionEdit, commit_finish, delete_session, insert_manual,
    list_session_details, list_sessions, update_session,
};
pub use settings_repo::{device_id, load_settings, save_settings, set_sync_provider};
pub use sync_repo::{
    Applied, last_sync_at, last_sync_of, load_webdav_config, save_webdav_config, set_last_sync_at,
    set_last_sync_of,
};
pub use timer_state::{load_timer, save_timer};
pub use todo_repo::{
    TodotxtImport, add_subtask, add_todo, delete_todo, export_todotxt, import_todotxt,
    increment_pomodoro_count, list_all_todos, list_todos, reorder_todos, set_todo_completed,
    set_todo_parent, update_todo,
};
