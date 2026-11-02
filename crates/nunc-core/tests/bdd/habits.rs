//! Steps für `features/habits/*.feature` und `features/sync/habit_crdt.feature` (Habit-Tracker, ADR 0028).

use std::collections::BTreeSet;

use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use nunc_core::habit::{
    Goal, Habit, HabitError, HabitItem, HabitLog, HabitProgress, PeriodCount, Schedule, StreakUnit,
    count_period, habit_overview, habit_progress, log_id, minutes_goal_days, validate_name,
};
use nunc_core::hlc::Hlc;
use nunc_core::model::{Session, SessionStatus, SessionType, TimerMode, Todo};
use nunc_core::sync_document::SyncDocument;
use nunc_core::todotxt::parse_line;

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    schedule: Option<Schedule>,
    done: BTreeSet<NaiveDate>,
    progress: Option<HabitProgress>,
    todos: Vec<Todo>,
    sessions: Vec<Session>,
    days: BTreeSet<NaiveDate>,
    pub(crate) today: Option<NaiveDate>,
    pub(crate) habits: Vec<Habit>,
    pub(crate) ticks: Vec<HabitLog>,
    overview: Vec<HabitItem>,
    period: Option<PeriodCount>,
}

impl State {
    fn progress(&self) -> &HabitProgress {
        self.progress.as_ref().expect("nicht ausgewertet")
    }
}

fn date(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("Datum")
}

fn describe_schedule(text: &str) -> String {
    Schedule::parse(text).map_or_else(|| "invalid".to_owned(), |s| s.encode())
}

fn describe<T>(result: Result<T, HabitError>, encode: impl FnOnce(T) -> String) -> String {
    result.map_or_else(|_| "invalid".to_owned(), encode)
}

#[then(expr = "the schedule text {string} reads as {string}")]
fn schedule_text(_world: &mut DomainWorld, text: String, expected: String) {
    assert_eq!(describe_schedule(&text), expected);
}

#[then(expr = "the weekdays {string} are checked as {string}")]
fn weekdays_checked(_world: &mut DomainWorld, days: String, expected: String) {
    let days: Vec<u8> = days
        .split(',')
        .filter(|d| !d.is_empty())
        .map(|d| d.parse().expect("Wochentag"))
        .collect();
    let result = Schedule::Days { days }.checked();
    assert_eq!(describe(result, |s| s.encode()), expected);
}

#[then(expr = "the weekly target {int} is checked as {string}")]
fn weekly_checked(_world: &mut DomainWorld, count: u8, expected: String) {
    let result = Schedule::Weekly { per_week: count }.checked();
    assert_eq!(describe(result, |s| s.encode()), expected);
}

#[then(expr = "the goal text {string} reads as {string}")]
fn goal_text(_world: &mut DomainWorld, text: String, expected: String) {
    let actual = Goal::parse(&text).map_or_else(|| "invalid".to_owned(), |g| g.encode());
    assert_eq!(actual, expected);
}

#[then(expr = "the minutes goal {int} on {string} is checked as {string}")]
fn goal_checked(_world: &mut DomainWorld, minutes: u32, tag: String, expected: String) {
    let result = Goal::Minutes { minutes, tag }.checked();
    assert_eq!(describe(result, |g| g.encode()), expected);
}

#[then(expr = "the name of {int} characters padded with {int} spaces is {word}")]
fn name_limit(_world: &mut DomainWorld, length: usize, pad: usize, result: String) {
    let padding = " ".repeat(pad);
    let name = format!("{padding}{}{padding}", "x".repeat(length));
    assert_eq!(validate_name(&name).is_ok(), result == "accepted");
    let error = validate_name(&name).err();
    match (length, error) {
        (0, error) => assert_eq!(error, Some(HabitError::EmptyName)),
        (101, error) => assert_eq!(error, Some(HabitError::NameTooLong)),
        (_, error) => assert_eq!(error, None),
    }
}

