//! Steps für Backup und Restore (`features/backup/*.feature`, Restore-Szenario in `existing_database.feature`).
//! Ordnernamen in den Szenarien sind relativ zum Temp-Ordner des Szenarios.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use cucumber::{gherkin::Step, given, then, when};
use nunc_core::backup_policy::{BackupEntry, BackupStatus};
use nunc_store::{BackupResult, StoreError};
use rusqlite::{Connection, OpenFlags};

use crate::StoreWorld;
use crate::settings::{as_text, rows};
use crate::todos::date;

#[derive(Debug, Default)]
pub struct State {
    result: Option<BackupResult>,
    ran: bool,
    status: Option<BackupStatus>,
    entries: Vec<BackupEntry>,
    error: Option<String>,
}

fn instant(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .unwrap_or_else(|_| panic!("Zeit {text:?}"))
        .with_timezone(&Utc)
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|_| panic!("Ordner {dir:?} fehlt"))
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

fn split(list: &str) -> Vec<String> {
    list.split_whitespace().map(str::to_owned).collect()
}

fn error_text(result: Result<(), StoreError>) -> Option<String> {
    result.err().map(|e| e.to_string())
}

/// Legt eine Datei mit `size` Bytes an und setzt die Änderungszeit.
fn write_file(path: &Path, size: usize, modified: Option<DateTime<Utc>>) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = fs::File::create(path).unwrap();
    file.set_len(size as u64).unwrap();
    if let Some(time) = modified {
        file.set_modified(time.into()).unwrap();
    }
}

fn nunc_db_with_todos(path: &Path, todos: &str) {
    let conn = nunc_store::open(path).unwrap();
    for (i, text) in todos.split(',').map(str::trim).enumerate() {
        let now = instant("2026-10-04T10:00:00Z");
        nunc_store::add_todo(&conn, text, &format!("src-{i}"), now).unwrap();
    }
}

// ---- Given / When ----------------------------------------------------------

#[when(expr = "I run the daily backup on {word}")]
fn daily(world: &mut StoreWorld, day: String) {
    let dir = world.root().join("backups");
    run_daily(world, &dir, &day);
}

#[when(expr = "I run the daily backup on {word} into the configured or default folder {string}")]
fn daily_configured(world: &mut StoreWorld, day: String, default: String) {
    let default = world.root().join(default);
    let dir = nunc_store::resolve_backup_dir(world.conn(), &default).unwrap();
    run_daily(world, &dir, &day);
}

fn run_daily(world: &mut StoreWorld, dir: &Path, day: &str) {
    let result = nunc_store::backup_if_needed(world.conn(), dir, date(day)).expect("Backup");
    world.backup.ran = result.is_some();
    world.backup.result = result;
}

#[when(expr = "I run a manual backup on {word}")]
fn manual(world: &mut StoreWorld, day: String) {
    let dir = world.root().join("backups");
    let result = nunc_store::run_backup(world.conn(), &dir, date(&day)).expect("Backup");
    world.backup.result = Some(result);
}

#[when(expr = "I choose the backup folder {string}")]
fn choose(world: &mut StoreWorld, name: String) {
    let dir = world.root().join(name);
    nunc_store::set_backup_dir(world.conn(), &dir).expect("Ordner wählen");
}

#[when(expr = "I try to choose the backup folder {string}")]
fn try_choose(world: &mut StoreWorld, name: String) {
    let dir = world.root().join(name);
    world.backup.error = error_text(nunc_store::set_backup_dir(world.conn(), &dir));
}

#[given(expr = "a file named {string} exists")]
#[when(expr = "a file named {string} exists")]
fn plain_file(world: &mut StoreWorld, name: String) {
    let path = world.root().join(name);
    fs::write(path, b"plain text, no database").unwrap();
}

#[given(expr = "the folder {string} has the empty files {string}")]
#[when(expr = "the folder {string} has the empty files {string}")]
fn empty_files(world: &mut StoreWorld, dir: String, files: String) {
    let dir = world.root().join(dir);
    for name in split(&files) {
        write_file(&dir.join(name), 0, None);
    }
}

