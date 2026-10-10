//! CRDT-Seite der Sessions (ADR 0017): Kennung, Notiz, Löschkennzeichen und Stempel je Feld. Die Spalten von
//! `sessions` bleiben der aufgelöste Stand; hier liegen die Zusätze (`session_meta`, `session_field_clock`).
//!
//! Wie bei den Todos stempelt [`stamp_changes`] nach einer lokalen Änderung nur die Felder, die sich gegenüber dem
//! Stand davor geändert haben. [`ensure_meta`] vergibt Bestandssessions beim ersten Zugriff ihre Kennung.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use nunc_core::crdt::{KIND_STOPWATCH, ORIGIN_MANUAL, ORIGIN_TIMER, Register, SessionCrdt};
use nunc_core::hlc::Hlc;
use nunc_core::model::SyncSession;
use nunc_core::sync_document::LEGACY_DEVICE;
use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;
use crate::crdt_repo::{local_device, next_hlc, observe};
use crate::sync_repo::sync_session;

/// Felder mit eigenem Stempel.
const FIELDS: [&str; 6] = [
    "started_at",
    "ended_at",
    "actual_min",
    "todo",
    "note",
    "deleted",
];

/// Spalten von `sessions` in der Reihenfolge, die [`sync_session`] erwartet (Index 0 bis 8).
const RAW_COLUMNS: &str =
    "COALESCE(s.sync_device_id, ''), s.mode, s.session_type, s.status, s.started_at,
                    COALESCE(s.ended_at, ''), COALESCE(s.planned_min, 0), COALESCE(s.actual_min, 0),
                    COALESCE(s.break_min, 0)";

/// Die gestempelten Felder einer Session, wie sie in der Datenbank stehen. `todo` ist die `sync_id` des Todos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Fields {
    started_at: String,
    ended_at: Option<String>,
    actual_min: u32,
    todo: Option<String>,
    note: String,
    deleted: bool,
}

impl Fields {
    pub(crate) fn is_deleted(&self) -> bool {
        self.deleted
    }

    /// Namen der Felder, die sich gegenüber `before` geändert haben (ohne `before`: alle).
    fn changed(&self, before: Option<&Fields>) -> Vec<&'static str> {
        let Some(before) = before else {
            return FIELDS.to_vec();
        };
        let differs = [
            self.started_at != before.started_at,
            self.ended_at != before.ended_at,
            self.actual_min != before.actual_min,
            self.todo != before.todo,
            self.note != before.note,
            self.deleted != before.deleted,
        ];
        FIELDS
            .into_iter()
            .zip(differs)
            .filter_map(|(field, differs)| differs.then_some(field))
            .collect()
    }
}

pub(crate) fn fields_of(conn: &Connection, id: i64) -> Result<Option<Fields>, StoreError> {
    Ok(conn
        .query_row(
            "SELECT s.started_at, COALESCE(s.ended_at, ''), COALESCE(s.actual_min, 0), t.sync_id,
                    COALESCE(m.note, ''), COALESCE(m.deleted, 0)
             FROM sessions s
             LEFT JOIN session_meta m ON m.session_id = s.id
             LEFT JOIN todos t ON t.id = s.todo_id
             WHERE s.id = ?1",
            [id],
            |row| {
                let ended: String = row.get(1)?;
                let todo: Option<String> = row.get(3)?;
                Ok(Fields {
                    started_at: row.get(0)?,
                    ended_at: (!ended.is_empty()).then_some(ended),
                    actual_min: row.get::<_, i64>(2)?.try_into().unwrap_or(0),
                    todo: todo.filter(|sync_id| !sync_id.is_empty()),
                    note: row.get(4)?,
                    deleted: row.get(5)?,
                })
            },
        )
        .optional()?)
}

/// Vergibt jeder Session ohne `session_meta`-Zeile ihre Kennung (deterministisch aus Gerät und Beginn, siehe
/// [`SessionCrdt::from_legacy`]). Zwei Zeilen mit demselben Schlüssel bekommen für die zweite einen Zusatz, damit
/// nichts verloren geht.
pub(crate) fn ensure_meta(conn: &Connection) -> Result<(), StoreError> {
    let device = local_device(conn)?;
    let missing: Vec<(i64, SyncSession)> = conn
        .prepare(&format!(
            "SELECT {RAW_COLUMNS}, s.id FROM sessions s
             WHERE NOT EXISTS (SELECT 1 FROM session_meta m WHERE m.session_id = s.id)
             ORDER BY s.id"
        ))?
        .query_map([], |row| Ok((row.get(9)?, sync_session(row, &device)?)))?
        .collect::<Result<_, _>>()?;
    for (id, session) in missing {
        let base = SessionCrdt::from_legacy(&session).sync_id;
        let taken: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM session_meta WHERE sync_id = ?1)",
            [&base],
            |row| row.get(0),
        )?;
        let sync_id = if taken { format!("{base}#{id}") } else { base };
        conn.execute(
            "INSERT INTO session_meta (session_id, sync_id, origin) VALUES (?1, ?2, ?3)",
            params![id, sync_id, ORIGIN_TIMER],
        )?;
    }
    Ok(())
}

