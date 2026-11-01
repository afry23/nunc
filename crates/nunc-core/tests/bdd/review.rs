//! Steps für `features/stats/review.feature` (Rückblick-Bericht, ADR 0025).
//! Sessions, Todos und die Zeitzone kommen aus den Steps der Module `stats` und `todo_query`.

use std::collections::HashMap;

use cucumber::{gherkin::Step, given, then, when};
use nunc_core::review::{
    GroupMinutes, Period, PeriodKind, Review, build_review, period_containing, review_markdown,
};

use crate::DomainWorld;
use crate::stats::day;
use crate::timetracking::local_date_fn;

const NO_GROUP: &str = "(none)";

#[derive(Debug, Default)]
pub struct State {
    review: Option<Review>,
    period: Option<Period>,
    markdown: Option<String>,
    notes: HashMap<i32, String>,
}

fn review(world: &DomainWorld) -> &Review {
    world.review.review.as_ref().expect("kein Bericht")
}

fn kind(text: &str) -> PeriodKind {
    match text {
        "week" => PeriodKind::Week,
        "month" => PeriodKind::Month,
        "quarter" => PeriodKind::Quarter,
        other => panic!("unbekannter Zeitraum {other}"),
    }
}

/// Spalten `start` (UTC) und `note`; vergibt dafür fortlaufende Session-Ids.
#[given("the session notes:")]
fn session_notes(world: &mut DomainWorld, step: &Step) {
    for (index, session) in world.stats.sessions.iter_mut().enumerate() {
        session.id = i32::try_from(index).expect("Index") + 1;
    }
    world.review.notes = crate::stats::rows(step)
        .into_iter()
        .map(|row| {
            let start = crate::stats::instant(&row[0]);
            let session = world
                .stats
                .sessions
                .iter()
                .find(|session| session.started_at == start)
                .expect("keine Session mit diesem Beginn");
            (session.id, row[1].clone())
        })
        .collect();
}

#[when(expr = "I build the review from {word} to {word}")]
fn build(world: &mut DomainWorld, from: String, to: String) {
    let local_date = local_date_fn(world);
    let period = Period {
        from: day(&from),
        to: day(&to),
    };
    world.review.review = Some(build_review(
        period,
        &world.stats.sessions,
        &world.todo_query.todos,
        &world.review.notes,
        &local_date,
    ));
}

#[when(expr = "I ask for the {word} containing {word}")]
fn ask_period(world: &mut DomainWorld, kind_text: String, date: String) {
    world.review.period = Some(period_containing(kind(&kind_text), day(&date)));
}

#[when("I export the review as Markdown")]
fn export(world: &mut DomainWorld) {
    let markdown = review_markdown(review(world));
    world.review.markdown = Some(markdown);
}

#[then(expr = "the review focus time is {int} minutes")]
fn focus_time(world: &mut DomainWorld, minutes: u32) {
    assert_eq!(review(world).focus_min, minutes);
}

#[then(expr = "the review has {int} completed and {int} aborted work sessions")]
fn session_counts(world: &mut DomainWorld, completed: u32, aborted: u32) {
    let review = review(world);
    assert_eq!(review.completed_sessions, completed);
    assert_eq!(review.aborted_sessions, aborted);
}

#[then(expr = "the review abort rate is {int} percent")]
fn abort_rate(world: &mut DomainWorld, percent: u32) {
    assert_eq!(review(world).abort_rate_percent, percent);
}

#[then(expr = "the previous period is {word} to {word}")]
fn previous_period(world: &mut DomainWorld, from: String, to: String) {
    let previous = review(world).previous_period;
    assert_eq!((previous.from, previous.to), (day(&from), day(&to)));
}

#[then(expr = "the previous focus time is {int} minutes")]
fn previous_focus(world: &mut DomainWorld, minutes: u32) {
    assert_eq!(review(world).previous_focus_min, minutes);
}

#[then(expr = "the focus time changed by {int} minutes")]
fn focus_delta(world: &mut DomainWorld, delta: i64) {
    let review = review(world);
    assert_eq!(
        i64::from(review.focus_min) - i64::from(review.previous_focus_min),
        delta
    );
}

#[then(expr = "the period is {word} to {word}")]
fn period_is(world: &mut DomainWorld, from: String, to: String) {
    let period = world.review.period.expect("kein Zeitraum");
    assert_eq!((period.from, period.to), (day(&from), day(&to)));
}

