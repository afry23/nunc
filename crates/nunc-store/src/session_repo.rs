//! Sessions in der Tabelle `sessions` (append-only). Die Go-App legt eine Session schon beim Start an;
//! nunc schreibt sie erst beim Ende, der laufende Zustand liegt in `active_timer`.

use chrono::{DateTime, Utc};
use nunc_core::model::Session;
use nunc_core::timer::{Finish, Timer};
use rusqlite::{Connection, Row, params};

use crate::time::{parse_instant, to_text};
use crate::{StoreError, increment_pomodoro_count, save_timer};

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

/// Alle Sessions nach Startzeit (Rohdaten für `nunc_core::stats` und den Export, M2-05). Nichts wird vorab
/// aggregiert; der Zeitraum und die lokale Zeitzone sind Sache der Fachlogik.
pub fn list_sessions(conn: &Connection) -> Result<Vec<Session>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, mode, session_type, status, started_at, ended_at, planned_min, actual_min,
                todo_id, break_min
         FROM sessions ORDER BY started_at, id",
    )?;
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
    let session = tx.query_row(
        "SELECT id, mode, session_type, status, started_at, ended_at, planned_min, actual_min,
                todo_id, break_min FROM sessions WHERE id = ?1",
        [id],
        session_from_row,
    )?;
    tx.commit()?;
    Ok(session)
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
}
