//! Steps für `features/sync/crdt.feature` (CRDT-Zustand, HLC, Fractional-Index; ADR 0016).

use cucumber::{given, then, when};
use nunc_core::crdt::{SyncState, TodoCrdt};
use nunc_core::hlc::{Hlc, MAX_DRIFT_MS};
use nunc_core::model::{SyncFile, SyncSession, SyncTodo};
use nunc_core::position::position_between;
use nunc_core::sync_document::{LEGACY_DEVICE, SyncDocument, legacy_position_key};

use crate::DomainWorld;

/// „Jetzt“ der Szenarien: 2026-03-01 12:00:00 UTC.
const NOW_MS: i64 = 1_772_366_400_000;
/// Mitternacht desselben Tages; Uhrzeiten der Szenarien zählen von hier.
const DAY_MS: i64 = NOW_MS - 12 * 3_600_000;
const DAY_30: i64 = 30 * 24 * 3_600_000;

#[derive(Debug, Default)]
pub struct State {
    todo_id: String,
    a: SyncState,
    b: SyncState,
    /// Ergebnis von `merge(a, b)`; `merged_reverse` ist `merge(b, a)`.
    merged: Option<SyncState>,
    merged_reverse: Option<SyncState>,
    /// Zähler für Schreibvorgänge ohne Uhrzeit, damit spätere Schritte später stempeln.
    writes: i64,
    last_stamp: Option<Hlc>,
    stamp: Option<Hlc>,
    new_position: Option<String>,
    legacy: Vec<SyncTodo>,
    converted: Option<SyncState>,
    document_state: Option<SyncState>,
    read_error: Option<String>,
}

fn device_id(device: &str) -> String {
    format!("dev-{}", device.to_lowercase())
}

/// `10:00:05` → Millisekunden seit Mitternacht des Szenario-Tags.
fn time_of_day(text: &str) -> i64 {
    let parts: Vec<i64> = text
        .split(':')
        .map(|part| part.parse().expect("Uhrzeit hh:mm:ss"))
        .collect();
    assert_eq!(parts.len(), 3, "Uhrzeit hh:mm:ss erwartet: {text}");
    DAY_MS + ((parts[0] * 60 + parts[1]) * 60 + parts[2]) * 1000
}

/// Stempel `wall+counter` (Gerät `d`).
fn parse_stamp(text: &str) -> Option<Hlc> {
    if text == "none" {
        return None;
    }
    let (wall, counter) = text.split_once('+').expect("Stempel wall+counter");
    Some(Hlc::new(
        wall.parse().expect("wall"),
        counter.parse().expect("counter"),
        "d",
    ))
}

fn format_stamp(stamp: &Hlc) -> String {
    format!("{}+{}", stamp.wall_ms, stamp.counter)
}

impl State {
    fn side(&mut self, device: &str) -> &mut SyncState {
        match device {
            "A" => &mut self.a,
            "B" => &mut self.b,
            other => panic!("unbekanntes Gerät {other}"),
        }
    }

    /// Ändert das Todo auf einem Gerät mit einem Stempel; ohne Uhrzeit mit dem nächsten freien Zeitpunkt.
    fn edit(&mut self, device: &str, at_ms: Option<i64>, change: impl FnOnce(&mut TodoCrdt, Hlc)) {
        let ms = at_ms.unwrap_or_else(|| {
            self.writes += 1;
            DAY_MS + 10 * 3_600_000 + self.writes * 1000
        });
        let id = self.todo_id.clone();
        let clock = Hlc::new(ms, 0, device_id(device));
        let todo = self
            .side(device)
            .todos
            .get_mut(&id)
            .expect("Todo fehlt auf dem Gerät");
        change(todo, clock);
    }

    fn merged_todo(&self) -> &TodoCrdt {
        self.merged
            .as_ref()
            .expect("noch nicht zusammengeführt")
            .todos
            .get(&self.todo_id)
            .expect("Todo fehlt im Ergebnis")
    }
}

