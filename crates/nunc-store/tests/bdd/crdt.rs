//! Steps für `features/sync/crdt_store.feature`: Stempel in der Datenbank und der Austausch zweier Geräte
//! (ADR 0016). Jedes Gerät ist eine eigene In-Memory-Datenbank.

use chrono::NaiveDate;
use cucumber::{given, then, when};
use nunc_store::{Applied, Connection};

use crate::{StoreWorld, at};

#[derive(Debug, Default)]
pub struct State {
    a: Option<Connection>,
    b: Option<Connection>,
    ids: u32,
    last_exchange: Applied,
}

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 4).unwrap()
}

impl State {
    pub(crate) fn conn(&mut self, device: &str) -> &mut Connection {
        let slot = match device {
            "A" => &mut self.a,
            "B" => &mut self.b,
            other => panic!("unbekanntes Gerät {other}"),
        };
        slot.get_or_insert_with(|| {
            let conn = nunc_store::open_in_memory().expect("open");
            nunc_store::device_id(&conn, || format!("dev-{}", device.to_lowercase()))
                .expect("device id");
            conn
        })
    }

    fn next_sync_id(&mut self) -> String {
        self.ids += 1;
        format!("sync-{}", self.ids)
    }

    fn add(&mut self, device: &str, text: &str, time: &str) {
        let sync_id = self.next_sync_id();
        nunc_store::add_todo(self.conn(device), text, &sync_id, at(time)).expect("add");
    }

    /// Beide Geräte tauschen ihren Stand und spielen das Ergebnis ein.
    fn exchange(&mut self) -> Applied {
        let mut ids = self.ids;
        let mut new_id = || {
            ids += 1;
            format!("sync-{ids}")
        };
        let state_a = nunc_store::local_state(self.conn("A"), &mut new_id).expect("state A");
        let state_b = nunc_store::local_state(self.conn("B"), &mut new_id).expect("state B");
        self.ids = ids;
        let now = at("12:00:00").timestamp_millis();
        let for_a = state_a.merge_remote(&state_b, now);
        let for_b = state_b.merge_remote(&state_a, now);
        let on_a = nunc_store::apply_state(self.conn("A"), &for_a).expect("apply A");
        let on_b = nunc_store::apply_state(self.conn("B"), &for_b).expect("apply B");
        Applied {
            todos_added: on_a.todos_added + on_b.todos_added,
            todos_replaced: on_a.todos_replaced + on_b.todos_replaced,
            sessions_added: on_a.sessions_added + on_b.sessions_added,
            sessions_replaced: on_a.sessions_replaced + on_b.sessions_replaced,
        }
    }
}

