//! BDD-Runner für `@integration`-Szenarien von `nunc-store` (SQLite, Dateisystem in Temp-Ordnern).
//!
//! Aufruf: `cargo test -p nunc-store --test bdd`
//! Szenarien mit `@wip` werden übersprungen.

use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::{DateTime, NaiveTime, Utc};
use cucumber::{World, given, then, when};
use nunc_core::model::{Settings, StatsResponse, TimerMode, TimerState, Todo};
use nunc_core::timer::Timer;
use rusqlite::Connection;
use tempfile::TempDir;

mod backup;
mod compat;
mod crdt;
mod legacy;
mod settings;
mod todos;
mod todotxt_file;

/// Gemeinsamer Zustand der `@integration`-Szenarien.
#[derive(Debug, Default, World)]
pub struct StoreWorld {
    dir: Option<TempDir>,
    path: Option<PathBuf>,
    conn: Option<Connection>,
    /// `schema_migrations` direkt nach dem ersten Öffnen: version → applied_at
    first_migrations: Option<BTreeMap<i64, String>>,
    /// `sqlite_master`-Definitionen vor dem Öffnen: Name → SQL
    schema_before: Option<BTreeMap<String, String>>,
    timer: Timer,
    settings: Settings,
    todos: Vec<Todo>,
    stats: Option<StatsResponse>,
    sync_counter: u32,
    backup: backup::State,
    crdt: crdt::State,
    legacy: legacy::State,
    todotxt: todotxt_file::State,
}

