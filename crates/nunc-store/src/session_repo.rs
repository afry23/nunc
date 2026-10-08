//! Sessions in der Tabelle `sessions`. Die Go-App legt eine Session schon beim Start an; nunc schreibt sie erst
//! beim Ende, der laufende Zustand liegt in `active_timer`. Seit ADR 0017 lassen sie sich nachtragen, bearbeiten und
//! löschen (Tombstone in `session_meta`); die Stempel dazu verwaltet `session_sync`.

use chrono::{DateTime, Utc};
use nunc_core::model::Session;
use nunc_core::timer::{Finish, Timer};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::crdt_repo::atomic;
use crate::session_sync::{ensure_meta, fields_of, stamp_changes};
use crate::time::{parse_instant, to_text};
use crate::{StoreError, increment_pomodoro_count, save_timer};

const COLUMNS: &str =
    "s.id, s.mode, s.session_type, s.status, s.started_at, s.ended_at, s.planned_min,
                s.actual_min, s.todo_id, s.break_min";

fn corrupt(index: usize, what: &str, text: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        rusqlite::types::Type::Text,
        format!("ungültiger Wert für {what}: {text:?}").into(),
    )
}

fn session_from_row(row: &Row<'_>) -> rusqlite::Result<Session> {
    let mode: String = row.get(1)?;
    let session_type: String = row.get(2)?;
    let status: String = row.get(3)?;
    let started_at: String = row.get(4)?;
    let ended_at: Option<String> = row.get(5)?;
    let minutes = |index: usize| -> rusqlite::Result<Option<u32>> {
        Ok(row
            .get::<_, Option<i64>>(index)?
            .map(|m| u32::try_from(m).unwrap_or(0)))
    };
    Ok(Session {
        id: row.get(0)?,
        mode: mode.parse().map_err(|_| corrupt(1, "mode", &mode))?,
        session_type: session_type
            .parse()
            .map_err(|_| corrupt(2, "session_type", &session_type))?,
        status: status.parse().map_err(|_| corrupt(3, "status", &status))?,
        started_at: parse_instant(&started_at)
            .ok_or_else(|| corrupt(4, "started_at", &started_at))?,
        ended_at: ended_at.as_deref().and_then(parse_instant),
        planned_min: minutes(6)?,
        actual_min: minutes(7)?,
        todo_id: row.get(8)?,
        break_min: minutes(9)?,
    })
}

/// Alle nicht gelöschten Sessions nach Startzeit (Rohdaten für `nunc_core::stats`, den Export und das Timesheet).
/// Nichts wird vorab aggregiert; der Zeitraum und die lokale Zeitzone sind Sache der Fachlogik.
pub fn list_sessions(conn: &Connection) -> Result<Vec<Session>, StoreError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM sessions s
         LEFT JOIN session_meta m ON m.session_id = s.id
         WHERE COALESCE(m.deleted, 0) = 0
         ORDER BY s.started_at, s.id"
    ))?;
    let sessions = stmt
        .query_map([], session_from_row)?
        .collect::<Result<_, _>>()?;
    Ok(sessions)
}

/// Übernimmt das Ende einer Session in **einer Transaktion**: die Session anfügen, den Pomodoro-Zähler des
/// Todos erhöhen und den neuen Timer-Zustand speichern. So gibt es nach einem Absturz weder doppelte Sessions
/// (Timer noch „abgelaufen“) noch eine fehlende Session. `device_id` wird als `sync_device_id` vermerkt.
pub fn commit_finish(
    conn: &mut Connection,
    finish: &Finish,
    device_id: &str,
    timer: &Timer,
    now: DateTime<Utc>,
) -> Result<Session, StoreError> {
    let tx = conn.transaction()?;
    let s = &finish.session;
    tx.execute(
        "INSERT INTO sessions (mode, session_type, status, started_at, ended_at, planned_min, actual_min,
                               todo_id, sync_device_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            s.mode.as_str(),
            s.session_type.as_str(),
            s.status.as_str(),
            to_text(s.started_at),
            to_text(s.ended_at),
            s.planned_min,
            s.actual_min,
            s.todo_id,
            device_id,
        ],
    )?;
    let id = i32::try_from(tx.last_insert_rowid()).unwrap_or(i32::MAX);
    if let Some(todo_id) = finish.increment_todo {
        increment_pomodoro_count(&tx, todo_id)?;
    }
    save_timer(&tx, timer, now)?;
    let session = get(&tx, id.into())?;
    tx.commit()?;
    Ok(session)
}