#[then(expr = "the name {string} is stored as {string}")]
fn name_trimmed(_world: &mut DomainWorld, name: String, stored: String) {
    assert_eq!(validate_name(&name), Ok(stored));
}

#[given(expr = "the schedule {string}")]
fn given_schedule(world: &mut DomainWorld, text: String) {
    world.habits.schedule = Some(Schedule::parse(&text).expect("Rhythmus"));
}

#[given(expr = "the habit was done on {string}")]
fn given_done(world: &mut DomainWorld, days: String) {
    world.habits.done = days
        .split(',')
        .filter(|d| !d.is_empty())
        .map(date)
        .collect();
}

#[when(expr = "I evaluate the habit for {word}")]
fn evaluate(world: &mut DomainWorld, today: String) {
    let state = &mut world.habits;
    let schedule = state.schedule.clone().expect("kein Rhythmus");
    state.progress = Some(habit_progress(&schedule, &state.done, date(&today)));
}

#[then(expr = "the current streak is {int} {word}")]
fn current_streak(world: &mut DomainWorld, count: u32, unit: String) {
    let progress = world.habits.progress();
    assert_eq!(progress.current_streak, count);
    let expected = if unit == "weeks" {
        StreakUnit::Weeks
    } else {
        StreakUnit::Days
    };
    assert_eq!(progress.unit, expected);
}

#[then(expr = "the best streak is {int}")]
fn best_streak(world: &mut DomainWorld, count: u32) {
    assert_eq!(world.habits.progress().best_streak, count);
}

#[then(expr = "the habit is {word} today")]
fn state_today(world: &mut DomainWorld, state: String) {
    let progress = world.habits.progress();
    let actual = if progress.done_today {
        "done"
    } else if progress.open_today {
        "open"
    } else {
        "idle"
    };
    assert_eq!(actual, state);
}

#[then(expr = "this week has {int} of {int}")]
fn week_progress(world: &mut DomainWorld, done: u32, target: u32) {
    let progress = world.habits.progress();
    assert_eq!((progress.week_done, progress.week_target), (done, target));
}

#[then(expr = "the week target is {int}")]
fn week_target(world: &mut DomainWorld, target: u32) {
    assert_eq!(world.habits.progress().week_target, target);
}

/// Spalten: `id`, `line` (todo.txt-Zeile).
#[given("the todos for the habits:")]
fn todos(world: &mut DomainWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    world.habits.todos = table.rows[1..]
        .iter()
        .map(|row| {
            let parsed = parse_line(&row[1]);
            Todo {
                id: row[0].parse().expect("id"),
                raw_text: row[1].clone(),
                priority: parsed.priority,
                completed: parsed.completed,
                completed_at: parsed.completed_at,
                created_at: parsed.created_at,
                description: parsed.description,
                project_tags: parsed.project_tags,
                context_tags: parsed.context_tags,
                pomodoro_count: 0,
                position: 0,
                parent_id: None,
            }
        })
        .collect();
}

/// Spalten: `start` (UTC), `type`, `status`, `minutes` (leer = unbekannt), `todo` (leer = keines), `projects`, `contexts`.
#[given("the work sessions:")]
fn work_sessions(world: &mut DomainWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    let words =
        |cell: &str| -> Vec<String> { cell.split_whitespace().map(str::to_owned).collect() };
    world.habits.sessions = table.rows[1..]
        .iter()
        .map(|row| Session {
            id: 0,
            mode: TimerMode::Pomodoro,
            session_type: row[1].parse::<SessionType>().expect("Typ"),
            status: row[2].parse::<SessionStatus>().expect("Status"),
            started_at: Utc.from_utc_datetime(
                &NaiveDateTime::parse_from_str(&row[0], "%Y-%m-%dT%H:%M").expect("Beginn"),
            ),
            ended_at: None,
            planned_min: None,
            actual_min: row[3].parse().ok(),
            todo_id: row[4].parse().ok(),
            break_min: None,
            project_tags: words(&row[5]),
            context_tags: words(&row[6]),
        })
        .collect();
}

