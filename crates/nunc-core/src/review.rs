//! Rückblick-Bericht für einen Zeitraum (ADR 0025), pur aus den Roh-Sessions und Todos.
//!
//! Fokuszeit zählt nur **abgeschlossene** Arbeitssessions; abgebrochene gehen in die Abbruchquote ein.
//! Ein Tag ist der lokale Kalendertag des Beginns (`local_date` kommt vom Aufrufer).

use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::{Session, SessionStatus, SessionType, TimerMode, Todo, effective_tags};

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

/// Abgeschlossene Arbeitssessions und Minuten einer Zeitmess-Technik.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ModeTotals {
    pub mode: TimerMode,
    pub sessions: u32,
    pub minutes: u32,
}

/// Fokuszeit und Pomodoros eines Tages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DayTotals {
    pub date: NaiveDate,
    pub minutes: u32,
    pub pomodoros: u32,
}

/// Fokuszeit, Pomodoros und Sessions einer Aufgabe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskTotals {
    pub todo_id: i32,
    pub text: String,
    pub minutes: u32,
    pub pomodoros: u32,
    pub sessions: u32,
}

/// Fokuszeit der Sessions, die genau dieselben Projekte und Kontexte zusammen trugen
/// (z. B. `+foo` mit `@meeting @work`). Die Tags sind sortiert; leer heißt „ohne“.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TagCombination {
    pub projects: Vec<String>,
    pub contexts: Vec<String>,
    pub sessions: u32,
    pub minutes: u32,
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
    /// Abgeschlossene Pomodoro-Arbeitssessions.
    pub pomodoro_count: u32,
    /// Immer Pomodoro, Flowtime, Stoppuhr (in dieser Reihenfolge).
    pub modes: Vec<ModeTotals>,
    /// Nur Tage mit Fokuszeit, aufsteigend.
    pub days: Vec<DayTotals>,
    /// Aufgaben mit Fokuszeit, meiste Zeit zuerst.
    pub tasks: Vec<TaskTotals>,
    /// Die Tag-Kombinationen der Sessions, wie sie zusammen vorkamen; meiste Zeit zuerst.
    pub combinations: Vec<TagCombination>,
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

const MODES: [TimerMode; 3] = [
    TimerMode::Pomodoro,
    TimerMode::Flowtime,
    TimerMode::Stopwatch,
];

fn mode_index(mode: TimerMode) -> usize {
    MODES
        .iter()
        .position(|candidate| *candidate == mode)
        .unwrap_or(0)
}

fn task_totals(tasks: BTreeMap<i32, (u32, u32, u32)>, todos: &[Todo]) -> Vec<TaskTotals> {
    let mut totals: Vec<TaskTotals> = tasks
        .into_iter()
        .filter_map(|(todo_id, (minutes, pomodoros, sessions))| {
            let todo = todos.iter().find(|todo| todo.id == todo_id)?;
            Some(TaskTotals {
                todo_id,
                text: todo.description.clone(),
                minutes,
                pomodoros,
                sessions,
            })
        })
        .collect();
    totals.sort_by(|a, b| b.minutes.cmp(&a.minutes).then(a.todo_id.cmp(&b.todo_id)));
    totals
}

/// Sortierte, eindeutige Tags.
fn sorted_tags(tags: &[String]) -> Vec<String> {
    let mut tags = tags.to_vec();
    tags.sort();
    tags.dedup();
    tags
}

