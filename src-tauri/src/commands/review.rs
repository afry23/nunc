//! Rückblick-Bericht (ADR 0025): Bericht für einen Zeitraum und Markdown-Export.

use std::collections::HashMap;

use chrono::NaiveDate;
use nunc_core::journal::JournalKind;
use nunc_core::model::Language;
use nunc_core::review::{Review, build_review, review_markdown};
use nunc_platform::to_local_date;
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FilePath;

use super::stats::write_file;
use crate::error::AppError;
use crate::state::AppState;

/// Längster Zeitraum eines Berichts (ein Jahr), wie beim Timesheet.
const MAX_REVIEW_DAYS: i64 = 366;

fn review_for(state: &AppState, from: NaiveDate, to: NaiveDate) -> Result<Review, AppError> {
    if to < from {
        return Err(AppError::validation(
            "Das Ende des Zeitraums liegt vor dem Beginn.",
        ));
    }
    if (to - from).num_days() >= MAX_REVIEW_DAYS {
        return Err(AppError::validation(
            "Der Zeitraum darf höchstens ein Jahr umfassen.",
        ));
    }
    let (sessions, todos, details, journal) = state.db.with(|conn| {
        Ok::<_, nunc_store::StoreError>((
            nunc_store::list_sessions(conn)?,
            nunc_store::list_all_todos(conn)?,
            nunc_store::list_session_details(conn)?,
            nunc_store::list_journal(conn, from, to)?,
        ))
    })?;
    let notes: HashMap<i32, String> = details
        .into_iter()
        .map(|(id, detail)| (id, detail.note))
        .collect();
    Ok(build_review(
        nunc_core::review::Period { from, to },
        &sessions,
        &todos,
        &notes,
        &journal,
        &to_local_date,
    ))
}

/// Rückblick für die lokalen Tage `from` bis `to` (inklusive), mit Vergleich zur Vorperiode.
#[tauri::command]
#[specta::specta]
pub fn get_review(
    state: State<'_, AppState>,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Review, AppError> {
    review_for(&state, from, to)
}

/// Schreibt den Rückblick als Markdown in eine vom Nutzer gewählte Datei, in der Sprache der Oberfläche (`system` gilt als Deutsch). `None`, wenn der Dialog abgebrochen wurde.
#[tauri::command]
#[specta::specta]
pub async fn save_review_markdown<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    from: NaiveDate,
    to: NaiveDate,
    language: Language,
) -> Result<Option<String>, AppError> {
    let content = review_markdown(&review_for(&state, from, to)?, language);
    let Some(target) = app
        .dialog()
        .file()
        .set_title("Rückblick exportieren")
        .add_filter("Markdown", &["md"])
        .set_file_name(format!("nunc-review-{from}-{to}.md"))
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = match &target {
        FilePath::Path(path) => path.display().to_string(),
        other => other.to_string(),
    };
    write_file(&app, target, content.as_bytes())?;
    Ok(Some(path))
}

/// Text der Tagesnotiz für `date`; leer, wenn es keine gibt.
#[tauri::command]
#[specta::specta]
pub fn get_day_note(state: State<'_, AppState>, date: NaiveDate) -> Result<String, AppError> {
    let entries = state
        .db
        .with(|conn| nunc_store::list_journal(conn, date, date))?;
    Ok(entries
        .into_iter()
        .find(|entry| entry.kind == JournalKind::DayNote)
        .map(|entry| entry.text.value)
        .unwrap_or_default())
}

/// Tagesfokus für `date`; leer, wenn es keinen gibt (ADR 0032).
#[tauri::command]
#[specta::specta]
pub fn get_day_focus(state: State<'_, AppState>, date: NaiveDate) -> Result<String, AppError> {
    let entries = state
        .db
        .with(|conn| nunc_store::list_journal(conn, date, date))?;
    Ok(entries
        .into_iter()
        .find(|entry| entry.kind == JournalKind::DayFocus)
        .map(|entry| entry.text.value)
        .unwrap_or_default())
}

/// Speichert den Tagesfokus, die Tages- oder Wochennotiz für `date` (bei `week_note` irgendein Tag der Woche). Ein leerer Text löscht die Notiz.
#[tauri::command]
#[specta::specta]
pub fn save_journal_note(
    state: State<'_, AppState>,
    kind: JournalKind,
    date: NaiveDate,
    text: String,
) -> Result<(), AppError> {
    let now = state.clock.now();
    state.db.with(|conn| match kind {
        JournalKind::DayFocus => nunc_store::save_day_focus(conn, date, &text, now),
        JournalKind::DayNote => nunc_store::save_day_note(conn, date, &text, now),
        JournalKind::WeekNote => nunc_store::save_week_note(conn, date, &text, now),
        JournalKind::Achievement => Err(nunc_core::journal::JournalError::NotANote.into()),
    })?;
    Ok(())
}

/// Legt einen Erfolg für `date` an und liefert seine Kennung.
#[tauri::command]
#[specta::specta]
pub fn add_achievement(
    state: State<'_, AppState>,
    date: NaiveDate,
    text: String,
) -> Result<String, AppError> {
    let now = state.clock.now();
    let sync_id = uuid::Uuid::new_v4().to_string();
    state
        .db
        .with(|conn| nunc_store::add_achievement(conn, &sync_id, date, &text, now))?;
    Ok(sync_id)
}

/// Ändert Tag und Text eines Erfolgs.
#[tauri::command]
#[specta::specta]
pub fn update_achievement(
    state: State<'_, AppState>,
    id: String,
    date: NaiveDate,
    text: String,
) -> Result<(), AppError> {
    let now = state.clock.now();
    state
        .db
        .with(|conn| nunc_store::update_achievement(conn, &id, date, &text, now))?;
    Ok(())
}

/// Löscht einen Erfolg.
#[tauri::command]
#[specta::specta]
pub fn delete_achievement(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let now = state.clock.now();
    state
        .db
        .with(|conn| nunc_store::delete_achievement(conn, &id, now))?;
    Ok(())
}