#[when(expr = "I find the days with {int} minutes on {string}")]
fn find_days(world: &mut DomainWorld, minutes: u32, tag: String) {
    let state = &mut world.habits;
    state.days = minutes_goal_days(&state.sessions, &state.todos, &tag, minutes, &|time| {
        time.date_naive()
    });
}

#[then(expr = "the days are {word}")]
fn days_are(world: &mut DomainWorld, days: String) {
    let expected: BTreeSet<NaiveDate> = days.split(',').map(date).collect();
    assert_eq!(world.habits.days, expected);
}

#[then("there are no days")]
fn no_days(world: &mut DomainWorld) {
    assert!(world.habits.days.is_empty());
}

// ---- Quote im Zeitraum ----

#[when(expr = "I count the period from {word} to {word} on {word} for a habit created on {word}")]
fn count(world: &mut DomainWorld, from: String, to: String, today: String, created: String) {
    let state = &mut world.habits;
    let schedule = state.schedule.clone().expect("kein Rhythmus");
    state.period = Some(count_period(
        &schedule,
        &state.done,
        date(&created),
        date(&from),
        date(&to),
        date(&today),
    ));
}

#[then(expr = "the habit was due on {int} days and done on {int} days")]
fn period_counts(world: &mut DomainWorld, due: u32, done: u32) {
    let counted = world.habits.period.expect("nicht gezählt");
    assert_eq!((counted.due_days, counted.done_days), (due, done));
}

#[then(expr = "the quota is {word}")]
fn period_rate(world: &mut DomainWorld, rate: String) {
    let actual = world
        .habits
        .period
        .expect("nicht gezählt")
        .rate_percent()
        .map_or_else(|| "none".to_owned(), |rate| rate.to_string());
    assert_eq!(actual, rate);
}

// ---- Übersicht ----

#[given(expr = "the habit day is {word}")]
fn today_is(world: &mut DomainWorld, today: String) {
    world.habits.today = Some(date(&today));
}

fn stamp() -> Hlc {
    Hlc::new(1, 0, "dev")
}

/// Spalten: `id`, `name`, `schedule`, `goal` (rohe Kodierung), `archived` (yes/no).
#[given("the habits:")]
fn overview_habits(world: &mut DomainWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    world.habits.habits = table.rows[1..]
        .iter()
        .map(|row| {
            let mut habit = Habit::new(
                &row[0],
                date("2026-03-01"),
                &row[1],
                &Schedule::Daily,
                &Goal::Check,
                &stamp(),
            );
            // roh gesetzt, damit auch Kodierungen einer neueren App vorkommen können
            habit.schedule.value.clone_from(&row[2]);
            habit.goal.value.clone_from(&row[3]);
            habit.archived.value = row[4] == "yes";
            habit
        })
        .collect();
}

/// Spalten: `habit`, `day`, `done` (yes/no).
#[given("the ticks:")]
fn overview_ticks(world: &mut DomainWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    world.habits.ticks = table.rows[1..]
        .iter()
        .map(|row| HabitLog::new(&row[0], date(&row[1]), row[2] == "yes", &stamp()))
        .collect();
}

#[when("I take the habit overview")]
fn take_overview(world: &mut DomainWorld) {
    let state = &mut world.habits;
    state.overview = habit_overview(
        &state.habits,
        &state.ticks,
        &state.sessions,
        &state.todos,
        state.today.expect("kein heute"),
        &|time| time.date_naive(),
    );
}

fn item<'a>(world: &'a DomainWorld, id: &str) -> &'a HabitItem {
    world
        .habits
        .overview
        .iter()
        .find(|item| item.id == id)
        .unwrap_or_else(|| panic!("{id} fehlt in der Übersicht"))
}

