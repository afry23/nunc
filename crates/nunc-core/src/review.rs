//! Rückblick-Bericht für einen Zeitraum (ADR 0025), pur aus den Roh-Sessions und Todos.
//!
//! Fokuszeit zählt nur **abgeschlossene** Arbeitssessions; abgebrochene gehen in die Abbruchquote ein.
//! Ein Tag ist der lokale Kalendertag des Beginns (`local_date` kommt vom Aufrufer).

use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::{Session, SessionStatus, SessionType, Todo, effective_tags};

/// Art eines Kalenderzeitraums.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PeriodKind {
    Week,
    Month,
    Quarter,
}

/// Zeitraum in lokalen Tagen, beide Grenzen inklusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

/// Summe einer Gruppe (`+projekt`, `@kontext`); `None` für Zeit ohne Tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GroupMinutes {
    pub group: Option<String>,
    pub minutes: u32,
}

/// Im Zeitraum erledigte Aufgabe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CompletedTodo {
    pub completed_at: NaiveDate,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub period: Period,
    pub previous_period: Period,
    pub focus_min: u32,
    pub previous_focus_min: u32,
    pub completed_sessions: u32,
    pub aborted_sessions: u32,
    pub abort_rate_percent: u32,
    pub projects: Vec<GroupMinutes>,
    pub contexts: Vec<GroupMinutes>,
    pub completed_todos: Vec<CompletedTodo>,
}

/// Erster Tag des Monats `month0` (0 = Januar), auch über Jahresgrenzen hinweg (`month0` darf negativ sein).
fn first_of_month(year: i32, month0: i32) -> Option<NaiveDate> {
    let year = year + month0.div_euclid(12);
    let month = u32::try_from(month0.rem_euclid(12)).ok()? + 1;
    NaiveDate::from_ymd_opt(year, month, 1)
}

/// Kalenderzeitraum (Woche ab Montag, Monat, Quartal), der `day` enthält.
pub fn period_containing(kind: PeriodKind, day: NaiveDate) -> Period {
    let months = |length: i32, start_month0: i32| {
        let from = first_of_month(day.year(), start_month0).unwrap_or(day);
        let to = first_of_month(day.year(), start_month0 + length)
            .map_or(day, |next| next - Duration::days(1));
        Period { from, to }
    };
    let month0 = i32::try_from(day.month0()).unwrap_or(0);
    match kind {
        PeriodKind::Week => {
            let from = day - Duration::days(i64::from(day.weekday().num_days_from_monday()));
            Period {
                from,
                to: from + Duration::days(6),
            }
        }
        PeriodKind::Month => months(1, month0),
        PeriodKind::Quarter => months(3, month0 - month0 % 3),
    }
}

/// Vorperiode: bei einem ganzen Kalendermonat oder -quartal der vorherige, sonst der Zeitraum gleicher Länge davor.
fn previous_period(period: Period) -> Period {
    let day_before = period.from - Duration::days(1);
    for kind in [PeriodKind::Month, PeriodKind::Quarter] {
        if period_containing(kind, period.from) == period {
            return period_containing(kind, day_before);
        }
    }
    let length = (period.to - period.from).num_days().max(0) + 1;
    Period {
        from: period.from - Duration::days(length),
        to: day_before,
    }
}

fn contains(period: Period, day: NaiveDate) -> bool {
    period.from <= day && day <= period.to
}

/// Zeilen: größte Summe zuerst, dann nach Name, Zeit ohne Tag ganz unten.
fn group_totals(totals: BTreeMap<Option<String>, u32>) -> Vec<GroupMinutes> {
    let mut groups: Vec<GroupMinutes> = totals
        .into_iter()
        .map(|(group, minutes)| GroupMinutes { group, minutes })
        .collect();
    groups.sort_by(|a, b| {
        a.group
            .is_none()
            .cmp(&b.group.is_none())
            .then_with(|| b.minutes.cmp(&a.minutes))
            .then_with(|| a.group.cmp(&b.group))
    });
    groups
}

fn add_tags(totals: &mut BTreeMap<Option<String>, u32>, tags: &[String], minutes: u32) {
    if tags.is_empty() {
        *totals.entry(None).or_default() += minutes;
    }
    for tag in tags {
        *totals.entry(Some(tag.clone())).or_default() += minutes;
    }
}

