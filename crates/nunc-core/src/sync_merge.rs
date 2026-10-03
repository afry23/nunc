//! Zusammenführen zweier Sync-Stände (pur). Verhalten der Go-App (`sync_merge.go`, `sync_webdav_merge.go`):
//!
//! - **Todos:** Last-Write-Wins nach `updated_at`, identifiziert über `sync_id`. Bei gleichem Zeitpunkt gewinnt
//!   die lokale Version. Das Löschkennzeichen (`deleted`) reist mit der gewinnenden Version.
//! - **Sessions:** append-only, nie überschrieben. Identität ist `(device_id, started_at)`; zusätzlich gelten zwei
//!   Sessions mit identischem Inhalt (ohne `device_id`) als dieselbe.
//!
//! Abweichungen: Zeitangaben werden als Zeitpunkte verglichen statt als Text (`…:00.5Z` vs. `…:00Z`,
//! unterschiedliche Offsets); doppelte `sync_id`s in der Remote-Liste ergeben ein Todo statt zwei.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, SecondsFormat, Utc};

use crate::model::{SyncFile, SyncSession, SyncTodo};

/// Ergebnis des Todo-Merges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoMerge {
    /// Lokale Todos in ihrer Reihenfolge (ggf. durch die neuere Remote-Version ersetzt), danach neue Remote-Todos.
    pub merged: Vec<SyncTodo>,
    /// Remote-Todos, die lokal fehlten.
    pub added_by_remote: Vec<SyncTodo>,
    /// Remote-Versionen, die eine lokale Version ersetzt haben (Konflikte).
    pub conflicted_by_remote: Vec<SyncTodo>,
}

/// Ergebnis des Session-Merges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMerge {
    pub all: Vec<SyncSession>,
    pub new_from_remote: Vec<SyncSession>,
}

/// Zeitpunkt aus einem RFC-3339-Text; leer oder ungültig = unbekannt (gilt als ältester Wert).
fn instant(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

/// Gleiche Zeitpunkte in unterschiedlicher Schreibweise ergeben denselben Text; Unlesbares bleibt unverändert.
fn canonical(text: &str) -> String {
    instant(text).map_or_else(
        || text.to_owned(),
        |t| t.to_rfc3339_opts(SecondsFormat::Nanos, true),
    )
}

/// Last-Write-Wins für Todos. Remote-Todos ohne `sync_id` werden übersprungen.
pub fn merge_todos(local: &[SyncTodo], remote: &[SyncTodo]) -> TodoMerge {
    let mut merged: Vec<SyncTodo> = local.to_vec();
    let mut index: HashMap<String, usize> = HashMap::new();
    for (position, todo) in local.iter().enumerate() {
        index.insert(todo.sync_id.clone(), position);
    }
    let mut added_by_remote = Vec::new();
    let mut conflicted_by_remote = Vec::new();

    for remote_todo in remote.iter().filter(|todo| !todo.sync_id.is_empty()) {
        match index.get(&remote_todo.sync_id) {
            None => {
                index.insert(remote_todo.sync_id.clone(), merged.len());
                merged.push(remote_todo.clone());
                added_by_remote.push(remote_todo.clone());
            }
            Some(&position) => {
                if instant(&remote_todo.updated_at) > instant(&merged[position].updated_at) {
                    merged[position] = remote_todo.clone();
                    conflicted_by_remote.push(remote_todo.clone());
                }
            }
        }
    }
    TodoMerge {
        merged,
        added_by_remote,
        conflicted_by_remote,
    }
}

fn session_key(session: &SyncSession) -> (String, String) {
    (session.device_id.clone(), canonical(&session.started_at))
}

/// Inhalt ohne Gerät: für Sessions, die auf zwei Geräten mit unterschiedlicher `device_id` entstanden sind.
fn session_fingerprint(
    session: &SyncSession,
) -> (
    String,
    String,
    String,
    String,
    Option<String>,
    u32,
    u32,
    u32,
) {
    (
        session.mode.clone(),
        session.session_type.clone(),
        session.status.clone(),
        canonical(&session.started_at),
        session.ended_at.as_deref().map(canonical),
        session.planned_min,
        session.actual_min,
        session.break_min,
    )
}

/// Append-only für Sessions: lokale bleiben unverändert, nur wirklich neue Remote-Sessions kommen dazu.
pub fn merge_sessions(local: &[SyncSession], remote: &[SyncSession]) -> SessionMerge {
    let mut keys: HashSet<_> = local.iter().map(session_key).collect();
    let mut fingerprints: HashSet<_> = local.iter().map(session_fingerprint).collect();
    let mut all = local.to_vec();
    let mut new_from_remote = Vec::new();

    for session in remote {
        let key = session_key(session);
        let fingerprint = session_fingerprint(session);
        if keys.contains(&key) || fingerprints.contains(&fingerprint) {
            continue;
        }
        keys.insert(key);
        fingerprints.insert(fingerprint);
        all.push(session.clone());
        new_from_remote.push(session.clone());
    }
    SessionMerge {
        all,
        new_from_remote,
    }
}

/// Führt zwei Sync-Dateien zusammen. Kopfdaten (`schema_version`, `synced_at`, `device_id`) bleiben die der
/// lokalen Datei.
pub fn merge_sync_data(local: &SyncFile, remote: &SyncFile) -> SyncFile {
    SyncFile {
        todos: merge_todos(&local.todos, &remote.todos).merged,
        sessions: merge_sessions(&local.sessions, &remote.sessions).all,
        ..local.clone()
    }
}
