//! Steps für `features/backup/*.feature` (Regeln ohne Dateizugriff).

use chrono::{DateTime, NaiveDate, Utc};
use cucumber::{gherkin::Step, given, then, when};
use nunc_core::backup_policy::{
    BackupCategory, BackupEntry, BackupFile, BackupStatus, MAX_BACKUPS, backup_file_name,
    backup_status, emergency_backup_file_name, files_to_delete, list_backups, needs_backup,
    parse_backup_date,
};

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    files: Vec<BackupFile>,
    last_backup_date: String,
    delete: Vec<String>,
    needed: Option<bool>,
    status: Option<BackupStatus>,
    entries: Vec<BackupEntry>,
    name: String,
    parsed_date: Option<NaiveDate>,
}

fn date(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap_or_else(|_| panic!("Datum {text:?}"))
}

fn instant(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .unwrap_or_else(|_| panic!("Zeit {text:?}"))
        .with_timezone(&Utc)
}

/// Spalten: `name`, `size` (Bytes, Standard 1), `modified` (RFC 3339, Standard 2026-01-01T00:00:00Z).
#[given("the backup files:")]
fn files(world: &mut DomainWorld, step: &Step) {
    let rows = &step.table.as_ref().expect("Tabelle fehlt").rows;
    let header = &rows[0];
    fn col<'a>(header: &[String], row: &'a [String], name: &str) -> Option<&'a str> {
        header
            .iter()
            .position(|h| h == name)
            .map(|i| row[i].as_str())
            .filter(|v| !v.is_empty())
    }
    world.backup.files = rows[1..]
        .iter()
        .map(|row| BackupFile {
            name: col(header, row, "name").expect("name").to_owned(),
            path: format!("/backups/{}", col(header, row, "name").unwrap()),
            size_bytes: col(header, row, "size").map_or(1, |v| v.parse().expect("Größe")),
            modified_at: instant(col(header, row, "modified").unwrap_or("2026-01-01T00:00:00Z")),
        })
        .collect();
}

#[given(expr = "the last backup date is {string}")]
fn last_date(world: &mut DomainWorld, last: String) {
    world.backup.last_backup_date = last;
}

#[when("I rotate the backups")]
fn rotate(world: &mut DomainWorld) {
    world.backup.delete = files_to_delete(&world.backup.files, MAX_BACKUPS);
}

#[when(expr = "I rotate the backups keeping at most {int}")]
fn rotate_with(world: &mut DomainWorld, max: usize) {
    world.backup.delete = files_to_delete(&world.backup.files, max);
}

#[then(expr = "the files to delete are {string}")]
fn to_delete(world: &mut DomainWorld, expected: String) {
    let expected: Vec<String> = expected.split_whitespace().map(str::to_owned).collect();
    assert_eq!(world.backup.delete, expected);
}

#[when("I check whether a backup is needed")]
fn check_needed(world: &mut DomainWorld) {
    let state = &world.backup;
    world.backup.needed = Some(needs_backup(&state.last_backup_date, world.stats.today));
}

#[then("a backup is needed")]
fn needed(world: &mut DomainWorld) {
    assert_eq!(world.backup.needed, Some(true));
}

#[then("no backup is needed")]
fn not_needed(world: &mut DomainWorld) {
    assert_eq!(world.backup.needed, Some(false));
}

#[when(expr = "I read the backup status at {word}")]
fn read_status(world: &mut DomainWorld, now: String) {
    world.backup.status = Some(backup_status(&world.backup.files, instant(&now)));
}

fn status(world: &DomainWorld) -> &BackupStatus {
    world.backup.status.as_ref().expect("Status nicht gelesen")
}

#[then(expr = "the status shows {int} backups with {int} bytes")]
fn status_counts(world: &mut DomainWorld, count: u32, bytes: u64) {
    let status = status(world);
    assert_eq!(
        (status.backup_count, status.total_size_bytes),
        (count, bytes)
    );
}

#[then(expr = "the last backup was at {word}")]
fn last_backup(world: &mut DomainWorld, expected: String) {
    let expected = (expected != "never").then(|| instant(&expected));
    assert_eq!(status(world).last_backup_at, expected);
}

#[then("the status is outdated")]
fn outdated(world: &mut DomainWorld) {
    assert!(status(world).is_outdated);
}

#[then("the status is up to date")]
fn up_to_date(world: &mut DomainWorld) {
    assert!(!status(world).is_outdated);
}

#[when("I list the backups")]
fn list(world: &mut DomainWorld) {
    world.backup.entries = list_backups(&world.backup.files, world.stats.today);
}

/// Zeilen `filename | created | category | size`.
#[then("the backup list is:")]
fn list_is(world: &mut DomainWorld, step: &Step) {
    let expected = step
        .table
        .as_ref()
        .map_or_else(Vec::new, |t| t.rows.clone());
    let actual: Vec<Vec<String>> = world
        .backup
        .entries
        .iter()
        .map(|e| {
            let category = match e.category {
                BackupCategory::Daily => "daily",
                BackupCategory::Weekly => "weekly",
                BackupCategory::Monthly => "monthly",
            };
            vec![
                e.filename.clone(),
                e.created_at.to_rfc3339(),
                category.to_owned(),
                e.size_bytes.to_string(),
            ]
        })
        .collect();
    assert_eq!(actual, expected);
}

#[then(expr = "the first listed backup has the path {string}")]
fn first_path(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.backup.entries[0].path, expected);
}

#[given(expr = "the file name {string}")]
fn file_name(world: &mut DomainWorld, name: String) {
    world.backup.name = name;
}

#[when("I read the backup date from the file name")]
fn read_date(world: &mut DomainWorld) {
    world.backup.parsed_date = parse_backup_date(&world.backup.name);
}

#[then(expr = "the backup date is {word}")]
fn backup_date(world: &mut DomainWorld, expected: String) {
    let expected = (expected != "none").then(|| date(&expected));
    assert_eq!(world.backup.parsed_date, expected);
}

#[then(expr = "the daily backup of {word} is named {string}")]
fn daily_name(_world: &mut DomainWorld, day: String, expected: String) {
    assert_eq!(backup_file_name(date(&day)), expected);
}

#[then(expr = "the emergency backup of {word} is named {string}")]
fn emergency_name(_world: &mut DomainWorld, day: String, expected: String) {
    assert_eq!(emergency_backup_file_name(date(&day)), expected);
}
