//! CRDT-Seite der Todos (ADR 0016): Stempel je Feld, Pomodoro-Zähler je Gerät und Reihenfolge als
//! Fractional-Index. Die Spalten von `todos` bleiben der aufgelöste Stand; hier liegen nur die Stempel dazu.
//!
//! Lokale Änderungen laufen unverändert über SQL auf `todos`. Danach vergleicht [`stamp_changes`] den Stand vor
//! und nach der Änderung und stempelt **nur die Felder, die sich geändert haben**; das ist die Grundlage dafür,
//! dass gleichzeitige Änderungen verschiedener Felder beim Sync beide erhalten bleiben.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, TimeZone, Utc};
use nunc_core::crdt::{Register, SyncState, TodoCrdt, session_key};
use nunc_core::hlc::Hlc;
use nunc_core::model::SyncSession;
use nunc_core::position::position_between;
use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;
use crate::sync_repo::{Applied, sync_session};
use crate::time::{day_text, parse_day, to_text};

/// Felder mit eigenem Stempel. Zeile (`raw_text`) und Tags folgen aus ihnen und haben keinen.
const FIELDS: [&str; 7] = [
    "priority",
    "completed",
    "completed_at",
    "created_at",
    "description",
    "position",
    "deleted",
];

/// Pseudo-Gerät der Bestandsdaten (Migration 007) und Ersatz für Felder ohne Stempel.
const LEGACY_DEVICE: &str = "legacy";

/// Gerätekennung, solange noch keine vergeben ist (nur Tests; die App legt sie beim Start an).
const FALLBACK_DEVICE: &str = "local";

/// Die gestempelten Felder eines Todos, wie sie in der Datenbank stehen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Fields {
    priority: Option<String>,
    completed: bool,
    completed_at: Option<String>,
    created_at: Option<String>,
    description: String,
    position: String,
    deleted: bool,
}

impl Fields {
    /// Namen der Felder, die sich gegenüber `before` geändert haben (ohne `before`: alle).
    fn changed(&self, before: Option<&Fields>) -> Vec<&'static str> {
        let Some(before) = before else {
            return FIELDS.to_vec();
        };
        let differs = [
            self.priority != before.priority,
            self.completed != before.completed,
            self.completed_at != before.completed_at,
            self.created_at != before.created_at,
            self.description != before.description,
            self.position != before.position,
            self.deleted != before.deleted,
        ];
        FIELDS
            .into_iter()
            .zip(differs)
            .filter_map(|(field, differs)| differs.then_some(field))
            .collect()
    }
}

fn non_empty(text: Option<String>) -> Option<String> {
    text.filter(|text| !text.is_empty())
}

/// Datum als `YYYY-MM-DD`; längere Texte der Go-App (`2026-09-18T00:00:00Z`) werden gekürzt, Unlesbares ist `None`.
fn day(text: Option<String>) -> Option<String> {
    non_empty(text).as_deref().and_then(parse_day).map(day_text)
}

pub(crate) fn fields_of(conn: &Connection, id: i64) -> Result<Option<Fields>, StoreError> {
    Ok(conn
        .query_row(
            "SELECT t.priority, t.completed, t.completed_at, t.created_at, t.description,
                    COALESCE(k.key, ''), t.deleted
             FROM todos t LEFT JOIN todo_position_key k ON k.todo_id = t.id
             WHERE t.id = ?1",
            [id],
            |row| {
                Ok(Fields {
                    priority: non_empty(row.get(0)?),
                    completed: row.get(1)?,
                    completed_at: day(row.get(2)?),
                    created_at: day(row.get(3)?),
                    description: row.get(4)?,
                    position: row.get(5)?,
                    deleted: row.get(6)?,
                })
            },
        )
        .optional()?)
}

/// Führt `body` in einem Sicherungspunkt aus (verschachtelbar, auch innerhalb einer Transaktion).
pub(crate) fn atomic<T>(
    conn: &Connection,
    body: impl FnOnce() -> Result<T, StoreError>,
) -> Result<T, StoreError> {
    conn.execute_batch("SAVEPOINT nunc_crdt")?;
    match body() {
        Ok(value) => {
            conn.execute_batch("RELEASE nunc_crdt")?;
            Ok(value)
        }
        Err(error) => {
            conn.execute_batch("ROLLBACK TO nunc_crdt; RELEASE nunc_crdt")?;
            Err(error)
        }
    }
}

