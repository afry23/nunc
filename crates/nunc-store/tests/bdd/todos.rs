//! Steps für Todos, Sessions und Statistik-Rohdaten (`features/todos/crud.feature`, `features/store/stats_input.feature`,
//! Lese-Szenarien in `features/compat_go.feature`).

use chrono::NaiveDate;
use cucumber::{gherkin::Step, then, when};
use nunc_core::model::{StatsRequest, Todo};
use nunc_core::stats::{compute_stats, utc_date};
use nunc_core::timer::FinishKind;
use serde::Serialize;

use crate::settings::{as_text, rows};
use crate::{StoreWorld, at};

const NOW: &str = "10:00:00";
const DEVICE: &str = "test-device";

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 4).unwrap()
}

fn date(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap_or_else(|_| panic!("Datum {text:?}"))
}

/// Tabelle mit Kopfzeile aus JSON-Feldnamen gegen eine Liste von Objekten (in Reihenfolge).
fn assert_rows<T: Serialize>(step: &Step, items: &[T]) {
    let table = &step.table.as_ref().expect("Tabelle fehlt").rows;
    let header = &table[0];
    assert_eq!(table.len() - 1, items.len(), "Anzahl Zeilen");
    for (row, item) in table[1..].iter().zip(items) {
        let json = serde_json::to_value(item).unwrap();
        for (field, expected) in header.iter().zip(row) {
            let actual = json.get(field).unwrap_or_else(|| panic!("Feld {field}"));
            assert_eq!(&as_text(actual), expected, "{field}");
        }
    }
}

/// Tabelle `field | value` gegen ein Objekt.
fn assert_fields<T: Serialize>(step: &Step, item: &T) {
    let json = serde_json::to_value(item).unwrap();
    for row in rows(step) {
        let actual = json
            .get(&row[0])
            .unwrap_or_else(|| panic!("Feld {}", row[0]));
        assert_eq!(as_text(actual), row[1], "{}", row[0]);
    }
}

fn todo(world: &StoreWorld, id: i32) -> Todo {
    nunc_store::list_all_todos(world.conn())
        .unwrap()
        .into_iter()
        .find(|t| t.id == id)
        .unwrap_or_else(|| panic!("Todo {id} fehlt"))
}

#[when(expr = "I add the todo {string}")]
fn add(world: &mut StoreWorld, text: String) {
    world.sync_counter += 1;
    let sync_id = format!("sync-{}", world.sync_counter);
    nunc_store::add_todo(world.conn(), &text, &sync_id, at(NOW)).expect("add");
}

#[when("I list the todos")]
fn list(world: &mut StoreWorld) {
    world.todos = nunc_store::list_todos(world.conn()).expect("list");
}

#[when(expr = "I update todo {int} to {string}")]
fn update(world: &mut StoreWorld, id: i32, text: String) {
    nunc_store::update_todo(world.conn(), id, &text, at(NOW)).expect("update");
}

#[when(expr = "I complete todo {int}")]
fn complete(world: &mut StoreWorld, id: i32) {
    nunc_store::set_todo_completed(world.conn(), id, true, today(), at(NOW)).expect("complete");
}

#[when(expr = "I uncomplete todo {int}")]
fn uncomplete(world: &mut StoreWorld, id: i32) {
    nunc_store::set_todo_completed(world.conn(), id, false, today(), at(NOW)).expect("uncomplete");
}

#[when(expr = "I delete todo {int}")]
fn delete(world: &mut StoreWorld, id: i32) {
    nunc_store::delete_todo(world.conn(), id, at(NOW)).expect("delete");
}

#[when(expr = "I reorder the todos to {string}")]
fn reorder(world: &mut StoreWorld, ids: String) {
    let ids: Vec<i32> = ids.split(',').map(|i| i.trim().parse().unwrap()).collect();
    nunc_store::reorder_todos(world.conn_mut(), &ids).expect("reorder");
}

#[when(expr = "todo {int} gets a pomodoro")]
fn pomodoro(world: &mut StoreWorld, id: i32) {
    nunc_store::increment_pomodoro_count(world.conn(), id).expect("increment");
}