fn write_clock(conn: &Connection, id: i64, field: &str, clock: &Hlc) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO session_field_clock (session_id, field, hlc_ms, hlc_counter, hlc_device)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(session_id, field)
         DO UPDATE SET hlc_ms = ?3, hlc_counter = ?4, hlc_device = ?5",
        params![id, field, clock.wall_ms, clock.counter, clock.device_id],
    )?;
    Ok(())
}

/// Stempelt die Felder von Session `id`, die sich gegenüber `before` geändert haben, mit einem neuen Stempel.
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

fn clocks_of(conn: &Connection, id: i64) -> Result<HashMap<String, Hlc>, StoreError> {
    let clocks = conn
        .prepare(
            "SELECT field, hlc_ms, hlc_counter, hlc_device FROM session_field_clock WHERE session_id = ?1",
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

/// Ids der abgeschlossenen Sessions (auch gelöschte) in Reihenfolge des Beginns, die zum Austausch gehören.
pub(crate) fn exportable_ids(conn: &Connection) -> Result<Vec<i64>, StoreError> {
    let ids = conn
        .prepare("SELECT id FROM sessions WHERE status = 'completed' ORDER BY started_at, id")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(ids)
}

/// Die Session `id` als CRDT: Werte aus den Spalten, Stempel aus `session_field_clock` (fehlend = uralt).
/// Setzt voraus, dass [`ensure_meta`] gelaufen ist.
pub(crate) fn read_session(conn: &Connection, id: i64) -> Result<SessionCrdt, StoreError> {
    let device = local_device(conn)?;
    let (raw, sync_id, origin, todo, note, deleted, kind) = conn.query_row(
        &format!(
            "SELECT {RAW_COLUMNS}, m.sync_id, m.origin, t.sync_id, m.note, m.deleted, m.kind
             FROM sessions s
             JOIN session_meta m ON m.session_id = s.id
             LEFT JOIN todos t ON t.id = s.todo_id
             WHERE s.id = ?1"
        ),
        [id],
        |row| {
            Ok((
                sync_session(row, &device)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, String>(12)?,
                row.get::<_, bool>(13)?,
                row.get::<_, String>(14)?,
            ))
        },
    )?;
    let clocks = clocks_of(conn, id)?;
    let clock = |field: &str| {
        clocks
            .get(field)
            .cloned()
            .unwrap_or_else(|| Hlc::new(0, 0, LEGACY_DEVICE))
    };
    let mut session = SessionCrdt::from_session(sync_id, &raw, &clock("started_at"));
    session.origin = origin;
    session.kind = kind;
    session.started_at.clock = clock("started_at");
    session.ended_at.clock = clock("ended_at");
    session.actual_min.clock = clock("actual_min");
    session.todo = Register::new(todo.filter(|sync_id| !sync_id.is_empty()), clock("todo"));
    session.note = Register::new(note, clock("note"));
    session.deleted = Register::new(deleted, clock("deleted"));
    Ok(session)
}

/// Das Löschkennzeichen als aufgelöster Wert: In der Datenbank steht nur dieser, nicht der rohe Registerwert (siehe
/// `crdt_repo::resolved` bei den Todos).
pub(crate) fn resolved(mut session: SessionCrdt) -> SessionCrdt {
    session.deleted.value = session.is_deleted();
    session
}

/// Legt die Zeile einer Session an, die bisher nur ein anderes Gerät kennt, und schreibt Felder und Stempel.
pub(crate) fn insert_session(conn: &Connection, session: &SessionCrdt) -> Result<i64, StoreError> {
    conn.execute(
        "INSERT INTO sessions (sync_device_id, mode, session_type, status, started_at, planned_min, break_min,
                               actual_min)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
        params![
            session.device_id,
            session.mode,
            session.session_type,
            session.status,
            session.started_at.value,
            (session.planned_min > 0).then_some(session.planned_min),
            (session.break_min > 0).then_some(session.break_min),
        ],
    )?;
    let id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO session_meta (session_id, sync_id, origin, kind) VALUES (?1, ?2, ?3, ?4)",
        params![id, session.sync_id, session.origin, kind_of(session)],
    )?;
    write_session_row(conn, id, session)?;
    Ok(id)
}

/// Nachgetragene Einträge zählen als Stoppuhr (ADR 0018), auch wenn die Datei kein `kind` kennt.
fn kind_of(session: &SessionCrdt) -> &str {
    if session.origin == ORIGIN_MANUAL {
        KIND_STOPWATCH
    } else {
        &session.kind
    }
}