fn get(conn: &Connection, id: i64) -> Result<Session, StoreError> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM sessions s WHERE s.id = ?1"),
        [id],
        session_from_row,
    )
    .optional()?
    .ok_or(StoreError::NotFound {
        entity: "Session",
        id: i32::try_from(id).unwrap_or(i32::MAX),
    })
}

/// Notiz und Herkunft einer Session (`timer` oder `manual`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDetails {
    pub note: String,
    pub origin: String,
}

/// Notiz und Herkunft der nicht gelöschten Sessions nach Id. Sessions ohne Metadaten (noch nie gesynct oder
/// bearbeitet) sind vom Timer aufgezeichnet und haben keine Notiz, sie fehlen in der Antwort.
pub fn list_session_details(
    conn: &Connection,
) -> Result<std::collections::HashMap<i32, SessionDetails>, StoreError> {
    let details = conn
        .prepare("SELECT session_id, note, origin FROM session_meta WHERE deleted = 0")?
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                SessionDetails {
                    note: row.get(1)?,
                    origin: row.get(2)?,
                },
            ))
        })?
        .collect::<Result<_, _>>()?;
    Ok(details)
}

/// Eine nachgetragene Arbeitssession.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualSession {
    /// Kennung für den Sync (UUID, vom Aufrufer vergeben).
    pub sync_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub todo_id: Option<i32>,
    pub note: String,
}

/// Die vollständigen, änderbaren Werte einer Session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEdit {
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub actual_min: u32,
    pub todo_id: Option<i32>,
    pub note: String,
}

/// Minuten zwischen zwei Zeitpunkten (nie negativ).
fn minutes_between(start: DateTime<Utc>, end: DateTime<Utc>) -> u32 {
    u32::try_from((end - start).num_minutes()).unwrap_or(0)
}

/// Trägt eine Arbeitssession nach (Modus `flowtime`, Typ `work`, Status `completed`, Herkunft `manual`) und
/// stempelt alle Felder, in **einer Transaktion**. Ob der Eintrag gültig ist, prüft die Fachlogik
/// (`nunc_core::manual_entry`) vorher. `device_id` wird als `sync_device_id` vermerkt.
pub fn insert_manual(
    conn: &Connection,
    entry: &ManualSession,
    device_id: &str,
    now: DateTime<Utc>,
) -> Result<Session, StoreError> {
    let id = atomic(conn, || {
        ensure_meta(conn)?;
        conn.execute(
            "INSERT INTO sessions (mode, session_type, status, started_at, ended_at, actual_min, todo_id,
                                   sync_device_id)
             VALUES ('flowtime', 'work', 'completed', ?1, ?2, ?3, ?4, ?5)",
            params![
                to_text(entry.started_at),
                to_text(entry.ended_at),
                minutes_between(entry.started_at, entry.ended_at),
                entry.todo_id,
                device_id,
            ],
        )?;
        let id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO session_meta (session_id, sync_id, origin, note) VALUES (?1, ?2, 'manual', ?3)",
            params![id, entry.sync_id, entry.note],
        )?;
        stamp_changes(conn, id, None, now)?;
        Ok(id)
    })?;
    get(conn, id)
}

