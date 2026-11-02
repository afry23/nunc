//! Tagesplaner (ADR 0027): Plan für heute und Planungsfelder eines Todos ändern.

use chrono::NaiveDate;
use nunc_core::day_plan::{DayPlan, PlanFields, apply_plan, day_plan};
use nunc_core::model::Todo;
use nunc_store::StoreError;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Der Plan für heute (lokaler Tag): überfällige, heute fällige und eingeplante offene Todos mit Schätzung,
/// erfasster Zeit und der Kapazität aus den Einstellungen.
#[tauri::command]
#[specta::specta]
pub fn get_day_plan(state: State<'_, AppState>) -> Result<DayPlan, AppError> {
    let today = state.today();
    let (todos, sessions, settings) = state.db.with(|conn| {
        Ok::<_, StoreError>((
            nunc_store::list_todos(conn)?,
            nunc_store::list_sessions(conn)?,
            nunc_store::load_settings(conn)?,
        ))
    })?;
    Ok(day_plan(
        &todos,
        &sessions,
        today,
        settings.day_capacity_min,
    ))
}

/// Setzt Frist, Plan-Tag und Schätzung (Minuten) eines Todos; `null` entfernt das jeweilige Feld.
#[tauri::command]
#[specta::specta]
pub fn set_todo_plan(
    state: State<'_, AppState>,
    id: i32,
    due: Option<NaiveDate>,
    plan: Option<NaiveDate>,
    estimate_min: Option<u32>,
) -> Result<Todo, AppError> {
    let now = state.clock.now();
    let fields = PlanFields {
        due,
        plan,
        estimate_min,
    };
    let todo = state.db.with(|conn| {
        let todo = nunc_store::list_all_todos(conn)?
            .into_iter()
            .find(|todo| todo.id == id)
            .ok_or(StoreError::NotFound { entity: "Todo", id })?;
        let raw_text = apply_plan(&todo.raw_text, &fields)
            .map_err(|error| AppError::validation(error.to_string()))?;
        if raw_text == todo.raw_text {
            return Ok(todo);
        }
        Ok::<_, AppError>(nunc_store::update_todo(conn, id, &raw_text, now)?)
    })?;
    state.todos_changed();
    Ok(todo)
}