/// Schreibt die Register einer Session samt Stempeln in die Datenbank.
pub(crate) fn write_session_row(
    conn: &Connection,
    id: i64,
    session: &SessionCrdt,
) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE sessions SET started_at = ?1, ended_at = ?2, actual_min = ?3,
                todo_id = (SELECT id FROM todos WHERE sync_id = ?4)
         WHERE id = ?5",
        params![
            session.started_at.value,
            session.ended_at.value,
            session.actual_min.value,
            session.todo.value,
            id,
        ],
    )?;
    conn.execute(
        "UPDATE session_meta SET note = ?1, deleted = ?2 WHERE session_id = ?3",
        params![session.note.value, session.is_deleted(), id],
    )?;
    let clocks = [
        ("started_at", &session.started_at.clock),
        ("ended_at", &session.ended_at.clock),
        ("actual_min", &session.actual_min.clock),
        ("todo", &session.todo.clock),
        ("note", &session.note.clock),
        ("deleted", &session.deleted.clock),
    ];
    for (field, clock) in clocks {
        write_clock(conn, id, field, clock)?;
        observe(conn, clock)?;
    }
    Ok(())
}

/// Row-Id der Session mit dieser Kennung.
pub(crate) fn row_of(conn: &Connection, sync_id: &str) -> Result<Option<i64>, StoreError> {
    Ok(conn
        .query_row(
            "SELECT session_id FROM session_meta WHERE sync_id = ?1",
            [sync_id],
            |row| row.get(0),
        )
        .optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;

    fn base() -> Fields {
        Fields {
            started_at: "2026-10-04T09:00:00Z".to_owned(),
            ended_at: None,
            actual_min: 25,
            todo: None,
            note: String::new(),
            deleted: false,
        }
    }

    #[test]
    fn each_field_alone_is_reported_as_changed() {
        let base = base();
        assert_eq!(base.changed(Some(&base)), Vec::<&str>::new());
        assert_eq!(base.changed(None), FIELDS.to_vec());
        let variants = [
            (
                "started_at",
                Fields {
                    started_at: "x".into(),
                    ..self::base()
                },
            ),
            (
                "ended_at",
                Fields {
                    ended_at: Some("x".into()),
                    ..self::base()
                },
            ),
            (
                "actual_min",
                Fields {
                    actual_min: 1,
                    ..self::base()
                },
            ),
            (
                "todo",
                Fields {
                    todo: Some("t".into()),
                    ..self::base()
                },
            ),
            (
                "note",
                Fields {
                    note: "x".into(),
                    ..self::base()
                },
            ),
            (
                "deleted",
                Fields {
                    deleted: true,
                    ..self::base()
                },
            ),
        ];
        for (field, changed) in variants {
            assert_eq!(changed.changed(Some(&base)), [field]);
        }
    }

    #[test]
    fn an_empty_end_is_none_and_a_set_end_is_kept() {
        let conn = open_in_memory().unwrap();
        for ended in ["NULL", "'2026-10-04T09:25:00Z'"] {
            conn.execute(
                &format!(
                    "INSERT INTO sessions (mode, session_type, status, started_at, ended_at)
                     VALUES ('pomodoro','work','completed','2026-10-04T09:00:00Z', {ended})"
                ),
                [],
            )
            .unwrap();
        }
        assert_eq!(fields_of(&conn, 1).unwrap().unwrap().ended_at, None);
        assert_eq!(
            fields_of(&conn, 2).unwrap().unwrap().ended_at.as_deref(),
            Some("2026-10-04T09:25:00Z")
        );
        assert!(fields_of(&conn, 3).unwrap().is_none());
    }

    #[test]
    fn two_rows_with_the_same_key_both_get_an_id() {
        let conn = open_in_memory().unwrap();
        for _ in 0..2 {
            conn.execute(
                "INSERT INTO sessions (mode, session_type, status, started_at, sync_device_id)
                 VALUES ('pomodoro','work','completed','2026-10-04T09:00:00Z', 'dev-x')",
                [],
            )
            .unwrap();
        }
        ensure_meta(&conn).unwrap();
        let ids: Vec<String> = conn
            .prepare("SELECT sync_id FROM session_meta ORDER BY session_id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(ids.len(), 2);
        assert!(ids[1].ends_with("#2") && !ids[0].contains('#'));
        // ein zweiter Lauf ändert nichts
        ensure_meta(&conn).unwrap();
        assert_eq!(row_of(&conn, &ids[0]).unwrap(), Some(1));
        assert_eq!(row_of(&conn, "nope").unwrap(), None);
    }

    #[test]
    fn a_read_session_carries_the_stored_stamps_and_values() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO sessions (mode, session_type, status, started_at, actual_min, sync_device_id)
             VALUES ('pomodoro','work','completed','2026-10-04T09:00:00Z', 25, 'dev-x')",
            [],
        )
        .unwrap();
        ensure_meta(&conn).unwrap();
        let fresh = read_session(&conn, 1).unwrap();
        assert_eq!(fresh.note.clock, Hlc::new(0, 0, LEGACY_DEVICE));
        conn.execute(
            "UPDATE session_meta SET note = 'x' WHERE session_id = 1",
            [],
        )
        .unwrap();
        write_clock(&conn, 1, "note", &Hlc::new(7, 2, "dev-y")).unwrap();
        let stored = read_session(&conn, 1).unwrap();
        assert_eq!(stored.note.value, "x");
        assert_eq!(stored.note.clock, Hlc::new(7, 2, "dev-y"));
        assert_eq!(stored.origin, "timer");
        assert_eq!(exportable_ids(&conn).unwrap(), [1]);
    }
}