fn local_device(conn: &Connection) -> Result<String, StoreError> {
    let stored: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'device_id'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(stored
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| FALLBACK_DEVICE.to_owned()))
}

fn stored_clock(conn: &Connection) -> Result<Option<(i64, u32)>, StoreError> {
    Ok(conn
        .query_row(
            "SELECT hlc_ms, hlc_counter FROM device_clock WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get::<_, i64>(1)?.try_into().unwrap_or(0))),
        )
        .optional()?)
}

fn store_clock(conn: &Connection, wall_ms: i64, counter: u32) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO device_clock (id, hlc_ms, hlc_counter) VALUES (1, ?1, ?2)
         ON CONFLICT(id) DO UPDATE SET hlc_ms = ?1, hlc_counter = ?2",
        params![wall_ms, counter],
    )?;
    Ok(())
}

/// Nächster Stempel dieses Geräts: immer jünger als der letzte vergebene oder gesehene.
pub(crate) fn next_hlc(conn: &Connection, at: DateTime<Utc>) -> Result<Hlc, StoreError> {
    let device = local_device(conn)?;
    let last = stored_clock(conn)?.map(|(wall, counter)| Hlc::new(wall, counter, device.clone()));
    let hlc = Hlc::next(last.as_ref(), at.timestamp_millis(), &device);
    store_clock(conn, hlc.wall_ms, hlc.counter)?;
    Ok(hlc)
}

/// Merkt sich einen fremden Stempel, damit lokale Schreibvorgänge danach immer jünger sind.
fn observe(conn: &Connection, clock: &Hlc) -> Result<(), StoreError> {
    let current = stored_clock(conn)?.unwrap_or((i64::MIN, 0));
    if (clock.wall_ms, clock.counter) > current {
        store_clock(conn, clock.wall_ms, clock.counter)?;
    }
    Ok(())
}

fn write_clock(conn: &Connection, id: i64, field: &str, clock: &Hlc) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO todo_field_clock (todo_id, field, hlc_ms, hlc_counter, hlc_device)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(todo_id, field)
         DO UPDATE SET hlc_ms = ?3, hlc_counter = ?4, hlc_device = ?5",
        params![id, field, clock.wall_ms, clock.counter, clock.device_id],
    )?;
    Ok(())
}

/// Stempelt die Felder von Todo `id`, die sich gegenüber `before` geändert haben, mit einem neuen Stempel.
pub(crate) fn stamp_changes(
    conn: &Connection,
    id: i64,
    before: Option<&Fields>,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    let Some(after) = fields_of(conn, id)? else {
        return Ok(());
    };
    let changed = after.changed(before);
    if changed.is_empty() {
        return Ok(());
    }
    let clock = next_hlc(conn, at)?;
    for field in changed {
        write_clock(conn, id, field, &clock)?;
    }
    Ok(())
}

fn last_key(conn: &Connection) -> Result<Option<String>, StoreError> {
    let key: Option<String> =
        conn.query_row("SELECT MAX(key) FROM todo_position_key", [], |row| {
            row.get(0)
        })?;
    Ok(key.filter(|key| !key.is_empty()))
}

fn set_key(conn: &Connection, id: i64, key: &str) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO todo_position_key (todo_id, key) VALUES (?1, ?2)
         ON CONFLICT(todo_id) DO UPDATE SET key = ?2",
        params![id, key],
    )?;
    Ok(())
}

/// Ein neu eingefügtes Todo: Schlüssel hinter allen anderen und Stempel für alle Felder.
pub(crate) fn register_new_todo(
    conn: &Connection,
    id: i64,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    let key = position_between(last_key(conn)?.as_deref(), None);
    set_key(conn, id, &key)?;
    stamp_changes(conn, id, None, at)
}

/// Alle Schlüssel in Reihenfolge der Spalte `position`, auch die gelöschter Todos.
fn keys_in_order(conn: &Connection) -> Result<Vec<(i64, String)>, StoreError> {
    let keys = conn
        .prepare(
            "SELECT t.id, COALESCE(k.key, '')
             FROM todos t LEFT JOIN todo_position_key k ON k.todo_id = t.id
             ORDER BY t.position, t.id",
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;
    Ok(keys)
}

/// Setzt `position` auf den Rang nach Schlüssel (bei gleichem Schlüssel nach `sync_id`).
fn renumber_positions(conn: &Connection) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE todos SET position = (
             SELECT ranked.rank FROM (
                 SELECT t.id AS id,
                        ROW_NUMBER() OVER (ORDER BY COALESCE(k.key, ''), COALESCE(t.sync_id, ''), t.id) - 1
                            AS rank
                 FROM todos t LEFT JOIN todo_position_key k ON k.todo_id = t.id
             ) AS ranked WHERE ranked.id = todos.id)",
        [],
    )?;
    Ok(())
}

