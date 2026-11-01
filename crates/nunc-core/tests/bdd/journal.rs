//! Steps für `features/sync/journal_crdt.feature` (Journal als CRDT, ADR 0026).

use chrono::NaiveDate;
use cucumber::{given, then, when};
use nunc_core::crdt::SyncState;
use nunc_core::hlc::Hlc;
use nunc_core::journal::{JournalEntry, JournalKind, MAX_TEXT_CHARS, validate_text, week_end};

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    a: SyncState,
    b: SyncState,
    merged: SyncState,
}

fn date(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("Datum")
}

impl State {
    fn side(&mut self, device: &str) -> &mut SyncState {
        match device {
            "A" => &mut self.a,
            "B" => &mut self.b,
            other => panic!("unbekanntes Gerät {other}"),
        }
    }

    /// Schreibt eine Notiz (neu oder als Änderung) mit dem Stempel `at` des Geräts.
    fn write(
        &mut self,
        device: &str,
        id: &str,
        mut make: impl FnMut(&Hlc) -> JournalEntry,
        text: &str,
        at: i64,
    ) {
        let clock = Hlc::new(at, 0, device);
        let journal = &mut self.side(device).journal;
        match journal.get_mut(id) {
            Some(entry) => entry.set_text(text, clock),
            None => {
                journal.insert(id.to_owned(), make(&clock));
            }
        }
    }

    fn entry(&self, id: &str) -> &JournalEntry {
        self.merged
            .journal
            .get(id)
            .unwrap_or_else(|| panic!("kein Eintrag {id}"))
    }
}

#[then(expr = "the week of {word} ends on {word}")]
fn week_ends(_world: &mut DomainWorld, day: String, sunday: String) {
    assert_eq!(week_end(date(&day)), date(&sunday));
}

#[when(expr = "device {word} writes the day note {string} for {word} at {int}")]
fn write_day_note(world: &mut DomainWorld, device: String, text: String, day: String, at: i64) {
    let day = date(&day);
    let id = format!("note:day:{day}");
    world.journal.write(
        &device,
        &id,
        |clock| JournalEntry::day_note(day, &text, clock),
        &text,
        at,
    );
}

#[when(expr = "device {word} writes the week note {string} for {word} at {int}")]
fn write_week_note(world: &mut DomainWorld, device: String, text: String, day: String, at: i64) {
    let day = date(&day);
    let id = format!("note:week:{}", week_end(day));
    world.journal.write(
        &device,
        &id,
        |clock| JournalEntry::week_note(day, &text, clock),
        &text,
        at,
    );
}

#[when(expr = "device {word} writes a day note of {int} characters for {word} at {int}")]
fn write_long_day_note(
    world: &mut DomainWorld,
    device: String,
    length: usize,
    day: String,
    at: i64,
) {
    let day = date(&day);
    // absichtlich ohne Prüfung: so sieht ein Gerät aus, das die Grenze nicht kennt
    let text = "x".repeat(length);
    let entry = JournalEntry::day_note(day, &text, &Hlc::new(at, 0, &device));
    world
        .journal
        .side(&device)
        .journal
        .insert(entry.sync_id.clone(), entry);
}

#[given(expr = "device {word} adds the achievement {string} with id {string} for {word} at {int}")]
#[when(expr = "device {word} adds the achievement {string} with id {string} for {word} at {int}")]
fn add_achievement(
    world: &mut DomainWorld,
    device: String,
    text: String,
    id: String,
    day: String,
    at: i64,
) {
    let entry = JournalEntry::achievement(&id, date(&day), &text, &Hlc::new(at, 0, &device));
    world.journal.side(&device).journal.insert(id, entry);
}

#[given(expr = "device B has a copy of the entry {string} of device A")]
fn copy_entry(world: &mut DomainWorld, id: String) {
    let entry = world.journal.a.journal[&id].clone();
    world.journal.b.journal.insert(id, entry);
}

#[when(expr = "device {word} deletes the entry {string} at {int}")]
fn delete_entry(world: &mut DomainWorld, device: String, id: String, at: i64) {
    let clock = Hlc::new(at, 0, &device);
    world
        .journal
        .side(&device)
        .journal
        .get_mut(&id)
        .expect("kein Eintrag")
        .delete(clock);
}

