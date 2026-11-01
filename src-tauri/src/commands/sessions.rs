//! Zeiterfassung (ADR 0017): Timesheet, Tages-Timeline sowie Nachtragen, Bearbeiten und Löschen von Sessions.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use nunc_core::manual_entry::validate_entry;
use nunc_core::model::{KnownTags, Session, known_tags, normalize_tags};
use nunc_core::timeline::{DayTimeline, day_timeline};
use nunc_core::timesheet::{GroupBy, Timesheet, timesheet};
use nunc_platform::to_local_date;
use nunc_store::{ManualSession, SessionEdit};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Längster Zeitraum eines Timesheets (ein Jahr), damit eine Anfrage die Oberfläche nicht lahmlegt.
const MAX_REPORT_DAYS: i64 = 366;

/// Beginn, Ende, Todo und Notiz einer Session.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionInput {
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub todo_id: Option<i32>,
    pub note: String,
    /// Eigene Projekte/Kontexte, nur wirksam ohne Todo (ADR 0023).
    #[serde(default)]
    pub project_tags: Vec<String>,
    #[serde(default)]
    pub context_tags: Vec<String>,
}

impl SessionInput {
    /// Normalisierte Projekte und Kontexte; mit Todo leer (es zählen dessen Tags).
    fn tags(&self) -> (Vec<String>, Vec<String>) {
        if self.todo_id.is_some() {
            return (Vec::new(), Vec::new());
        }
        (
            normalize_tags(&self.project_tags, '+'),
            normalize_tags(&self.context_tags, '@'),
        )
    }
}

/// Die gespeicherte Session und die Sessions, die sie überlappt (Warnung, kein Fehler).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionSaved {
    pub session: Session,
    pub overlaps: Vec<i32>,
}

/// Arbeitszeit je Projekt, Kontext oder Aufgabe und Tag für die lokalen Tage `start_date` bis `end_date`.
#[tauri::command]
#[specta::specta]
pub fn get_timesheet(
    state: State<'_, AppState>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    group_by: GroupBy,
) -> Result<Timesheet, AppError> {
    if (end_date - start_date).num_days() >= MAX_REPORT_DAYS {
        return Err(AppError::validation(
            "Der Zeitraum darf höchstens ein Jahr umfassen.",
        ));
    }
    let (sessions, todos) = state.db.with(|conn| {
        Ok::<_, nunc_store::StoreError>((
            nunc_store::list_sessions(conn)?,
            nunc_store::list_all_todos(conn)?,
        ))
    })?;
    Ok(timesheet(
        &sessions,
        &todos,
        start_date,
        end_date,
        group_by,
        &to_local_date,
    ))
}

/// Die Sessions eines lokalen Tages als Blöcke, mit Notiz und Herkunft.
#[tauri::command]
#[specta::specta]
pub fn get_day_timeline(
    state: State<'_, AppState>,
    date: NaiveDate,
) -> Result<DayTimeline, AppError> {
    let (sessions, details) = state.db.with(|conn| {
        Ok::<_, nunc_store::StoreError>((
            nunc_store::list_sessions(conn)?,
            nunc_store::list_session_details(conn)?,
        ))
    })?;
    let mut timeline = day_timeline(&sessions, date, &to_local_date);
    for block in &mut timeline.blocks {
        if let Some(detail) = details.get(&block.session_id) {
            block.note.clone_from(&detail.note);
            block.origin.clone_from(&detail.origin);
        }
    }
    Ok(timeline)
}

/// Alle bekannten Projekte und Kontexte (Todos und Sessions) als Vorschläge für die Auswahl.
#[tauri::command]
#[specta::specta]
pub fn get_known_tags(state: State<'_, AppState>) -> Result<KnownTags, AppError> {
    let (sessions, todos) = state.db.with(|conn| {
        Ok::<_, nunc_store::StoreError>((
            nunc_store::list_sessions(conn)?,
            nunc_store::list_all_todos(conn)?,
        ))
    })?;
    Ok(known_tags(&sessions, &todos))
}

/// Trägt eine Arbeitssession nach.
#[tauri::command]
#[specta::specta]
pub fn create_manual_session(
    state: State<'_, AppState>,
    input: SessionInput,
) -> Result<SessionSaved, AppError> {
    let now = state.clock.now();
    let (project_tags, context_tags) = input.tags();
    let saved = state.db.with(|conn| {
        let overlaps = validate_entry(
            input.started_at,
            input.ended_at,
            now,
            &nunc_store::list_sessions(conn)?,
            None,
        )?;
        let device = nunc_store::device_id(conn, || uuid::Uuid::new_v4().to_string())?;
        let entry = ManualSession {
            sync_id: uuid::Uuid::new_v4().to_string(),
            started_at: input.started_at,
            ended_at: input.ended_at,
            todo_id: input.todo_id,
            note: input.note,
            project_tags,
            context_tags,
        };
        let session = nunc_store::insert_manual(conn, &entry, &device, now)?;
        Ok::<_, AppError>(SessionSaved { session, overlaps })
    })?;
    state.sessions_changed();
    Ok(saved)
}

/// Ersetzt Beginn, Ende, Todo und Notiz einer Session; die Dauer folgt aus Beginn und Ende.
#[tauri::command]
#[specta::specta]
pub fn update_session(
    state: State<'_, AppState>,
    id: i32,
    input: SessionInput,
) -> Result<SessionSaved, AppError> {
    let now = state.clock.now();
    let (project_tags, context_tags) = input.tags();
    let saved = state.db.with(|conn| {
        let overlaps = validate_entry(
            input.started_at,
            input.ended_at,
            now,
            &nunc_store::list_sessions(conn)?,
            Some(id),
        )?;
        let edit = SessionEdit {
            started_at: input.started_at,
            ended_at: Some(input.ended_at),
            actual_min: minutes(input.ended_at - input.started_at),
            todo_id: input.todo_id,
            note: input.note,
            project_tags,
            context_tags,
        };
        let session = nunc_store::update_session(conn, id, &edit, now)?;
        Ok::<_, AppError>(SessionSaved { session, overlaps })
    })?;
    state.sessions_changed();
    Ok(saved)
}

/// Löscht eine Session (Löschmarke, damit der Sync sie weiterträgt).
#[tauri::command]
#[specta::specta]
pub fn delete_session(state: State<'_, AppState>, id: i32) -> Result<(), AppError> {
    let now = state.clock.now();
    state
        .db
        .with(|conn| nunc_store::delete_session(conn, id, now))?;
    state.sessions_changed();
    Ok(())
}

fn minutes(duration: Duration) -> u32 {
    u32::try_from(duration.num_minutes()).unwrap_or(0)
}
