//! Austauschformat `nunc-sync.v2.json` (ADR 0016, Format 3 nach ADR 0017) und der einmalige Import der alten Datei `nunc-sync.json`.
//!
//! Die Datei ist der [`SyncState`] als Listen (Todos mit Feldregistern, Sessions) plus Kopfdaten. Die alte Datei
//! kennt keine Stempel: Beim Import trägt jedes Feld den Zeitpunkt `updated_at` und das Pseudo-Gerät `legacy`
//! (wie die Bestandsdaten in Migration 007), die Reihenfolge wird aus `position` abgeleitet.

use std::collections::BTreeMap;
use std::io::{Read, Write};

use chrono::{DateTime, NaiveDate, Utc};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};

use crate::crdt::{Register, SessionCrdt, SyncState, TodoCrdt, session_key};
use crate::hlc::Hlc;
use crate::journal::{JournalEntry, JournalKind};
use crate::model::{SyncFile, SyncSession, SyncTodo};
use crate::todotxt::parse_line;

/// Version des Formats in `schema_version`, die geschrieben wird (ADR 0017: Sessions als CRDT, ADR 0018: `kind`, ADR 0022: `parent`, ADR 0023: Projekte/Kontexte der Session, ADR 0026: Journal, ADR 0028: Gewohnheiten, ADR 0029: kompakte Stempel und gzip, ADR 0033: Gewohnheiten entfallen wieder, ADR 0034: der Tagesfokus entfällt).
pub const SCHEMA_VERSION: u32 = 12;

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
    #[error("Die Sync-Datei ist beschädigt: {0}")]
    Invalid(String),
}

/// Die ersten Bytes jeder gzip-Datei.
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// Obergrenze der entpackten Größe in Bytes (512 MiB; Schutz vor Zip-Bomben auf dem Server).
const MAX_DECODED_BYTES: usize = 536_870_912;

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
    /// Tages-/Wochennotizen und Erfolge (ab Version 7, ADR 0026); ältere Dateien haben keine.
    #[serde(default, deserialize_with = "known_journal_entries")]
    pub journal: Vec<JournalEntry>,
}

/// Liest die Journal-Einträge und lässt solche mit unbekannter Art weg (der Tagesfokus älterer Dateien, ADR 0034),
/// statt die ganze Datei als beschädigt abzulehnen.
fn known_journal_entries<'de, D>(deserializer: D) -> Result<Vec<JournalEntry>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Vec::<serde_json::Value>::deserialize(deserializer)?;
    values
        .into_iter()
        .filter(|value| {
            value
                .get("kind")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|kind| JournalKind::parse(kind).is_some())
        })
        .map(|value| serde_json::from_value(value).map_err(serde::de::Error::custom))
        .collect()
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
            journal: state.journal.values().cloned().collect(),
        }
    }

    /// Die Bytes für den Server: kompaktes JSON, gzip-komprimiert (ADR 0029).
    pub fn encode(&self) -> Result<Vec<u8>, DocumentError> {
        let json =
            serde_json::to_vec(self).map_err(|error| DocumentError::Invalid(error.to_string()))?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&json)
            .and_then(|()| encoder.finish())
            .map_err(|error| DocumentError::Invalid(error.to_string()))
    }

    /// Liest die Bytes vom Server: gzip (ab Format 9) oder unkomprimiertes JSON (Format 2 bis 8).
    pub fn decode(bytes: &[u8]) -> Result<Self, DocumentError> {
        Self::decode_limited(bytes, MAX_DECODED_BYTES)
    }

    /// Wie [`decode`](Self::decode) mit einer Obergrenze für die entpackte Größe (genau `max_decoded` Bytes sind erlaubt).
    fn decode_limited(bytes: &[u8], max_decoded: usize) -> Result<Self, DocumentError> {
        if bytes.starts_with(&GZIP_MAGIC) {
            let mut json = Vec::new();
            GzDecoder::new(bytes)
                .take(max_decoded as u64 + 1)
                .read_to_end(&mut json)
                .map_err(|error| DocumentError::Invalid(error.to_string()))?;
            if json.len() > max_decoded {
                return Err(DocumentError::Invalid("entpackt zu groß".to_owned()));
            }
            serde_json::from_slice(&json)
        } else {
            serde_json::from_slice(bytes)
        }
        .map_err(|error| DocumentError::Invalid(error.to_string()))
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
            journal: self
                .journal
                .into_iter()
                .map(|entry| (entry.sync_id.clone(), entry))
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
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> SyncDocument {
        SyncDocument::from_state(&SyncState::default(), DateTime::UNIX_EPOCH)
    }

    #[test]
    fn the_limit_is_512_mib() {
        assert_eq!(MAX_DECODED_BYTES, 512 * 1024 * 1024);
    }

    #[test]
    fn a_file_exactly_at_the_limit_is_read_and_one_byte_more_is_refused() {
        let document = document();
        let bytes = document.encode().unwrap();
        let size = serde_json::to_vec(&document).unwrap().len();
        assert_eq!(
            SyncDocument::decode_limited(&bytes, size).unwrap(),
            document
        );
        assert_eq!(
            SyncDocument::decode_limited(&bytes, size - 1),
            Err(DocumentError::Invalid("entpackt zu groß".to_owned()))
        );
    }

    #[test]
    fn habits_of_an_older_file_are_ignored() {
        let json = br#"{"schema_version":10,"synced_at":"2026-03-01T00:00:00Z","todos":[],"sessions":[],"journal":[],
            "habits":[{"sync_id":"h"}],"habit_logs":[{"habit":"h"}]}"#;
        let document = SyncDocument::decode(json).unwrap();
        assert_eq!(document.schema_version, 10);
        assert!(document.todos.is_empty() && document.sessions.is_empty());
    }

    #[test]
    fn the_day_focus_of_an_older_file_is_ignored_and_the_notes_stay() {
        let entry = |id: &str, kind: &str| {
            format!(
                r#"{{"sync_id":"{id}","kind":"{kind}",
                "date":{{"value":"2026-03-02","hlc":{{"ms":1,"counter":0,"device":"a"}}}},
                "text":{{"value":"x","hlc":{{"ms":1,"counter":0,"device":"a"}}}},
                "deleted":{{"value":false,"hlc":{{"ms":1,"counter":0,"device":"a"}}}}}}"#
            )
        };
        let journal = JournalEntry::day_note(
            NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
            "x",
            &Hlc::new(1, 0, "a"),
        );
        let note = serde_json::to_string(&journal).unwrap();
        let json = format!(
            r#"{{"schema_version":11,"synced_at":"2026-03-01T00:00:00Z","journal":[{},{}]}}"#,
            note,
            entry("note:focus:2026-03-02", "day_focus")
        );
        let document = SyncDocument::decode(json.as_bytes()).unwrap();
        assert_eq!(document.journal, vec![journal]);
    }
}