fn lines(conn: &Connection) -> String {
    nunc_store::list_todos(conn)
        .expect("list")
        .iter()
        .map(|todo| todo.raw_text.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn clock(conn: &Connection, todo_id: i64, field: &str) -> (i64, i64) {
    conn.query_row(
        "SELECT hlc_ms, hlc_counter FROM todo_field_clock WHERE todo_id = ?1 AND field = ?2",
        rusqlite::params![todo_id, field],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .unwrap_or_else(|_| panic!("kein Stempel für {field}"))
}

#[given(expr = "device {word} has the todo {string} created at {word}")]
fn has_todo(world: &mut StoreWorld, device: String, text: String, time: String) {
    world.crdt.add(&device, &text, &time);
}

#[given(expr = "device {word} has the todos {string} created at {word}")]
fn has_todos(world: &mut StoreWorld, device: String, texts: String, time: String) {
    for text in texts.split(", ") {
        world.crdt.add(&device, text, &time);
    }
}

#[given(expr = "device A and device B share the todo {string}")]
fn share_todo(world: &mut StoreWorld, text: String) {
    world.crdt.add("A", &text, "09:00:00");
    world.crdt.exchange();
}

#[when(expr = "device {word} changes the todo to {string} at {word}")]
fn changes_todo(world: &mut StoreWorld, device: String, text: String, time: String) {
    nunc_store::update_todo(world.crdt.conn(&device), 1, &text, at(&time)).expect("update");
}

#[when(expr = "device {word} completes the todo at {word}")]
fn completes_todo(world: &mut StoreWorld, device: String, time: String) {
    nunc_store::set_todo_completed(world.crdt.conn(&device), 1, true, today(), at(&time))
        .expect("complete");
}

#[given(expr = "device {word} deletes the todo at {word}")]
#[when(expr = "device {word} deletes the todo at {word}")]
fn deletes_todo(world: &mut StoreWorld, device: String, time: String) {
    nunc_store::delete_todo(world.crdt.conn(&device), 1, at(&time)).expect("delete");
}

#[when(expr = "device {word} counts {int} pomodoro(s) for the todo")]
fn counts_pomodoros(world: &mut StoreWorld, device: String, count: u32) {
    for _ in 0..count {
        nunc_store::increment_pomodoro_count(world.crdt.conn(&device), 1).expect("pomodoro");
    }
}

#[when(expr = "device {word} reorders the todos to {string} at {word}")]
fn reorders(world: &mut StoreWorld, device: String, ids: String, time: String) {
    let ids: Vec<i32> = ids
        .split(',')
        .map(|id| id.trim().parse().expect("id"))
        .collect();
    nunc_store::reorder_todos(world.crdt.conn(&device), &ids, at(&time)).expect("reorder");
}

#[given("the devices exchange their state")]
#[when("the devices exchange their state")]
fn exchange(world: &mut StoreWorld) {
    world.crdt.last_exchange = world.crdt.exchange();
}

#[then(expr = "on device {word} the field {string} of the todo was last written at {word}")]
fn field_written_at(world: &mut StoreWorld, device: String, field: String, time: String) {
    let (ms, _) = clock(world.crdt.conn(&device), 1, &field);
    assert_eq!(ms, at(&time).timestamp_millis(), "{field}");
}

#[then(
    expr = "on device {word} the field {string} of the todo was last written at {word} with counter {int}"
)]
fn field_written_with_counter(
    world: &mut StoreWorld,
    device: String,
    field: String,
    time: String,
    counter: i64,
) {
    let (ms, count) = clock(world.crdt.conn(&device), 1, &field);
    assert_eq!(
        (ms, count),
        (at(&time).timestamp_millis(), counter),
        "{field}"
    );
}

#[then(expr = "on device {word} the position of todo {int} was last written at {word}")]
fn position_written_at(world: &mut StoreWorld, device: String, id: i64, time: String) {
    let (ms, _) = clock(world.crdt.conn(&device), id, "position");
    assert_eq!(ms, at(&time).timestamp_millis(), "Todo {id}");
}

#[then(expr = "on device {word} the todos are listed as {string}")]
fn device_lists(world: &mut StoreWorld, device: String, expected: String) {
    assert_eq!(lines(world.crdt.conn(&device)), expected);
}

#[then(expr = "on both devices the todos are listed as {string}")]
fn both_list(world: &mut StoreWorld, expected: String) {
    for device in ["A", "B"] {
        assert_eq!(lines(world.crdt.conn(device)), expected, "Gerät {device}");
    }
}

#[then(expr = "on both devices the todo line is {string}")]
fn both_line(world: &mut StoreWorld, expected: String) {
    both_list(world, expected);
}

#[then(expr = "on both devices the todo has {int} pomodoros")]
fn both_pomodoros(world: &mut StoreWorld, expected: u32) {
    for device in ["A", "B"] {
        let todos = nunc_store::list_todos(world.crdt.conn(device)).expect("list");
        assert_eq!(todos[0].pomodoro_count, expected, "Gerät {device}");
    }
}

#[then("the second exchange changed nothing")]
fn nothing_changed(world: &mut StoreWorld) {
    assert_eq!(world.crdt.last_exchange, Applied::default());
}

#[then(expr = "every todo has a stamp for each of its {int} fields")]
fn every_field_stamped(world: &mut StoreWorld, fields: i64) {
    let todos: i64 = world
        .conn()
        .query_row("SELECT COUNT(*) FROM todos", [], |row| row.get(0))
        .unwrap();
    let stamps: i64 = world
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM todo_field_clock WHERE hlc_device = 'legacy'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(todos > 0);
    assert_eq!(stamps, todos * fields);
}

#[then("the position keys follow the old order")]
fn keys_follow_order(world: &mut StoreWorld) {
    let keys: Vec<String> = world
        .conn()
        .prepare(
            "SELECT k.key FROM todos t JOIN todo_position_key k ON k.todo_id = t.id
             ORDER BY t.position, t.id",
        )
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let todos: i64 = world
        .conn()
        .query_row("SELECT COUNT(*) FROM todos", [], |row| row.get(0))
        .unwrap();
    assert_eq!(i64::try_from(keys.len()).unwrap(), todos);
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]), "{keys:?}");
    assert!(keys.iter().all(|key| !key.ends_with('0')));
}

#[given(expr = "device {word} has a completed session started at {word}")]
fn has_session(world: &mut StoreWorld, device: String, time: String) {
    let started = at(&time).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    world
        .crdt
        .conn(&device)
        .execute(
            "INSERT INTO sessions (mode, session_type, status, started_at, ended_at, planned_min,
                                   actual_min, break_min, sync_device_id)
             VALUES ('pomodoro', 'work', 'completed', ?1, ?1, 25, 25, 5, ?2)",
            rusqlite::params![started, format!("dev-{}", device.to_lowercase())],
        )
        .expect("session");
}

#[then(expr = "the exchange added {int} todos, replaced {int} and added {int} sessions")]
fn exchange_counts(world: &mut StoreWorld, added: u32, replaced: u32, sessions: u32) {
    let last = world.crdt.last_exchange;
    assert_eq!(
        (last.todos_added, last.todos_replaced, last.sessions_added),
        (added, replaced, sessions)
    );
}

#[then(expr = "on both devices the todo was last changed at {word}")]
fn both_changed_at(world: &mut StoreWorld, time: String) {
    let expected = at(&time).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    for device in ["A", "B"] {
        let updated: String = world
            .crdt
            .conn(device)
            .query_row("SELECT updated_at FROM todos WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(updated, expected, "Gerät {device}");
    }
}

#[then(expr = "on device {word} the database holds {int} todos")]
fn holds_todos(world: &mut StoreWorld, device: String, expected: i64) {
    let count: i64 = world
        .crdt
        .conn(&device)
        .query_row("SELECT COUNT(*) FROM todos", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, expected);
}

#[then(expr = "on device {word} the database holds {int} sessions")]
fn holds_sessions(world: &mut StoreWorld, device: String, expected: i64) {
    let count: i64 = world
        .crdt
        .conn(&device)
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, expected);
}
