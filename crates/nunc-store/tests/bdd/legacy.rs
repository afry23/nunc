//! Steps für die Übernahme der Go-Datenbank (`features/existing_database.feature`, M2-07).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use cucumber::{given, then, when};
use nunc_store::LegacyImport;
use rusqlite::Connection;

use crate::StoreWorld;

#[derive(Debug, Default)]
pub struct State {
    result: Option<LegacyImport>,
    /// Ordnerinhalt vor dem Start: Dateiname → Bytes
    remembered: Option<BTreeMap<String, Vec<u8>>>,
}

fn snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            (
                e.file_name().into_string().unwrap(),
                fs::read(e.path()).unwrap(),
            )
        })
        .collect()
}

#[given(expr = "the Go app left its database in the user config folder {string}")]
fn go_database(world: &mut StoreWorld, config: String) {
    let src = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/go-v5.db");
    let dest = nunc_store::legacy_db_path(&world.root().join(config));
    fs::create_dir_all(dest.parent().unwrap()).unwrap();
    fs::copy(src, dest).unwrap();
}

#[given(expr = "the Go database has the backup folder {string}")]
fn go_backup_dir(world: &mut StoreWorld, dir: String) {
    let path = nunc_store::legacy_db_path(&world.root().join("config"));
    Connection::open(path)
        .unwrap()
        .execute(
            "UPDATE settings SET value = ?1 WHERE key = 'backup_dir'",
            [dir],
        )
        .unwrap();
}

#[given(expr = "the state of the folder {string} is remembered")]
#[when(expr = "the state of the folder {string} is remembered")]
fn remember(world: &mut StoreWorld, dir: String) {
    world.legacy.remembered = Some(snapshot(&world.root().join(dir)));
}

#[when(expr = "nunc starts with the data folder {string}")]
fn start(world: &mut StoreWorld, data: String) {
    let root = world.root();
    let target = root.join(data).join("nunc.db");
    let legacy = nunc_store::legacy_db_path(&root.join("config"));
    world.legacy.result = Some(nunc_store::import_legacy_db(&legacy, &target).expect("Übernahme"));
    world.open(target);
}

#[then(expr = "the legacy import result is {string}")]
fn result_is(world: &mut StoreWorld, expected: String) {
    let actual = match world.legacy.result.expect("kein Ergebnis") {
        LegacyImport::Imported => "imported",
        LegacyImport::AlreadyPresent => "already_present",
        LegacyImport::NoLegacyDatabase => "no_legacy_database",
    };
    assert_eq!(actual, expected);
}

#[then(expr = "the folder {string} is unchanged")]
fn unchanged(world: &mut StoreWorld, dir: String) {
    let before = world.legacy.remembered.clone().expect("nichts gemerkt");
    let now = snapshot(&world.root().join(dir));
    let names = |m: &BTreeMap<String, Vec<u8>>| m.keys().cloned().collect::<Vec<_>>();
    assert_eq!(names(&now), names(&before), "Dateien");
    assert!(now == before, "Inhalt einer Datei hat sich geändert");
}

#[then(expr = "the todo list has {int} entries")]
fn todo_count(world: &mut StoreWorld, expected: usize) {
    assert_eq!(world.todos.len(), expected);
}
