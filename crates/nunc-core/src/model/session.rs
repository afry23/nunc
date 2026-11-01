use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::{TimerMode, text_enum};

/// Art eines Intervalls innerhalb eines Zyklus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

text_enum!(SessionType, "SessionType", {
    Work => "work",
    ShortBreak => "short_break",
    LongBreak => "long_break",
});

impl SessionType {
    pub fn is_break(self) -> bool {
        !matches!(self, Self::Work)
    }
}

/// Ergebnis einer Session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Completed,
    Aborted,
    Skipped,
}

text_enum!(SessionStatus, "SessionStatus", {
    Completed => "completed",
    Aborted => "aborted",
    Skipped => "skipped",
});

/// Gespeichertes Arbeits- oder Pausenintervall.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i32,
    pub mode: TimerMode,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    /// Zieldauer in Minuten (nur Pomodoro).
    pub planned_min: Option<u32>,
    /// Tatsächlich verstrichene Minuten.
    pub actual_min: Option<u32>,
    pub todo_id: Option<i32>,
    /// Empfohlene bzw. genutzte Pausendauer in Minuten.
    pub break_min: Option<u32>,
    /// Eigene Projekte (`+name`); zählen nur, wenn die Session kein Todo hat (ADR 0023).
    #[serde(default)]
    pub project_tags: Vec<String>,
    /// Eigene Kontexte (`@name`); zählen nur, wenn die Session kein Todo hat (ADR 0023).
    #[serde(default)]
    pub context_tags: Vec<String>,
}

/// Zuordnung einer Session ohne Todo: eigene Projekte, Kontexte und eine Notiz (ADR 0023).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Labels {
    pub project_tags: Vec<String>,
    pub context_tags: Vec<String>,
    pub note: String,
}

impl Labels {
    /// Normalisiert die Tags (`+`/`@`) und trimmt die Notiz.
    pub fn normalized(&self) -> Self {
        Self {
            project_tags: normalize_tags(&self.project_tags, '+'),
            context_tags: normalize_tags(&self.context_tags, '@'),
            note: self.note.trim().to_owned(),
        }
    }
}

/// Vereinheitlicht Tags: trimmt, setzt `prefix` voran, wenn er fehlt, verwirft leere Tags (nur Präfix) und Tags
/// mit Leerraum, entfernt Duplikate und behält die Reihenfolge.
pub fn normalize_tags<S: AsRef<str>>(raw: &[S], prefix: char) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tag in raw {
        let word = tag.as_ref().trim();
        let name = word.strip_prefix(prefix).unwrap_or(word);
        if name.is_empty() || name.chars().any(char::is_whitespace) {
            continue;
        }
        let tag = format!("{prefix}{name}");
        if !out.contains(&tag) {
            out.push(tag);
        }
    }
    out
}

/// Alle bekannten Projekte und Kontexte (Vorschläge für die Auswahl), sortiert und ohne Duplikate.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct KnownTags {
    pub projects: Vec<String>,
    pub contexts: Vec<String>,
}

/// Sammelt die Tags aller Todos und aller Sessions.
pub fn known_tags(sessions: &[Session], todos: &[crate::model::Todo]) -> KnownTags {
    use std::collections::BTreeSet;
    let mut projects = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    for todo in todos {
        projects.extend(todo.project_tags.iter().cloned());
        contexts.extend(todo.context_tags.iter().cloned());
    }
    for session in sessions {
        projects.extend(session.project_tags.iter().cloned());
        contexts.extend(session.context_tags.iter().cloned());
    }
    KnownTags {
        projects: projects.into_iter().collect(),
        contexts: contexts.into_iter().collect(),
    }
}