/// Ersetzt die änderbaren Werte einer Session und stempelt nur die Felder, die sich geändert haben. Eine gelöschte
/// Session gilt als nicht gefunden.
pub fn update_session(
    conn: &Connection,
    id: i32,
    edit: &SessionEdit,
    now: DateTime<Utc>,
) -> Result<Session, StoreError> {
    let id = i64::from(id);
    atomic(conn, || {
        ensure_meta(conn)?;
        let before = live_fields(conn, id)?;
        conn.execute(
            "UPDATE sessions SET started_at = ?1, ended_at = ?2, actual_min = ?3, todo_id = ?4 WHERE id = ?5",
            params![
                to_text(edit.started_at),
                edit.ended_at.map(to_text),
                edit.actual_min,
                edit.todo_id,
                id,
            ],
        )?;
        conn.execute(
            "UPDATE session_meta SET note = ?1 WHERE session_id = ?2",
            params![edit.note, id],
        )?;
        stamp_changes(conn, id, Some(&before), now)
    })?;
    get(conn, id)
}

/// Löscht eine Session (Tombstone): Sie verschwindet aus Statistik, Export und Timesheet, die Zeile bleibt für den
/// Sync erhalten. Eine schon gelöschte Session gilt als nicht gefunden.
pub fn delete_session(conn: &Connection, id: i32, now: DateTime<Utc>) -> Result<(), StoreError> {
    let id = i64::from(id);
    atomic(conn, || {
        ensure_meta(conn)?;
        let before = live_fields(conn, id)?;
        conn.execute(
            "UPDATE session_meta SET deleted = 1 WHERE session_id = ?1",
            [id],
        )?;
        stamp_changes(conn, id, Some(&before), now)
    })
}

