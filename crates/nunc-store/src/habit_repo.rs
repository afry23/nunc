//! Gewohnheiten und Haken in der Datenbank (ADR 0028): [`Habit`] und [`HabitLog`] als CRDT.
//!
//! Die Tabellen halten die rohen Registerwerte, `habit_field_clock` (bzw. `hlc_*` beim Haken) die Stempel dazu.
//! Lokale Schreibvorgänge lesen den Wert, ändern ihn über die Methoden des CRDT (neuer Stempel dieses Geräts) und
//! schreiben ihn zurück. Ob eine Gewohnheit gilt, entscheidet [`Habit::is_deleted`], nicht die Spalte `deleted`.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use nunc_core::crdt::Register;
use nunc_core::habit::{Goal, Habit, HabitLog, Schedule, log_id, validate_name};
use nunc_core::hlc::Hlc;
use rusqlite::{Connection, Row, params};

use crate::StoreError;
use crate::crdt_repo::{atomic, next_hlc, observe};
use crate::time::{day_text, parse_day};

/// Stempel für Felder ohne Zeile in `habit_field_clock`.
const LEGACY_DEVICE: &str = "legacy";

const SELECT: &str = "SELECT sync_id, created, name, schedule, goal, archived, deleted FROM habits";
const SELECT_LOG: &str =
    "SELECT sync_id, habit, day, done, hlc_ms, hlc_counter, hlc_device FROM habit_logs";