#[then(expr = "the overview lists {string}")]
fn overview_lists(world: &mut DomainWorld, expected: String) {
    let names: Vec<String> = world
        .habits
        .overview
        .iter()
        .map(|item| {
            if item.archived {
                format!("{} (archived)", item.name)
            } else {
                item.name.clone()
            }
        })
        .collect();
    assert_eq!(names.join(" | "), expected);
}

#[then(expr = "the overview of {string} is {word} today with a streak of {int}")]
fn overview_state(world: &mut DomainWorld, id: String, state: String, streak: u32) {
    let progress = item(world, &id)
        .progress
        .as_ref()
        .expect("nicht auswertbar");
    let actual = if progress.done_today {
        "done"
    } else if progress.open_today {
        "open"
    } else {
        "idle"
    };
    assert_eq!((actual, progress.current_streak), (state.as_str(), streak));
}

#[then(expr = "the overview of {string} has the recent days {string}")]
fn overview_recent(world: &mut DomainWorld, id: String, days: String) {
    let expected: Vec<NaiveDate> = days
        .split(',')
        .filter(|d| !d.is_empty())
        .map(date)
        .collect();
    assert_eq!(item(world, &id).recent, expected);
}

#[then(expr = "the overview of {string} can be evaluated")]
fn overview_evaluable(world: &mut DomainWorld, id: String) {
    assert!(item(world, &id).progress.is_some());
}

#[then(expr = "the overview of {string} cannot be evaluated")]
fn overview_not_evaluable(world: &mut DomainWorld, id: String) {
    assert!(item(world, &id).progress.is_none());
}

// ---- CRDT ----

const CREATED: &str = "2026-03-01";

fn new_habit(id: &str, name: &str, created: &str, at: i64, device: &str) -> Habit {
    Habit::new(
        id,
        date(created),
        name,
        &Schedule::Daily,
        &Goal::Check,
        &Hlc::new(at, 0, device),
    )
}

#[given(expr = "device {word} creates the habit {string} named {string} at {int}")]
#[when(expr = "device {word} creates the habit {string} named {string} at {int}")]
fn create(world: &mut DomainWorld, device: String, id: String, name: String, at: i64) {
    let habit = new_habit(&id, &name, CREATED, at, &device);
    world.journal.side(&device).habits.insert(id, habit);
}

#[given(expr = "device {word} creates the habit {string} named {string} on {word} at {int}")]
fn create_on(
    world: &mut DomainWorld,
    device: String,
    id: String,
    name: String,
    created: String,
    at: i64,
) {
    let habit = new_habit(&id, &name, &created, at, &device);
    world.journal.side(&device).habits.insert(id, habit);
}

#[given(expr = "device {word} creates a habit {string} named with {int} characters at {int}")]
fn create_long(world: &mut DomainWorld, device: String, id: String, length: usize, at: i64) {
    // absichtlich ohne Prüfung: so sieht ein Gerät aus, das die Grenze nicht kennt
    let habit = new_habit(&id, &"x".repeat(length), CREATED, at, &device);
    world.journal.side(&device).habits.insert(id, habit);
}

#[given(expr = "device B has a copy of the habit {string} of device A")]
fn copy(world: &mut DomainWorld, id: String) {
    let habit = world.journal.a.habits[&id].clone();
    world.journal.b.habits.insert(id, habit);
}

fn edit(world: &mut DomainWorld, device: &str, id: &str, change: impl FnOnce(&mut Habit)) {
    let habit = world
        .journal
        .side(device)
        .habits
        .get_mut(id)
        .expect("keine Gewohnheit");
    change(habit);
}

#[when(expr = "device {word} renames the habit {string} to {string} at {int}")]
fn rename(world: &mut DomainWorld, device: String, id: String, name: String, at: i64) {
    let clock = Hlc::new(at, 0, &device);
    edit(world, &device, &id, |habit| habit.set_name(&name, clock));
}