fn start_todo(world: &mut DomainWorld, id: String, pomodoros: u32) {
    let state = &mut world.crdt;
    let mut todo = TodoCrdt::new(id.clone(), &Hlc::new(DAY_MS + 9 * 3_600_000, 0, "base"));
    if pomodoros > 0 {
        todo.pomodoros.insert("base".to_owned(), pomodoros);
    }
    state.a.todos.insert(id.clone(), todo.clone());
    state.b.todos.insert(id.clone(), todo);
    state.todo_id = id;
}

#[given(expr = "a todo {string} on both devices")]
fn todo_on_both(world: &mut DomainWorld, id: String) {
    start_todo(world, id, 0);
}

#[given(expr = "a todo {string} with {int} pomodoros on both devices")]
fn todo_with_pomodoros(world: &mut DomainWorld, id: String, pomodoros: u32) {
    start_todo(world, id, pomodoros);
}

#[given(expr = "device {word} completes the todo")]
fn completes(world: &mut DomainWorld, device: String) {
    world.crdt.edit(&device, None, |todo, clock| {
        todo.completed.set(true, clock.clone());
        todo.completed_at.set(Some("2026-03-01".to_owned()), clock);
    });
}

#[given(expr = "device {word} changes its text to {string}")]
fn changes_text(world: &mut DomainWorld, device: String, text: String) {
    world.crdt.edit(&device, None, |todo, clock| {
        todo.description.set(text, clock)
    });
}

#[given(expr = "device {word} changes its text at {word}")]
fn changes_text_at(world: &mut DomainWorld, device: String, time: String) {
    world
        .crdt
        .edit(&device, Some(time_of_day(&time)), |todo, clock| {
            todo.description.set("edited".to_owned(), clock);
        });
}

#[given(expr = "device {word} changes its text with a clock 30 days ahead")]
fn changes_text_far_ahead(world: &mut DomainWorld, device: String) {
    world
        .crdt
        .edit(&device, Some(NOW_MS + DAY_30), |todo, clock| {
            todo.description.set("from the future".to_owned(), clock);
        });
}

#[given(expr = "device {word} sets the priority to {string} at {word}")]
fn sets_priority(world: &mut DomainWorld, device: String, priority: String, time: String) {
    world
        .crdt
        .edit(&device, Some(time_of_day(&time)), |todo, clock| {
            todo.priority.set(Some(priority), clock);
        });
}

#[given("both devices set the priority at the same instant")]
fn sets_priority_same_instant(world: &mut DomainWorld) {
    for device in ["A", "B"] {
        world
            .crdt
            .edit(device, Some(DAY_MS + 10 * 3_600_000), |todo, clock| {
                todo.priority.set(Some(device.to_owned()), clock);
            });
    }
}

#[given(expr = "device {word} deletes the todo at {word}")]
fn deletes(world: &mut DomainWorld, device: String, time: String) {
    world
        .crdt
        .edit(&device, Some(time_of_day(&time)), |todo, clock| {
            todo.deleted.set(true, clock);
        });
}

#[given(expr = "device {word} adds {int} pomodoro(s)")]
fn adds_pomodoros(world: &mut DomainWorld, device: String, count: u32) {
    world.crdt.edit(&device, None, |todo, clock| {
        for _ in 0..count {
            todo.add_pomodoro(&clock.device_id);
        }
    });
}

fn session(device: &str, started_at: &str) -> SyncSession {
    SyncSession {
        device_id: device_id(device),
        mode: "pomodoro".to_owned(),
        session_type: "work".to_owned(),
        status: "completed".to_owned(),
        started_at: started_at.to_owned(),
        ended_at: None,
        planned_min: 25,
        actual_min: 25,
        break_min: 5,
    }
}