/// Fokuszeit abgeschlossener Arbeitssessions, die im Zeitraum begannen.
fn focus_minutes(
    period: Period,
    sessions: &[Session],
    local_date: &dyn Fn(DateTime<Utc>) -> NaiveDate,
) -> u32 {
    sessions
        .iter()
        .filter(|session| {
            session.session_type == SessionType::Work
                && session.status == SessionStatus::Completed
                && contains(period, local_date(session.started_at))
        })
        .map(|session| session.actual_min.unwrap_or(0))
        .sum()
}

/// Berechnet den Bericht; die Vorperiode ist der vorherige Kalendermonat/-quartal bzw. der Zeitraum gleicher Länge davor.
pub fn build_review(
    period: Period,
    sessions: &[Session],
    todos: &[Todo],
    local_date: &dyn Fn(DateTime<Utc>) -> NaiveDate,
) -> Review {
    let previous = previous_period(period);
    let mut projects = BTreeMap::new();
    let mut contexts = BTreeMap::new();
    let (mut completed_sessions, mut aborted_sessions) = (0u32, 0u32);
    for session in sessions.iter().filter(|session| {
        session.session_type == SessionType::Work
            && contains(period, local_date(session.started_at))
    }) {
        match session.status {
            SessionStatus::Completed => completed_sessions += 1,
            SessionStatus::Aborted => {
                aborted_sessions += 1;
                continue;
            }
            SessionStatus::Skipped => continue,
        }
        let todo = session
            .todo_id
            .and_then(|id| todos.iter().find(|todo| todo.id == id));
        let (project_tags, context_tags) = effective_tags(session, todo);
        let minutes = session.actual_min.unwrap_or(0);
        add_tags(&mut projects, project_tags, minutes);
        add_tags(&mut contexts, context_tags, minutes);
    }

    let started = completed_sessions + aborted_sessions;
    let mut finished: Vec<(&Todo, NaiveDate)> = todos
        .iter()
        .filter(|todo| todo.completed)
        .filter_map(|todo| Some((todo, todo.completed_at?)))
        .filter(|(_, date)| contains(period, *date))
        .collect();
    finished.sort_by_key(|(todo, date)| (*date, todo.position, todo.id));

    Review {
        period,
        previous_period: previous,
        focus_min: focus_minutes(period, sessions, local_date),
        previous_focus_min: focus_minutes(previous, sessions, local_date),
        completed_sessions,
        aborted_sessions,
        abort_rate_percent: (aborted_sessions * 100 + started / 2)
            .checked_div(started)
            .unwrap_or(0),
        projects: group_totals(projects),
        contexts: group_totals(contexts),
        completed_todos: finished
            .into_iter()
            .map(|(todo, completed_at)| CompletedTodo {
                completed_at,
                text: todo.description.clone(),
            })
            .collect(),
    }
}

fn group_lines(markdown: &mut String, title: &str, groups: &[GroupMinutes]) {
    if groups.is_empty() {
        return;
    }
    markdown.push_str(&format!("\n## {title}\n\n"));
    for entry in groups {
        let name = entry.group.as_deref().unwrap_or("(none)");
        markdown.push_str(&format!("- {name}: {} min\n", entry.minutes));
    }
}

/// Markdown für die Gesprächsvorbereitung.
pub fn review_markdown(review: &Review) -> String {
    let delta = i64::from(review.focus_min) - i64::from(review.previous_focus_min);
    let mut markdown = format!(
        "# Review {} – {}\n\n- Focus time: {} min ({delta:+} min vs. {} – {})\n- Work sessions: {} completed, {} aborted ({} % aborted)\n",
        review.period.from,
        review.period.to,
        review.focus_min,
        review.previous_period.from,
        review.previous_period.to,
        review.completed_sessions,
        review.aborted_sessions,
        review.abort_rate_percent,
    );
    group_lines(&mut markdown, "Projects", &review.projects);
    group_lines(&mut markdown, "Contexts", &review.contexts);
    if !review.completed_todos.is_empty() {
        markdown.push_str("\n## Completed tasks\n\n");
        for todo in &review.completed_todos {
            markdown.push_str(&format!("- {}: {}\n", todo.completed_at, todo.text));
        }
    }
    markdown
}
