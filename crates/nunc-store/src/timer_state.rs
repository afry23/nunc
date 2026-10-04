//! Persistierter Timer-Zustand (`active_timer`, Migration 006).

use chrono::{DateTime, SecondsFormat, Utc};
use nunc_core::timer::Timer;
use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;

/// Speichert den kompletten Timer-Zustand (laufende Session, Zyklus, Break-Vorschlag).
/// Nach jeder Transition aufrufen. Ein Timer im Ausgangszustand löscht die Zeile.
pub fn save_timer(conn: &Connection, timer: &Timer, now: DateTime<Utc>) -> Result<(), StoreError> {
    if *timer == Timer::default() {
        conn.execute("DELETE FROM active_timer", [])?;
        return Ok(());
    }
    conn.execute(
        "INSERT INTO active_timer (id, state, updated_at) VALUES (1, ?1, ?2)
         ON CONFLICT(id) DO UPDATE SET state = excluded.state, updated_at = excluded.updated_at",
        params![
            serde_json::to_string(timer)?,
            now.to_rfc3339_opts(SecondsFormat::Secs, true)
        ],
    )?;
    Ok(())
}

/// Lädt den gespeicherten Timer; ohne Eintrag den Ruhezustand.
pub fn load_timer(conn: &Connection) -> Result<Timer, StoreError> {
    let json: Option<String> = conn
        .query_row("SELECT state FROM active_timer WHERE id = 1", [], |r| {
            r.get(0)
        })
        .optional()?;
    match json {
        Some(json) => Ok(serde_json::from_str(&json)?),
        None => Ok(Timer::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;
    use nunc_core::model::{Settings, TimerMode, TimerState};

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn rows(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM active_timer", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn empty_table_loads_idle() {
        let conn = open_in_memory().unwrap();
        assert_eq!(load_timer(&conn).unwrap(), Timer::default());
    }

    #[test]
    fn save_overwrites_single_row_and_roundtrips() {
        let conn = open_in_memory().unwrap();
        let t0 = at("2026-10-04T09:00:00Z");
        let mut timer = Timer::default();
        timer
            .start(t0, TimerMode::Pomodoro, Some(4), &Settings::default())
            .unwrap();
        save_timer(&conn, &timer, t0).unwrap();
        timer.pause(at("2026-10-04T09:10:00Z")).unwrap();
        save_timer(&conn, &timer, at("2026-10-04T09:10:00Z")).unwrap();
        assert_eq!(rows(&conn), 1);
        let loaded = load_timer(&conn).unwrap();
        assert_eq!(loaded, timer);
        assert_eq!(loaded.state(), TimerState::Paused);
        let updated: String = conn
            .query_row("SELECT updated_at FROM active_timer", [], |r| r.get(0))
            .unwrap();
        assert_eq!(updated, "2026-10-04T09:10:00Z");
    }

    #[test]
    fn default_timer_removes_the_row() {
        let conn = open_in_memory().unwrap();
        let t0 = at("2026-10-04T09:00:00Z");
        let mut timer = Timer::default();
        timer
            .start(t0, TimerMode::Flowtime, None, &Settings::default())
            .unwrap();
        save_timer(&conn, &timer, t0).unwrap();
        save_timer(&conn, &Timer::default(), t0).unwrap();
        assert_eq!(rows(&conn), 0);
    }

    #[test]
    fn corrupt_state_is_an_error_not_a_panic() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO active_timer (id, state, updated_at) VALUES (1, 'not json', '')",
            [],
        )
        .unwrap();
        assert!(matches!(load_timer(&conn), Err(StoreError::Json(_))));
    }
}