#[given(expr = "a session started at {string} on device {word}")]
fn session_on_device(world: &mut DomainWorld, started_at: String, device: String) {
    let session = session(&device, &started_at);
    world
        .crdt
        .side(&device)
        .sessions
        .insert(nunc_core::crdt::session_key(&session), session);
}

#[given("the same session of device A is also known to device B")]
fn session_known_to_b(world: &mut DomainWorld) {
    let (key, session) = world
        .crdt
        .a
        .sessions
        .iter()
        .next()
        .map(|(key, session)| (key.clone(), session.clone()))
        .expect("Gerät A hat keine Session");
    world.crdt.b.sessions.insert(key, session);
}

#[when("the states are merged in either order")]
fn merged_either_order(world: &mut DomainWorld) {
    let state = &mut world.crdt;
    let forward = state.a.merge(&state.b);
    let reverse = state.b.merge(&state.a);
    assert_eq!(forward, reverse, "Merge hängt von der Reihenfolge ab");
    state.merged = Some(forward);
    state.merged_reverse = Some(reverse);
}

#[when("device A receives the state of device B")]
fn a_receives_b(world: &mut DomainWorld) {
    let state = &mut world.crdt;
    state.merged = Some(state.a.merge_remote(&state.b, NOW_MS));
}

#[then("the merged todo is completed")]
fn todo_completed(world: &mut DomainWorld) {
    assert!(world.crdt.merged_todo().completed.value);
}

#[then(expr = "its text is {string}")]
fn text_is(world: &mut DomainWorld, text: String) {
    assert_eq!(world.crdt.merged_todo().description.value, text);
}

#[then(expr = "the merged line is {string}")]
fn line_is(world: &mut DomainWorld, line: String) {
    assert_eq!(world.crdt.merged_todo().raw_text(), line);
}

#[then(expr = "the merged priority is {string}")]
fn priority_is(world: &mut DomainWorld, priority: String) {
    assert_eq!(
        world.crdt.merged_todo().priority.value.as_deref(),
        Some(priority.as_str())
    );
}

#[then("both orders give the same priority")]
fn same_priority(world: &mut DomainWorld) {
    let state = &world.crdt;
    let id = &state.todo_id;
    let priority = |merged: &Option<SyncState>| {
        merged.as_ref().expect("zusammengeführt").todos[id]
            .priority
            .value
            .clone()
    };
    assert!(priority(&state.merged).is_some(), "Priorität fehlt");
    assert_eq!(priority(&state.merged), priority(&state.merged_reverse));
}

#[then("the todo exists")]
fn todo_exists(world: &mut DomainWorld) {
    assert!(!world.crdt.merged_todo().is_deleted());
}

#[then("the todo is deleted")]
fn todo_is_deleted(world: &mut DomainWorld) {
    assert!(world.crdt.merged_todo().is_deleted());
}

#[then(expr = "the todo has {int} pomodoros")]
fn todo_pomodoros(world: &mut DomainWorld, count: u32) {
    assert_eq!(world.crdt.merged_todo().pomodoro_count(), count);
}

#[then(expr = "there are {int} sessions")]
fn session_count(world: &mut DomainWorld, count: usize) {
    let merged = world.crdt.merged.as_ref().expect("zusammengeführt");
    assert_eq!(merged.sessions.len(), count);
}

#[then("the stored clock is at most 24 hours ahead of now")]
fn clock_is_capped(world: &mut DomainWorld) {
    let todo = world.crdt.merged_todo();
    let latest = [
        &todo.description.clock,
        &todo.completed.clock,
        &todo.deleted.clock,
    ]
    .into_iter()
    .map(|clock| clock.wall_ms)
    .max()
    .expect("Stempel");
    assert_eq!(latest, NOW_MS + MAX_DRIFT_MS, "auf die Grenze gekappt");
}

#[given(expr = "the last known stamp is {word}")]
fn last_stamp(world: &mut DomainWorld, stamp: String) {
    world.crdt.last_stamp = parse_stamp(&stamp);
}

