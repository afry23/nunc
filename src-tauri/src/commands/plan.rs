//! Tagesplaner (ADR 0027): Plan für heute und Planungsfelder eines Todos ändern.

use chrono::NaiveDate;
use nunc_core::day_plan::{
    CloseAction, DayPlan, PlanFields, apply_plan, closing_changes, day_plan, plan_fields,
};
use nunc_core::model::Todo;
use nunc_store::{Connection, StoreError};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Der Plan für `today` aus Todos, Sessions und der Kapazität der Einstellungen.
fn plan_for(conn: &Connection, today: NaiveDate) -> Result<DayPlan, StoreError> {
    let todos = nunc_store::list_todos(conn)?;
    let sessions = nunc_store::list_sessions(conn)?;
    let settings = nunc_store::load_settings(conn)?;
    Ok(day_plan(
        &todos,
        &sessions,
        today,
        settings.day_capacity_min,
    ))
}

/// Der Plan für heute (lokaler Tag): überfällige, heute fällige und eingeplante offene Todos mit Schätzung,
/// erfasster Zeit und der Kapazität aus den Einstellungen.
#[tauri::command]
#[specta::specta]
pub fn get_day_plan(state: State<'_, AppState>) -> Result<DayPlan, AppError> {
    let today = state.today();
    state
        .db
        .with(|conn| plan_for(conn, today).map_err(AppError::from))
}

/// Ändert die Planungsfelder des Todos `id` mit `change`; was `change` nicht anfasst, bleibt.
fn update_plan(
    conn: &Connection,
    id: i32,
    now: chrono::DateTime<chrono::Utc>,
    change: impl FnOnce(&mut PlanFields),
) -> Result<Todo, AppError> {
    let todo = nunc_store::list_all_todos(conn)?
        .into_iter()
        .find(|todo| todo.id == id)
        .ok_or(StoreError::NotFound { entity: "Todo", id })?;
    let mut fields = plan_fields(&todo.description);
    change(&mut fields);
    let raw_text = apply_plan(&todo.raw_text, &fields)
        .map_err(|error| AppError::validation(error.to_string()))?;
    if raw_text == todo.raw_text {
        return Ok(todo);
    }
    Ok(nunc_store::update_todo(conn, id, &raw_text, now)?)
}

fn change_plan(
    state: &AppState,
    id: i32,
    change: impl FnOnce(&mut PlanFields),
) -> Result<Todo, AppError> {
    let now = state.clock.now();
    let todo = state.db.with(|conn| update_plan(conn, id, now, change))?;
    state.todos_changed();
    Ok(todo)
}

/// Plant das Todo für den Tag `day` ein; `null` nimmt es vom Plan. Frist und Schätzung bleiben.
#[tauri::command]
#[specta::specta]
pub fn plan_todo(
    state: State<'_, AppState>,
    id: i32,
    day: Option<NaiveDate>,
) -> Result<Todo, AppError> {
    change_plan(&state, id, |fields| fields.plan = day)
}

/// Setzt die Schätzung (Minuten, 1–10080); `null` entfernt sie samt einer Pomodoro-Schätzung. Frist und Plan-Tag bleiben.
#[tauri::command]
#[specta::specta]
pub fn set_todo_estimate(
    state: State<'_, AppState>,
    id: i32,
    estimate_min: Option<u32>,
) -> Result<Todo, AppError> {
    change_plan(&state, id, |fields| {
        // je Aufgabe eine Einheit (ADR 0035): die Pomodoro-Schätzung entfällt mit
        fields.estimate_min = estimate_min;
        fields.estimate_pomodoros = None;
    })
}

/// Tagesabschluss (ADR 0027): behandelt die noch eingeplanten Einträge von heute nach `action` und speichert die
/// Tagesnotiz, wenn `note` gesetzt ist (ein leerer Text löscht sie). Alles oder nichts. Liefert, wie viele Einträge
/// sich geändert haben.
#[tauri::command]
#[specta::specta]
pub fn close_day(
    state: State<'_, AppState>,
    action: CloseAction,
    note: Option<String>,
) -> Result<u32, AppError> {
    let today = state.today();
    let now = state.clock.now();
    let changed = state.db.with(|conn| {
        nunc_store::atomically(conn, || {
            let changes = closing_changes(&plan_for(conn, today)?, action);
            for (id, day) in &changes {
                update_plan(conn, *id, now, |fields| fields.plan = *day)?;
            }
            if let Some(text) = &note {
                nunc_store::save_day_note(conn, today, text, now)?;
            }
            Ok::<_, AppError>(u32::try_from(changes.len()).unwrap_or(u32::MAX))
        })
    })?;
    if changed > 0 {
        state.todos_changed();
    }
    Ok(changed)
}