/// Die Felder einer vorhandenen, nicht gelöschten Session.
fn live_fields(conn: &Connection, id: i64) -> Result<crate::session_sync::Fields, StoreError> {
    fields_of(conn, id)?
        .filter(|fields| !fields.is_deleted())
        .ok_or(StoreError::NotFound {
            entity: "Session",
            id: i32::try_from(id).unwrap_or(i32::MAX),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{add_todo, list_todos, load_timer, open_in_memory};
    use nunc_core::model::{Settings, TimerMode};

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    /// Timer, der um 09:25 abgelaufen ist, samt Ergebnis.
    fn finished(todo_id: Option<i32>) -> (Timer, Finish) {
        let t0 = at("2026-10-04T09:00:00Z");
        let mut timer = Timer::default();
        timer
            .start(t0, TimerMode::Pomodoro, todo_id, &Settings::default())
            .unwrap();
        let finish = timer
            .tick(at("2026-10-04T09:30:00Z"), &Settings::default())
            .unwrap();
        (timer, finish)
    }

    #[test]
    fn commit_records_session_count_and_timer() {
        let mut conn = open_in_memory().unwrap();
        let todo = add_todo(&conn, "t", "s1", at("2026-10-04T08:00:00Z")).unwrap();
        let (timer, finish) = finished(Some(todo.id));
        let session = commit_finish(
            &mut conn,
            &finish,
            "dev-1",
            &timer,
            at("2026-10-04T09:30:00Z"),
        )
        .unwrap();
        assert_eq!(session.started_at, at("2026-10-04T09:00:00Z"));
        assert_eq!(session.ended_at, Some(at("2026-10-04T09:25:00Z")));
        assert_eq!(
            (session.planned_min, session.actual_min),
            (Some(25), Some(25))
        );
        assert_eq!(session.todo_id, Some(todo.id));
        assert_eq!(list_todos(&conn).unwrap()[0].pomodoro_count, 1);
        assert_eq!(load_timer(&conn).unwrap(), timer);
        let device: String = conn
            .query_row("SELECT sync_device_id FROM sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(device, "dev-1");
        assert_eq!(list_sessions(&conn).unwrap(), [session]);
    }

    #[test]
    fn without_a_todo_nothing_is_counted() {
        let mut conn = open_in_memory().unwrap();
        let (timer, finish) = finished(None);
        commit_finish(&mut conn, &finish, "d", &timer, at("2026-10-04T09:30:00Z")).unwrap();
        assert_eq!(list_sessions(&conn).unwrap().len(), 1);
    }

    #[test]
    fn a_failing_insert_leaves_nothing_behind() {
        let mut conn = open_in_memory().unwrap();
        // Todo existiert nicht -> Foreign Key verletzt, nichts bleibt zurück
        let (timer, finish) = finished(Some(99));
        let result = commit_finish(&mut conn, &finish, "d", &timer, at("2026-10-04T09:30:00Z"));
        assert!(result.is_err());
        assert!(list_sessions(&conn).unwrap().is_empty());
        assert_eq!(load_timer(&conn).unwrap(), Timer::default());
    }

    #[test]
    fn sessions_are_listed_by_start_time_and_invalid_rows_are_errors() {
        let conn = open_in_memory().unwrap();
        for (start, status) in [
            ("2026-10-02T09:00:00Z", "completed"),
            ("2026-10-01T09:00:00Z", "aborted"),
        ] {
            conn.execute(
                "INSERT INTO sessions (mode, session_type, status, started_at) VALUES ('pomodoro','work',?1,?2)",
                [status, start],
            )
            .unwrap();
        }
        let list = list_sessions(&conn).unwrap();
        assert_eq!(list[0].started_at, at("2026-10-01T09:00:00Z"));
        assert_eq!(list[0].ended_at, None);
        assert_eq!(list[0].actual_min, None);
        conn.execute(
            "INSERT INTO sessions (mode, session_type, status, started_at) VALUES ('pomodoro','work','completed','irgendwann')",
            [],
        )
        .unwrap();
        assert!(list_sessions(&conn).is_err());
    }

    fn manual(sync_id: &str, start: &str, end: &str) -> ManualSession {
        ManualSession {
            sync_id: sync_id.to_owned(),
            started_at: at(start),
            ended_at: at(end),
            todo_id: None,
            note: "n".to_owned(),
        }
    }

    #[test]
    fn a_manual_entry_is_a_completed_flowtime_work_session() {
        let conn = open_in_memory().unwrap();
        let entry = manual("m-1", "2026-10-04T09:00:00Z", "2026-10-04T10:30:00Z");
        let session = insert_manual(&conn, &entry, "dev-1", at("2026-10-04T11:00:00Z")).unwrap();
        assert_eq!(session.mode, TimerMode::Flowtime);
        assert_eq!(session.actual_min, Some(90));
        assert_eq!(session.ended_at, Some(at("2026-10-04T10:30:00Z")));
        assert_eq!(session.planned_min, None);
        let (origin, note, device): (String, String, String) = conn
            .query_row(
                "SELECT m.origin, m.note, s.sync_device_id FROM session_meta m
                 JOIN sessions s ON s.id = m.session_id",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            (origin.as_str(), note.as_str(), device.as_str()),
            ("manual", "n", "dev-1")
        );
        let stamped: i64 = conn
            .query_row("SELECT COUNT(*) FROM session_field_clock", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stamped, 6);
    }

    #[test]
    fn details_list_note_and_origin_without_deleted_sessions() {
        let conn = open_in_memory().unwrap();
        let now = at("2026-10-04T11:00:00Z");
        let kept = manual("m-1", "2026-10-04T09:00:00Z", "2026-10-04T10:00:00Z");
        let gone = manual("m-2", "2026-10-04T10:00:00Z", "2026-10-04T10:30:00Z");
        let kept = insert_manual(&conn, &kept, "d", now).unwrap();
        let gone = insert_manual(&conn, &gone, "d", now).unwrap();
        delete_session(&conn, gone.id, now).unwrap();
        let details = list_session_details(&conn).unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(
            details[&kept.id],
            SessionDetails {
                note: "n".to_owned(),
                origin: "manual".to_owned()
            }
        );
    }

    #[test]
    fn an_end_before_the_start_gives_zero_minutes() {
        let conn = open_in_memory().unwrap();
        let entry = manual("m-1", "2026-10-04T10:00:00Z", "2026-10-04T09:00:00Z");
        let session = insert_manual(&conn, &entry, "d", at("2026-10-04T11:00:00Z")).unwrap();
        assert_eq!(session.actual_min, Some(0));
    }

    #[test]
    fn updating_replaces_the_values_and_stamps_only_the_changes() {
        let conn = open_in_memory().unwrap();
        let todo = add_todo(&conn, "t", "s1", at("2026-10-04T08:00:00Z")).unwrap();
        let entry = manual("m-1", "2026-10-04T09:00:00Z", "2026-10-04T10:00:00Z");
        let session = insert_manual(&conn, &entry, "d", at("2026-10-04T10:00:00Z")).unwrap();
        let edit = SessionEdit {
            started_at: at("2026-10-04T09:10:00Z"),
            ended_at: Some(at("2026-10-04T10:00:00Z")),
            actual_min: 50,
            todo_id: Some(todo.id),
            note: "n".to_owned(),
        };
        let updated = update_session(&conn, session.id, &edit, at("2026-10-04T12:00:00Z")).unwrap();
        assert_eq!(updated.started_at, at("2026-10-04T09:10:00Z"));
        assert_eq!(
            (updated.actual_min, updated.todo_id),
            (Some(50), Some(todo.id))
        );
        let wall = |field: &str| -> i64 {
            conn.query_row(
                "SELECT hlc_ms FROM session_field_clock WHERE session_id = ?1 AND field = ?2",
                params![session.id, field],
                |r| r.get(0),
            )
            .unwrap()
        };
        let edited = at("2026-10-04T12:00:00Z").timestamp_millis();
        let created = at("2026-10-04T10:00:00Z").timestamp_millis();
        assert_eq!(
            [
                "started_at",
                "actual_min",
                "todo",
                "note",
                "ended_at",
                "deleted"
            ]
            .map(wall),
            [edited, edited, edited, created, created, created]
        );
    }

    #[test]
    fn a_deleted_session_is_hidden_and_cannot_be_changed_again() {
        let conn = open_in_memory().unwrap();
        let entry = manual("m-1", "2026-10-04T09:00:00Z", "2026-10-04T10:00:00Z");
        let session = insert_manual(&conn, &entry, "d", at("2026-10-04T10:00:00Z")).unwrap();
        delete_session(&conn, session.id, at("2026-10-04T11:00:00Z")).unwrap();
        assert!(list_sessions(&conn).unwrap().is_empty());
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 1);
        let edit = SessionEdit {
            started_at: at("2026-10-04T09:00:00Z"),
            ended_at: None,
            actual_min: 1,
            todo_id: None,
            note: String::new(),
        };
        let now = at("2026-10-04T12:00:00Z");
        assert!(matches!(
            update_session(&conn, session.id, &edit, now),
            Err(StoreError::NotFound {
                entity: "Session",
                ..
            })
        ));
        assert!(matches!(
            delete_session(&conn, session.id, now),
            Err(StoreError::NotFound { .. })
        ));
        assert!(matches!(
            delete_session(&conn, 99, now),
            Err(StoreError::NotFound { id: 99, .. })
        ));
    }

    #[test]
    fn editing_a_session_without_metadata_assigns_its_id_first() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO sessions (mode, session_type, status, started_at, actual_min, sync_device_id)
             VALUES ('pomodoro','work','completed','2026-10-04T09:00:00Z', 25, 'dev-x')",
            [],
        )
        .unwrap();
        let edit = SessionEdit {
            started_at: at("2026-10-04T09:00:00Z"),
            ended_at: None,
            actual_min: 30,
            todo_id: None,
            note: String::new(),
        };
        update_session(&conn, 1, &edit, at("2026-10-04T12:00:00Z")).unwrap();
        let sync_id: String = conn
            .query_row("SELECT sync_id FROM session_meta", [], |r| r.get(0))
            .unwrap();
        assert!(sync_id.starts_with("legacy:dev-x|2026-10-04T09:00:00"));
    }
}