#[when(expr = "the clock reads {int}")]
fn clock_reads(world: &mut DomainWorld, now: i64) {
    let state = &mut world.crdt;
    state.stamp = Some(Hlc::next(state.last_stamp.as_ref(), now, "d"));
}

#[then(expr = "the next stamp is {word}")]
fn next_stamp(world: &mut DomainWorld, expected: String) {
    let stamp = world.crdt.stamp.as_ref().expect("kein Stempel");
    assert_eq!(format_stamp(stamp), expected);
}

#[when(expr = "I place a todo between {string} and {string}")]
fn place_between(world: &mut DomainWorld, before: String, after: String) {
    let bound = |text: &str| (!text.is_empty()).then(|| text.to_owned());
    world.crdt.new_position = Some(position_between(
        bound(&before).as_deref(),
        bound(&after).as_deref(),
    ));
    world.crdt.todo_id = format!("{before}|{after}");
}

#[then("the new position sorts strictly between them")]
fn position_is_between(world: &mut DomainWorld) {
    let position = world.crdt.new_position.clone().expect("keine Position");
    let (before, after) = world.crdt.todo_id.split_once('|').expect("Grenzen");
    assert!(
        before < position.as_str(),
        "{position} nicht hinter {before}"
    );
    if !after.is_empty() {
        assert!(position.as_str() < after, "{position} nicht vor {after}");
    }
    assert!(
        !position.ends_with('0'),
        "Schlüssel endet auf 0: {position}"
    );
}

fn legacy_todo(
    sync_id: &str,
    line: &str,
    description: &str,
    position: u32,
    updated: &str,
) -> SyncTodo {
    SyncTodo {
        sync_id: sync_id.to_owned(),
        device_id: "fixture".to_owned(),
        raw_text: line.to_owned(),
        priority: None,
        completed: false,
        completed_at: None,
        created_at: None,
        description: description.to_owned(),
        project_tags: vec![],
        context_tags: vec![],
        pomodoro_count: 0,
        position,
        updated_at: updated.to_owned(),
        deleted: false,
    }
}

#[given(
    expr = "a legacy todo {string} with the line {string} and the description {string} at position {int} updated at {string}"
)]
fn legacy_todo_step(
    world: &mut DomainWorld,
    sync_id: String,
    line: String,
    description: String,
    position: u32,
    updated: String,
) {
    let mut todo = legacy_todo(&sync_id, &line, &description, position, &updated);
    // Priorität und Erstellungsdatum stehen in der Go-Datei auch in eigenen Feldern.
    if line.starts_with("(A) ") {
        todo.priority = Some("A".to_owned());
        todo.created_at = Some("2026-09-20T00:00:00Z".to_owned());
    }
    world.crdt.legacy.push(todo);
}

#[given(expr = "a deleted legacy todo {string} updated at {string}")]
fn deleted_legacy_todo(world: &mut DomainWorld, sync_id: String, updated: String) {
    let mut todo = legacy_todo(&sync_id, "Gone", "Gone", 1, &updated);
    todo.deleted = true;
    world.crdt.legacy.push(todo);
}

#[when("the legacy file is converted")]
fn convert_legacy(world: &mut DomainWorld) {
    let file = SyncFile {
        schema_version: 1,
        synced_at: chrono::DateTime::from_timestamp_millis(NOW_MS).unwrap(),
        device_id: "fixture".to_owned(),
        todos: world.crdt.legacy.clone(),
        sessions: vec![],
    };
    world.crdt.converted = Some(SyncState::from_legacy(&file));
}

fn converted_todo<'a>(world: &'a DomainWorld, id: &str) -> &'a TodoCrdt {
    world
        .crdt
        .converted
        .as_ref()
        .expect("nicht konvertiert")
        .todos
        .get(id)
        .unwrap_or_else(|| panic!("Todo {id} fehlt"))
}