#[given(expr = "the folder {string} has the files:")]
#[when(expr = "the folder {string} has the files:")]
fn files_table(world: &mut StoreWorld, dir: String, step: &Step) {
    let dir = world.root().join(dir);
    let header = &step.table.as_ref().unwrap().rows[0];
    for row in rows(step) {
        let cell = |name: &str| {
            let index = header.iter().position(|h| h == name)?;
            Some(row[index].as_str()).filter(|c| !c.is_empty())
        };
        let size = cell("size").map_or(0, |s| s.parse().unwrap());
        write_file(&dir.join(&row[0]), size, cell("modified").map(instant));
    }
}

#[when(expr = "I read the status of the folder {string} at {word}")]
fn status(world: &mut StoreWorld, dir: String, now: String) {
    let dir = world.root().join(dir);
    world.backup.status = Some(nunc_store::backup_dir_status(&dir, instant(&now)).unwrap());
}

#[when(expr = "I list the backups of the folder {string} on {word}")]
fn list(world: &mut StoreWorld, dir: String, today: String) {
    let dir = world.root().join(dir);
    world.backup.entries = nunc_store::list_backup_entries(&dir, date(&today)).unwrap();
}

#[given(expr = "a backup file {string} with the todos {string} exists")]
#[when(expr = "a backup file {string} with the todos {string} exists")]
fn backup_with_todos(world: &mut StoreWorld, name: String, todos: String) {
    nunc_db_with_todos(&world.root().join(name), &todos);
}

#[given(expr = "a backup file {string} with schema version {int} exists")]
#[when(expr = "a backup file {string} with schema version {int} exists")]
fn backup_with_version(world: &mut StoreWorld, name: String, version: i64) {
    let path = world.root().join(name);
    let conn = nunc_store::open(&path).unwrap();
    conn.execute(
        "INSERT INTO schema_migrations (version) VALUES (?1)",
        [version],
    )
    .unwrap();
}

#[given(expr = "a broken nunc backup {string} exists")]
#[when(expr = "a broken nunc backup {string} exists")]
fn broken_backup(world: &mut StoreWorld, name: String) {
    // besteht die Prüfung (Basis-Migration vermerkt), lässt sich aber nicht migrieren: Tabellen fehlen
    let conn = Connection::open(world.root().join(name)).unwrap();
    conn.execute_batch(
        "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at DATETIME NOT NULL DEFAULT (datetime('now')));
         INSERT INTO schema_migrations (version) VALUES (1);",
    )
    .unwrap();
}

#[given(expr = "a plain SQLite file {string} exists")]
#[when(expr = "a plain SQLite file {string} exists")]
fn plain_sqlite(world: &mut StoreWorld, name: String) {
    let conn = Connection::open(world.root().join(name)).unwrap();
    conn.execute_batch("CREATE TABLE other (x)").unwrap();
}

#[when(expr = "the Go fixture is copied to {string}")]
fn copy_fixture(world: &mut StoreWorld, name: String) {
    let src = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/go-v5.db");
    let dest = world.root().join(name);
    fs::create_dir_all(dest.parent().unwrap()).unwrap();
    fs::copy(src, dest).unwrap();
}

fn restore(world: &mut StoreWorld, name: &str, day: &str) -> Result<(), StoreError> {
    let root = world.root();
    let db_path = world.path.clone().expect("kein Pfad");
    let conn = world.conn.as_mut().expect("Datenbank nicht geöffnet");
    nunc_store::restore_backup(
        conn,
        &db_path,
        &root.join(name),
        &root.join("backups"),
        date(day),
    )
}

#[when(expr = "I restore {string} on {word}")]
fn restore_ok(world: &mut StoreWorld, name: String, day: String) {
    restore(world, &name, &day).expect("Restore");
}

#[when(expr = "I try to restore {string} on {word}")]
fn restore_try(world: &mut StoreWorld, name: String, day: String) {
    world.backup.error = error_text(restore(world, &name, &day));
}

// ---- Then ------------------------------------------------------------------

#[then("a backup was made")]
fn made(world: &mut StoreWorld) {
    assert!(world.backup.ran);
}

#[then("no backup was made")]
fn not_made(world: &mut StoreWorld) {
    assert!(!world.backup.ran);
}

