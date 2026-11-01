//! Steps für `features/stats/aggregation.feature` und `features/stats/streaks.feature`.
//! Die Todos kommen aus dem Step `the todos:` (Modul `todo_query`).

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use cucumber::{gherkin::Step, given, then, when};
use nunc_core::model::{
    Session, SessionStatus, SessionType, StatsPeriod, StatsRequest, StatsResponse, TimerMode,
};
use nunc_core::stats::compute_stats;

use crate::DomainWorld;

#[derive(Debug)]
pub struct State {
    pub(crate) sessions: Vec<Session>,
    pub(crate) today: NaiveDate,
    pub(crate) range: (NaiveDate, NaiveDate),
    pub(crate) offset_hours: i32,
    result: Option<StatsResponse>,
}

impl Default for State {
    fn default() -> Self {
        let today = day("2025-03-14");
        Self {
            sessions: Vec::new(),
            today,
            range: (today, today),
            offset_hours: 0,
            result: None,
        }
    }
}

pub(crate) fn day(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .unwrap_or_else(|_| panic!("Datum {text:?} ungültig"))
}

pub(crate) fn instant(text: &str) -> DateTime<Utc> {
    let naive =
        NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M").expect("Zeit YYYY-MM-DD HH:MM");
    Utc.from_utc_datetime(&naive)
}

pub(crate) fn rows(step: &Step) -> Vec<Vec<String>> {
    step.table.as_ref().expect("Tabelle fehlt").rows[1..].to_vec()
}

fn assert_rows(actual: Vec<Vec<String>>, step: &Step) {
    assert_eq!(actual, rows(step));
}

fn result(world: &DomainWorld) -> &StatsResponse {
    world
        .stats
        .result
        .as_ref()
        .expect("Statistik wurde nicht berechnet")
}

fn session(
    start: DateTime<Utc>,
    mode: &str,
    kind: &str,
    status: &str,
    minutes: Option<u32>,
    todo: Option<i32>,
) -> Session {
    Session {
        id: 0,
        mode: mode.parse::<TimerMode>().expect("Modus"),
        session_type: kind.parse::<SessionType>().expect("Typ"),
        status: status.parse::<SessionStatus>().expect("Status"),
        started_at: start,
        ended_at: None,
        planned_min: None,
        actual_min: minutes,
        todo_id: todo,
        break_min: None,
        project_tags: Vec::new(),
        context_tags: Vec::new(),
    }
}

/// Spalten: `start` (UTC), `mode`, `type`, `status`, `minutes` (leer = unbekannt), `todo` (leer = keines),
/// optional `end` (UTC).
#[given("the sessions:")]
fn sessions(world: &mut DomainWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    let header = &table.rows[0];
    let column = |name: &str| header.iter().position(|h| h == name);
    let need = |name: &str| column(name).unwrap_or_else(|| panic!("Spalte {name} fehlt"));
    let (start, mode, kind, status, minutes, todo) = (
        need("start"),
        need("mode"),
        need("type"),
        need("status"),
        need("minutes"),
        need("todo"),
    );
    let end = column("end");
    let (projects_col, contexts_col) = (column("projects"), column("contexts"));
    let words = |row: &[String], col: Option<usize>| -> Vec<String> {
        col.map(|i| row[i].split_whitespace().map(str::to_owned).collect())
            .unwrap_or_default()
    };

    world.stats.sessions = table.rows[1..]
        .iter()
        .map(|row| {
            let mut session = session(
                instant(&row[start]),
                &row[mode],
                &row[kind],
                &row[status],
                row[minutes].parse().ok(),
                row[todo].parse().ok(),
            );
            session.project_tags = words(row, projects_col);
            session.context_tags = words(row, contexts_col);
            session.ended_at = end
                .map(|i| row[i].as_str())
                .filter(|text| !text.is_empty())
                .map(instant);
            session
        })
        .collect();
}

#[given(expr = "pomodoros were completed on {string}")]
fn pomodoros_on(world: &mut DomainWorld, dates: String) {
    world.stats.sessions = dates
        .split_whitespace()
        .map(|date| {
            session(
                instant(&format!("{date} 12:00")),
                "pomodoro",
                "work",
                "completed",
                Some(25),
                None,
            )
        })
        .collect();
}

#[given(expr = "today is {word}")]
fn today(world: &mut DomainWorld, date: String) {
    world.stats.today = day(&date);
}

#[given(expr = "the statistics range is {word} to {word}")]
fn range(world: &mut DomainWorld, start: String, end: String) {
    world.stats.range = (day(&start), day(&end));
}

#[given(expr = "the local time zone is UTC{word}")]
fn time_zone(world: &mut DomainWorld, offset: String) {
    world.stats.offset_hours = offset.parse().expect("Stundenversatz wie +2 oder -5");
}

#[when("I calculate the statistics")]
fn calculate(world: &mut DomainWorld) {
    let state = &world.stats;
    let offset = FixedOffset::east_opt(state.offset_hours * 3600).expect("Zeitzone");
    let request = StatsRequest {
        period: StatsPeriod::Week,
        start_date: state.range.0,
        end_date: state.range.1,
    };
    let response = compute_stats(
        &state.sessions,
        &world.todo_query.todos,
        &request,
        state.today,
        &|time| time.with_timezone(&offset).date_naive(),
    );
    world.stats.result = Some(response);
}

