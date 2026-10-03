//! Steps für `features/sync/merge.feature` und `features/sync/compat_go.feature` (Dateiformat, Merge).

use chrono::{TimeZone, Utc};
use cucumber::{gherkin::Step, given, then, when};
use nunc_core::model::{SyncFile, SyncSession, SyncTodo};
use nunc_core::sync_merge::{
    SessionMerge, TodoMerge, merge_sessions, merge_sync_data, merge_todos,
};

use crate::DomainWorld;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/");

#[derive(Debug, Default)]
pub struct State {
    local_todos: Vec<SyncTodo>,
    remote_todos: Vec<SyncTodo>,
    local_sessions: Vec<SyncSession>,
    remote_sessions: Vec<SyncSession>,
    todo_merge: Option<TodoMerge>,
    session_merge: Option<SessionMerge>,
    local_file: Option<SyncFile>,
    remote_file: Option<SyncFile>,
    merged_file: Option<SyncFile>,
    read_file: Option<SyncFile>,
    read_text: String,
    written_json: String,
}

fn table(step: &Step) -> (Vec<String>, Vec<Vec<String>>) {
    let rows = &step.table.as_ref().expect("Tabelle fehlt").rows;
    (rows[0].clone(), rows[1..].to_vec())
}

/// Erwartete Zeilen ohne Überschrift; ein fehlender Tabellenblock heißt „keine Zeilen“.
fn expected_rows(step: &Step) -> Vec<Vec<String>> {
    step.table
        .as_ref()
        .map_or_else(Vec::new, |table| table.rows.clone())
}

fn cell<'a>(header: &[String], row: &'a [String], name: &str) -> Option<&'a str> {
    header
        .iter()
        .position(|h| h == name)
        .map(|i| row[i].as_str())
        .filter(|v| !v.is_empty())
}

/// Spalten: `sync_id`, `text`, `updated`, `deleted` (`yes` oder leer).
fn todos(step: &Step) -> Vec<SyncTodo> {
    let (header, rows) = table(step);
    rows.iter()
        .map(|row| SyncTodo {
            sync_id: cell(&header, row, "sync_id").unwrap_or_default().to_owned(),
            device_id: "device".to_owned(),
            raw_text: cell(&header, row, "text").unwrap_or_default().to_owned(),
            priority: None,
            completed: false,
            completed_at: None,
            created_at: None,
            description: cell(&header, row, "text").unwrap_or_default().to_owned(),
            project_tags: vec![],
            context_tags: vec![],
            pomodoro_count: 0,
            position: 1,
            updated_at: cell(&header, row, "updated").unwrap_or_default().to_owned(),
            deleted: cell(&header, row, "deleted") == Some("yes"),
        })
        .collect()
}

/// Spalten: `device`, `started`, optional `ended`, `planned`, `actual`, `break`.
fn sessions(step: &Step) -> Vec<SyncSession> {
    let (header, rows) = table(step);
    let number = |row: &[String], name: &str, default: u32| {
        cell(&header, row, name).map_or(default, |v| v.parse().expect("Zahl"))
    };
    rows.iter()
        .map(|row| SyncSession {
            device_id: cell(&header, row, "device").unwrap_or_default().to_owned(),
            mode: "pomodoro".to_owned(),
            session_type: "work".to_owned(),
            status: "completed".to_owned(),
            started_at: cell(&header, row, "started").expect("started").to_owned(),
            ended_at: cell(&header, row, "ended").map(str::to_owned),
            planned_min: number(row, "planned", 25),
            actual_min: number(row, "actual", 25),
            break_min: number(row, "break", 0),
        })
        .collect()
}

fn ids(todos: &[SyncTodo]) -> Vec<String> {
    todos.iter().map(|t| t.sync_id.clone()).collect()
}

fn words(list: &str) -> Vec<String> {
    list.split_whitespace().map(str::to_owned).collect()
}

fn read_fixture(name: &str) -> (String, SyncFile) {
    let text = std::fs::read_to_string(format!("{FIXTURES}{name}"))
        .unwrap_or_else(|e| panic!("{name}: {e}"));
    let file = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name} nicht lesbar: {e}"));
    (text, file)
}

// ---- Given -----------------------------------------------------------------

