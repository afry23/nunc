//! Steps für `features/sync/habit_store.feature`: Gewohnheiten und Haken in der Datenbank und im Austausch zweier
//! Geräte (ADR 0028). Die Geräte stammen aus `crdt.rs`.

use chrono::NaiveDate;
use cucumber::{given, then, when};
use nunc_core::habit::{Goal, Schedule, log_id};
use nunc_store::Connection;

use crate::{StoreWorld, at};

fn day(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap_or_else(|_| panic!("Datum {text:?}"))
}

/// Rhythmus aus der Kodierung; Unlesbares (`days:`, `weekly:8`) wird zu einem ungültigen Rhythmus, den der Store
/// ablehnen muss.
fn schedule(text: &str) -> Schedule {
    Schedule::parse(text).unwrap_or(Schedule::Weekly { per_week: 8 })
}

fn goal(text: &str) -> Goal {
    Goal::parse(text).unwrap_or(Goal::Minutes {
        minutes: 30,
        tag: "work".to_owned(),
    })
}

/// Merkt das Ergebnis eines Schreibvorgangs; ein Fehler wird zur Meldung für „the habits report“.
fn record<T>(world: &mut StoreWorld, result: Result<T, nunc_store::StoreError>) {
    world.crdt.habit_error = result.err().map(|error| error.to_string());
}

fn listed(conn: &Connection) -> String {
    nunc_store::list_habits(conn)
        .expect("list")
        .iter()
        .map(|habit| {
            let archived = if habit.archived.value {
                " archived"
            } else {
                ""
            };
            format!(
                "{} [{}, {}]{archived}",
                habit.name.value, habit.schedule.value, habit.goal.value
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn ticks(conn: &Connection, from: &str, to: &str) -> String {
    nunc_store::list_habit_logs(conn, day(from), day(to))
        .expect("list")
        .iter()
        .map(|log| format!("{} {}", log.day, log.habit))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .expect("count")
}

#[given(
    expr = "device {word} creates the habit {string} {string} on {word} with schedule {string} and goal {string} at {word}"
)]
#[when(
    expr = "device {word} creates the habit {string} {string} on {word} with schedule {string} and goal {string} at {word}"
)]
#[expect(clippy::too_many_arguments, reason = "ein Parameter je Gherkin-Wert")]
fn creates_habit(
    world: &mut StoreWorld,
    device: String,
    id: String,
    name: String,
    created: String,
    schedule_text: String,
    goal_text: String,
    time: String,
) {
    let result = nunc_store::create_habit(
        world.crdt.conn(&device),
        &id,
        day(&created),
        &name,
        schedule(&schedule_text),
        goal(&goal_text),
        at(&time),
    );
    record(world, result);
}

#[when(expr = "device {word} creates a habit with a name of {int} characters at {word}")]
fn creates_long_habit(world: &mut StoreWorld, device: String, length: usize, time: String) {
    let result = nunc_store::create_habit(
        world.crdt.conn(&device),
        "h1",
        day("2026-03-02"),
        &"x".repeat(length),
        Schedule::Daily,
        Goal::Check,
        at(&time),
    );
    record(world, result);
}

#[when(
    expr = "device {word} changes the habit {string} to {string} with schedule {string} and goal {string} at {word}"
)]
fn changes_habit(
    world: &mut StoreWorld,
    device: String,
    id: String,
    name: String,
    schedule_text: String,
    goal_text: String,
    time: String,
) {
    let result = nunc_store::update_habit(
        world.crdt.conn(&device),
        &id,
        &name,
        schedule(&schedule_text),
        goal(&goal_text),
        at(&time),
    );
    record(world, result);
}

#[given(expr = "device {word} archives the habit {string} at {word}")]
#[when(expr = "device {word} archives the habit {string} at {word}")]
fn archives_habit(world: &mut StoreWorld, device: String, id: String, time: String) {
    let result = nunc_store::set_habit_archived(world.crdt.conn(&device), &id, true, at(&time));
    record(world, result);
}

#[when(expr = "device {word} unarchives the habit {string} at {word}")]
fn unarchives_habit(world: &mut StoreWorld, device: String, id: String, time: String) {
    let result = nunc_store::set_habit_archived(world.crdt.conn(&device), &id, false, at(&time));
    record(world, result);
}

