//! Steps für `features/todos/todotxt_parsing.feature`.

use chrono::NaiveDate;
use cucumber::{given, then, when};
use nunc_core::todotxt::{ParsedTodo, build_raw_text, parse_line};

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    parsed: ParsedTodo,
    structured: ParsedTodo,
    raw: String,
}

fn date(text: &str) -> Option<NaiveDate> {
    if text.is_empty() || text == "none" {
        return None;
    }
    Some(NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("Datum im Szenario ungültig"))
}

fn tags(list: &str) -> Vec<String> {
    list.split_whitespace().map(str::to_owned).collect()
}

#[when(expr = "I parse the line {string}")]
fn parse(world: &mut DomainWorld, line: String) {
    world.todotxt.parsed = parse_line(&line);
}

#[then(expr = "the todo is completed")]
fn is_completed(world: &mut DomainWorld) {
    assert!(world.todotxt.parsed.completed);
}

#[then(expr = "the todo is not completed")]
fn is_not_completed(world: &mut DomainWorld) {
    assert!(!world.todotxt.parsed.completed);
}

#[then(expr = "the priority is {word}")]
fn priority(world: &mut DomainWorld, expected: String) {
    let actual = world
        .todotxt
        .parsed
        .priority
        .map(String::from)
        .unwrap_or_else(|| "none".to_owned());
    assert_eq!(actual, expected);
}

#[then(expr = "the creation date is {word}")]
fn creation_date(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.todotxt.parsed.created_at, date(&expected));
}

#[then(expr = "the completion date is {word}")]
fn completion_date(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.todotxt.parsed.completed_at, date(&expected));
}

#[then(expr = "the description is {string}")]
fn description(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.todotxt.parsed.description, expected);
}

#[then(expr = "the project tags are {string}")]
fn project_tags(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.todotxt.parsed.project_tags, tags(&expected));
}

#[then(expr = "the context tags are {string}")]
fn context_tags(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.todotxt.parsed.context_tags, tags(&expected));
}

#[given(
    expr = "a todo with completed {word}, priority {string}, completion date {string}, creation date {string} and description {string}"
)]
fn structured_todo(
    world: &mut DomainWorld,
    completed: String,
    priority: String,
    completed_at: String,
    created_at: String,
    description: String,
) {
    world.todotxt.structured = ParsedTodo {
        completed: completed == "true",
        priority: priority.chars().next(),
        completed_at: date(&completed_at),
        created_at: date(&created_at),
        description,
        ..ParsedTodo::default()
    };
}

#[when(expr = "I build the raw text")]
fn build(world: &mut DomainWorld) {
    world.todotxt.raw = build_raw_text(&world.todotxt.structured);
}

#[then(expr = "the raw text is {string}")]
fn raw_text(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.todotxt.raw, expected);
}

#[when(expr = "I parse the line {string} and build the raw text again")]
fn roundtrip(world: &mut DomainWorld, line: String) {
    world.todotxt.parsed = parse_line(&line);
    world.todotxt.raw = build_raw_text(&world.todotxt.parsed);
}
