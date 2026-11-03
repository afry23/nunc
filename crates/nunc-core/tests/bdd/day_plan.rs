//! Steps für `features/todos/day_plan.feature` (Tagesplaner, ADR 0027).

use chrono::{NaiveDate, TimeZone, Utc};
use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use nunc_core::day_plan::{
    CloseAction, DayPlan, PlanError, PlanFields, PlanKey, PlanReason, apply_plan, closing_changes,
    day_plan, plan_fields, with_field,
};
use nunc_core::model::{Session, SessionStatus, SessionType, TimerMode, Todo};

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    fields: PlanFields,
    description: String,
    today: Option<NaiveDate>,
    capacity: u32,
    todos: Vec<Todo>,
    sessions: Vec<Session>,
    plan: Option<DayPlan>,
    line: Option<Result<String, PlanError>>,
    changes: Vec<(i32, Option<NaiveDate>)>,
}

fn date(text: &str) -> Option<NaiveDate> {
    (text != "none").then(|| NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("Datum"))
}

fn plan(world: &DomainWorld) -> &DayPlan {
    world.day_plan.plan.as_ref().expect("Plan nicht gebaut")
}

#[when(expr = "I read the planning fields of {string}")]
fn read_fields(world: &mut DomainWorld, description: String) {
    world.day_plan.fields = plan_fields(&description);
}

#[then(expr = "the due date is {word}")]
fn due_is(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.day_plan.fields.due, date(&expected));
}

#[then(expr = "the plan date is {word}")]
fn plan_is(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.day_plan.fields.plan, date(&expected));
}

#[then(expr = "the estimate is {word}")]
fn estimate_is(world: &mut DomainWorld, expected: String) {
    let expected = (expected != "none").then(|| expected.parse::<u32>().expect("Minuten"));
    assert_eq!(world.day_plan.fields.estimate_min, expected);
}

#[when(expr = "I set {string} to {string} in {string}")]
fn set_field(world: &mut DomainWorld, key: String, value: String, description: String) {
    let key = match key.as_str() {
        "due" => PlanKey::Due,
        "plan" => PlanKey::Plan,
        "est" => PlanKey::Estimate,
        other => panic!("unbekannter Schlüssel {other}"),
    };
    let value = (!value.is_empty()).then_some(value.as_str());
    world.day_plan.description = with_field(&description, key, value);
}

#[then(expr = "the description reads {string}")]
fn description_reads(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.day_plan.description, expected);
}

#[given(expr = "today is {word} and the capacity is {int} minutes")]
fn today_and_capacity(world: &mut DomainWorld, today: String, capacity: u32) {
    world.day_plan.today = date(&today);
    world.day_plan.capacity = capacity;
}

/// Spalten: `id`, `priority` (leer = keine), `position`, `description`.
#[given("the open todos")]
fn open_todos(world: &mut DomainWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    world.day_plan.todos = table.rows[1..]
        .iter()
        .map(|row| Todo {
            id: row[0].parse().expect("id"),
            raw_text: row[3].clone(),
            priority: row[1].chars().next(),
            completed: false,
            completed_at: None,
            created_at: None,
            description: row[3].clone(),
            project_tags: Vec::new(),
            context_tags: Vec::new(),
            pomodoro_count: 0,
            position: row[2].parse().expect("position"),
            parent_id: None,
        })
        .collect();
}

#[given(expr = "todo {int} is completed")]
fn todo_completed(world: &mut DomainWorld, id: i32) {
    for todo in world.day_plan.todos.iter_mut().filter(|t| t.id == id) {
        todo.completed = true;
    }
}

/// Spalten: `todo`, `status`, `minutes`.
#[given("the work sessions")]
fn work_sessions(world: &mut DomainWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    world.day_plan.sessions = table.rows[1..]
        .iter()
        .map(|row| Session {
            id: 0,
            mode: TimerMode::Pomodoro,
            session_type: SessionType::Work,
            status: row[1].parse::<SessionStatus>().expect("Status"),
            started_at: Utc.with_ymd_and_hms(2026, 3, 4, 9, 0, 0).unwrap(),
            ended_at: None,
            planned_min: None,
            actual_min: Some(row[2].parse().expect("Minuten")),
            todo_id: Some(row[0].parse().expect("todo")),
            break_min: None,
            project_tags: Vec::new(),
            context_tags: Vec::new(),
        })
        .collect();
}