#[given("the local todos:")]
fn local_todos(world: &mut DomainWorld, step: &Step) {
    world.sync.local_todos = todos(step);
}

#[given("the remote todos:")]
fn remote_todos(world: &mut DomainWorld, step: &Step) {
    world.sync.remote_todos = todos(step);
}

#[given("the local sessions:")]
fn local_sessions(world: &mut DomainWorld, step: &Step) {
    world.sync.local_sessions = sessions(step);
}

#[given("the remote sessions:")]
fn remote_sessions(world: &mut DomainWorld, step: &Step) {
    world.sync.remote_sessions = sessions(step);
}

#[given(expr = "the sync file {string} is the local state")]
fn file_local(world: &mut DomainWorld, name: String) {
    world.sync.local_file = Some(read_fixture(&name).1);
}

#[given(expr = "the sync file {string} is the remote state")]
fn file_remote(world: &mut DomainWorld, name: String) {
    world.sync.remote_file = Some(read_fixture(&name).1);
}

// ---- When ------------------------------------------------------------------

#[when("I merge the remote state into the local state")]
fn merge(world: &mut DomainWorld) {
    let state = &mut world.sync;
    state.todo_merge = Some(merge_todos(&state.local_todos, &state.remote_todos));
    state.session_merge = Some(merge_sessions(
        &state.local_sessions,
        &state.remote_sessions,
    ));
}

#[when("I merge the remote file into the local file")]
fn merge_files(world: &mut DomainWorld) {
    let state = &mut world.sync;
    let (local, remote) = (
        state.local_file.as_ref().expect("lokal"),
        state.remote_file.as_ref().expect("remote"),
    );
    state.todo_merge = Some(merge_todos(&local.todos, &remote.todos));
    state.session_merge = Some(merge_sessions(&local.sessions, &remote.sessions));
    state.merged_file = Some(merge_sync_data(local, remote));
}

#[when(expr = "I read the sync file {string}")]
fn read_file(world: &mut DomainWorld, name: String) {
    let (text, file) = read_fixture(&name);
    world.sync.read_text = text;
    world.sync.read_file = Some(file);
}

#[when("I write the sync file again")]
fn write_file(world: &mut DomainWorld) {
    let file = world.sync.read_file.as_ref().expect("Datei nicht gelesen");
    world.sync.written_json = serde_json::to_string_pretty(file).expect("serialisieren");
}

// ---- Then (Merge) ----------------------------------------------------------

fn todo_merge(world: &DomainWorld) -> &TodoMerge {
    world
        .sync
        .todo_merge
        .as_ref()
        .expect("nicht zusammengeführt")
}

fn session_merge(world: &DomainWorld) -> &SessionMerge {
    world
        .sync
        .session_merge
        .as_ref()
        .expect("nicht zusammengeführt")
}

#[then("the merged todos are:")]
fn merged_todos(world: &mut DomainWorld, step: &Step) {
    let expected = expected_rows(step);
    let actual: Vec<Vec<String>> = todo_merge(world)
        .merged
        .iter()
        .map(|t| {
            vec![
                t.sync_id.clone(),
                t.raw_text.clone(),
                if t.deleted { "yes" } else { "" }.to_owned(),
            ]
        })
        .collect();
    assert_eq!(actual, expected);
}

#[then(expr = "the todos added by the remote are {string}")]
fn added(world: &mut DomainWorld, expected: String) {
    assert_eq!(ids(&todo_merge(world).added_by_remote), words(&expected));
}

#[then(expr = "the todos taken over from the remote are {string}")]
fn conflicted(world: &mut DomainWorld, expected: String) {
    assert_eq!(
        ids(&todo_merge(world).conflicted_by_remote),
        words(&expected)
    );
}

#[then("the merged sessions are:")]
fn merged_sessions(world: &mut DomainWorld, step: &Step) {
    let expected = expected_rows(step);
    let actual: Vec<Vec<String>> = session_merge(world)
        .all
        .iter()
        .map(|s| {
            vec![
                s.device_id.clone(),
                s.started_at.clone(),
                s.actual_min.to_string(),
            ]
        })
        .collect();
    assert_eq!(actual, expected);
}

