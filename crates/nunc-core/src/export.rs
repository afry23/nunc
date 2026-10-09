//! Export der Sessions als CSV oder JSON (pur, REQUIREMENTS 11).
//!
//! Spalten wie in der Go-App (`date, start_time, end_time, duration_min, mode, status, todo_text, project_tags,
//! context_tags`), zusätzlich am Ende `session_type`, weil sich Arbeit und Pausen sonst nicht unterscheiden
//! lassen (Go exportierte nur den Modus), sowie `note` und `origin` (`timer` oder `manual`, ADR 0017). Datum und Uhrzeiten sind **Ortszeit**; der Aufrufer liefert die
//! Umrechnung. JSON verwendet dieselben Schlüssel mit echten Typen (Zahlen, Arrays) statt lauter Strings.

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde::Serialize;

use crate::model::{Session, SessionStatus, SessionType, StatsRequest, TimerMode, Todo};

/// Eine Exportzeile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportRow {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: Option<NaiveTime>,
    pub duration_min: Option<u32>,
    pub mode: TimerMode,
    pub status: SessionStatus,
    /// Rohtext des verknüpften Todos, leer ohne Todo.
    pub todo_text: String,
    pub project_tags: Vec<String>,
    pub context_tags: Vec<String>,
    pub session_type: SessionType,
    /// Notiz der Session, leer ohne Notiz.
    pub note: String,
    /// `timer` (aufgezeichnet) oder `manual` (nachgetragen).
    pub origin: String,
}

/// Notiz und Herkunft einer Session; Sessions ohne Eintrag gelten als aufgezeichnet und ohne Notiz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportDetails {
    pub note: String,
    pub origin: String,
}

const ORIGIN_TIMER: &str = "timer";

const CSV_HEADER: [&str; 12] = [
    "date",
    "start_time",
    "end_time",
    "duration_min",
    "mode",
    "status",
    "todo_text",
    "project_tags",
    "context_tags",
    "session_type",
    "note",
    "origin",
];

/// Wählt die Sessions im Zeitraum (inklusive, nach lokalem Startdatum) und sortiert sie nach Startzeit.
pub fn export_rows(
    sessions: &[Session],
    todos: &[Todo],
    request: &StatsRequest,
    details: &HashMap<i32, ExportDetails>,
    to_local: &dyn Fn(DateTime<Utc>) -> NaiveDateTime,
) -> Vec<ExportRow> {
    let todo_by_id: HashMap<i32, &Todo> = todos.iter().map(|todo| (todo.id, todo)).collect();
    let mut selected: Vec<&Session> = sessions
        .iter()
        .filter(|session| {
            let date = to_local(session.started_at).date();
            date >= request.start_date && date <= request.end_date
        })
        .collect();
    selected.sort_by_key(|session| (session.started_at, session.id));

    selected
        .into_iter()
        .map(|session| {
            let start = to_local(session.started_at);
            let todo = session.todo_id.and_then(|id| todo_by_id.get(&id));
            let detail = details.get(&session.id);
            ExportRow {
                date: start.date(),
                start_time: start.time(),
                end_time: session.ended_at.map(|end| to_local(end).time()),
                duration_min: session.actual_min,
                mode: session.mode,
                status: session.status,
                todo_text: todo.map(|t| t.raw_text.clone()).unwrap_or_default(),
                project_tags: todo.map(|t| t.project_tags.clone()).unwrap_or_default(),
                context_tags: todo.map(|t| t.context_tags.clone()).unwrap_or_default(),
                session_type: session.session_type,
                note: detail.map(|d| d.note.clone()).unwrap_or_default(),
                origin: detail.map_or_else(|| ORIGIN_TIMER.to_owned(), |d| d.origin.clone()),
            }
        })
        .collect()
}

/// CSV nach RFC 4180 (Zeilenende `\n`): Kopfzeile plus eine Zeile je Session.
///
/// Felder mit Komma, Anführungszeichen oder Zeilenumbruch werden in Anführungszeichen gesetzt. Die freien
/// Textspalten (`todo_text`, `project_tags`, `context_tags`, `note`) bekommen ein `'` vorangestellt, wenn sie mit
/// `=`, `+`, `-`, `@`, Tab oder Zeilenumbruch beginnen, damit Tabellenprogramme sie nicht als Formel auswerten
/// (CSV-Injection). Tags stehen mit Präfix und durch Leerzeichen getrennt in einer Zelle.
pub fn to_csv(rows: &[ExportRow]) -> String {
    let mut out = CSV_HEADER.join(",");
    out.push('\n');
    for row in rows {
        let fields = [
            row.date.format("%Y-%m-%d").to_string(),
            row.start_time.format("%H:%M:%S").to_string(),
            row.end_time
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_default(),
            row.duration_min.map(|m| m.to_string()).unwrap_or_default(),
            row.mode.as_str().to_owned(),
            row.status.as_str().to_owned(),
            guard_formula(&row.todo_text),
            guard_formula(&row.project_tags.join(" ")),
            guard_formula(&row.context_tags.join(" ")),
            row.session_type.as_str().to_owned(),
            guard_formula(&row.note),
            row.origin.clone(),
        ];
        let line: Vec<String> = fields.iter().map(|field| csv_escape(field)).collect();
        out.push_str(&line.join(","));
        out.push('\n');
    }
    out
}

/// JSON-Liste (zwei Leerzeichen Einrückung), bei keinen Sessions `[]`.
pub fn to_json(rows: &[ExportRow]) -> serde_json::Result<String> {
    serde_json::to_string_pretty(rows)
}

fn csv_escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_owned()
    }
}

fn guard_formula(field: &str) -> String {
    if field.starts_with(['=', '+', '-', '@', '\t', '\r', '\n']) {
        format!("'{field}")
    } else {
        field.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_only_when_needed() {
        assert_eq!(csv_escape("plain text"), "plain text");
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_escape("two\nlines"), "\"two\nlines\"");
        assert_eq!(csv_escape("cr\rhere"), "\"cr\rhere\"");
        assert_eq!(csv_escape(""), "");
    }

    #[test]
    fn guards_formula_prefixes() {
        for prefix in ["=", "+", "-", "@", "\t", "\r", "\n"] {
            assert_eq!(guard_formula(&format!("{prefix}x")), format!("'{prefix}x"));
        }
        assert_eq!(guard_formula("x=1"), "x=1");
        assert_eq!(guard_formula(""), "");
        assert_eq!(guard_formula("(A) task"), "(A) task");
    }
}
