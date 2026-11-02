//! Journal in der Datenbank (ADR 0026): Tages-/Wochennotizen und Erfolge als [`JournalEntry`].
//!
//! Die Tabelle hält die rohen Registerwerte, `journal_field_clock` die Stempel dazu. Lokale Schreibvorgänge lesen
//! den Eintrag, ändern ihn über die Methoden des CRDT (neuer Stempel dieses Geräts) und schreiben ihn zurück.
//! Ob ein Eintrag gilt, entscheidet [`JournalEntry::is_deleted`], nicht die Spalte `deleted`.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use nunc_core::crdt::Register;
use nunc_core::hlc::Hlc;
use nunc_core::journal::{
    JournalEntry, JournalError, JournalKind, day_note_id, validate_text, week_note_id,
};
use rusqlite::{Connection, Row, params};

use crate::StoreError;
use crate::crdt_repo::{atomic, next_hlc, observe};
use crate::time::{day_text, parse_day};

/// Stempel für Felder ohne Zeile in `journal_field_clock`.
const LEGACY_DEVICE: &str = "legacy";

const SELECT: &str = "SELECT sync_id, kind, date, text, deleted FROM journal_entries";

fn clocks_of(conn: &Connection, sync_id: &str) -> Result<BTreeMap<String, Hlc>, StoreError> {
    let clocks = conn
        .prepare(
            "SELECT field, hlc_ms, hlc_counter, hlc_device FROM journal_field_clock WHERE sync_id = ?1",
        )?
        .query_map([sync_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                Hlc::new(
                    row.get(1)?,
                    row.get::<_, i64>(2)?.try_into().unwrap_or(0),
                    row.get::<_, String>(3)?,
                ),
            ))
        })?
        .collect::<Result<_, _>>()?;
    Ok(clocks)
}

/// Zeile samt Stempeln als Eintrag. Unlesbare Zeilen (unbekannte Art, kaputtes Datum) fehlen.
fn entry_of(conn: &Connection, row: &Row<'_>) -> Result<Option<JournalEntry>, StoreError> {
    let sync_id: String = row.get(0)?;
    let kind: String = row.get(1)?;
    let date: String = row.get(2)?;
    let (Some(kind), Some(date)) = (JournalKind::parse(&kind), parse_day(&date)) else {
        return Ok(None);
    };
    let clocks = clocks_of(conn, &sync_id)?;
    let clock = |field: &str| {
        clocks
            .get(field)
            .cloned()
            .unwrap_or_else(|| Hlc::new(0, 0, LEGACY_DEVICE))
    };
    Ok(Some(JournalEntry {
        kind,
        date: Register::new(date, clock("date")),
        text: Register::new(row.get(3)?, clock("text")),
        deleted: Register::new(row.get(4)?, clock("deleted")),
        sync_id,
    }))
}

fn read_entry(conn: &Connection, sync_id: &str) -> Result<Option<JournalEntry>, StoreError> {
    let mut stmt = conn.prepare(&format!("{SELECT} WHERE sync_id = ?1"))?;
    let mut rows = stmt.query([sync_id])?;
    match rows.next()? {
        Some(row) => entry_of(conn, row),
        None => Ok(None),
    }
}

/// Alle Einträge samt gelöschten, für den Austausch.
pub(crate) fn entries(conn: &Connection) -> Result<BTreeMap<String, JournalEntry>, StoreError> {
    let mut stmt = conn.prepare(&format!("{SELECT} ORDER BY sync_id"))?;
    let mut rows = stmt.query([])?;
    let mut all = BTreeMap::new();
    while let Some(row) = rows.next()? {
        if let Some(entry) = entry_of(conn, row)? {
            all.insert(entry.sync_id.clone(), entry);
        }
    }
    Ok(all)
}

fn write_entry(conn: &Connection, entry: &JournalEntry) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO journal_entries (sync_id, kind, date, text, deleted) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(sync_id) DO UPDATE SET kind = ?2, date = ?3, text = ?4, deleted = ?5",
        params![
            entry.sync_id,
            entry.kind.as_str(),
            day_text(entry.date.value),
            entry.text.value,
            entry.deleted.value,
        ],
    )?;
    let clocks = [
        ("date", &entry.date.clock),
        ("text", &entry.text.clock),
        ("deleted", &entry.deleted.clock),
    ];
    for (field, clock) in clocks {
        conn.execute(
            "INSERT INTO journal_field_clock (sync_id, field, hlc_ms, hlc_counter, hlc_device)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(sync_id, field)
             DO UPDATE SET hlc_ms = ?3, hlc_counter = ?4, hlc_device = ?5",
            params![
                entry.sync_id,
                field,
                clock.wall_ms,
                clock.counter,
                clock.device_id
            ],
        )?;
        observe(conn, clock)?;
    }
    Ok(())
}

/// Die geltenden (nicht gelöschten) Einträge mit Datum von `from` bis `to` (einschließlich), nach Datum, Art und
/// Kennung. Eine Wochennotiz trägt den Sonntag ihrer Woche als Datum.
pub fn list_journal(
    conn: &Connection,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<JournalEntry>, StoreError> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE date >= ?1 AND date <= ?2 ORDER BY date, kind, sync_id"
    ))?;
    let mut rows = stmt.query(params![day_text(from), day_text(to)])?;
    let mut live = Vec::new();
    while let Some(row) = rows.next()? {
        if let Some(entry) = entry_of(conn, row)?
            && !entry.is_deleted()
        {
            live.push(entry);
        }
    }
    Ok(live)
}