#[then(expr = "the folder {string} contains exactly {string}")]
fn contains_exactly(world: &mut StoreWorld, dir: String, files: String) {
    let mut expected = split(&files);
    expected.sort();
    assert_eq!(names(&world.root().join(dir)), expected);
}

#[then(expr = "the folder {string} does not exist")]
fn folder_missing(world: &mut StoreWorld, dir: String) {
    assert!(!world.root().join(dir).exists());
}

#[then(expr = "the folder {string} has no temporary files")]
fn no_tmp(world: &mut StoreWorld, dir: String) {
    let tmp: Vec<_> = names(&world.root().join(dir))
        .into_iter()
        .filter(|n| n.ends_with(".tmp"))
        .collect();
    assert!(tmp.is_empty(), "{tmp:?}");
}

#[then("the data directory has no leftover restore files")]
fn no_leftovers(world: &mut StoreWorld) {
    let left: Vec<_> = names(&world.root())
        .into_iter()
        .filter(|n| n.contains(".restore"))
        .collect();
    assert!(left.is_empty(), "{left:?}");
}

#[then(expr = "the backup file {string} is valid")]
fn valid(world: &mut StoreWorld, name: String) {
    nunc_store::validate_backup_file(&world.root().join(name)).expect("gültig");
}

#[then(expr = "the backup file {string} is rejected with {string}")]
fn rejected(world: &mut StoreWorld, name: String, reason: String) {
    let error = nunc_store::validate_backup_file(&world.root().join(name)).expect_err("abgelehnt");
    assert!(matches!(error, StoreError::InvalidBackup(_)), "{error:?}");
    assert!(error.to_string().contains(&reason), "{error}");
}

#[then(expr = "the database {string} has the todos {string}")]
fn database_has_todos(world: &mut StoreWorld, name: String, todos: String) {
    let path = world.root().join(name);
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let actual: Vec<String> = nunc_store::list_todos(&conn)
        .unwrap()
        .into_iter()
        .map(|t| t.raw_text)
        .collect();
    let expected: Vec<String> = todos.split(',').map(|t| t.trim().to_owned()).collect();
    assert_eq!(actual, expected);
}

#[then(expr = "the backup folder resolves to the default {string} when none is configured")]
fn resolves_default(world: &mut StoreWorld, default: String) {
    let default = world.root().join(default);
    let resolved = nunc_store::resolve_backup_dir(world.conn(), &default).unwrap();
    assert_eq!(resolved, default);
}

#[then(expr = "the backup folder setting is {string}")]
fn folder_setting(world: &mut StoreWorld, name: String) {
    let expected = world.root().join(name);
    let actual = nunc_store::load_settings(world.conn()).unwrap().backup_dir;
    assert_eq!(Path::new(&actual), expected);
}

#[then("choosing the folder failed")]
fn choosing_failed(world: &mut StoreWorld) {
    assert!(world.backup.error.is_some());
}

#[then("the restore failed")]
fn restore_failed(world: &mut StoreWorld) {
    assert!(world.backup.error.is_some());
}

#[then(expr = "the backup result is {string} with the real file size")]
fn result_is(world: &mut StoreWorld, name: String) {
    let expected = world.root().join("backups").join(name);
    let result = world.backup.result.as_ref().expect("kein Ergebnis");
    assert_eq!(result.path, expected);
    assert!(result.size_bytes > 0);
    assert_eq!(result.size_bytes, fs::metadata(&result.path).unwrap().len());
}

#[then("the backup status is:")]
fn status_is(world: &mut StoreWorld, step: &Step) {
    let json = serde_json::to_value(world.backup.status.as_ref().expect("kein Status")).unwrap();
    for row in rows(step) {
        assert_eq!(as_text(&json[&row[0]]), row[1], "{}", row[0]);
    }
}

#[then("the listed backups are:")]
fn listed(world: &mut StoreWorld, step: &Step) {
    let table = &step.table.as_ref().unwrap().rows;
    assert_eq!(table.len() - 1, world.backup.entries.len(), "Anzahl");
    for (row, entry) in table[1..].iter().zip(&world.backup.entries) {
        let json = serde_json::to_value(entry).unwrap();
        for (field, expected) in table[0].iter().zip(row) {
            assert_eq!(&as_text(&json[field]), expected, "{field}");
        }
    }
}