#[given(expr = "device {word} deletes the habit {string} at {word}")]
#[when(expr = "device {word} deletes the habit {string} at {word}")]
fn deletes_habit(world: &mut StoreWorld, device: String, id: String, time: String) {
    let result = nunc_store::delete_habit(world.crdt.conn(&device), &id, at(&time));
    record(world, result);
}

#[given(expr = "device {word} ticks the habit {string} on {word} at {word}")]
#[when(expr = "device {word} ticks the habit {string} on {word} at {word}")]
fn ticks_habit(world: &mut StoreWorld, device: String, id: String, date: String, time: String) {
    let result =
        nunc_store::set_habit_done(world.crdt.conn(&device), &id, day(&date), true, at(&time));
    record(world, result);
}

#[given(expr = "device {word} unticks the habit {string} on {word} at {word}")]
#[when(expr = "device {word} unticks the habit {string} on {word} at {word}")]
fn unticks_habit(world: &mut StoreWorld, device: String, id: String, date: String, time: String) {
    let result =
        nunc_store::set_habit_done(world.crdt.conn(&device), &id, day(&date), false, at(&time));
    record(world, result);
}

#[then(expr = "on device {word} the habits list {string}")]
fn device_lists(world: &mut StoreWorld, device: String, expected: String) {
    assert_eq!(listed(world.crdt.conn(&device)), expected);
}

#[then(expr = "on both devices the habits list {string}")]
fn both_list(world: &mut StoreWorld, expected: String) {
    for device in ["A", "B"] {
        assert_eq!(listed(world.crdt.conn(device)), expected, "Gerät {device}");
    }
}

#[then(expr = "on device {word} the ticks from {word} to {word} list {string}")]
fn device_ticks(
    world: &mut StoreWorld,
    device: String,
    from: String,
    to: String,
    expected: String,
) {
    assert_eq!(ticks(world.crdt.conn(&device), &from, &to), expected);
}

#[then(expr = "on both devices the ticks from {word} to {word} list {string}")]
fn both_ticks(world: &mut StoreWorld, from: String, to: String, expected: String) {
    for device in ["A", "B"] {
        assert_eq!(
            ticks(world.crdt.conn(device), &from, &to),
            expected,
            "Gerät {device}"
        );
    }
}

#[then(expr = "device {word} has {int} habit rows")]
fn has_habit_rows(world: &mut StoreWorld, device: String, expected: i64) {
    assert_eq!(count(world.crdt.conn(&device), "habits"), expected);
}

#[then(expr = "device {word} has {int} habit log rows")]
fn has_log_rows(world: &mut StoreWorld, device: String, expected: i64) {
    assert_eq!(count(world.crdt.conn(&device), "habit_logs"), expected);
}

#[then(expr = "the habits report {string}")]
fn reports(world: &mut StoreWorld, expected: String) {
    assert_eq!(world.crdt.habit_error.as_deref(), Some(expected.as_str()));
}

#[then(
    expr = "on device {word} the field {string} of the habit {string} was last written at {word}"
)]
fn field_written_at(
    world: &mut StoreWorld,
    device: String,
    field: String,
    id: String,
    time: String,
) {
    let ms: i64 = world
        .crdt
        .conn(&device)
        .query_row(
            "SELECT hlc_ms FROM habit_field_clock WHERE sync_id = ?1 AND field = ?2",
            rusqlite::params![id, field],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| panic!("kein Stempel für {id}/{field}"));
    assert_eq!(ms, at(&time).timestamp_millis(), "{id}/{field}");
}

#[then(
    expr = "on device {word} the tick of the habit {string} on {word} was last written at {word}"
)]
fn tick_written_at(world: &mut StoreWorld, device: String, id: String, date: String, time: String) {
    let ms: i64 = world
        .crdt
        .conn(&device)
        .query_row(
            "SELECT hlc_ms FROM habit_logs WHERE sync_id = ?1",
            [log_id(&id, day(&date))],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| panic!("kein Haken für {id} am {date}"));
    assert_eq!(ms, at(&time).timestamp_millis(), "{id} am {date}");
}
