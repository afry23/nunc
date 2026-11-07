//! Steps für `features/todos/recurrence.feature` (wiederkehrende Aufgaben, ADR 0036).

use chrono::NaiveDate;
use cucumber::{given, then, when};
use nunc_core::recurrence::{
    follow_up_description, follow_up_sync_id, parse, recurrence_of, with_recurrence,
};

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    rule: Option<String>,
    follow_up: Option<Option<String>>,
    description: String,
}

#[when(expr = "I read the recurrence of {string}")]
fn read_rule(world: &mut DomainWorld, description: String) {
    world.recurrence.rule = recurrence_of(&description).map(|r| r.to_string());
}

#[then(expr = "the recurrence is {word}")]
fn rule_is(world: &mut DomainWorld, expected: String) {
    let expected = (expected != "none").then_some(expected);
    assert_eq!(world.recurrence.rule, expected);
}

#[given(expr = "the task {string} is completed on {word}")]
fn completed_on(world: &mut DomainWorld, description: String, done: String) {
    let done = NaiveDate::parse_from_str(&done, "%Y-%m-%d").expect("Datum");
    world.recurrence.follow_up = Some(follow_up_description(&description, done));
}

#[then(expr = "the follow-up line is {string}")]
fn follow_up_is(world: &mut DomainWorld, expected: String) {
    let actual = world.recurrence.follow_up.clone().expect("nicht berechnet");
    assert_eq!(actual, (expected != "none").then_some(expected));
}

#[when(expr = "I set the recurrence {string} on {string}")]
fn set_rule(world: &mut DomainWorld, rule: String, description: String) {
    let rule = (rule != "none").then(|| parse(&rule));
    // „0d“ ist keine gültige Regel: es ändert nichts
    world.recurrence.description = match rule {
        Some(None) => description,
        Some(Some(rule)) => with_recurrence(&description, Some(rule)),
        None => with_recurrence(&description, None),
    };
}

#[then(expr = "the description with recurrence is {string}")]
fn description_is(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.recurrence.description, expected);
}

#[then(expr = "the follow-up sync ids of {string} for {string} are equal")]
fn ids_equal(_: &mut DomainWorld, template: String, due: String) {
    assert_eq!(
        follow_up_sync_id(&template, &due),
        follow_up_sync_id(&template, &due)
    );
}

#[then(expr = "the follow-up sync ids of {string} for {string} and {string} differ")]
fn ids_differ_by_due(_: &mut DomainWorld, template: String, a: String, b: String) {
    assert_ne!(
        follow_up_sync_id(&template, &a),
        follow_up_sync_id(&template, &b)
    );
}

#[then(expr = "the follow-up sync ids of {string} and {string} for {string} differ")]
fn ids_differ_by_template(_: &mut DomainWorld, a: String, b: String, due: String) {
    assert_ne!(follow_up_sync_id(&a, &due), follow_up_sync_id(&b, &due));
}

#[then(expr = "the follow-up sync id of {string} for {string} is {string}")]
fn id_is(_: &mut DomainWorld, template: String, due: String, expected: String) {
    assert_eq!(follow_up_sync_id(&template, &due), expected);
}