#[then(expr = "{int} sessions are new from the remote")]
fn new_sessions(world: &mut DomainWorld, expected: usize) {
    assert_eq!(session_merge(world).new_from_remote.len(), expected);
}

// ---- Then (Dateiformat) ----------------------------------------------------

#[then(expr = "the file has schema version {int} and device {string}")]
fn header(world: &mut DomainWorld, version: u32, device: String) {
    let file = world.sync.read_file.as_ref().expect("Datei nicht gelesen");
    assert_eq!(
        (file.schema_version, file.device_id.as_str()),
        (version, device.as_str())
    );
    assert_eq!(
        file.synced_at,
        Utc.with_ymd_and_hms(2026, 9, 29, 7, 0, 0).unwrap()
    );
}

#[then(expr = "the file has {int} todos and {int} sessions")]
fn counts(world: &mut DomainWorld, todos: usize, sessions: usize) {
    let file = world.sync.read_file.as_ref().expect("Datei nicht gelesen");
    assert_eq!((file.todos.len(), file.sessions.len()), (todos, sessions));
}

#[then(expr = "the todo {string} is completed on {string}")]
fn todo_completed(world: &mut DomainWorld, sync_id: String, date: String) {
    let file = world.sync.read_file.as_ref().expect("Datei nicht gelesen");
    let todo = file
        .todos
        .iter()
        .find(|t| t.sync_id == sync_id)
        .expect("Todo nicht gefunden");
    assert!(todo.completed);
    assert_eq!(todo.completed_at.as_deref(), Some(date.as_str()));
}

#[then(expr = "the todo {string} is marked as deleted")]
fn todo_deleted(world: &mut DomainWorld, sync_id: String) {
    let file = world.sync.read_file.as_ref().expect("Datei nicht gelesen");
    assert!(
        file.todos
            .iter()
            .find(|t| t.sync_id == sync_id)
            .expect("Todo nicht gefunden")
            .deleted
    );
}

#[then("the written file equals the original file")]
fn same_json(world: &mut DomainWorld) {
    let original: serde_json::Value =
        serde_json::from_str(&world.sync.read_text).expect("Original");
    let written: serde_json::Value =
        serde_json::from_str(&world.sync.written_json).expect("Geschrieben");
    assert_eq!(written, original);
}

#[then(expr = "the written todo {string} has the keys {string}")]
fn written_keys(world: &mut DomainWorld, sync_id: String, keys: String) {
    let value: serde_json::Value =
        serde_json::from_str(&world.sync.written_json).expect("Geschrieben");
    let todo = value["todos"]
        .as_array()
        .and_then(|todos| todos.iter().find(|t| t["sync_id"] == sync_id))
        .expect("Todo nicht gefunden");
    let mut actual: Vec<&str> = todo
        .as_object()
        .expect("Objekt")
        .keys()
        .map(String::as_str)
        .collect();
    let mut expected: Vec<&str> = keys.split_whitespace().collect();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}

// ---- Then (Datei-Merge) ----------------------------------------------------

#[then(expr = "the merged file has {int} todos and {int} sessions")]
fn merged_counts(world: &mut DomainWorld, todos: usize, sessions: usize) {
    let file = world
        .sync
        .merged_file
        .as_ref()
        .expect("nicht zusammengeführt");
    assert_eq!((file.todos.len(), file.sessions.len()), (todos, sessions));
}

#[then(expr = "the merged file keeps the header of the local file")]
fn merged_header(world: &mut DomainWorld) {
    let (merged, local) = (
        world
            .sync
            .merged_file
            .as_ref()
            .expect("nicht zusammengeführt"),
        world.sync.local_file.as_ref().unwrap(),
    );
    assert_eq!(
        (merged.schema_version, &merged.synced_at, &merged.device_id),
        (local.schema_version, &local.synced_at, &local.device_id)
    );
}

#[then(expr = "the merged todo {string} says {string}")]
fn merged_todo_text(world: &mut DomainWorld, sync_id: String, text: String) {
    let file = world
        .sync
        .merged_file
        .as_ref()
        .expect("nicht zusammengeführt");
    assert_eq!(
        file.todos
            .iter()
            .find(|t| t.sync_id == sync_id)
            .expect("Todo")
            .raw_text,
        text
    );
}
