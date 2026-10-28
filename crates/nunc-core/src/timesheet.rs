//! Zeiterfassung: Arbeitszeit je Projekt, Kontext oder Aufgabe und Tag (ADR 0017), pur aus den Roh-Sessions.
//!
//! Zählregeln wie in [`crate::stats`]: Nur Arbeitssessions zählen (Pausen nicht), abgeschlossene und abgebrochene,
//! mit `actual_min`. Ein Tag ist der lokale Kalendertag des Beginns (`local_date` kommt vom Aufrufer). Tags und
//! Beschreibung stammen vom **aktuellen** Todo. Hat ein Todo mehrere `+projekt`- oder `@kontext`-Tags, zählt die
//! Zeit in jeder Zeile, die Tages- und Gesamtsummen zählen jede Session aber nur einmal.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::{Session, SessionType, Todo};

/// Wonach die Zeilen des Timesheets gruppiert werden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    Project,
    Context,
    Todo,
}

/// Eine Zeile: eine Gruppe mit ihren Minuten je Tag (gleiche Reihenfolge wie [`Timesheet::days`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TimesheetRow {
    /// Tag (`+projekt` bzw. `@kontext`) oder Beschreibung des Todos; `None` für Sessions ohne Tag bzw. Todo.
    pub group: Option<String>,
    /// Nur bei Gruppierung nach Todo.
    pub todo_id: Option<i32>,
    pub minutes: Vec<u32>,
    pub total: u32,
}

/// Wochen- oder Zeitraumbericht: Zeilen × Tage mit Summen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Timesheet {
    pub days: Vec<NaiveDate>,
    pub rows: Vec<TimesheetRow>,
    pub day_totals: Vec<u32>,
    pub total: u32,
}

/// Berechnet das Timesheet für die lokalen Tage `start` bis `end` (inklusive); ist `end` vor `start`, ist es leer.
/// Zeilen: die Zeile „ohne“ ganz unten, sonst größte Summe zuerst, dann nach Name.
pub fn timesheet(
    sessions: &[Session],
    todos: &[Todo],
    start: NaiveDate,
    end: NaiveDate,
    group_by: GroupBy,
    local_date: &dyn Fn(DateTime<Utc>) -> NaiveDate,
) -> Timesheet {
    let days: Vec<NaiveDate> = (0..)
        .map(|offset| start + Duration::days(offset))
        .take_while(|day| *day <= end)
        .collect();
    let todo_by_id: HashMap<i32, &Todo> = todos.iter().map(|todo| (todo.id, todo)).collect();

    let mut rows: BTreeMap<(Option<String>, Option<i32>), Vec<u32>> = BTreeMap::new();
    let mut day_totals = vec![0u32; days.len()];
    for session in sessions
        .iter()
        .filter(|session| session.session_type == SessionType::Work)
    {
        let date = local_date(session.started_at);
        if date < start || date > end {
            continue;
        }
        let index = usize::try_from((date - start).num_days()).unwrap_or(0);
        let minutes = session.actual_min.unwrap_or(0);
        day_totals[index] += minutes;
        let todo = session.todo_id.and_then(|id| todo_by_id.get(&id).copied());
        for key in group_keys(todo, group_by) {
            rows.entry(key).or_insert_with(|| vec![0; days.len()])[index] += minutes;
        }
    }

    let mut rows: Vec<TimesheetRow> = rows
        .into_iter()
        .map(|((group, todo_id), minutes)| TimesheetRow {
            group,
            todo_id,
            total: minutes.iter().sum(),
            minutes,
        })
        .collect();
    rows.sort_by(|a, b| {
        a.group
            .is_none()
            .cmp(&b.group.is_none())
            .then_with(|| b.total.cmp(&a.total))
            .then_with(|| a.group.cmp(&b.group))
            .then_with(|| a.todo_id.cmp(&b.todo_id))
    });
    Timesheet {
        total: day_totals.iter().sum(),
        days,
        rows,
        day_totals,
    }
}