#[when(expr = "device {word} moves the achievement {string} to {word} at {int}")]
fn move_achievement(world: &mut DomainWorld, device: String, id: String, day: String, at: i64) {
    let clock = Hlc::new(at, 0, &device);
    let moved = world
        .journal
        .side(&device)
        .journal
        .get_mut(&id)
        .expect("kein Eintrag")
        .set_date(date(&day), clock);
    assert!(moved);
}

#[then(expr = "moving the entry {string} of device {word} to {word} is {word}")]
fn moving_is(world: &mut DomainWorld, id: String, device: String, day: String, result: String) {
    let clock = Hlc::new(9_000, 0, &device);
    let moved = world
        .journal
        .side(&device)
        .journal
        .get_mut(&id)
        .expect("kein Eintrag")
        .set_date(date(&day), clock);
    assert_eq!(moved, result == "accepted", "Verschieben {result}");
}

#[when(expr = "the devices are merged at {int}")]
fn merge(world: &mut DomainWorld, now: i64) {
    let state = &mut world.journal;
    state.merged = state.a.merge_remote(&state.b, now);
}

#[when(expr = "the garbage is collected at {int}")]
fn collect(world: &mut DomainWorld, now: i64) {
    world.journal.merged = world.journal.merged.collect_garbage(now);
}

#[then("the merge does not depend on the order")]
fn order_free(world: &mut DomainWorld) {
    let state = &world.journal;
    assert_eq!(state.a.merge(&state.b), state.b.merge(&state.a));
}

#[then(expr = "the merged state has {int} journal entries")]
fn entry_count(world: &mut DomainWorld, count: usize) {
    assert_eq!(world.journal.merged.journal.len(), count);
}

#[then(expr = "the merged entry {string} reads {string}")]
fn reads(world: &mut DomainWorld, id: String, text: String) {
    let entry = world.journal.entry(&id);
    assert!(!entry.is_deleted(), "{id} ist gelöscht");
    assert_eq!(entry.text.value, text);
}

#[then(expr = "the merged entry {string} is deleted")]
fn is_deleted(world: &mut DomainWorld, id: String) {
    assert!(world.journal.entry(&id).is_deleted());
}

#[then(expr = "the merged entry {string} does not exist")]
fn does_not_exist(world: &mut DomainWorld, id: String) {
    assert!(!world.journal.merged.journal.contains_key(&id));
}

#[then(expr = "the merged entry {string} is dated {word}")]
fn is_dated(world: &mut DomainWorld, id: String, day: String) {
    assert_eq!(world.journal.entry(&id).date.value, date(&day));
}

#[then(expr = "the merged entry {string} has {int} characters")]
fn has_characters(world: &mut DomainWorld, id: String, count: usize) {
    assert_eq!(world.journal.entry(&id).text.value.chars().count(), count);
}

#[then(expr = "a text of {int} characters is {word}")]
fn text_limit(_world: &mut DomainWorld, length: usize, result: String) {
    let outcome = validate_text(&"x".repeat(length));
    assert_eq!(outcome.is_ok(), result == "accepted");
    assert_eq!(MAX_TEXT_CHARS, 4000);
}

#[then(expr = "the text {string} is stored as {string}")]
fn text_trimmed(_world: &mut DomainWorld, text: String, stored: String) {
    assert_eq!(validate_text(&text), Ok(stored));
}

#[when(expr = "device {word} edits the entry {string} to {string} at {int}")]
fn edit_entry(world: &mut DomainWorld, device: String, id: String, text: String, at: i64) {
    let clock = Hlc::new(at, 0, &device);
    world
        .journal
        .side(&device)
        .journal
        .get_mut(&id)
        .expect("kein Eintrag")
        .set_text(&text, clock);
}

#[then(expr = "the kind {word} is stored as {string}")]
fn kind_text(_world: &mut DomainWorld, kind: String, stored: String) {
    let kind = match kind.as_str() {
        "DayNote" => JournalKind::DayNote,
        "WeekNote" => JournalKind::WeekNote,
        "Achievement" => JournalKind::Achievement,
        other => panic!("unbekannte Art {other}"),
    };
    assert_eq!(kind.as_str(), stored);
}