/// Welche Einträge ihren Schlüssel behalten: die längste Folge streng aufsteigender, nicht leerer Schlüssel.
/// Alle anderen bekommen neue Schlüssel, so ändern sich beim Verschieben möglichst wenige Todos.
fn keep_flags(keys: &[&str]) -> Vec<bool> {
    let mut tails: Vec<usize> = Vec::new();
    let mut previous: Vec<Option<usize>> = vec![None; keys.len()];
    for (index, key) in keys.iter().enumerate() {
        if key.is_empty() {
            continue;
        }
        let length = tails.partition_point(|&tail| keys[tail] < *key);
        previous[index] = length.checked_sub(1).map(|before| tails[before]);
        if length == tails.len() {
            tails.push(index);
        } else {
            tails[length] = index;
        }
    }
    let mut keep = vec![false; keys.len()];
    let mut cursor = tails.last().copied();
    while let Some(index) = cursor {
        keep[index] = true;
        cursor = previous[index];
    }
    keep
}

/// Passt die Schlüssel an die Reihenfolge der Spalte `position` an (nach `reorder_todos`) und stempelt nur die
/// Todos, deren Schlüssel sich ändert. Todos in der längsten schon passenden Folge behalten ihren Schlüssel; die
/// übrigen bekommen einen Schlüssel zwischen ihren Nachbarn.
pub(crate) fn rekey_positions(conn: &Connection, at: DateTime<Utc>) -> Result<(), StoreError> {
    let order = keys_in_order(conn)?;
    let keys: Vec<&str> = order.iter().map(|(_, key)| key.as_str()).collect();
    let keep = keep_flags(&keys);
    let mut previous: Option<String> = None;
    let mut changed = Vec::new();
    for (index, (id, key)) in order.iter().enumerate() {
        if keep[index] {
            previous = Some(key.clone());
            continue;
        }
        let next = order[index + 1..]
            .iter()
            .zip(&keep[index + 1..])
            .find_map(|((_, later), kept)| kept.then_some(later.as_str()));
        let fresh = position_between(previous.as_deref(), next);
        set_key(conn, *id, &fresh)?;
        previous = Some(fresh);
        changed.push(*id);
    }
    if !changed.is_empty() {
        let clock = next_hlc(conn, at)?;
        for id in changed {
            write_clock(conn, id, "position", &clock)?;
        }
    }
    renumber_positions(conn)
}

/// Zählt einen Pomodoro für dieses Gerät und hält `todos.pomodoro_count` als Summe aktuell.
pub(crate) fn add_pomodoro(conn: &Connection, id: i64) -> Result<(), StoreError> {
    reconcile_counters(conn, id)?;
    conn.execute(
        "INSERT INTO todo_pomodoro_counter (todo_id, device_id, count) VALUES (?1, ?2, 1)
         ON CONFLICT(todo_id, device_id) DO UPDATE SET count = count + 1",
        params![id, local_device(conn)?],
    )?;
    sync_pomodoro_column(conn, id)
}

/// Steht in `pomodoro_count` mehr als die Zähler ergeben (Zeile von außen eingefügt), geht der Rest aufs
/// Pseudo-Gerät `legacy`.
fn reconcile_counters(conn: &Connection, id: i64) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO todo_pomodoro_counter (todo_id, device_id, count)
         SELECT t.id, ?2, t.pomodoro_count - COALESCE(
                    (SELECT SUM(count) FROM todo_pomodoro_counter WHERE todo_id = t.id), 0)
         FROM todos t
         WHERE t.id = ?1 AND t.pomodoro_count > COALESCE(
                    (SELECT SUM(count) FROM todo_pomodoro_counter WHERE todo_id = t.id), 0)
         ON CONFLICT(todo_id, device_id) DO UPDATE SET count = count + excluded.count",
        params![id, LEGACY_DEVICE],
    )?;
    Ok(())
}

fn sync_pomodoro_column(conn: &Connection, id: i64) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE todos SET pomodoro_count = (
             SELECT COALESCE(SUM(count), 0) FROM todo_pomodoro_counter WHERE todo_id = ?1)
         WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

