//! Persistenz: SQLite (rusqlite), Migrationen, Backup

mod connection;
mod error;
mod migrations;
mod session_repo;
mod settings_repo;
mod time;
mod timer_state;
mod todo_repo;

pub use connection::{open, open_in_memory};
pub use error::StoreError;
pub use migrations::{MIGRATIONS, Migration, migrate};
pub use session_repo::{commit_finish, list_sessions};
pub use settings_repo::{load_settings, save_settings};
pub use timer_state::{load_timer, save_timer};
pub use todo_repo::{
    add_todo, delete_todo, increment_pomodoro_count, list_all_todos, list_todos, reorder_todos,
    set_todo_completed, update_todo,
};
