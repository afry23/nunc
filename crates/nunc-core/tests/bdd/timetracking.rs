//! Steps für `features/timetracking/*.feature` (Timesheet, Timeline, manuelle Einträge; ADR 0017).
//! Sessions, Todos und die Zeitzone kommen aus den Steps der Module `stats` und `todo_query`.

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use cucumber::{gherkin::Step, given, then, when};
use nunc_core::manual_entry::{EntryError, validate_entry};
use nunc_core::model::SessionType;
use nunc_core::timeline::{DayTimeline, day_timeline};
use nunc_core::timesheet::{GroupBy, Timesheet, timesheet};

use crate::DomainWorld;
use crate::stats::{day, instant};

const NO_GROUP: &str = "(none)";

#[derive(Debug, Default)]
pub struct State {
    sheet: Option<Timesheet>,
    timeline: Option<DayTimeline>,
    now: Option<DateTime<Utc>>,
    entry: Option<Result<Vec<i32>, EntryError>>,
}

fn local_date_fn(world: &DomainWorld) -> impl Fn(DateTime<Utc>) -> NaiveDate + use<> {
    let offset = FixedOffset::east_opt(world.stats.offset_hours * 3600).expect("Zeitzone");
    move |time| time.with_timezone(&offset).date_naive()
}

fn sheet(world: &DomainWorld) -> &Timesheet {
    world.timetracking.sheet.as_ref().expect("kein Timesheet")
}

fn timeline(world: &DomainWorld) -> &DayTimeline {
    world
        .timetracking
        .timeline
        .as_ref()
        .expect("keine Timeline")
}

fn group_by(text: &str) -> GroupBy {
    match text {
        "project" => GroupBy::Project,
        "context" => GroupBy::Context,
        "todo" => GroupBy::Todo,
        other => panic!("unbekannte Gruppierung {other}"),
    }
}

#[when(expr = "I calculate the timesheet from {word} to {word} grouped by {word}")]
fn calculate_sheet(world: &mut DomainWorld, start: String, end: String, group: String) {
    let local_date = local_date_fn(world);
    let sheet = timesheet(
        &world.stats.sessions,
        &world.todo_query.todos,
        day(&start),
        day(&end),
        group_by(&group),
        &local_date,
    );
    world.timetracking.sheet = Some(sheet);
}

#[when(expr = "I assign the session starting {string} to todo {int}")]
fn assign_session(world: &mut DomainWorld, start: String, todo: i32) {
    let start = instant(&start);
    let session = world
        .stats
        .sessions
        .iter_mut()
        .find(|session| session.started_at == start)
        .expect("keine Session mit diesem Beginn");
    session.todo_id = Some(todo);
}

#[then("the timesheet is:")]
fn sheet_is(world: &mut DomainWorld, step: &Step) {
    let sheet = sheet(world);
    let mut actual = vec![{
        let mut header = vec!["group".to_owned()];
        header.extend(sheet.days.iter().map(ToString::to_string));
        header.push("total".to_owned());
        header
    }];
    for row in &sheet.rows {
        let mut cells = vec![row.group.clone().unwrap_or_else(|| NO_GROUP.to_owned())];
        cells.extend(row.minutes.iter().map(ToString::to_string));
        cells.push(row.total.to_string());
        actual.push(cells);
    }
    let expected: Vec<Vec<String>> = step.table.as_ref().expect("Tabelle fehlt").rows.to_vec();
    assert_eq!(actual, expected);
}

#[then(expr = "the day totals are {string} and the total is {int}")]
fn totals(world: &mut DomainWorld, days: String, total: u32) {
    let sheet = sheet(world);
    let expected: Vec<u32> = days
        .split(',')
        .map(|part| part.trim().parse().expect("Minuten"))
        .collect();
    assert_eq!(sheet.day_totals, expected);
    assert_eq!(sheet.total, total);
}