#[when("I build the day plan")]
fn build(world: &mut DomainWorld) {
    let state = &mut world.day_plan;
    let today = state.today.expect("Heute nicht gesetzt");
    state.plan = Some(day_plan(
        &state.todos,
        &state.sessions,
        today,
        state.capacity,
    ));
}

#[then(expr = "the plan lists the todos {}")]
fn lists(world: &mut DomainWorld, expected: String) {
    let ids: Vec<i32> = plan(world).items.iter().map(|i| i.todo_id).collect();
    let expected: Vec<i32> = expected
        .split(',')
        .map(|id| id.trim().parse().expect("id"))
        .collect();
    assert_eq!(ids, expected);
}

#[then(expr = "the plan reasons are {}")]
fn reasons(world: &mut DomainWorld, expected: String) {
    let reasons: Vec<PlanReason> = plan(world).items.iter().map(|i| i.reason).collect();
    let expected: Vec<PlanReason> = expected
        .split(',')
        .map(|r| match r.trim() {
            "overdue" => PlanReason::Overdue,
            "due" => PlanReason::Due,
            "planned" => PlanReason::Planned,
            other => panic!("unbekannter Grund {other}"),
        })
        .collect();
    assert_eq!(reasons, expected);
}

#[then(expr = "the planned estimate is {int} minutes")]
fn planned_estimate(world: &mut DomainWorld, expected: u32) {
    assert_eq!(plan(world).estimate_min, expected);
}

#[then(expr = "{int} plan entries have no estimate")]
fn unestimated(world: &mut DomainWorld, expected: u32) {
    assert_eq!(plan(world).unestimated, expected);
}

#[then(expr = "the remaining capacity is {int} minutes")]
fn remaining(world: &mut DomainWorld, expected: i64) {
    assert_eq!(plan(world).remaining_min, expected);
}

#[then(expr = "todo {int} on the plan has spent {int} minutes")]
fn spent(world: &mut DomainWorld, id: i32, expected: u32) {
    let item = plan(world)
        .items
        .iter()
        .find(|i| i.todo_id == id)
        .expect("Todo nicht im Plan");
    assert_eq!(item.spent_min, expected);
}

#[when(expr = "I apply due {string}, plan {string} and estimate {string} to the line {string}")]
fn apply(world: &mut DomainWorld, due: String, plan: String, estimate: String, line: String) {
    let fields = PlanFields {
        due: date_or_empty(&due),
        plan: date_or_empty(&plan),
        estimate_min: (!estimate.is_empty()).then(|| estimate.parse().expect("Minuten")),
    };
    world.day_plan.line = Some(apply_plan(&line, &fields));
}

fn date_or_empty(text: &str) -> Option<NaiveDate> {
    (!text.is_empty()).then(|| NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("Datum"))
}

#[then(expr = "the line reads {string}")]
fn line_reads(world: &mut DomainWorld, expected: String) {
    let line = world.day_plan.line.clone().expect("nichts angewendet");
    assert_eq!(line, Ok(expected));
}

#[then("the planning is rejected")]
fn rejected(world: &mut DomainWorld) {
    let line = world.day_plan.line.clone().expect("nichts angewendet");
    assert_eq!(line, Err(PlanError::InvalidEstimate));
}

#[when(expr = "I close the day with the action {string}")]
fn close(world: &mut DomainWorld, action: String) {
    let action = match action.as_str() {
        "postpone" => CloseAction::Postpone,
        "unplan" => CloseAction::Unplan,
        "keep" => CloseAction::Keep,
        other => panic!("unbekannte Aktion {other}"),
    };
    world.day_plan.changes = closing_changes(plan(world), action);
}

#[then(expr = "the closing changes are {string}")]
fn closing(world: &mut DomainWorld, expected: String) {
    let actual: Vec<String> = world
        .day_plan
        .changes
        .iter()
        .map(|(id, day)| {
            let day = day.map_or_else(|| "none".to_owned(), |day| day.to_string());
            format!("{id} -> {day}")
        })
        .collect();
    assert_eq!(actual.join(", "), expected);
}
