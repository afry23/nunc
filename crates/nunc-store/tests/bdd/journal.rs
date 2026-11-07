//! Steps für `features/sync/journal_store.feature`: Journal in der Datenbank und im Austausch zweier Geräte
//! (ADR 0026). Die Geräte stammen aus `crdt.rs`.

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use cucumber::{given, then, when};
use nunc_store::Connection;

use crate::{StoreWorld, at};

fn day(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap_or_else(|_| panic!("Datum {text:?}"))
}

/// Merkt das Ergebnis eines Schreibvorgangs; ein Fehler wird zur Meldung für „the journal reports“.
fn record<T>(world: &mut StoreWorld, result: Result<T, nunc_store::StoreError>) {
    world.crdt.journal_error = result.err().map(|error| error.to_string());
}

fn listed(conn: &Connection, from: &str, to: &str) -> String {
    nunc_store::list_journal(conn, day(from), day(to))
        .expect("list")
        .iter()
        .map(|entry| {
            format!(
                "{} {}: {}",
                entry.date.value,
                entry.kind.as_str(),
                entry.text.value
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn rows(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get(0))
        .expect("count")
}

#[given(expr = "device {word} writes the day note {string} for {word} at {word}")]
#[when(expr = "device {word} writes the day note {string} for {word} at {word}")]
fn writes_day_note(
    world: &mut StoreWorld,
    device: String,
    text: String,
    date: String,
    time: String,
) {
    let result = nunc_store::save_day_note(world.crdt.conn(&device), day(&date), &text, at(&time));
    record(world, result);
}

#[given(expr = "device {word} writes the week note {string} for {word} at {word}")]
#[when(expr = "device {word} writes the week note {string} for {word} at {word}")]
fn writes_week_note(
    world: &mut StoreWorld,
    device: String,
    text: String,
    date: String,
    time: String,
) {
    let result = nunc_store::save_week_note(world.crdt.conn(&device), day(&date), &text, at(&time));
    record(world, result);
}

#[when(expr = "device {word} writes a day note of {int} characters for {word} at {word}")]
fn writes_long_note(
    world: &mut StoreWorld,
    device: String,
    length: usize,
    date: String,
    time: String,
) {
    let text = "x".repeat(length);
    let result = nunc_store::save_day_note(world.crdt.conn(&device), day(&date), &text, at(&time));
    record(world, result);
}

#[given(expr = "device {word} adds the achievement {string} {string} for {word} at {word}")]
#[when(expr = "device {word} adds the achievement {string} {string} for {word} at {word}")]
fn adds_achievement(
    world: &mut StoreWorld,
    device: String,
    id: String,
    text: String,
    date: String,
    time: String,
) {
    let result =
        nunc_store::add_achievement(world.crdt.conn(&device), &id, day(&date), &text, at(&time));
    record(world, result);
}

#[when(expr = "device {word} changes the achievement {string} to {word} {string} at {word}")]
fn changes_achievement(
    world: &mut StoreWorld,
    device: String,
    id: String,
    date: String,
    text: String,
    time: String,
) {
    let result =
        nunc_store::update_achievement(world.crdt.conn(&device), &id, day(&date), &text, at(&time));
    record(world, result);
}

#[given(expr = "device {word} deletes the achievement {string} at {word}")]
#[when(expr = "device {word} deletes the achievement {string} at {word}")]
fn deletes_achievement(world: &mut StoreWorld, device: String, id: String, time: String) {
    let result = nunc_store::delete_achievement(world.crdt.conn(&device), &id, at(&time));
    record(world, result);
}

#[when(expr = "the devices exchange their state on {word}")]
fn exchange_on(world: &mut StoreWorld, date: String) {
    let noon = day(&date).and_time(NaiveTime::from_hms_opt(12, 0, 0).expect("Zeit"));
    let now = Utc.from_utc_datetime(&noon).timestamp_millis();
    world.crdt.last_exchange = world.crdt.exchange_at(now);
}

#[then(expr = "on device {word} the journal from {word} to {word} lists {string}")]
fn device_lists(
    world: &mut StoreWorld,
    device: String,
    from: String,
    to: String,
    expected: String,
) {
    assert_eq!(listed(world.crdt.conn(&device), &from, &to), expected);
}

#[then(expr = "on both devices the journal from {word} to {word} lists {string}")]
fn both_list(world: &mut StoreWorld, from: String, to: String, expected: String) {
    for device in ["A", "B"] {
        assert_eq!(
            listed(world.crdt.conn(device), &from, &to),
            expected,
            "Gerät {device}"
        );
    }
}

#[then(expr = "device {word} has {int} journal rows")]
fn has_rows(world: &mut StoreWorld, device: String, expected: i64) {
    assert_eq!(rows(world.crdt.conn(&device)), expected);
}

#[then(expr = "the journal reports {string}")]
fn reports(world: &mut StoreWorld, expected: String) {
    assert_eq!(world.crdt.journal_error.as_deref(), Some(expected.as_str()));
}

#[then(
    expr = "on device {word} the field {string} of the entry {string} was last written at {word}"
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
            "SELECT hlc_ms FROM journal_field_clock WHERE sync_id = ?1 AND field = ?2",
            rusqlite::params![id, field],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| panic!("kein Stempel für {id}/{field}"));
    assert_eq!(ms, at(&time).timestamp_millis(), "{id}/{field}");
}
