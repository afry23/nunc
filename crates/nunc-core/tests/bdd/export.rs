//! Steps für `features/stats/export.feature`. Sessions, Zeitraum und Zeitzone kommen aus dem Modul `stats`,
//! die Todos aus `todo_query`.

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use cucumber::{gherkin::Step, given, then, when};
use nunc_core::export::{ExportDetails, export_rows, to_csv, to_json};
use nunc_core::model::{StatsPeriod, StatsRequest};
use std::collections::HashMap;

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    csv: String,
    json: String,
    /// Notiz und Herkunft je Beginn (`YYYY-MM-DD HH:MM`, UTC).
    details: Vec<(String, ExportDetails)>,
}

fn docstring(step: &Step) -> String {
    step.docstring
        .as_deref()
        .expect("Docstring fehlt")
        .trim_matches('\n')
        .to_owned()
}

#[given("the session details:")]
fn details(world: &mut DomainWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    world.export.details = table.rows[1..]
        .iter()
        .map(|row| {
            (
                row[0].clone(),
                ExportDetails {
                    note: row[1].clone(),
                    origin: row[2].clone(),
                },
            )
        })
        .collect();
}

#[when("I export the sessions")]
fn export(world: &mut DomainWorld) {
    let stats = &world.stats;
    let offset = FixedOffset::east_opt(stats.offset_hours * 3600).expect("Zeitzone");
    let request = StatsRequest {
        period: StatsPeriod::Month,
        start_date: stats.range.0,
        end_date: stats.range.1,
    };
    let to_local =
        |time: DateTime<Utc>| -> NaiveDateTime { time.with_timezone(&offset).naive_local() };
    // Die Sessions der Tabelle tragen Id 0; für die Details bekommt jede eine eigene.
    let mut sessions = stats.sessions.clone();
    let mut details = HashMap::new();
    for (index, session) in sessions.iter_mut().enumerate() {
        session.id = i32::try_from(index).expect("wenige Sessions") + 1;
        let start = session.started_at.format("%Y-%m-%d %H:%M").to_string();
        if let Some((_, detail)) = world.export.details.iter().find(|(key, _)| *key == start) {
            details.insert(session.id, detail.clone());
        }
    }
    let rows = export_rows(
        &sessions,
        &world.todo_query.todos,
        &request,
        &details,
        &to_local,
    );
    world.export.csv = to_csv(&rows);
    world.export.json = to_json(&rows).expect("JSON erzeugen");
}

#[then("the CSV export is:")]
fn csv_is(world: &mut DomainWorld, step: &Step) {
    assert_eq!(world.export.csv, format!("{}\n", docstring(step)));
}

#[then(expr = "the CSV export has {int} lines")]
fn csv_lines(world: &mut DomainWorld, expected: usize) {
    assert_eq!(world.export.csv.lines().count(), expected);
}

#[then("the JSON export is:")]
fn json_is(world: &mut DomainWorld, step: &Step) {
    let actual: serde_json::Value = serde_json::from_str(&world.export.json).expect("JSON lesen");
    let expected: serde_json::Value =
        serde_json::from_str(&docstring(step)).expect("Erwartung ist kein JSON");
    assert_eq!(actual, expected);
}

#[then("the JSON export is an empty list")]
fn json_empty(world: &mut DomainWorld) {
    assert_eq!(world.export.json, "[]");
}

#[then(expr = "the JSON export is indented with {int} spaces")]
fn json_indent(world: &mut DomainWorld, spaces: usize) {
    let second_line = world.export.json.lines().nth(1).expect("zweite Zeile");
    assert_eq!(second_line.len() - second_line.trim_start().len(), spaces);
}