/// Legt die Notiz `id` an oder ändert ihren Text. Ein leerer Text löscht sie; gibt es sie nicht, passiert dann
/// nichts. Bleibt der Text, wie er ist, bekommt kein Feld einen neuen Stempel.
fn save_note(
    conn: &Connection,
    id: &str,
    create: impl FnOnce(&str, &Hlc) -> JournalEntry,
    text: &str,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    let text = validate_text(text)?;
    atomic(conn, || match read_entry(conn, id)? {
        Some(entry) if entry.is_deleted() == text.is_empty() && is_same(&entry, &text) => Ok(()),
        Some(mut entry) => {
            entry.set_text(&text, next_hlc(conn, at)?);
            write_entry(conn, &entry)
        }
        None if text.is_empty() => Ok(()),
        None => write_entry(conn, &create(&text, &next_hlc(conn, at)?)),
    })
}

/// Hat der Eintrag schon diesen Text? Für einen leeren Text zählt nur, dass er gelöscht ist.
fn is_same(entry: &JournalEntry, text: &str) -> bool {
    text.is_empty() || entry.text.value == text
}

/// Speichert die Tagesnotiz für `date` (leerer Text löscht sie).
pub fn save_day_note(
    conn: &Connection,
    date: NaiveDate,
    text: &str,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    save_note(
        conn,
        &day_note_id(date),
        |text, clock| JournalEntry::day_note(date, text, clock),
        text,
        at,
    )
}

/// Speichert die Wochennotiz der Woche, die `day` enthält (leerer Text löscht sie).
pub fn save_week_note(
    conn: &Connection,
    day: NaiveDate,
    text: &str,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    save_note(
        conn,
        &week_note_id(day),
        |text, clock| JournalEntry::week_note(day, text, clock),
        text,
        at,
    )
}

/// Legt einen Erfolg an. `sync_id` vergibt der Aufrufer (UUID).
pub fn add_achievement(
    conn: &Connection,
    sync_id: &str,
    date: NaiveDate,
    text: &str,
    at: DateTime<Utc>,
) -> Result<JournalEntry, StoreError> {
    let text = validate_text(text)?;
    if text.is_empty() {
        return Err(JournalError::EmptyText.into());
    }
    atomic(conn, || {
        let entry = JournalEntry::achievement(sync_id, date, &text, &next_hlc(conn, at)?);
        write_entry(conn, &entry)?;
        Ok(entry)
    })
}

/// Der geltende Erfolg `sync_id`; Notizen und gelöschte Einträge zählen nicht.
fn live_achievement(conn: &Connection, sync_id: &str) -> Result<JournalEntry, StoreError> {
    read_entry(conn, sync_id)?
        .filter(|entry| entry.kind == JournalKind::Achievement && !entry.is_deleted())
        .ok_or_else(|| StoreError::JournalEntryNotFound(sync_id.to_owned()))
}

/// Ändert Datum und Text eines Erfolgs; nur was sich ändert, bekommt einen neuen Stempel.
pub fn update_achievement(
    conn: &Connection,
    sync_id: &str,
    date: NaiveDate,
    text: &str,
    at: DateTime<Utc>,
) -> Result<JournalEntry, StoreError> {
    let text = validate_text(text)?;
    if text.is_empty() {
        return Err(JournalError::EmptyText.into());
    }
    atomic(conn, || {
        let mut entry = live_achievement(conn, sync_id)?;
        let clock = next_hlc(conn, at)?;
        if entry.date.value != date {
            entry.set_date(date, clock.clone());
        }
        if entry.text.value != text {
            entry.set_text(&text, clock);
        }
        write_entry(conn, &entry)?;
        Ok(entry)
    })
}

/// Löscht einen Erfolg (Tombstone, der mit dem Sync verschwindet).
pub fn delete_achievement(
    conn: &Connection,
    sync_id: &str,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    atomic(conn, || {
        let mut entry = live_achievement(conn, sync_id)?;
        entry.delete(next_hlc(conn, at)?);
        write_entry(conn, &entry)
    })
}

/// Schreibt den zusammengeführten Stand: neue und geänderte Einträge (bereits gelöschte nicht neu anlegen).
/// Gelöschte Einträge, die der Stand nicht mehr enthält (Müllabfuhr), verschwinden auch hier.
pub(crate) fn apply_entries(
    conn: &Connection,
    merged: &BTreeMap<String, JournalEntry>,
) -> Result<(), StoreError> {
    for (sync_id, entry) in merged {
        match read_entry(conn, sync_id)? {
            None if entry.is_deleted() => {}
            Some(local) if local == *entry => {}
            _ => write_entry(conn, entry)?,
        }
    }
    for (sync_id, local) in entries(conn)? {
        if !merged.contains_key(&sync_id) && local.is_deleted() {
            conn.execute("DELETE FROM journal_entries WHERE sync_id = ?1", [sync_id])?;
        }
    }
    Ok(())
}