#[when(expr = "the pomodoro is finished at {word} and stored")]
fn finish_and_store(world: &mut StoreWorld, time: String) {
    let now = at(&time);
    let finish = world
        .timer
        .tick(now, &nunc_core::model::Settings::default())
        .expect("Timer nicht abgelaufen");
    assert_eq!(finish.kind, FinishKind::Completed);
    let timer = world.timer.clone();
    nunc_store::commit_finish(world.conn_mut(), &finish, DEVICE, &timer, now).expect("commit");
}

#[when(expr = "I calculate the statistics for {word} to {word} as of {word}")]
fn calculate(world: &mut StoreWorld, start: String, end: String, as_of: String) {
    let request: StatsRequest = serde_json::from_value(serde_json::json!({
        "period": "month", "startDate": start, "endDate": end
    }))
    .unwrap();
    let sessions = nunc_store::list_sessions(world.conn()).unwrap();
    let todos = nunc_store::list_all_todos(world.conn()).unwrap();
    world.stats = Some(compute_stats(
        &sessions,
        &todos,
        &request,
        date(&as_of),
        &utc_date,
    ));
}

#[then("the todo list is empty")]
fn list_empty(world: &mut StoreWorld) {
    assert!(world.todos.is_empty());
}

#[then(expr = "the todo {int} is:")]
fn todo_is(world: &mut StoreWorld, id: i32, step: &Step) {
    assert_fields(step, &todo(world, id));
}

#[then("the listed todos are:")]
fn listed(world: &mut StoreWorld, step: &Step) {
    assert_rows(step, &world.todos);
}

#[then(expr = "the todo {int} is marked as deleted")]
fn marked_deleted(world: &mut StoreWorld, id: i32) {
    let deleted: i64 = world
        .conn()
        .query_row("SELECT deleted FROM todos WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(deleted, 1);
}

#[then(expr = "the todo with sync id {string} is marked as deleted")]
fn sync_id_deleted(world: &mut StoreWorld, sync_id: String) {
    let deleted: i64 = world
        .conn()
        .query_row(
            "SELECT deleted FROM todos WHERE sync_id = ?1",
            [sync_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(deleted, 1);
}

#[then(expr = "the todo with sync id {string} has the id {int}")]
fn sync_id_has_id(world: &mut StoreWorld, sync_id: String, id: i32) {
    let actual: i32 = world
        .conn()
        .query_row("SELECT id FROM todos WHERE sync_id = ?1", [sync_id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(actual, id);
}

#[then(expr = "there are {int} sessions")]
fn session_count(world: &mut StoreWorld, expected: usize) {
    assert_eq!(
        nunc_store::list_sessions(world.conn()).unwrap().len(),
        expected
    );
}

#[then(expr = "session {int} is:")]
fn session_is(world: &mut StoreWorld, id: i32, step: &Step) {
    let session = nunc_store::list_sessions(world.conn())
        .unwrap()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap_or_else(|| panic!("Session {id} fehlt"));
    assert_fields(step, &session);
}

#[then("the sessions are:")]
fn sessions_are(world: &mut StoreWorld, step: &Step) {
    assert_rows(step, &nunc_store::list_sessions(world.conn()).unwrap());
}

#[then(expr = "all sessions have the sync device id {string}")]
fn device_id(world: &mut StoreWorld, expected: String) {
    let others: i64 = world
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE sync_device_id IS NOT ?1",
            [expected],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(others, 0);
}

#[then("the statistics are:")]
fn stats_are(world: &mut StoreWorld, step: &Step) {
    let stats = world.stats.as_ref().expect("keine Statistik");
    assert_fields(step, stats);
}

#[then(expr = "the top todo is {int} with {int} pomodoros")]
fn top_todo(world: &mut StoreWorld, id: i32, pomodoros: u32) {
    let top = &world.stats.as_ref().expect("keine Statistik").top_todos[0];
    assert_eq!((top.todo_id, top.pomodoro_count), (id, pomodoros));
}

#[then(expr = "the top todos include {int} with {int} pomodoros")]
fn top_todos_include(world: &mut StoreWorld, id: i32, pomodoros: u32) {
    let found = world
        .stats
        .as_ref()
        .expect("keine Statistik")
        .top_todos
        .iter()
        .find(|t| t.todo_id == id)
        .unwrap_or_else(|| panic!("Todo {id} nicht in der Top-Liste"));
    assert_eq!(found.pomodoro_count, pomodoros);
    assert!(!found.description.is_empty());
}