fn clocks_of(conn: &Connection, sync_id: &str) -> Result<BTreeMap<String, Hlc>, StoreError> {
    let clocks = conn
        .prepare(
            "SELECT field, hlc_ms, hlc_counter, hlc_device FROM habit_field_clock WHERE sync_id = ?1",
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

/// Zeile samt Stempeln als Gewohnheit. Unlesbare Zeilen (kaputtes Datum) fehlen.
fn habit_of(conn: &Connection, row: &Row<'_>) -> Result<Option<Habit>, StoreError> {
    let sync_id: String = row.get(0)?;
    let created: String = row.get(1)?;
    let Some(created) = parse_day(&created) else {
        return Ok(None);
    };
    let clocks = clocks_of(conn, &sync_id)?;
    let clock = |field: &str| {
        clocks
            .get(field)
            .cloned()
            .unwrap_or_else(|| Hlc::new(0, 0, LEGACY_DEVICE))
    };
    Ok(Some(Habit {
        created,
        name: Register::new(row.get(2)?, clock("name")),
        schedule: Register::new(row.get(3)?, clock("schedule")),
        goal: Register::new(row.get(4)?, clock("goal")),
        archived: Register::new(row.get(5)?, clock("archived")),
        deleted: Register::new(row.get(6)?, clock("deleted")),
        sync_id,
    }))
}

fn log_of(row: &Row<'_>) -> Result<Option<HabitLog>, StoreError> {
    let day: String = row.get(2)?;
    let Some(day) = parse_day(&day) else {
        return Ok(None);
    };
    let clock = Hlc::new(
        row.get(4)?,
        row.get::<_, i64>(5)?.try_into().unwrap_or(0),
        row.get::<_, String>(6)?,
    );
    Ok(Some(HabitLog {
        habit: row.get(1)?,
        day,
        done: Register::new(row.get(3)?, clock),
    }))
}

fn read_habit(conn: &Connection, sync_id: &str) -> Result<Option<Habit>, StoreError> {
    let mut stmt = conn.prepare(&format!("{SELECT} WHERE sync_id = ?1"))?;
    let mut rows = stmt.query([sync_id])?;
    match rows.next()? {
        Some(row) => habit_of(conn, row),
        None => Ok(None),
    }
}

fn read_log(conn: &Connection, sync_id: &str) -> Result<Option<HabitLog>, StoreError> {
    let mut stmt = conn.prepare(&format!("{SELECT_LOG} WHERE sync_id = ?1"))?;
    let mut rows = stmt.query([sync_id])?;
    match rows.next()? {
        Some(row) => log_of(row),
        None => Ok(None),
    }
}

/// Alle Gewohnheiten samt gelöschten, für den Austausch.
pub(crate) fn habits(conn: &Connection) -> Result<BTreeMap<String, Habit>, StoreError> {
    let mut stmt = conn.prepare(&format!("{SELECT} ORDER BY sync_id"))?;
    let mut rows = stmt.query([])?;
    let mut all = BTreeMap::new();
    while let Some(row) = rows.next()? {
        if let Some(habit) = habit_of(conn, row)? {
            all.insert(habit.sync_id.clone(), habit);
        }
    }
    Ok(all)
}

/// Alle Haken samt entfernten, für den Austausch.
pub(crate) fn logs(conn: &Connection) -> Result<BTreeMap<String, HabitLog>, StoreError> {
    let mut stmt = conn.prepare(&format!("{SELECT_LOG} ORDER BY sync_id"))?;
    let mut rows = stmt.query([])?;
    let mut all = BTreeMap::new();
    while let Some(row) = rows.next()? {
        if let Some(log) = log_of(row)? {
            all.insert(log.sync_id(), log);
        }
    }
    Ok(all)
}

fn write_habit(conn: &Connection, habit: &Habit) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO habits (sync_id, created, name, schedule, goal, archived, deleted)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(sync_id) DO UPDATE
         SET created = ?2, name = ?3, schedule = ?4, goal = ?5, archived = ?6, deleted = ?7",
        params![
            habit.sync_id,
            day_text(habit.created),
            habit.name.value,
            habit.schedule.value,
            habit.goal.value,
            habit.archived.value,
            habit.deleted.value,
        ],
    )?;
    let clocks = [
        ("name", &habit.name.clock),
        ("schedule", &habit.schedule.clock),
        ("goal", &habit.goal.clock),
        ("archived", &habit.archived.clock),
        ("deleted", &habit.deleted.clock),
    ];
    for (field, clock) in clocks {
        conn.execute(
            "INSERT INTO habit_field_clock (sync_id, field, hlc_ms, hlc_counter, hlc_device)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(sync_id, field)
             DO UPDATE SET hlc_ms = ?3, hlc_counter = ?4, hlc_device = ?5",
            params![
                habit.sync_id,
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

fn write_log(conn: &Connection, log: &HabitLog) -> Result<(), StoreError> {
    let clock = &log.done.clock;
    conn.execute(
        "INSERT INTO habit_logs (sync_id, habit, day, done, hlc_ms, hlc_counter, hlc_device)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(sync_id) DO UPDATE
         SET done = ?4, hlc_ms = ?5, hlc_counter = ?6, hlc_device = ?7",
        params![
            log.sync_id(),
            log.habit,
            day_text(log.day),
            log.done.value,
            clock.wall_ms,
            clock.counter,
            clock.device_id,
        ],
    )?;
    observe(conn, clock)
}

/// Die geltenden (nicht gelöschten) Gewohnheiten, auch archivierte, nach Anlage und Kennung.
pub fn list_habits(conn: &Connection) -> Result<Vec<Habit>, StoreError> {
    let mut live: Vec<Habit> = habits(conn)?
        .into_values()
        .filter(|habit| !habit.is_deleted())
        .collect();
    live.sort_by(|a, b| (a.created, &a.sync_id).cmp(&(b.created, &b.sync_id)));
    Ok(live)
}

/// Die gesetzten Haken (nicht die entfernten) mit Tag von `from` bis `to` (einschließlich), nach Tag und Gewohnheit.
pub fn list_habit_logs(
    conn: &Connection,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<HabitLog>, StoreError> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT_LOG} WHERE day >= ?1 AND day <= ?2 AND done = 1 ORDER BY day, habit"
    ))?;
    let mut rows = stmt.query(params![day_text(from), day_text(to)])?;
    let mut done = Vec::new();
    while let Some(row) = rows.next()? {
        if let Some(log) = log_of(row)? {
            done.push(log);
        }
    }
    Ok(done)
}

/// Legt eine Gewohnheit an. `sync_id` vergibt der Aufrufer (UUID), `created` ist der lokale Tag der Anlage.
pub fn create_habit(
    conn: &Connection,
    sync_id: &str,
    created: NaiveDate,
    name: &str,
    schedule: Schedule,
    goal: Goal,
    at: DateTime<Utc>,
) -> Result<Habit, StoreError> {
    let name = validate_name(name)?;
    let (schedule, goal) = (schedule.checked()?, goal.checked()?);
    atomic(conn, || {
        let habit = Habit::new(
            sync_id,
            created,
            &name,
            &schedule,
            &goal,
            &next_hlc(conn, at)?,
        );
        write_habit(conn, &habit)?;
        Ok(habit)
    })
}

/// Die geltende Gewohnheit `sync_id`; gelöschte zählen nicht.
fn live_habit(conn: &Connection, sync_id: &str) -> Result<Habit, StoreError> {
    read_habit(conn, sync_id)?
        .filter(|habit| !habit.is_deleted())
        .ok_or_else(|| StoreError::HabitNotFound(sync_id.to_owned()))
}

/// Ändert Name, Rhythmus und Ziel; nur was sich ändert, bekommt einen neuen Stempel.
pub fn update_habit(
    conn: &Connection,
    sync_id: &str,
    name: &str,
    schedule: Schedule,
    goal: Goal,
    at: DateTime<Utc>,
) -> Result<Habit, StoreError> {
    let name = validate_name(name)?;
    let (schedule, goal) = (schedule.checked()?, goal.checked()?);
    atomic(conn, || {
        let mut habit = live_habit(conn, sync_id)?;
        let clock = next_hlc(conn, at)?;
        if habit.name.value != name {
            habit.set_name(&name, clock.clone());
        }
        if habit.schedule.value != schedule.encode() {
            habit.set_schedule(&schedule, clock.clone());
        }
        if habit.goal.value != goal.encode() {
            habit.set_goal(&goal, clock);
        }
        write_habit(conn, &habit)?;
        Ok(habit)
    })
}

/// Blendet eine Gewohnheit aus oder wieder ein; der Verlauf bleibt. Ändert sich nichts, bleibt der Stempel.
pub fn set_habit_archived(
    conn: &Connection,
    sync_id: &str,
    archived: bool,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    atomic(conn, || {
        let mut habit = live_habit(conn, sync_id)?;
        if habit.archived.value != archived {
            habit.set_archived(archived, next_hlc(conn, at)?);
            write_habit(conn, &habit)?;
        }
        Ok(())
    })
}

/// Löscht eine Gewohnheit (Tombstone, der mit dem Sync verschwindet; ihre Haken gehen mit).
pub fn delete_habit(conn: &Connection, sync_id: &str, at: DateTime<Utc>) -> Result<(), StoreError> {
    atomic(conn, || {
        let mut habit = live_habit(conn, sync_id)?;
        habit.delete(next_hlc(conn, at)?);
        write_habit(conn, &habit)
    })
}

/// Setzt oder entfernt den Haken für (Gewohnheit, Tag). Nur Gewohnheiten mit Ziel `check` haben Haken. Ändert sich
/// nichts (schon so gesetzt, oder nichts zu entfernen), bleibt der Stempel.
pub fn set_habit_done(
    conn: &Connection,
    habit: &str,
    day: NaiveDate,
    done: bool,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    atomic(conn, || {
        if live_habit(conn, habit)?.goal() != Some(Goal::Check) {
            return Err(StoreError::NotCheckable);
        }
        match read_log(conn, &log_id(habit, day))? {
            Some(log) if log.done.value == done => Ok(()),
            Some(mut log) => {
                log.set_done(done, next_hlc(conn, at)?);
                write_log(conn, &log)
            }
            None if !done => Ok(()),
            None => write_log(conn, &HabitLog::new(habit, day, true, &next_hlc(conn, at)?)),
        }
    })
}

/// Schreibt den zusammengeführten Stand: neue und geänderte Gewohnheiten und Haken (bereits gelöschte bzw.
/// entfernte nicht neu anlegen). Gelöschte Gewohnheiten, die der Stand nicht mehr enthält (Müllabfuhr), verschwinden
/// auch hier, ebenso entfernte Haken und Haken verschwundener Gewohnheiten.
pub(crate) fn apply(
    conn: &Connection,
    merged_habits: &BTreeMap<String, Habit>,
    merged_logs: &BTreeMap<String, HabitLog>,
) -> Result<(), StoreError> {
    for (sync_id, habit) in merged_habits {
        if habit.is_deleted() && read_habit(conn, sync_id)?.is_none() {
            continue;
        }
        write_habit(conn, habit)?;
    }
    for (sync_id, local) in habits(conn)? {
        if !merged_habits.contains_key(&sync_id) && local.is_deleted() {
            conn.execute("DELETE FROM habits WHERE sync_id = ?1", [sync_id])?;
        }
    }
    for (sync_id, log) in merged_logs {
        if !log.done.value && read_log(conn, sync_id)?.is_none() {
            continue;
        }
        write_log(conn, log)?;
    }
    for (sync_id, local) in logs(conn)? {
        if !merged_logs.contains_key(&sync_id)
            && (!local.done.value || !merged_habits.contains_key(&local.habit))
        {
            conn.execute("DELETE FROM habit_logs WHERE sync_id = ?1", [sync_id])?;
        }
    }
    Ok(())
}
