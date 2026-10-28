//! Austauschformat `nunc-sync.v2.json` (ADR 0016, Format 3 nach ADR 0017) und der einmalige Import der alten Datei `nunc-sync.json`.
//!
//! Die Datei ist der [`SyncState`] als Listen (Todos mit Feldregistern, Sessions) plus Kopfdaten. Die alte Datei
//! kennt keine Stempel: Beim Import trägt jedes Feld den Zeitpunkt `updated_at` und das Pseudo-Gerät `legacy`
//! (wie die Bestandsdaten in Migration 007), die Reihenfolge wird aus `position` abgeleitet.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::crdt::{Register, SessionCrdt, SyncState, TodoCrdt, session_key};
use crate::hlc::Hlc;
use crate::model::{SyncFile, SyncSession, SyncTodo};
use crate::todotxt::parse_line;

/// Version des Formats in `schema_version`, die geschrieben wird (ADR 0017: Sessions als CRDT, ADR 0018: `kind`, ADR 0022: `parent`).
pub const SCHEMA_VERSION: u32 = 5;

/// Älteste Version, die noch gelesen wird: Version 2 kennt Sessions nur als Rohdaten ([`SyncSession`]).
pub const MIN_SCHEMA_VERSION: u32 = 2;

/// Pseudo-Gerät für Stände ohne Stempel.
pub const LEGACY_DEVICE: &str = "legacy";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocumentError {
    #[error(
        "Die Sync-Datei hat das Format {found}, diese Version versteht {MIN_SCHEMA_VERSION} bis {SCHEMA_VERSION}. Bitte nunc aktualisieren."
    )]
    UnsupportedVersion { found: u32 },
}

/// Eine Session in der Datei: Version 3 schreibt [`SessionCrdt`], Version 2 die Rohdaten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionEntry {
    Crdt(Box<SessionCrdt>),
    Legacy(SyncSession),
}

/// Die Sync-Datei auf dem Server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncDocument {
    pub schema_version: u32,
    pub synced_at: DateTime<Utc>,
    #[serde(default)]
    pub todos: Vec<TodoCrdt>,
    #[serde(default)]
    pub sessions: Vec<SessionEntry>,
}

impl SyncDocument {
    pub fn from_state(state: &SyncState, synced_at: DateTime<Utc>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            synced_at,
            todos: state.todos.values().cloned().collect(),
            sessions: state
                .sessions
                .values()
                .cloned()
                .map(|session| SessionEntry::Crdt(Box::new(session)))
                .collect(),
        }
    }

    /// Der Zustand aus der Datei (Version 2 und 3); eine andere Version ist ein Fehler (neuere Dateien nicht raten).
    /// Sessions der Version 2 werden zu [`SessionCrdt::from_legacy`].
    pub fn into_state(self) -> Result<SyncState, DocumentError> {
        if !(MIN_SCHEMA_VERSION..=SCHEMA_VERSION).contains(&self.schema_version) {
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
                .map(|entry| match entry {
                    SessionEntry::Crdt(session) => *session,
                    SessionEntry::Legacy(session) => SessionCrdt::from_legacy(&session),
                })
                .map(|session| (session.sync_id.clone(), session))
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
        // Version-2-Dateien kennen keine Unteraufgaben; der uralte Stempel entspricht Bestandsdaten ohne Zeile
        parent: reg(None, &Hlc::new(0, 0, LEGACY_DEVICE)),
        deleted: reg(todo.deleted, &clock),
        pomodoros: if todo.pomodoro_count > 0 {
            BTreeMap::from([(LEGACY_DEVICE.to_owned(), todo.pomodoro_count)])
        } else {
            BTreeMap::new()
        },
    }
}

impl SessionCrdt {
    /// Eine Session ohne `sync_id` und ohne Stempel (Version-2-Datei, alte Datei, Bestandsdaten): Die ID folgt
    /// deterministisch aus [`session_key`], damit zwei Geräte für dieselbe Session dieselbe ID bilden und nichts
    /// doppelt entsteht. Alle Felder tragen den Stempel `0` des Pseudo-Geräts `legacy`; jede echte Bearbeitung
    /// gewinnt dagegen.
    pub fn from_legacy(session: &SyncSession) -> Self {
        Self::from_session(
            format!("legacy:{}", session_key(session)),
            session,
            &Hlc::new(0, 0, LEGACY_DEVICE),
        )
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
                .map(SessionCrdt::from_legacy)
                .map(|session| (session.sync_id.clone(), session))
                .collect(),
        }
    }
}