#[then(expr = "the group {string} totals {int} minutes")]
fn group_total(world: &mut DomainWorld, group: String, minutes: u32) {
    let found = sheet(world)
        .rows
        .iter()
        .find(|row| row.group.as_deref().unwrap_or(NO_GROUP) == group)
        .unwrap_or_else(|| panic!("keine Gruppe {group}"));
    assert_eq!(found.total, minutes);
}

#[when(expr = "I build the timeline for {word}")]
fn build_timeline(world: &mut DomainWorld, date: String) {
    let local_date = local_date_fn(world);
    let timeline = day_timeline(&world.stats.sessions, day(&date), &local_date);
    world.timetracking.timeline = Some(timeline);
}

fn time_text(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M").to_string()
}

#[then("the timeline blocks are:")]
fn blocks_are(world: &mut DomainWorld, step: &Step) {
    let actual: Vec<Vec<String>> = timeline(world)
        .blocks
        .iter()
        .map(|block| {
            vec![
                time_text(block.start),
                time_text(block.end),
                match block.session_type {
                    SessionType::Work => "work",
                    SessionType::ShortBreak => "short_break",
                    SessionType::LongBreak => "long_break",
                }
                .to_owned(),
                block.todo_id.map_or_else(String::new, |id| id.to_string()),
                if block.overlaps { "yes" } else { "no" }.to_owned(),
            ]
        })
        .collect();
    let expected = crate::stats::rows(step);
    assert_eq!(actual, expected);
}

#[then(expr = "the block starting {word} {word} has projects {string} and contexts {string}")]
fn block_labels(
    world: &mut DomainWorld,
    date: String,
    time: String,
    projects: String,
    contexts: String,
) {
    let start = crate::stats::instant(&format!("{date} {time}"));
    let block = timeline(world)
        .blocks
        .iter()
        .find(|block| block.start == start)
        .expect("kein Block zu diesem Beginn");
    assert_eq!(block.project_tags.join(" "), projects);
    assert_eq!(block.context_tags.join(" "), contexts);
}

#[then("the timeline has no blocks")]
fn no_blocks(world: &mut DomainWorld) {
    assert!(timeline(world).blocks.is_empty());
}

#[then("the timeline gaps are:")]
fn gaps_are(world: &mut DomainWorld, step: &Step) {
    let actual: Vec<Vec<String>> = timeline(world)
        .gaps
        .iter()
        .map(|gap| {
            vec![
                time_text(gap.start),
                time_text(gap.end),
                gap.minutes.to_string(),
            ]
        })
        .collect();
    assert_eq!(actual, crate::stats::rows(step));
}

#[given(expr = "the current time is {string}")]
fn current_time(world: &mut DomainWorld, time: String) {
    world.timetracking.now = Some(instant(&time));
}

#[when(expr = "I check a manual entry from {string} to {string}")]
fn check_entry(world: &mut DomainWorld, start: String, end: String) {
    let now = world.timetracking.now.expect("keine aktuelle Zeit");
    world.timetracking.entry = Some(validate_entry(
        instant(&start),
        instant(&end),
        now,
        &world.stats.sessions,
        None,
    ));
}

#[then("the entry is accepted without warnings")]
fn accepted(world: &mut DomainWorld) {
    assert_eq!(world.timetracking.entry, Some(Ok(vec![])));
}

#[then(expr = "the entry is accepted with a warning about {int} overlapping session")]
fn accepted_with_warning(world: &mut DomainWorld, count: usize) {
    let Some(Ok(overlaps)) = &world.timetracking.entry else {
        panic!("Eintrag nicht akzeptiert: {:?}", world.timetracking.entry);
    };
    assert_eq!(overlaps.len(), count);
}

#[then(expr = "the entry is rejected with {string}")]
fn rejected(world: &mut DomainWorld, message: String) {
    let Some(Err(error)) = &world.timetracking.entry else {
        panic!("Eintrag nicht abgelehnt: {:?}", world.timetracking.entry);
    };
    assert_eq!(error.to_string(), message);
}