/// Uhrzeit `HH:MM:SS` am festen Testtag (UTC).
fn at(time: &str) -> DateTime<Utc> {
    let t = NaiveTime::parse_from_str(time, "%H:%M:%S").unwrap_or_else(|_| panic!("Zeit {time:?}"));
    DateTime::parse_from_rfc3339("2026-10-04T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
        + (t - NaiveTime::MIN)
}

fn schema(conn: &Connection) -> BTreeMap<String, String> {
    let mut stmt = conn
        .prepare("SELECT name, sql FROM sqlite_master WHERE sql IS NOT NULL")
        .unwrap();
    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

impl StoreWorld {
    fn root(&mut self) -> PathBuf {
        self.dir().path().to_path_buf()
    }

    fn dir(&mut self) -> &TempDir {
        self.dir
            .get_or_insert_with(|| tempfile::tempdir().expect("Temp-Ordner"))
    }

    fn conn_mut(&mut self) -> &mut Connection {
        self.conn.as_mut().expect("Datenbank nicht geöffnet")
    }

    fn conn(&self) -> &Connection {
        self.conn.as_ref().expect("Datenbank nicht geöffnet")
    }

    fn migrations(&self) -> BTreeMap<i64, String> {
        let mut stmt = self
            .conn()
            .prepare("SELECT version, applied_at FROM schema_migrations")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    fn open(&mut self, path: PathBuf) {
        self.conn = None; // alte Verbindung zuerst schließen
        self.conn = Some(nunc_store::open(&path).expect("open"));
        self.path = Some(path);
        if self.first_migrations.is_none() {
            self.first_migrations = Some(self.migrations());
        }
    }
}

#[given("no database exists yet")]
fn no_database(world: &mut StoreWorld) {
    let path = world.dir().path().join("nunc.db");
    world.path = Some(path);
}

#[given("the Go fixture database")]
fn go_fixture(world: &mut StoreWorld) {
    let src = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/go-v5.db");
    let dest = world.dir().path().join("nunc.db");
    std::fs::copy(src, &dest).expect("Fixture kopieren");
    world.path = Some(dest);
    // Zustand vor dem Öffnen: Go schreibt WAL, die Kopie hat kein -wal/-shm
    let before = Connection::open(world.path.as_ref().unwrap()).unwrap();
    let mut stmt = before
        .prepare("SELECT version, applied_at FROM schema_migrations")
        .unwrap();
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    drop(stmt);
    world.first_migrations = Some(rows);
    world.schema_before = Some(schema(&before));
}

#[when("nunc opens the database")]
fn open_db(world: &mut StoreWorld) {
    let path = world.path.clone().expect("kein Pfad");
    world.open(path);
}

#[when("nunc opens the database in a not yet existing folder")]
fn open_in_new_folder(world: &mut StoreWorld) {
    let path = world.dir().path().join("a").join("b").join("nunc.db");
    assert!(!path.parent().unwrap().exists());
    world.open(path);
}

#[then(expr = "the migrations {int} to {int} are recorded as applied")]
fn migrations_recorded(world: &mut StoreWorld, from: i64, to: i64) {
    let versions: Vec<i64> = world.migrations().into_keys().collect();
    let expected: Vec<i64> = (from..=to).collect();
    assert!(
        expected.iter().all(|v| versions.contains(v)),
        "erwartet {expected:?}, vorhanden {versions:?}"
    );
}

#[then("the migration timestamps did not change")]
fn timestamps_unchanged(world: &mut StoreWorld) {
    let first = world.first_migrations.clone().expect("kein Ausgangsstand");
    let now = world.migrations();
    for (version, applied_at) in first {
        assert_eq!(now.get(&version), Some(&applied_at), "Version {version}");
    }
}

#[then(expr = "exactly the migrations {int} to {int} are recorded as applied")]
fn migrations_exactly(world: &mut StoreWorld, from: i64, to: i64) {
    let versions: Vec<i64> = world.migrations().into_keys().collect();
    assert_eq!(versions, (from..=to).collect::<Vec<_>>());
}

#[then("the Go tables keep their definition")]
fn go_tables_unchanged(world: &mut StoreWorld) {
    let before = world.schema_before.clone().expect("kein Ausgangsstand");
    let now = schema(world.conn());
    for (name, sql) in before {
        assert_eq!(now.get(&name), Some(&sql), "{name}");
    }
}

#[then(expr = "the table {string} exists")]
fn table_exists(world: &mut StoreWorld, table: String) {
    tables_exist(world, table);
}

#[then(expr = "the tables {string} exist")]
fn tables_exist(world: &mut StoreWorld, tables: String) {
    for table in tables.split(',').map(str::trim) {
        let n: i64 = world
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "Tabelle {table}");
    }
}

#[then(expr = "the table {string} contains {int} rows")]
fn row_count(world: &mut StoreWorld, table: String, expected: i64) {
    assert!(table.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
    let n: i64 = world
        .conn()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, expected);
}

#[then(expr = "inserting a session for the todo {int} is rejected")]
fn fk_rejected(world: &mut StoreWorld, todo_id: i64) {
    let result = world.conn().execute(
        "INSERT INTO sessions (mode, session_type, status, started_at, todo_id)
         VALUES ('pomodoro', 'work', 'completed', '2026-01-01T00:00:00Z', ?1)",
        [todo_id],
    );
    assert!(result.is_err(), "Foreign Key nicht erzwungen");
}

#[then(expr = "the journal mode is {string}")]
fn journal_mode(world: &mut StoreWorld, expected: String) {
    let mode: String = world
        .conn()
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(mode, expected);
}

#[when(expr = "a pomodoro for todo {int} is started at {word} and saved")]
fn start_and_save(world: &mut StoreWorld, todo_id: i32, time: String) {
    let now = at(&time);
    world
        .timer
        .start(
            now,
            TimerMode::Pomodoro,
            Some(todo_id),
            &Settings::default(),
        )
        .expect("start");
    nunc_store::save_timer(world.conn(), &world.timer, now).expect("save");
}

#[when(expr = "the timer is paused at {word} and saved")]
fn pause_and_save(world: &mut StoreWorld, time: String) {
    let now = at(&time);
    world.timer.pause(now).expect("pause");
    nunc_store::save_timer(world.conn(), &world.timer, now).expect("save");
}

#[when(expr = "the timer is stopped at {word} and saved")]
fn stop_and_save(world: &mut StoreWorld, time: String) {
    let now = at(&time);
    world.timer.stop(now, &Settings::default()).expect("stop");
    nunc_store::save_timer(world.conn(), &world.timer, now).expect("save");
}

#[when(expr = "the timer is expired at {word} and saved")]
fn expire_and_save(world: &mut StoreWorld, time: String) {
    let now = at(&time);
    let finish = world.timer.tick(now, &Settings::default());
    assert!(finish.is_some(), "Timer nicht abgelaufen");
    nunc_store::save_timer(world.conn(), &world.timer, now).expect("save");
}

#[when("the app restarts")]
fn restart(world: &mut StoreWorld) {
    let path = world.path.clone().expect("kein Pfad");
    world.open(path);
    world.timer = nunc_store::load_timer(world.conn()).expect("Timer laden");
}

/// Ein beendeter Prozess schließt seine Verbindung nicht: kein Checkpoint, die WAL-Datei bleibt liegen.
#[when("the app is killed and restarted")]
fn kill_and_restart(world: &mut StoreWorld) {
    if let Some(conn) = world.conn.take() {
        std::mem::forget(conn);
    }
    restart(world);
}

#[then(expr = "the restored timer is running for todo {int}")]
fn restored_running(world: &mut StoreWorld, todo_id: i32) {
    assert_eq!(world.timer.state(), TimerState::Running);
    assert_eq!(world.timer.active().and_then(|a| a.todo_id), Some(todo_id));
}

#[then("the restored timer is paused")]
fn restored_paused(world: &mut StoreWorld) {
    assert_eq!(world.timer.state(), TimerState::Paused);
}

#[then("the restored timer is idle")]
fn restored_idle(world: &mut StoreWorld) {
    assert_eq!(world.timer.state(), TimerState::Idle);
}

#[then(expr = "{int} seconds remain at {word}")]
fn remaining(world: &mut StoreWorld, seconds: u32, time: String) {
    assert_eq!(world.timer.snapshot(at(&time)).remaining, seconds);
}

const FEATURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../features");

/// Nur `@integration`-Szenarien dieses Crates, `@wip` wird übersprungen.
/// Todoist-/Sync-Szenarien gehören anderen Crates (wiremock) und werden hier nicht erwartet.
fn selected(
    feature: &cucumber::gherkin::Feature,
    _rule: Option<&cucumber::gherkin::Rule>,
    scenario: &cucumber::gherkin::Scenario,
) -> bool {
    let has = |tag: &str| feature.tags.iter().chain(&scenario.tags).any(|t| t == tag);
    // `@sync`-Szenarien (WebDAV, wiremock) laufen im Runner von nunc-sync
    has("integration") && !has("sync") && !has("api") && !has("wip")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    StoreWorld::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit(FEATURES, selected)
        .await;
}