#[when(expr = "device {word} sets the schedule of the habit {string} to {string} at {int}")]
fn set_schedule(world: &mut DomainWorld, device: String, id: String, text: String, at: i64) {
    let clock = Hlc::new(at, 0, &device);
    let schedule = Schedule::parse(&text).expect("Rhythmus");
    edit(world, &device, &id, |habit| {
        habit.set_schedule(&schedule, clock)
    });
}

#[when(expr = "device {word} sets the goal of the habit {string} to {string} at {int}")]
fn set_goal(world: &mut DomainWorld, device: String, id: String, text: String, at: i64) {
    let clock = Hlc::new(at, 0, &device);
    let goal = Goal::parse(&text).expect("Ziel");
    edit(world, &device, &id, |habit| habit.set_goal(&goal, clock));
}

#[when(expr = "device {word} archives the habit {string} at {int}")]
fn archive(world: &mut DomainWorld, device: String, id: String, at: i64) {
    let clock = Hlc::new(at, 0, &device);
    edit(world, &device, &id, |habit| habit.set_archived(true, clock));
}

#[given(expr = "device {word} deletes the habit {string} at {int}")]
#[when(expr = "device {word} deletes the habit {string} at {int}")]
fn delete(world: &mut DomainWorld, device: String, id: String, at: i64) {
    let clock = Hlc::new(at, 0, &device);
    edit(world, &device, &id, |habit| habit.delete(clock));
}

fn tick(world: &mut DomainWorld, device: &str, habit: &str, day: &str, done: bool, at: i64) {
    let day = date(day);
    let clock = Hlc::new(at, 0, device);
    let logs = &mut world.journal.side(device).habit_logs;
    match logs.get_mut(&log_id(habit, day)) {
        Some(log) => log.set_done(done, clock),
        None => {
            logs.insert(log_id(habit, day), HabitLog::new(habit, day, done, &clock));
        }
    }
}

#[given(expr = "device {word} ticks the habit {string} on {word} at {int}")]
#[when(expr = "device {word} ticks the habit {string} on {word} at {int}")]
fn tick_on(world: &mut DomainWorld, device: String, habit: String, day: String, at: i64) {
    tick(world, &device, &habit, &day, true, at);
}

#[given(expr = "device {word} unticks the habit {string} on {word} at {int}")]
#[when(expr = "device {word} unticks the habit {string} on {word} at {int}")]
fn untick_on(world: &mut DomainWorld, device: String, habit: String, day: String, at: i64) {
    tick(world, &device, &habit, &day, false, at);
}

#[then(expr = "the log id of {string} on {word} is {string}")]
fn log_id_is(_world: &mut DomainWorld, habit: String, day: String, expected: String) {
    assert_eq!(log_id(&habit, date(&day)), expected);
    assert_eq!(
        HabitLog::new(&habit, date(&day), true, &Hlc::new(1, 0, "d")).sync_id(),
        expected
    );
}

#[then(expr = "the merged state has {int} logs")]
fn log_count(world: &mut DomainWorld, count: usize) {
    assert_eq!(world.journal.merged.habit_logs.len(), count);
}

#[then(expr = "the merged state has {int} habits and {int} logs")]
fn counts(world: &mut DomainWorld, habits: usize, logs: usize) {
    assert_eq!(world.journal.merged.habits.len(), habits);
    assert_eq!(world.journal.merged.habit_logs.len(), logs);
}

#[then(expr = "the merged log of {string} on {word} is {word}")]
fn log_is(world: &mut DomainWorld, habit: String, day: String, state: String) {
    let log = &world.journal.merged.habit_logs[&log_id(&habit, date(&day))];
    // `done` oder `undone`
    assert_eq!(log.done.value, state == "done");
    assert!(state == "done" || state == "undone");
}

