//! Steps für die Prüfung der Einstellungen (`features/timer/flowtime.feature`, `features/settings.feature`).

use cucumber::{given, then, when};
use nunc_core::model::{Settings, SettingsIssue};

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    settings: Settings,
    issues: Option<Vec<SettingsIssue>>,
}

/// Setzt ein Feld über seinen JSON-Namen (`flowtimeRatio`); der Wert ist ein JSON-Literal (`0.09`).
#[given(expr = "the setting {string} is {string}")]
fn set_setting(world: &mut DomainWorld, field: String, json: String) {
    let state = &mut world.settings;
    let mut value = serde_json::to_value(&state.settings).expect("Einstellungen serialisieren");
    let literal: serde_json::Value = serde_json::from_str(&json).expect("Wert ist kein JSON");
    let object = value.as_object_mut().expect("Objekt");
    assert!(object.contains_key(&field), "unbekanntes Feld {field}");
    object.insert(field, literal);
    state.settings = serde_json::from_value(value).expect("Einstellungen lesen");
}

#[when("I validate the settings")]
fn validate(world: &mut DomainWorld) {
    let state = &mut world.settings;
    state.issues = state.settings.validate().err();
}

#[then("the settings are valid")]
fn valid(world: &mut DomainWorld) {
    assert_eq!(world.settings.issues, None);
}

#[then(expr = "the settings are invalid because of {string}")]
fn invalid(world: &mut DomainWorld, fields: String) {
    let actual: Vec<String> = world
        .settings
        .issues
        .as_ref()
        .expect("die Einstellungen hätten ungültig sein müssen")
        .iter()
        .map(|issue| issue.field.clone())
        .collect();
    let expected: Vec<String> = fields.split_whitespace().map(str::to_owned).collect();
    assert_eq!(actual, expected);
}

#[then(expr = "the sync provider {string} is {word} on {word}")]
fn provider_availability(
    _world: &mut DomainWorld,
    provider: String,
    result: String,
    platform: String,
) {
    let provider: nunc_core::model::SyncProvider = provider.parse().expect("unbekannter Anbieter");
    let mobile = match platform.as_str() {
        "mobile" => true,
        "desktop" => false,
        other => panic!("unbekannte Plattform {other}"),
    };
    let expected = match result.as_str() {
        "available" => true,
        "unavailable" => false,
        other => panic!("unbekanntes Ergebnis {other}"),
    };
    assert_eq!(provider.is_available(mobile), expected);
}