/// Größte Summe zuerst; bei gleicher Summe nach Projekten, dann Kontexten, jeweils „ohne“ zuletzt.
fn combination_totals(
    combinations: BTreeMap<(Vec<String>, Vec<String>), (u32, u32)>,
) -> Vec<TagCombination> {
    let mut totals: Vec<TagCombination> = combinations
        .into_iter()
        .map(
            |((projects, contexts), (sessions, minutes))| TagCombination {
                projects,
                contexts,
                sessions,
                minutes,
            },
        )
        .collect();
    totals.sort_by(|a, b| {
        b.minutes
            .cmp(&a.minutes)
            .then_with(|| a.projects.is_empty().cmp(&b.projects.is_empty()))
            .then_with(|| a.projects.cmp(&b.projects))
            .then_with(|| a.contexts.is_empty().cmp(&b.contexts.is_empty()))
            .then_with(|| a.contexts.cmp(&b.contexts))
    });
    totals
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
    let mut combinations: BTreeMap<(Vec<String>, Vec<String>), (u32, u32)> = BTreeMap::new();
    let mut modes: BTreeMap<usize, (u32, u32)> = BTreeMap::new();
    let mut days: BTreeMap<NaiveDate, (u32, u32)> = BTreeMap::new();
    let mut tasks: BTreeMap<i32, (u32, u32, u32)> = BTreeMap::new();
    let mut pomodoro_count = 0u32;
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
        let combination = combinations
            .entry((sorted_tags(project_tags), sorted_tags(context_tags)))
            .or_default();
        combination.0 += 1;
        combination.1 += minutes;
        let pomodoro = u32::from(session.mode == TimerMode::Pomodoro);
        pomodoro_count += pomodoro;
        let mode = modes.entry(mode_index(session.mode)).or_default();
        mode.0 += 1;
        mode.1 += minutes;
        let day = days.entry(local_date(session.started_at)).or_default();
        day.0 += minutes;
        day.1 += pomodoro;
        if let Some(todo) = todo {
            let task = tasks.entry(todo.id).or_default();
            task.0 += minutes;
            task.1 += pomodoro;
            task.2 += 1;
        }
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
        pomodoro_count,
        modes: MODES
            .iter()
            .enumerate()
            .map(|(index, mode)| {
                let (sessions, minutes) = modes.get(&index).copied().unwrap_or_default();
                ModeTotals {
                    mode: *mode,
                    sessions,
                    minutes,
                }
            })
            .collect(),
        days: days
            .into_iter()
            .map(|(date, (minutes, pomodoros))| DayTotals {
                date,
                minutes,
                pomodoros,
            })
            .collect(),
        tasks: task_totals(tasks, todos),
        combinations: combination_totals(combinations),
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
    markdown.push_str(&format!("- Pomodoros: {}\n", review.pomodoro_count));
    if review.modes.iter().any(|entry| entry.sessions > 0) {
        markdown.push_str("\n## Techniques\n\n");
        for entry in &review.modes {
            markdown.push_str(&format!(
                "- {}: {} sessions, {} min\n",
                entry.mode.as_str(),
                entry.sessions,
                entry.minutes
            ));
        }
    }
    if !review.tasks.is_empty() {
        markdown.push_str("\n## Tasks\n\n");
        for task in &review.tasks {
            markdown.push_str(&format!(
                "- {}: {} min, {} pomodoros, {} sessions\n",
                task.text, task.minutes, task.pomodoros, task.sessions
            ));
        }
    }
    group_lines(&mut markdown, "Projects", &review.projects);
    group_lines(&mut markdown, "Contexts", &review.contexts);
    if !review.combinations.is_empty() {
        markdown.push_str("\n## Projects and contexts together\n\n");
        for combination in &review.combinations {
            let tags = combination
                .projects
                .iter()
                .chain(&combination.contexts)
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" ");
            let tags = if tags.is_empty() { "(none)" } else { &tags };
            markdown.push_str(&format!(
                "- {tags}: {} min, {} sessions\n",
                combination.minutes, combination.sessions
            ));
        }
    }
    if !review.days.is_empty() {
        markdown.push_str("\n## Days\n\n");
        for day in &review.days {
            markdown.push_str(&format!(
                "- {}: {} min, {} pomodoros\n",
                day.date, day.minutes, day.pomodoros
            ));
        }
    }
    if !review.completed_todos.is_empty() {
        markdown.push_str("\n## Completed tasks\n\n");
        for todo in &review.completed_todos {
            markdown.push_str(&format!("- {}: {}\n", todo.completed_at, todo.text));
        }
    }
    markdown
}
