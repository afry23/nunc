//! Habit-Tracker (ADR 0028): Gewohnheiten anlegen, ändern, abhaken und die Übersicht mit Serien.

use chrono::NaiveDate;
use nunc_core::habit::{Goal, HabitItem, Schedule, habit_overview};
use nunc_platform::to_local_date;
use nunc_store::StoreError;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Frühester Tag, ab dem Haken gelesen werden (die Serie braucht den ganzen Verlauf).
const EPOCH: NaiveDate = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

/// Alle Gewohnheiten (auch archivierte) mit Stand für heute (lokaler Tag), Serien und den zuletzt erledigten Tagen.
#[tauri::command]
#[specta::specta]
pub fn get_habit_overview(state: State<'_, AppState>) -> Result<Vec<HabitItem>, AppError> {
    let today = state.today();
    let (habits, logs, sessions, todos) = state.db.with(|conn| {
        Ok::<_, StoreError>((
            nunc_store::list_habits(conn)?,
            nunc_store::list_habit_logs(conn, EPOCH, today)?,
            nunc_store::list_sessions(conn)?,
            nunc_store::list_all_todos(conn)?,
        ))
    })?;
    Ok(habit_overview(
        &habits,
        &logs,
        &sessions,
        &todos,
        today,
        &to_local_date,
    ))
}

/// Legt eine Gewohnheit an (`id` leer) oder ändert Name, Rhythmus und Ziel der Gewohnheit `id`. Liefert die Kennung.
#[tauri::command]
#[specta::specta]
pub fn save_habit(
    state: State<'_, AppState>,
    id: Option<String>,
    name: String,
    schedule: Schedule,
    goal: Goal,
) -> Result<String, AppError> {
    let now = state.clock.now();
    let created = state.today();
    let id = state.db.with(|conn| match id {
        Some(id) => nunc_store::update_habit(conn, &id, &name, schedule, goal, now).map(|_| id),
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            nunc_store::create_habit(conn, &id, created, &name, schedule, goal, now).map(|_| id)
        }
    })?;
    Ok(id)
}

/// Blendet eine Gewohnheit aus (`archived`) oder wieder ein; der Verlauf bleibt.
#[tauri::command]
#[specta::specta]
pub fn archive_habit(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> Result<(), AppError> {
    let now = state.clock.now();
    state
        .db
        .with(|conn| nunc_store::set_habit_archived(conn, &id, archived, now))?;
    Ok(())
}

/// Löscht eine Gewohnheit samt ihren Haken.
#[tauri::command]
#[specta::specta]
pub fn delete_habit(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let now = state.clock.now();
    state
        .db
        .with(|conn| nunc_store::delete_habit(conn, &id, now))?;
    Ok(())
}

/// Setzt oder entfernt den Haken der Gewohnheit `id` für `day`. Nur Gewohnheiten mit Ziel „abhaken“ haben Haken.
#[tauri::command]
#[specta::specta]
pub fn set_habit_done(
    state: State<'_, AppState>,
    id: String,
    day: NaiveDate,
    done: bool,
) -> Result<(), AppError> {
    let now = state.clock.now();
    state
        .db
        .with(|conn| nunc_store::set_habit_done(conn, &id, day, done, now))?;
    Ok(())
}