fn group_keys(todo: Option<&Todo>, group_by: GroupBy) -> Vec<(Option<String>, Option<i32>)> {
    let tags = |tags: &[String]| -> Vec<(Option<String>, Option<i32>)> {
        if tags.is_empty() {
            vec![(None, None)]
        } else {
            tags.iter().map(|tag| (Some(tag.clone()), None)).collect()
        }
    };
    match (group_by, todo) {
        (_, None) => vec![(None, None)],
        (GroupBy::Project, Some(todo)) => tags(&todo.project_tags),
        (GroupBy::Context, Some(todo)) => tags(&todo.context_tags),
        (GroupBy::Todo, Some(todo)) => vec![(Some(todo.description.clone()), Some(todo.id))],
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::model::{SessionStatus, TimerMode};

    fn day(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
    }

    fn utc_day(time: DateTime<Utc>) -> NaiveDate {
        time.date_naive()
    }

    fn work(date: &str, minutes: u32, todo_id: Option<i32>) -> Session {
        Session {
            id: 0,
            mode: TimerMode::Flowtime,
            session_type: SessionType::Work,
            status: SessionStatus::Completed,
            started_at: Utc.from_utc_datetime(&day(date).and_hms_opt(10, 0, 0).unwrap()),
            ended_at: None,
            planned_min: None,
            actual_min: Some(minutes),
            todo_id,
            break_min: None,
        }
    }

    fn todo(id: i32, description: &str, projects: &[&str], contexts: &[&str]) -> Todo {
        Todo {
            id,
            raw_text: description.to_owned(),
            priority: None,
            completed: false,
            completed_at: None,
            created_at: None,
            description: description.to_owned(),
            project_tags: projects.iter().map(|tag| (*tag).to_owned()).collect(),
            context_tags: contexts.iter().map(|tag| (*tag).to_owned()).collect(),
            pomodoro_count: 0,
            position: 0,
            parent_id: None,
        }
    }

    #[test]
    fn a_session_with_two_projects_counts_in_both_rows_but_once_in_the_totals() {
        let todos = [todo(1, "Both +a +b", &["+a", "+b"], &[])];
        let sheet = timesheet(
            &[work("2026-03-02", 30, Some(1))],
            &todos,
            day("2026-03-02"),
            day("2026-03-03"),
            GroupBy::Project,
            &utc_day,
        );
        assert_eq!(sheet.rows.len(), 2);
        assert!(sheet.rows.iter().all(|row| row.minutes == [30, 0]));
        assert_eq!(sheet.day_totals, [30, 0]);
        assert_eq!(sheet.total, 30);
    }

    #[test]
    fn rows_are_sorted_by_total_then_name_with_the_untagged_row_last() {
        let todos = [todo(1, "A +b", &["+b"], &[]), todo(2, "B +a", &["+a"], &[])];
        let sessions = [
            work("2026-03-02", 10, Some(1)),
            work("2026-03-02", 10, Some(2)),
            work("2026-03-02", 90, None),
            work("2026-03-02", 20, Some(1)),
        ];
        let sheet = timesheet(
            &sessions,
            &todos,
            day("2026-03-02"),
            day("2026-03-02"),
            GroupBy::Project,
            &utc_day,
        );
        let groups: Vec<_> = sheet.rows.iter().map(|row| row.group.as_deref()).collect();
        assert_eq!(groups, [Some("+b"), Some("+a"), None]);
    }

    #[test]
    fn an_end_before_the_start_gives_an_empty_sheet() {
        let sheet = timesheet(
            &[work("2026-03-02", 30, None)],
            &[],
            day("2026-03-03"),
            day("2026-03-02"),
            GroupBy::Todo,
            &utc_day,
        );
        assert!(sheet.days.is_empty() && sheet.rows.is_empty());
        assert_eq!(sheet.total, 0);
    }

    #[test]
    fn a_session_on_the_last_day_is_included_and_one_after_it_is_not() {
        let sessions = [work("2026-03-03", 5, None), work("2026-03-04", 7, None)];
        let sheet = timesheet(
            &sessions,
            &[],
            day("2026-03-02"),
            day("2026-03-03"),
            GroupBy::Context,
            &utc_day,
        );
        assert_eq!(sheet.day_totals, [0, 5]);
    }
}