/// Projekte und Kontexte, die für eine Session zählen: die des Todos, sonst die eigenen (ADR 0023).
pub fn effective_tags<'a>(
    session: &'a Session,
    todo: Option<&'a crate::model::Todo>,
) -> (&'a [String], &'a [String]) {
    match todo {
        Some(todo) => (&todo.project_tags, &todo.context_tags),
        None => (&session.project_tags, &session.context_tags),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_values_match_the_database() {
        assert_eq!(SessionType::Work.as_str(), "work");
        assert_eq!(SessionType::ShortBreak.as_str(), "short_break");
        assert_eq!(SessionType::LongBreak.as_str(), "long_break");
        assert_eq!(SessionStatus::Completed.as_str(), "completed");
        assert_eq!(SessionStatus::Aborted.as_str(), "aborted");
        assert_eq!(SessionStatus::Skipped.as_str(), "skipped");
    }

    #[test]
    fn parses_every_text_value() {
        for t in [
            SessionType::Work,
            SessionType::ShortBreak,
            SessionType::LongBreak,
        ] {
            assert_eq!(t.as_str().parse::<SessionType>(), Ok(t));
        }
        for s in [
            SessionStatus::Completed,
            SessionStatus::Aborted,
            SessionStatus::Skipped,
        ] {
            assert_eq!(s.as_str().parse::<SessionStatus>(), Ok(s));
        }
        assert!("long-break".parse::<SessionType>().is_err());
    }

    #[test]
    fn only_work_is_not_a_break() {
        assert!(!SessionType::Work.is_break());
        assert!(SessionType::ShortBreak.is_break());
        assert!(SessionType::LongBreak.is_break());
    }

    #[test]
    fn reads_the_json_written_by_the_go_app() {
        let json = r#"{
            "id": 7, "mode": "pomodoro", "sessionType": "work", "status": "completed",
            "startedAt": "2026-09-22T09:00:00Z", "endedAt": "2026-09-22T09:30:00Z",
            "plannedMin": 30, "actualMin": 30, "todoId": 1, "breakMin": 6
        }"#;
        let s: Session = serde_json::from_str(json).unwrap();
        assert_eq!(s.id, 7);
        assert_eq!(s.mode, TimerMode::Pomodoro);
        assert_eq!(s.session_type, SessionType::Work);
        assert_eq!(s.status, SessionStatus::Completed);
        assert_eq!(s.started_at.to_rfc3339(), "2026-09-22T09:00:00+00:00");
        assert_eq!(s.planned_min, Some(30));
        assert_eq!(s.todo_id, Some(1));
        assert_eq!(s.break_min, Some(6));
    }

    #[test]
    fn open_sessions_have_null_optional_fields() {
        let json = r#"{
            "id": 1, "mode": "flowtime", "sessionType": "work", "status": "aborted",
            "startedAt": "2026-09-22T09:00:00Z", "endedAt": null,
            "plannedMin": null, "actualMin": null, "todoId": null, "breakMin": null
        }"#;
        let s: Session = serde_json::from_str(json).unwrap();
        assert!(s.ended_at.is_none() && s.planned_min.is_none() && s.todo_id.is_none());
        let back = serde_json::to_value(&s).unwrap();
        assert_eq!(back["sessionType"], "work");
        assert!(back["endedAt"].is_null());
    }

    fn session_with(projects: &[&str], contexts: &[&str]) -> Session {
        Session {
            id: 1,
            mode: crate::model::TimerMode::Stopwatch,
            session_type: SessionType::Work,
            status: SessionStatus::Completed,
            started_at: chrono::DateTime::UNIX_EPOCH,
            ended_at: None,
            planned_min: None,
            actual_min: Some(10),
            todo_id: None,
            break_min: None,
            project_tags: projects.iter().map(|t| (*t).to_owned()).collect(),
            context_tags: contexts.iter().map(|t| (*t).to_owned()).collect(),
        }
    }

    fn todo_with(projects: &[&str], contexts: &[&str]) -> crate::model::Todo {
        crate::model::Todo {
            id: 1,
            raw_text: "Task".to_owned(),
            priority: None,
            completed: false,
            completed_at: None,
            created_at: None,
            description: "Task".to_owned(),
            project_tags: projects.iter().map(|t| (*t).to_owned()).collect(),
            context_tags: contexts.iter().map(|t| (*t).to_owned()).collect(),
            pomodoro_count: 0,
            position: 1,
            parent_id: None,
        }
    }

    #[test]
    fn tags_are_prefixed_trimmed_and_deduplicated() {
        assert_eq!(
            normalize_tags(&["meetings", " +meetings ", "+", "", "a b", "team"], '+'),
            ["+meetings", "+team"]
        );
        assert_eq!(normalize_tags(&["office", "@office"], '@'), ["@office"]);
        assert!(normalize_tags::<&str>(&[], '+').is_empty());
    }

    #[test]
    fn labels_are_normalised_and_the_note_is_trimmed() {
        let labels = Labels {
            project_tags: vec!["a".to_owned()],
            context_tags: vec!["b".to_owned()],
            note: "  Weekly ".to_owned(),
        };
        assert_eq!(
            labels.normalized(),
            Labels {
                project_tags: vec!["+a".to_owned()],
                context_tags: vec!["@b".to_owned()],
                note: "Weekly".to_owned(),
            }
        );
    }

    #[test]
    fn a_todo_takes_precedence_over_the_own_tags() {
        let session = session_with(&["+own"], &["@own"]);
        let todo = todo_with(&["+work"], &["@desk"]);
        assert_eq!(
            effective_tags(&session, Some(&todo)),
            (&["+work".to_owned()][..], &["@desk".to_owned()][..])
        );
        assert_eq!(
            effective_tags(&session, None),
            (&["+own".to_owned()][..], &["@own".to_owned()][..])
        );
    }

    #[test]
    fn known_tags_merge_todos_and_sessions_sorted() {
        let todo = todo_with(&["+work"], &["@desk"]);
        let known = known_tags(
            &[session_with(&["+meetings", "+work"], &["@office"])],
            &[todo],
        );
        assert_eq!(known.projects, ["+meetings", "+work"]);
        assert_eq!(known.contexts, ["@desk", "@office"]);
    }
}
