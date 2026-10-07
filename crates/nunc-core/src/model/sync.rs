//! Austauschformat `nunc-sync.json` (Go: `models.SyncFile`). Feldnamen snake_case wie in der Go-App; Zeitangaben
//! bleiben als Text erhalten, damit Lesen und Schreiben verlustfrei ist (die Vergleiche parsen sie).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

/// Die Sync-Datei auf dem Server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncFile {
    pub schema_version: u32,
    pub synced_at: DateTime<Utc>,
    pub device_id: String,
    #[serde(default, deserialize_with = "null_as_empty")]
    pub todos: Vec<SyncTodo>,
    #[serde(default, deserialize_with = "null_as_empty")]
    pub sessions: Vec<SyncSession>,
}

/// Ein Todo in der Sync-Datei. `sync_id` ist die geräteübergreifende Identität.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncTodo {
    pub sync_id: String,
    /// Ursprungsgerät.
    pub device_id: String,
    pub raw_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    pub completed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub description: String,
    #[serde(default, deserialize_with = "null_as_empty")]
    pub project_tags: Vec<String>,
    #[serde(default, deserialize_with = "null_as_empty")]
    pub context_tags: Vec<String>,
    pub pomodoro_count: u32,
    pub position: u32,
    /// RFC 3339; leer bei Todos, die vor der Sync-Einführung entstanden.
    pub updated_at: String,
    pub deleted: bool,
}

/// Eine Session in der Sync-Datei; Sessions sind append-only und werden über `(device_id, started_at)`
/// identifiziert.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SyncSession {
    pub device_id: String,
    pub mode: String,
    pub session_type: String,
    pub status: String,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    pub planned_min: u32,
    pub actual_min: u32,
    pub break_min: u32,
}

/// Go schreibt `nil`-Listen als `null`.
fn null_as_empty<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}
