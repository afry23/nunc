use nunc_core::model::Todo;
use nunc_core::todo_query::{SortDirection, TodoFilter, TodoSort, query_todos};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Aktive (nicht gelöschte) Todos, gefiltert und sortiert. Ohne Angaben: alle in manueller Reihenfolge.
#[tauri::command]
#[specta::specta]
pub fn get_todos(
    state: State<'_, AppState>,
    filter: Option<TodoFilter>,
    sort: Option<TodoSort>,
    direction: Option<SortDirection>,
) -> Result<Vec<Todo>, AppError> {
    let todos = state.db.with(|conn| nunc_store::list_todos(conn))?;
    Ok(query_todos(
        &todos,
        &filter.unwrap_or_default(),
        sort.unwrap_or(TodoSort::Manual),
        direction.unwrap_or(SortDirection::Ascending),
    ))
}

/// Legt ein Todo aus einer todo.txt-Zeile an.
#[tauri::command]
#[specta::specta]
pub fn add_todo(state: State<'_, AppState>, raw_text: String) -> Result<Todo, AppError> {
    let raw_text = non_empty(&raw_text)?;
    let sync_id = uuid::Uuid::new_v4().to_string();
    let now = state.clock.now();
    let todo = state
        .db
        .with(|conn| nunc_store::add_todo(conn, raw_text, &sync_id, now))?;
    state.todos_changed();
    Ok(todo)
}

/// Ersetzt den Text eines Todos (Position, Pomodoro-Zähler und Sync-Kennung bleiben).
#[tauri::command]
#[specta::specta]
pub fn update_todo(
    state: State<'_, AppState>,
    id: i32,
    raw_text: String,
) -> Result<Todo, AppError> {
    let raw_text = non_empty(&raw_text)?;
    let now = state.clock.now();
    let todo = state
        .db
        .with(|conn| nunc_store::update_todo(conn, id, raw_text, now))?;
    state.todos_changed();
    Ok(todo)
}

/// Löscht ein Todo (als Löschmarke, damit der Sync es weiterträgt).
#[tauri::command]
#[specta::specta]
pub fn delete_todo(state: State<'_, AppState>, id: i32) -> Result<(), AppError> {
    let now = state.clock.now();
    state
        .db
        .with(|conn| nunc_store::delete_todo(conn, id, now))?;
    state.todos_changed();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_todo_completed(
    state: State<'_, AppState>,
    id: i32,
    completed: bool,
) -> Result<Todo, AppError> {
    let now = state.clock.now();
    let today = state.today();
    let todo = state
        .db
        .with(|conn| nunc_store::set_todo_completed(conn, id, completed, today, now))?;
    state.todos_changed();
    Ok(todo)
}

/// Setzt die manuelle Reihenfolge: das erste Element bekommt Position 0.
#[tauri::command]
#[specta::specta]
pub fn reorder_todos(state: State<'_, AppState>, ids: Vec<i32>) -> Result<(), AppError> {
    let now = state.clock.now();
    state
        .db
        .with(|conn| nunc_store::reorder_todos(conn, &ids, now))
}

fn non_empty(raw_text: &str) -> Result<&str, AppError> {
    if raw_text.trim().is_empty() {
        return Err(AppError::validation(
            "Der Text einer Aufgabe darf nicht leer sein.",
        ));
    }
    Ok(raw_text)
}