#[then("the completed todos are:")]
fn completed_todos(world: &mut DomainWorld, step: &Step) {
    let actual: Vec<Vec<String>> = review(world)
        .completed_todos
        .iter()
        .map(|todo| vec![todo.completed_at.to_string(), todo.text.clone()])
        .collect();
    assert_eq!(actual, crate::stats::rows(step));
}

fn group_rows(groups: &[GroupMinutes]) -> Vec<Vec<String>> {
    groups
        .iter()
        .map(|entry| {
            vec![
                entry.group.clone().unwrap_or_else(|| NO_GROUP.to_owned()),
                entry.minutes.to_string(),
            ]
        })
        .collect()
}

#[then("the review projects are:")]
fn projects_are(world: &mut DomainWorld, step: &Step) {
    assert_eq!(
        group_rows(&review(world).projects),
        crate::stats::rows(step)
    );
}

#[then(expr = "the Markdown contains {string}")]
fn markdown_contains(world: &mut DomainWorld, text: String) {
    let markdown = world.review.markdown.as_ref().expect("kein Markdown");
    assert!(markdown.contains(&text), "{text:?} fehlt in:\n{markdown}");
}

#[then(expr = "the review has {int} pomodoros")]
fn pomodoros(world: &mut DomainWorld, count: u32) {
    assert_eq!(review(world).pomodoro_count, count);
}

#[then("the review modes are:")]
fn modes_are(world: &mut DomainWorld, step: &Step) {
    let actual: Vec<Vec<String>> = review(world)
        .modes
        .iter()
        .map(|entry| {
            vec![
                entry.mode.as_str().to_owned(),
                entry.sessions.to_string(),
                entry.minutes.to_string(),
            ]
        })
        .collect();
    assert_eq!(actual, crate::stats::rows(step));
}

#[then("the review days are:")]
fn days_are(world: &mut DomainWorld, step: &Step) {
    let actual: Vec<Vec<String>> = review(world)
        .days
        .iter()
        .map(|entry| {
            vec![
                entry.date.to_string(),
                entry.minutes.to_string(),
                entry.pomodoros.to_string(),
            ]
        })
        .collect();
    assert_eq!(actual, crate::stats::rows(step));
}

#[then("the review tasks are:")]
fn tasks_are(world: &mut DomainWorld, step: &Step) {
    let actual: Vec<Vec<String>> = review(world)
        .tasks
        .iter()
        .map(|entry| {
            vec![
                entry.text.clone(),
                entry.minutes.to_string(),
                entry.pomodoros.to_string(),
                entry.sessions.to_string(),
            ]
        })
        .collect();
    assert_eq!(actual, crate::stats::rows(step));
}

#[then("the review tag combinations are:")]
fn combinations_are(world: &mut DomainWorld, step: &Step) {
    let words = |tags: &[String]| {
        if tags.is_empty() {
            NO_GROUP.to_owned()
        } else {
            tags.join(" ")
        }
    };
    let actual: Vec<Vec<String>> = review(world)
        .combinations
        .iter()
        .map(|entry| {
            vec![
                words(&entry.projects),
                words(&entry.contexts),
                entry.sessions.to_string(),
                entry.minutes.to_string(),
            ]
        })
        .collect();
    assert_eq!(actual, crate::stats::rows(step));
}

#[then(expr = "the Markdown does not contain {string}")]
fn markdown_lacks(world: &mut DomainWorld, text: String) {
    let markdown = world.review.markdown.as_ref().expect("kein Markdown");
    assert!(!markdown.contains(&text), "{text:?} steht in:\n{markdown}");
}

#[then(expr = "the combination {string} has the notes:")]
fn combination_notes(world: &mut DomainWorld, tags: String, step: &Step) {
    let found = review(world)
        .combinations
        .iter()
        .find(|entry| {
            entry
                .projects
                .iter()
                .chain(&entry.contexts)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
                == tags
        })
        .unwrap_or_else(|| panic!("keine Kombination {tags}"));
    let actual: Vec<Vec<String>> = found
        .notes
        .iter()
        .map(|note| vec![note.date.to_string(), note.text.clone()])
        .collect();
    assert_eq!(actual, crate::stats::rows(step));
}

#[then("the Markdown contains:")]
fn markdown_contains_block(world: &mut DomainWorld, step: &Step) {
    let text = step.docstring.as_ref().expect("Docstring fehlt").trim();
    let markdown = world.review.markdown.as_ref().expect("kein Markdown");
    assert!(markdown.contains(text), "{text:?} fehlt in:\n{markdown}");
}
