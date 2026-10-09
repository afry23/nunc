//! Tages-Timeline (ADR 0017): die Sessions eines lokalen Tages als Blöcke auf einer Zeitachse, mit Lücken und
//! Überlappungen. Pur; der Aufrufer liefert die Zuordnung Zeitpunkt → lokaler Tag.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::crdt::ORIGIN_TIMER;
use crate::model::{Session, SessionStatus, SessionType};

/// Ein Block der Timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TimelineBlock {
    pub session_id: i32,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub todo_id: Option<i32>,
    /// Notiz der Session; [`day_timeline`] kennt sie nicht und lässt sie leer, der Aufrufer füllt sie nach.
    pub note: String,
    /// `timer` oder `manual`, siehe `note`.
    pub origin: String,
    /// Der Block überlappt mit mindestens einem anderen Block des Tages.
    pub overlaps: bool,
}

/// Zeit ohne Session zwischen zwei Blöcken.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TimelineGap {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub minutes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DayTimeline {
    pub date: NaiveDate,
    pub blocks: Vec<TimelineBlock>,
    pub gaps: Vec<TimelineGap>,
}

/// Beginn und Ende einer Session: `ended_at`, sonst Beginn + `actual_min`.
pub fn span(session: &Session) -> (DateTime<Utc>, DateTime<Utc>) {
    let end = session.ended_at.unwrap_or_else(|| {
        session.started_at + Duration::minutes(i64::from(session.actual_min.unwrap_or(0)))
    });
    (session.started_at, end.max(session.started_at))
}

/// Überlappen sich zwei Zeitspannen? Aneinandergrenzende (Ende = Beginn) überlappen nicht.
pub fn spans_overlap(a: (DateTime<Utc>, DateTime<Utc>), b: (DateTime<Utc>, DateTime<Utc>)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

/// Die Timeline des lokalen Tages `date`. Eine Session gehört zu dem Tag, an dem sie begann (auch über
/// Mitternacht hinaus). Blöcke nach Beginn, bei Gleichstand nach `id`.
pub fn day_timeline(
    sessions: &[Session],
    date: NaiveDate,
    local_date: &dyn Fn(DateTime<Utc>) -> NaiveDate,
) -> DayTimeline {
    let mut blocks: Vec<TimelineBlock> = sessions
        .iter()
        .filter(|session| local_date(session.started_at) == date)
        .map(|session| {
            let (start, end) = span(session);
            TimelineBlock {
                session_id: session.id,
                start,
                end,
                session_type: session.session_type,
                status: session.status,
                todo_id: session.todo_id,
                note: String::new(),
                origin: ORIGIN_TIMER.to_owned(),
                overlaps: false,
            }
        })
        .collect();
    blocks.sort_by_key(|block| (block.start, block.session_id));

    for i in 0..blocks.len() {
        for j in (i + 1)..blocks.len() {
            if spans_overlap(
                (blocks[i].start, blocks[i].end),
                (blocks[j].start, blocks[j].end),
            ) {
                blocks[i].overlaps = true;
                blocks[j].overlaps = true;
            }
        }
    }

    let mut gaps = Vec::new();
    let mut covered_until: Option<DateTime<Utc>> = None;
    for block in &blocks {
        if let Some(until) = covered_until.filter(|until| *until < block.start) {
            gaps.push(TimelineGap {
                start: until,
                end: block.start,
                minutes: u32::try_from((block.start - until).num_minutes()).unwrap_or(u32::MAX),
            });
        }
        covered_until = Some(covered_until.map_or(block.end, |until| until.max(block.end)));
    }
    DayTimeline { date, blocks, gaps }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::model::TimerMode;

    fn at(text: &str) -> DateTime<Utc> {
        Utc.from_utc_datetime(
            &chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M").unwrap(),
        )
    }

    fn session(id: i32, start: &str, minutes: u32) -> Session {
        Session {
            id,
            mode: TimerMode::Flowtime,
            session_type: SessionType::Work,
            status: SessionStatus::Completed,
            started_at: at(start),
            ended_at: None,
            planned_min: None,
            actual_min: Some(minutes),
            todo_id: None,
            break_min: None,
        }
    }

    fn day(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
    }

    fn utc_day(time: DateTime<Utc>) -> NaiveDate {
        time.date_naive()
    }

    #[test]
    fn the_end_is_the_recorded_end_or_start_plus_minutes() {
        let mut open = session(1, "2026-03-02 09:00", 30);
        assert_eq!(span(&open).1, at("2026-03-02 09:30"));
        open.ended_at = Some(at("2026-03-02 09:20"));
        assert_eq!(span(&open).1, at("2026-03-02 09:20"));
        open.ended_at = Some(at("2026-03-02 08:00"));
        assert_eq!(span(&open).1, open.started_at);
    }

    #[test]
    fn touching_spans_do_not_overlap() {
        let a = (at("2026-03-02 09:00"), at("2026-03-02 09:30"));
        let b = (at("2026-03-02 09:30"), at("2026-03-02 10:00"));
        assert!(!spans_overlap(a, b));
        let c = (at("2026-03-02 09:29"), at("2026-03-02 10:00"));
        assert!(spans_overlap(a, c) && spans_overlap(c, a));
    }

    #[test]
    fn a_gap_is_measured_from_the_latest_end_so_far() {
        let sessions = [
            session(1, "2026-03-02 09:00", 120),
            session(2, "2026-03-02 09:30", 10),
            session(3, "2026-03-02 12:00", 10),
        ];
        let timeline = day_timeline(&sessions, day("2026-03-02"), &utc_day);
        assert_eq!(timeline.gaps.len(), 1);
        assert_eq!(timeline.gaps[0].minutes, 60);
        assert!(timeline.blocks[0].overlaps && timeline.blocks[1].overlaps);
        assert!(!timeline.blocks[2].overlaps);
    }
}
