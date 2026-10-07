//! Steps für `features/settings.feature` (Persistenz) und die Settings-Szenarien aus `existing_database.feature`.

use cucumber::{gherkin::Step, then, when};
use serde_json::Value;

use crate::StoreWorld;

pub(crate) fn rows(step: &Step) -> &[Vec<String>] {
    &step.table.as_ref().expect("Tabelle fehlt").rows[1..]
}

/// JSON-Literal (`30`, `true`), sonst Text (`webdav`).
fn literal(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()))
}

pub(crate) fn as_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[when("I load the settings")]
fn load(world: &mut StoreWorld) {
    world.settings = nunc_store::load_settings(world.conn()).expect("Einstellungen laden");
}

/// Setzt ein Feld über seinen JSON-Namen (`flowtimeRatio`).
#[when(expr = "I set {string} to {string}")]
fn set(world: &mut StoreWorld, field: String, value: String) {
    let mut json = serde_json::to_value(&world.settings).expect("serialisieren");
    let object = json.as_object_mut().expect("Objekt");
    assert!(object.contains_key(&field), "unbekanntes Feld {field}");
    object.insert(field, literal(&value));
    world.settings = serde_json::from_value(json).expect("Einstellungen lesen");
}

#[when("I save the settings")]
fn save(world: &mut StoreWorld) {
    let settings = world.settings.clone();
    let conn = world.conn.as_mut().expect("Datenbank nicht geöffnet");
    nunc_store::save_settings(conn, &settings).expect("Einstellungen speichern");
}

#[when(expr = "I select the sync provider {string}")]
fn select_provider(world: &mut StoreWorld, provider: String) {
    let provider = provider.parse().expect("unbekannter Sync-Anbieter");
    nunc_store::set_sync_provider(world.conn(), provider).expect("Anbieter wählen");
}

#[when(expr = "the settings table has {string} = {string}")]
fn raw_set(world: &mut StoreWorld, key: String, value: String) {
    world
        .conn()
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [key, value],
        )
        .unwrap();
}

#[then("the loaded settings are:")]
fn loaded_are(world: &mut StoreWorld, step: &Step) {
    let json = serde_json::to_value(&world.settings).expect("serialisieren");
    for row in rows(step) {
        let actual = json
            .get(&row[0])
            .unwrap_or_else(|| panic!("Feld {}", row[0]));
        assert_eq!(as_text(actual), row[1], "{}", row[0]);
    }
}

#[then("the settings table contains:")]
fn table_contains(world: &mut StoreWorld, step: &Step) {
    for row in rows(step) {
        let value: String = world
            .conn()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                [&row[0]],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| panic!("Schlüssel {} fehlt", row[0]));
        assert_eq!(value, row[1], "{}", row[0]);
    }
}