#[then("the daily statistics are:")]
fn daily(world: &mut DomainWorld, step: &Step) {
    let actual = result(world)
        .daily
        .iter()
        .map(|d| {
            vec![
                d.date.to_string(),
                d.pomodoro_count.to_string(),
                d.work_minutes.to_string(),
                d.break_minutes.to_string(),
            ]
        })
        .collect();
    assert_rows(actual, step);
}

#[then("the daily statistics per timer are:")]
fn daily_per_timer(world: &mut DomainWorld, step: &Step) {
    let actual = result(world)
        .daily
        .iter()
        .map(|d| {
            vec![
                d.date.to_string(),
                d.pomodoro_count.to_string(),
                d.pomodoro_minutes.to_string(),
                d.flowtime_count.to_string(),
                d.flowtime_minutes.to_string(),
                d.stopwatch_count.to_string(),
                d.stopwatch_minutes.to_string(),
            ]
        })
        .collect();
    assert_rows(actual, step);
}

#[then("the projects are:")]
fn projects(world: &mut DomainWorld, step: &Step) {
    let actual = result(world)
        .projects
        .iter()
        .map(|p| vec![p.project.clone(), p.work_minutes.to_string()])
        .collect();
    assert_rows(actual, step);
}

#[then("the contexts are:")]
fn contexts(world: &mut DomainWorld, step: &Step) {
    let actual = result(world)
        .contexts
        .iter()
        .map(|c| vec![c.context.clone(), c.work_minutes.to_string()])
        .collect();
    assert_rows(actual, step);
}

#[then("the top todos are:")]
fn top_todos(world: &mut DomainWorld, step: &Step) {
    let actual = result(world)
        .top_todos
        .iter()
        .map(|t| {
            vec![
                t.todo_id.to_string(),
                t.pomodoro_count.to_string(),
                t.work_minutes.to_string(),
            ]
        })
        .collect();
    assert_rows(actual, step);
}

#[then("the months are:")]
fn months(world: &mut DomainWorld, step: &Step) {
    let actual = result(world)
        .monthly
        .iter()
        .map(|m| {
            vec![
                format!("{}-{:02}", m.year, m.month),
                m.pomodoro_count.to_string(),
                m.work_minutes.to_string(),
            ]
        })
        .collect();
    assert_rows(actual, step);
}

#[then(expr = "{word} has {int} pomodoros and {int} work minutes")]
fn weekday(world: &mut DomainWorld, name: String, pomodoros: u32, work: u32) {
    let index = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .iter()
        .position(|n| *n == name)
        .unwrap_or_else(|| panic!("unbekannter Wochentag {name:?}"));
    let weekdays = &result(world).weekdays;
    assert_eq!(weekdays.len(), 7);
    let entry = &weekdays[index];
    assert_eq!(
        (
            usize::from(entry.weekday),
            entry.pomodoro_count,
            entry.work_minutes
        ),
        (index, pomodoros, work)
    );
}

#[then(expr = "the total pomodoros are {int}")]
fn total_pomodoros(world: &mut DomainWorld, expected: u32) {
    assert_eq!(result(world).total_pomodoros, expected);
}

#[then(expr = "the total work minutes are {int}")]
fn total_work(world: &mut DomainWorld, expected: u32) {
    assert_eq!(result(world).total_work_min, expected);
}

#[then(expr = "the average over {int} days is about {float}")]
fn average(world: &mut DomainWorld, days: u32, expected: f64) {
    let actual = match days {
        7 => result(world).average_pomodoros_7,
        30 => result(world).average_pomodoros_30,
        other => panic!("nur 7 und 30 Tage, nicht {other}"),
    };
    assert!((actual - expected).abs() < 0.0005, "{actual} != {expected}");
}

#[then(expr = "the current streak is {int}")]
fn current_streak(world: &mut DomainWorld, expected: u32) {
    assert_eq!(result(world).current_streak, expected);
}

#[then(expr = "the longest streak is {int}")]
fn longest_streak(world: &mut DomainWorld, expected: u32) {
    assert_eq!(result(world).longest_streak, expected);
}

#[then("there are no statistics")]
fn nothing(world: &mut DomainWorld) {
    let r = result(world);
    assert!(
        r.daily.is_empty()
            && r.projects.is_empty()
            && r.contexts.is_empty()
            && r.top_todos.is_empty()
            && r.monthly.is_empty()
    );
    assert_eq!(
        (
            r.total_pomodoros,
            r.total_work_min,
            r.current_streak,
            r.longest_streak
        ),
        (0, 0, 0, 0)
    );
    assert_eq!((r.average_pomodoros_7, r.average_pomodoros_30), (0.0, 0.0));
    assert_eq!(r.weekdays.len(), 7);
    assert!(
        r.weekdays
            .iter()
            .all(|w| w.pomodoro_count == 0 && w.work_minutes == 0)
    );
}
