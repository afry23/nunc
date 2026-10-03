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
}
