//! Steps für `features/stats/review.feature` (Rückblick-Bericht, ADR 0025).
//! Sessions, Todos und die Zeitzone kommen aus den Steps der Module `stats` und `todo_query`.

use std::collections::HashMap;

use cucumber::{gherkin::Step, given, then, when};
use nunc_core::estimate::{Estimate, Verdict, check, estimate_of, summarize};
use nunc_core::hlc::Hlc;
use nunc_core::journal::JournalEntry;
use nunc_core::model::Language;
use nunc_core::review::{
    GroupMinutes, JournalItem, Period, PeriodKind, Review, build_review, period_containing,
    review_markdown,
};
use nunc_core::task_time::{TaskTime, task_time};

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
    journal: Vec<JournalEntry>,
    task_time: Option<TaskTime>,
    check: Option<nunc_core::estimate::EstimateCheck>,
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
        &world.review.journal,
        &local_date,
    ));
}

#[when(expr = "I ask for the {word} containing {word}")]
fn ask_period(world: &mut DomainWorld, kind_text: String, date: String) {
    world.review.period = Some(period_containing(kind(&kind_text), day(&date)));
}

#[when(expr = "I export the review as Markdown in {word}")]
fn export(world: &mut DomainWorld, language: String) {
    let language = match language.as_str() {
        "English" => Language::En,
        "German" => Language::De,
        other => panic!("unbekannte Sprache {other}"),
    };
    let markdown = review_markdown(review(world), language);
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
        .map(|todo| {
            vec![
                todo.completed_at.to_string(),
                todo.text.clone(),
                todo.minutes.to_string(),
            ]
        })
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

/// Spalten `kind` (`day_note`, `week_note`, `achievement`), `date`, `text` und `deleted` (`yes`/leer).
#[given("the journal entries:")]
fn journal_entries(world: &mut DomainWorld, step: &Step) {
    let clock = Hlc::new(1_000, 0, "d");
    for (index, row) in crate::stats::rows(step).into_iter().enumerate() {
        let date = day(&row[1]);
        let mut entry = match row[0].as_str() {
            "day_note" => JournalEntry::day_note(date, &row[2], &clock),
            "week_note" => JournalEntry::week_note(date, &row[2], &clock),
            "achievement" => JournalEntry::achievement(format!("a-{index}"), date, &row[2], &clock),
            other => panic!("unbekannte Art {other}"),
        };
        if row[3] == "yes" {
            entry.delete(Hlc::new(2_000, 0, "d"));
        }
        world.review.journal.push(entry);
    }
}

fn lines(items: &[JournalItem]) -> String {
    items
        .iter()
        .map(|item| format!("{}: {}", item.date, item.text))
        .collect::<Vec<_>>()
        .join(" | ")
}

#[then(expr = "the review {word} are {string}")]
fn journal_lists(world: &mut DomainWorld, group: String, expected: String) {
    let journal = &review(world).journal;
    let items = match group.as_str() {
        "achievements" => &journal.achievements,
        "week_notes" => &journal.week_notes,
        "day_notes" => &journal.day_notes,
        other => panic!("unbekannte Gruppe {other}"),
    };
    assert_eq!(lines(items), expected);
}

#[when(expr = "I list the time entries of todo {int}")]
fn list_task_time(world: &mut DomainWorld, todo_id: i32) {
    world.review.task_time = Some(task_time(
        &world.stats.sessions,
        todo_id,
        &world.review.notes,
    ));
}

#[then(expr = "the total time is {int} minutes")]
fn task_time_total(world: &mut DomainWorld, expected: u32) {
    let time = world.review.task_time.as_ref().expect("keine Zeiteinträge");
    assert_eq!(time.total_min, expected);
}

#[then("the time entries are:")]
fn task_time_entries(world: &mut DomainWorld, step: &Step) {
    let time = world.review.task_time.as_ref().expect("keine Zeiteinträge");
    let actual: Vec<Vec<String>> = time
        .entries
        .iter()
        .map(|entry| {
            vec![
                entry.started_at.format("%Y-%m-%d %H:%M").to_string(),
                entry.minutes.to_string(),
                entry.note.clone(),
            ]
        })
        .collect();
    assert_eq!(actual, crate::stats::rows(step));
}

fn estimate_text(estimate: Option<Estimate>) -> String {
    match estimate {
        None => "none".to_owned(),
        Some(Estimate::Minutes(value)) => format!("{value} min"),
        Some(Estimate::Pomodoros(value)) => format!("{value} pom"),
    }
}

fn parse_estimate(text: &str) -> Option<Estimate> {
    if text == "none" {
        return None;
    }
    let (value, unit) = text.split_once(' ').expect("Wert und Einheit");
    let value = value.parse().expect("Zahl");
    Some(match unit {
        "min" => Estimate::Minutes(value),
        "pom" => Estimate::Pomodoros(value),
        other => panic!("unbekannte Einheit {other}"),
    })
}

fn verdict_text(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Under => "under",
        Verdict::OnTarget => "on_target",
        Verdict::Over => "over",
    }
}

#[then("the estimate checks are:")]
fn estimate_checks(world: &mut DomainWorld, step: &Step) {
    let actual: Vec<Vec<String>> = review(world)
        .completed_todos
        .iter()
        .map(|todo| match todo.estimate_check {
            Some(check) => vec![
                todo.text.clone(),
                estimate_text(Some(check.estimate)),
                check.actual.to_string(),
                verdict_text(check.verdict).to_owned(),
            ],
            None => vec![
                todo.text.clone(),
                "none".to_owned(),
                "none".to_owned(),
                "none".to_owned(),
            ],
        })
        .collect();
    assert_eq!(actual, crate::stats::rows(step));
}

#[then(expr = "the estimate summary is {int} estimated, {int} on target, {int} under, {int} over")]
fn estimate_summary(world: &mut DomainWorld, count: u32, on_target: u32, under: u32, over: u32) {
    let summary = review(world).estimate_summary;
    assert_eq!(
        (
            summary.count,
            summary.on_target,
            summary.under,
            summary.over
        ),
        (count, on_target, under, over)
    );
    let again = summarize(
        review(world)
            .completed_todos
            .iter()
            .filter_map(|todo| todo.estimate_check),
    );
    assert_eq!(again, summary);
}

#[when(expr = "I compare an estimate of {string} with {int} minutes and {int} pomodoros")]
fn compare(world: &mut DomainWorld, estimate: String, minutes: u32, pomodoros: u32) {
    let estimate = parse_estimate(&estimate).expect("Schätzung");
    world.review.check = Some(check(estimate, minutes, pomodoros));
}

#[then(expr = "the verdict is {word} with a deviation of {int}")]
fn verdict_is(world: &mut DomainWorld, verdict: String, delta: i32) {
    let check = world.review.check.expect("kein Vergleich");
    assert_eq!(verdict_text(check.verdict), verdict);
    assert_eq!(check.delta, delta);
}

#[then(expr = "the estimate of {string} is {string}")]
fn estimate_of_is(_world: &mut DomainWorld, description: String, expected: String) {
    assert_eq!(estimate_text(estimate_of(&description)), expected);
}