#[then(expr = "the todo {string} has the line {string}")]
fn converted_line(world: &mut DomainWorld, id: String, line: String) {
    assert_eq!(converted_todo(world, &id).raw_text(), line);
}

#[then(expr = "every field of the todo {string} is stamped by the legacy device at {string}")]
fn converted_stamps(world: &mut DomainWorld, id: String, updated: String) {
    let todo = converted_todo(world, &id);
    let expected = Hlc::new(
        chrono::DateTime::parse_from_rfc3339(&updated)
            .unwrap()
            .timestamp_millis(),
        0,
        LEGACY_DEVICE,
    );
    let clocks = [
        &todo.priority.clock,
        &todo.completed.clock,
        &todo.completed_at.clock,
        &todo.created_at.clock,
        &todo.description.clock,
        &todo.position.clock,
        &todo.deleted.clock,
    ];
    assert!(clocks.iter().all(|clock| **clock == expected), "{clocks:?}");
}

#[then(expr = "the todo {string} sorts before the todo {string}")]
fn converted_order(world: &mut DomainWorld, first: String, second: String) {
    let first = converted_todo(world, &first).position.value.clone();
    let second = converted_todo(world, &second).position.value.clone();
    assert!(first < second, "{first} nicht vor {second}");
}

#[then(expr = "the position key of the todo {string} is {string}")]
fn converted_key(world: &mut DomainWorld, id: String, key: String) {
    assert_eq!(converted_todo(world, &id).position.value, key);
    assert_eq!(legacy_position_key(1), key);
}

#[then(expr = "the todo {string} is deleted")]
fn converted_deleted(world: &mut DomainWorld, id: String) {
    assert!(converted_todo(world, &id).is_deleted());
}

#[when("the state of device A is written to a sync file and read back")]
fn document_roundtrip(world: &mut DomainWorld) {
    let state = &mut world.crdt;
    let document = SyncDocument::from_state(
        &state.a,
        chrono::DateTime::from_timestamp_millis(NOW_MS).unwrap(),
    );
    let json = serde_json::to_string_pretty(&document).unwrap();
    let read: SyncDocument = serde_json::from_str(&json).unwrap();
    state.document_state = Some(read.into_state().unwrap());
}

#[then("the state is unchanged")]
fn state_unchanged(world: &mut DomainWorld) {
    assert_eq!(world.crdt.document_state.as_ref(), Some(&world.crdt.a));
}

#[when(expr = "a sync file of format {int} is read")]
fn read_other_format(world: &mut DomainWorld, format: u32) {
    let json = serde_json::json!({
        "schema_version": format,
        "synced_at": "2026-03-01T12:00:00Z",
    });
    let document: SyncDocument = serde_json::from_value(json).unwrap();
    world.crdt.read_error = document.into_state().err().map(|error| error.to_string());
}

#[then(expr = "reading fails with {string}")]
fn reading_fails(world: &mut DomainWorld, message: String) {
    assert_eq!(world.crdt.read_error.as_deref(), Some(message.as_str()));
}

#[given(expr = "the legacy todo {string} has {int} pomodoros")]
fn legacy_pomodoros(world: &mut DomainWorld, id: String, count: u32) {
    world
        .crdt
        .legacy
        .iter_mut()
        .find(|todo| todo.sync_id == id)
        .expect("Todo")
        .pomodoro_count = count;
}

#[then(expr = "the todo {string} counts {int} pomodoros for the legacy device")]
fn legacy_counter(world: &mut DomainWorld, id: String, count: u32) {
    assert_eq!(
        converted_todo(world, &id).pomodoros.get(LEGACY_DEVICE),
        Some(&count)
    );
}

#[then(expr = "the todo {string} has no pomodoro counters")]
fn no_counters(world: &mut DomainWorld, id: String) {
    assert!(converted_todo(world, &id).pomodoros.is_empty());
}
