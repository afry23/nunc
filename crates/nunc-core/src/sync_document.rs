//! Austauschformat `nunc-sync.v2.json` (ADR 0016) und der einmalige Import der alten Datei `nunc-sync.json`.
//!
//! Die Datei ist der [`SyncState`] als Listen (Todos mit Feldregistern, Sessions) plus Kopfdaten. Die alte Datei
//! kennt keine Stempel: Beim Import trägt jedes Feld den Zeitpunkt `updated_at` und das Pseudo-Gerät `legacy`
//! (wie die Bestandsdaten in Migration 007), die Reihenfolge wird aus `position` abgeleitet.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::crdt::{Register, SyncState, TodoCrdt, session_key};
use crate::hlc::Hlc;
use crate::model::{SyncFile, SyncSession, SyncTodo};
use crate::todotxt::parse_line;

/// Version des Formats in `schema_version`.
pub const SCHEMA_VERSION: u32 = 2;

/// Pseudo-Gerät für Stände ohne Stempel.
pub const LEGACY_DEVICE: &str = "legacy";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocumentError {
    #[error(
        "Die Sync-Datei hat das Format {found}, diese Version versteht {SCHEMA_VERSION}. Bitte nunc aktualisieren."
    )]
    UnsupportedVersion { found: u32 },
}

/// Die Sync-Datei auf dem Server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncDocument {
    pub schema_version: u32,
    pub synced_at: DateTime<Utc>,
    #[serde(default)]
    pub todos: Vec<TodoCrdt>,
    #[serde(default)]
    pub sessions: Vec<SyncSession>,
}

impl SyncDocument {
    pub fn from_state(state: &SyncState, synced_at: DateTime<Utc>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            synced_at,
            todos: state.todos.values().cloned().collect(),
            sessions: state.sessions.values().cloned().collect(),
        }
    }

    /// Der Zustand aus der Datei; eine andere Version ist ein Fehler (neuere Dateien nicht raten).
    pub fn into_state(self) -> Result<SyncState, DocumentError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(DocumentError::UnsupportedVersion {
                found: self.schema_version,
            });
        }
        Ok(SyncState {
            todos: self
                .todos
                .into_iter()
                .filter(|todo| !todo.sync_id.is_empty())
                .map(|todo| (todo.sync_id.clone(), todo))
                .collect(),
            sessions: self
                .sessions
                .into_iter()
                .map(|session| (session_key(&session), session))
                .collect(),
        })
    }
}

fn day(text: &Option<String>) -> Option<String> {
    let text = text.as_deref()?;
    let day = NaiveDate::parse_from_str(text.get(..10)?, "%Y-%m-%d").ok()?;
    Some(day.format("%Y-%m-%d").to_string())
}

/// Reihenfolgeschlüssel aus dem 1-basierten Rang: feste Breite, endet nie auf `0` (siehe [`crate::position`]).
/// Dieselbe Regel wendet Migration 007 auf Bestandsdaten an, damit beide Seiten gleiche Schlüssel haben.
pub fn legacy_position_key(rank: usize) -> String {
    format!("{rank:06}5")
}

fn legacy_todo(todo: &SyncTodo, rank: usize) -> TodoCrdt {
    fn reg<T: Ord + Clone>(value: T, clock: &Hlc) -> Register<T> {
        Register::new(value, clock.clone())
    }
    let wall =
        DateTime::parse_from_rfc3339(&todo.updated_at).map_or(0, |time| time.timestamp_millis());
    let clock = Hlc::new(wall, 0, LEGACY_DEVICE);
    let priority = todo
        .priority
        .clone()
        .filter(|priority| !priority.is_empty());
    // Die Go-App speichert die Beschreibung ohne Tags; die Zeile ist maßgeblich (hier gehören Tags zur Beschreibung).
    let description = parse_line(&todo.raw_text).description;
    TodoCrdt {
        sync_id: todo.sync_id.clone(),
        priority: reg(priority, &clock),
        completed: reg(todo.completed, &clock),
        completed_at: reg(day(&todo.completed_at), &clock),
        created_at: reg(day(&todo.created_at), &clock),
        description: reg(description, &clock),
        position: reg(legacy_position_key(rank), &clock),
        deleted: reg(todo.deleted, &clock),
        pomodoros: if todo.pomodoro_count > 0 {
            BTreeMap::from([(LEGACY_DEVICE.to_owned(), todo.pomodoro_count)])
        } else {
            BTreeMap::new()
        },
    }
}

impl SyncState {
    /// Der Stand aus der alten Datei `nunc-sync.json` (einmaliger Import beim ersten Lauf). Todos ohne `sync_id`
    /// werden übersprungen.
    pub fn from_legacy(file: &SyncFile) -> Self {
        // Rang nach `position` (bei Gleichstand nach Reihenfolge in der Datei), wie `ROW_NUMBER` in Migration 007.
        let mut ranked: Vec<&SyncTodo> = file
            .todos
            .iter()
            .filter(|todo| !todo.sync_id.is_empty())
            .collect();
        ranked.sort_by_key(|todo| todo.position);
        Self {
            todos: ranked
                .into_iter()
                .enumerate()
                .map(|(index, todo)| (todo.sync_id.clone(), legacy_todo(todo, index + 1)))
                .collect(),
            sessions: file
                .sessions
                .iter()
                .map(|session| (session_key(session), session.clone()))
                .collect(),
        }
    }
}