fn clocks_of(conn: &Connection, id: i64) -> Result<HashMap<String, Hlc>, StoreError> {
    let clocks = conn
        .prepare(
            "SELECT field, hlc_ms, hlc_counter, hlc_device FROM todo_field_clock WHERE todo_id = ?1",
        )?
        .query_map([id], |row| {
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

fn pomodoros_of(conn: &Connection, id: i64) -> Result<BTreeMap<String, u32>, StoreError> {
    let counters = conn
        .prepare("SELECT device_id, count FROM todo_pomodoro_counter WHERE todo_id = ?1")?
        .query_map([id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?.try_into().unwrap_or(0),
            ))
        })?
        .collect::<Result<_, _>>()?;
    Ok(counters)
}

/// Das Todo `id` als CRDT: Werte aus den Spalten, Stempel aus `todo_field_clock` (fehlend = uralt).
fn read_todo(conn: &Connection, id: i64, sync_id: &str) -> Result<TodoCrdt, StoreError> {
    reconcile_counters(conn, id)?;
    let fields = fields_of(conn, id)?.ok_or(StoreError::NotFound {
        entity: "Todo",
        id: i32::try_from(id).unwrap_or(i32::MAX),
    })?;
    let clocks = clocks_of(conn, id)?;
    let clock = |field: &str| {
        clocks
            .get(field)
            .cloned()
            .unwrap_or_else(|| Hlc::new(0, 0, LEGACY_DEVICE))
    };
    Ok(TodoCrdt {
        sync_id: sync_id.to_owned(),
        priority: Register::new(fields.priority, clock("priority")),
        completed: Register::new(fields.completed, clock("completed")),
        completed_at: Register::new(fields.completed_at, clock("completed_at")),
        created_at: Register::new(fields.created_at, clock("created_at")),
        description: Register::new(fields.description, clock("description")),
        position: Register::new(fields.position, clock("position")),
        deleted: Register::new(fields.deleted, clock("deleted")),
        pomodoros: pomodoros_of(conn, id)?,
    })
}

/// Der lokale Stand für den Austausch: alle Todos (auch gelöschte) und alle abgeschlossenen Sessions.
/// `new_sync_id` vergibt Kennungen an Todos, die noch keine haben.
pub fn local_state(
    conn: &Connection,
    new_sync_id: impl FnMut() -> String,
) -> Result<SyncState, StoreError> {
    crate::sync_repo::assign_missing_sync_ids(conn, new_sync_id)?;
    let rows: Vec<(i64, String)> = conn
        .prepare("SELECT id, sync_id FROM todos ORDER BY id")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;
    let mut todos = BTreeMap::new();
    for (id, sync_id) in rows {
        todos.insert(sync_id.clone(), read_todo(conn, id, &sync_id)?);
    }
    let device = local_device(conn)?;
    let sessions: Vec<SyncSession> = conn
        .prepare(
            "SELECT COALESCE(sync_device_id, ''), mode, session_type, status, started_at,
                    COALESCE(ended_at, ''), COALESCE(planned_min, 0), COALESCE(actual_min, 0),
                    COALESCE(break_min, 0)
             FROM sessions WHERE status = 'completed' ORDER BY started_at, id",
        )?
        .query_map([], |row| sync_session(row, &device))?
        .collect::<Result<_, _>>()?;
    Ok(SyncState {
        todos,
        sessions: sessions
            .into_iter()
            .map(|session| (session_key(&session), session))
            .collect(),
    })
}

/// Jüngster Stempel eines Todos als Zeitpunkt (Spalte `updated_at`).
fn updated_at(todo: &TodoCrdt) -> String {
    let wall = [
        &todo.priority.clock,
        &todo.completed.clock,
        &todo.completed_at.clock,
        &todo.created_at.clock,
        &todo.description.clock,
        &todo.position.clock,
        &todo.deleted.clock,
    ]
    .into_iter()
    .map(|clock| clock.wall_ms)
    .max()
    .unwrap_or(0);
    to_text(Utc.timestamp_millis_opt(wall).single().unwrap_or_default())
}

fn write_todo_row(conn: &Connection, id: i64, todo: &TodoCrdt) -> Result<(), StoreError> {
    let raw_text = todo.raw_text();
    let project_tags = serde_json::to_string(&todo.project_tags())?;
    let context_tags = serde_json::to_string(&todo.context_tags())?;
    conn.execute(
        "UPDATE todos SET raw_text = ?1, priority = ?2, completed = ?3, completed_at = ?4, created_at = ?5,
                description = ?6, project_tags = ?7, context_tags = ?8, pomodoro_count = ?9, updated_at = ?10,
                deleted = ?11
         WHERE id = ?12",
        params![
            raw_text,
            todo.priority.value,
            todo.completed.value,
            todo.completed_at.value,
            todo.created_at.value,
            todo.description.value,
            project_tags,
            context_tags,
            todo.pomodoro_count(),
            updated_at(todo),
            todo.is_deleted(),
            id,
        ],
    )?;
    set_key(conn, id, &todo.position.value)?;
    let clocks = [
        ("priority", &todo.priority.clock),
        ("completed", &todo.completed.clock),
        ("completed_at", &todo.completed_at.clock),
        ("created_at", &todo.created_at.clock),
        ("description", &todo.description.clock),
        ("position", &todo.position.clock),
        ("deleted", &todo.deleted.clock),
    ];
    for (field, clock) in clocks {
        write_clock(conn, id, field, clock)?;
        observe(conn, clock)?;
    }
    for (device, count) in &todo.pomodoros {
        conn.execute(
            "INSERT INTO todo_pomodoro_counter (todo_id, device_id, count) VALUES (?1, ?2, ?3)
             ON CONFLICT(todo_id, device_id) DO UPDATE SET count = ?3",
            params![id, device, count],
        )?;
    }
    Ok(())
}

/// Schreibt den zusammengeführten Stand in die Datenbank (**eine Transaktion**): neue Todos werden angelegt
/// (bereits gelöschte nicht), geänderte in allen Feldern, Stempeln und Zählern nachgezogen, `raw_text` und
/// Tags aus den Feldern neu aufgebaut, neue Sessions angefügt und die Reihenfolge neu nummeriert.
pub fn apply_state(conn: &mut Connection, merged: &SyncState) -> Result<Applied, StoreError> {
    let tx = conn.transaction()?;
    let mut applied = Applied::default();
    for (sync_id, todo) in &merged.todos {
        let local: Option<i64> = tx
            .query_row(
                "SELECT id FROM todos WHERE sync_id = ?1",
                [sync_id],
                |row| row.get(0),
            )
            .optional()?;
        match local {
            None if todo.is_deleted() => {}
            None => {
                tx.execute(
                    "INSERT INTO todos (raw_text, description, position, sync_id) VALUES ('', '', 0, ?1)",
                    [sync_id],
                )?;
                let id = tx.last_insert_rowid();
                write_todo_row(&tx, id, todo)?;
                applied.todos_added += 1;
            }
            Some(id) => {
                if read_todo(&tx, id, sync_id)? != *todo {
                    write_todo_row(&tx, id, todo)?;
                    applied.todos_replaced += 1;
                }
            }
        }
    }
    for session in merged.sessions.values() {
        let known: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions
                           WHERE started_at = ?1 AND COALESCE(sync_device_id, '') = ?2)",
            params![session.started_at, session.device_id],
            |row| row.get(0),
        )?;
        if known {
            continue;
        }
        applied.sessions_added += 1;
        tx.execute(
            "INSERT INTO sessions (sync_device_id, mode, session_type, status, started_at, ended_at,
                                   planned_min, actual_min, break_min)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                session.device_id,
                session.mode,
                session.session_type,
                session.status,
                session.started_at,
                session.ended_at,
                (session.planned_min > 0).then_some(session.planned_min),
                session.actual_min,
                session.break_min,
            ],
        )?;
    }
    renumber_positions(&tx)?;
    tx.commit()?;
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;

    #[test]
    fn equal_keys_are_not_both_kept() {
        let kept = keep_flags(&["a", "a", "b"]);
        assert_eq!(kept.iter().filter(|kept| **kept).count(), 2);
        assert!(kept[2]);
        assert_eq!(keep_flags(&["b", "a"]), [false, true]);
        assert_eq!(keep_flags(&["", "a", ""]), [false, true, false]);
    }

    #[test]
    fn a_pomodoro_count_written_from_outside_is_not_lost_when_counting() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO todos (raw_text, description, position, pomodoro_count) VALUES ('t', 't', 0, 2)",
            [],
        )
        .unwrap();
        add_pomodoro(&conn, 1).unwrap();
        let column: i64 = conn
            .query_row("SELECT pomodoro_count FROM todos WHERE id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(column, 3);
        assert_eq!(pomodoros_of(&conn, 1).unwrap().get(LEGACY_DEVICE), Some(&2));
    }
}
