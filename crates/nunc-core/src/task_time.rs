//! Zeiteinträge einer Aufgabe (Details in „Aufgaben“): die abgeschlossenen Arbeitssessions des Todos, neueste zuerst.
//! Pur aus den Roh-Sessions; die Notizen liefert der Aufrufer.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::estimate::EstimateCheck;
use crate::model::{Session, SessionStatus, SessionType, TimerMode};

/// Ein Zeiteintrag an einer Aufgabe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskEntry {
    pub session_id: i32,
    pub started_at: DateTime<Utc>,
    pub minutes: u32,
    pub mode: TimerMode,
    /// Notiz der Session, leer ohne.
    pub note: String,
}

/// Alle Zeiteinträge einer Aufgabe und ihre Summe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskTime {
    pub todo_id: i32,
    pub total_min: u32,
    pub entries: Vec<TaskEntry>,
    /// Schätzung gegen bisherigen Aufwand; vom Aufrufer gesetzt (`estimate::todo_check`), `None` ohne Schätzung.
    pub estimate_check: Option<EstimateCheck>,
}

/// Die abgeschlossenen Arbeitssessions von `todo_id`; abgebrochene, übersprungene und Pausen zählen nicht.
pub fn task_time(sessions: &[Session], todo_id: i32, notes: &HashMap<i32, String>) -> TaskTime {
    let mut entries: Vec<TaskEntry> = sessions
        .iter()
        .filter(|session| {
            session.todo_id == Some(todo_id)
                && session.session_type == SessionType::Work
                && session.status == SessionStatus::Completed
        })
        .map(|session| TaskEntry {
            session_id: session.id,
            started_at: session.started_at,
            minutes: session.actual_min.unwrap_or(0),
            mode: session.mode,
            note: notes
                .get(&session.id)
                .map(|note| note.trim().to_owned())
                .unwrap_or_default(),
        })
        .collect();
    entries.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
            .then(b.session_id.cmp(&a.session_id))
    });
    TaskTime {
        todo_id,
        total_min: entries.iter().map(|entry| entry.minutes).sum(),
        entries,
        estimate_check: None,
    }
}
