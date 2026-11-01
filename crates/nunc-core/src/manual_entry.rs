//! Prüfung manueller Zeiteinträge (ADR 0017), pur. „Jetzt“ kommt vom Aufrufer (injizierte `Clock`).

use chrono::{DateTime, Duration, Utc};

use crate::model::Session;
use crate::timeline::{span, spans_overlap};

/// Längster erlaubter Eintrag.
pub const MAX_ENTRY_MINUTES: i64 = 24 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EntryError {
    #[error("Das Ende muss nach dem Beginn liegen.")]
    EndNotAfterStart,
    #[error("Ein Eintrag darf höchstens 24 Stunden lang sein.")]
    TooLong,
    #[error("Das Ende darf nicht in der Zukunft liegen.")]
    InFuture,
}

/// Prüft einen Eintrag von `start` bis `end` und liefert die Ids der Sessions, die er überlappt (Warnung, kein
/// Fehler). `ignore_id` nimmt die bearbeitete Session selbst aus der Überlappungsprüfung.
pub fn validate_entry(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    now: DateTime<Utc>,
    existing: &[Session],
    ignore_id: Option<i32>,
) -> Result<Vec<i32>, EntryError> {
    if end <= start {
        return Err(EntryError::EndNotAfterStart);
    }
    if end - start > Duration::minutes(MAX_ENTRY_MINUTES) {
        return Err(EntryError::TooLong);
    }
    if end > now {
        return Err(EntryError::InFuture);
    }
    Ok(existing
        .iter()
        .filter(|session| Some(session.id) != ignore_id)
        .filter(|session| spans_overlap((start, end), span(session)))
        .map(|session| session.id)
        .collect())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::model::{SessionStatus, SessionType, TimerMode};

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
    }

    fn session(id: i32, start: DateTime<Utc>, minutes: u32) -> Session {
        Session {
            id,
            mode: TimerMode::Flowtime,
            session_type: SessionType::Work,
            status: SessionStatus::Completed,
            started_at: start,
            ended_at: None,
            planned_min: None,
            actual_min: Some(minutes),
            todo_id: None,
            break_min: None,
            project_tags: Vec::new(),
            context_tags: Vec::new(),
        }
    }

    #[test]
    fn the_boundaries_are_inclusive_where_they_should_be() {
        let now = at(12, 0);
        assert_eq!(
            validate_entry(at(9, 0), at(9, 0), now, &[], None),
            Err(EntryError::EndNotAfterStart)
        );
        assert_eq!(
            validate_entry(at(9, 0), at(9, 1), now, &[], None),
            Ok(vec![])
        );
        // genau 24 h sind erlaubt, eine Minute mehr nicht
        let later = Utc.with_ymd_and_hms(2026, 3, 4, 0, 0, 0).unwrap();
        let start = Utc.with_ymd_and_hms(2026, 3, 3, 0, 0, 0).unwrap();
        assert_eq!(validate_entry(start, later, later, &[], None), Ok(vec![]));
        assert_eq!(
            validate_entry(start - Duration::minutes(1), later, later, &[], None),
            Err(EntryError::TooLong)
        );
        // ein Ende genau jetzt ist keine Zukunft
        assert_eq!(validate_entry(at(11, 0), now, now, &[], None), Ok(vec![]));
        assert_eq!(
            validate_entry(at(11, 0), at(12, 1), now, &[], None),
            Err(EntryError::InFuture)
        );
    }

    #[test]
    fn overlaps_are_reported_but_the_edited_session_is_ignored() {
        let existing = [session(1, at(9, 0), 60), session(2, at(10, 0), 30)];
        let now = at(12, 0);
        assert_eq!(
            validate_entry(at(9, 30), at(10, 15), now, &existing, None),
            Ok(vec![1, 2])
        );
        assert_eq!(
            validate_entry(at(9, 30), at(10, 15), now, &existing, Some(1)),
            Ok(vec![2])
        );
        assert_eq!(
            validate_entry(at(10, 30), at(11, 0), now, &existing, None),
            Ok(vec![])
        );
    }
}