#[then(expr = "the merged habit {string} is named {string}")]
fn named(world: &mut DomainWorld, id: String, name: String) {
    assert_eq!(merged_habit(world, &id).name.value, name);
}

#[then(expr = "the merged habit {string} has the schedule {string}")]
fn has_schedule(world: &mut DomainWorld, id: String, text: String) {
    assert_eq!(merged_habit(world, &id).schedule.value, text);
    assert_eq!(
        merged_habit(world, &id).schedule().map(|s| s.encode()),
        Some(text)
    );
}

#[then(expr = "the merged habit {string} has the goal {string}")]
fn has_goal(world: &mut DomainWorld, id: String, text: String) {
    assert_eq!(merged_habit(world, &id).goal.value, text);
    assert_eq!(
        merged_habit(world, &id).goal().map(|g| g.encode()),
        Some(text)
    );
}

#[then(expr = "the merged habit {string} is archived")]
fn archived(world: &mut DomainWorld, id: String) {
    assert!(merged_habit(world, &id).archived.value);
}

#[then(expr = "the merged habit {string} is deleted")]
fn deleted(world: &mut DomainWorld, id: String) {
    assert!(merged_habit(world, &id).is_deleted());
}

#[then(expr = "the merged habit {string} is not deleted")]
fn not_deleted(world: &mut DomainWorld, id: String) {
    assert!(!merged_habit(world, &id).is_deleted());
}

#[then(expr = "the merged habit {string} has a name of {int} characters")]
fn name_length(world: &mut DomainWorld, id: String, count: usize) {
    assert_eq!(merged_habit(world, &id).name.value.chars().count(), count);
}

#[then(expr = "the merged habit {string} has a stamp not after {int}")]
fn stamp_capped(world: &mut DomainWorld, id: String, limit: i64) {
    let habit = merged_habit(world, &id);
    // die Uhr darf höchstens um die erlaubte Abweichung vorauseilen
    let allowed = limit + 24 * 60 * 60 * 1000;
    for stamp in [
        &habit.name.clock,
        &habit.schedule.clock,
        &habit.goal.clock,
        &habit.archived.clock,
        &habit.deleted.clock,
    ] {
        assert!(
            stamp.wall_ms <= allowed,
            "Stempel {} zu weit voraus",
            stamp.wall_ms
        );
    }
}

#[then(expr = "the merged habit {string} was created on {word}")]
fn created_on(world: &mut DomainWorld, id: String, day: String) {
    assert_eq!(merged_habit(world, &id).created, date(&day));
}

#[when("the habits of device A are written to a sync file and read back")]
fn file_roundtrip(world: &mut DomainWorld) {
    let document = SyncDocument::from_state(
        &world.journal.a,
        chrono::DateTime::from_timestamp_millis(0).expect("Zeit"),
    );
    let json = serde_json::to_string(&document).expect("json");
    let read: SyncDocument = serde_json::from_str(&json).expect("lesen");
    world.journal.merged = read.into_state().expect("Zustand");
}

#[when("a version 7 sync file without habits is read")]
fn old_file(world: &mut DomainWorld) {
    let json = serde_json::json!({ "schema_version": 7, "synced_at": "2026-03-01T12:00:00Z" });
    let document: SyncDocument = serde_json::from_value(json).expect("lesen");
    world.journal.merged = document.into_state().expect("Zustand");
}

fn merged_habit<'a>(world: &'a DomainWorld, id: &str) -> &'a Habit {
    world
        .journal
        .merged
        .habits
        .get(id)
        .unwrap_or_else(|| panic!("keine Gewohnheit {id}"))
}

#[then(expr = "the merged log of {string} on {word} has a stamp not after {int}")]
fn log_stamp_capped(world: &mut DomainWorld, habit: String, day: String, limit: i64) {
    let log = &world.journal.merged.habit_logs[&log_id(&habit, date(&day))];
    assert!(log.done.clock.wall_ms <= limit + 24 * 60 * 60 * 1000);
}
