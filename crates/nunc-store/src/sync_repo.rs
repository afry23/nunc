//! Einstellungen der Synchronisierung: WebDAV-Konfiguration, Zeitpunkte der letzten Syncs und die Hilfen für den
//! CRDT-Stand (`crdt_repo`). Das Zusammenführen selbst ist pur (`nunc_core::crdt`).

use chrono::{DateTime, Utc};
use nunc_core::model::{SyncProvider, SyncSession, WebDavAuth, WebDavConfig};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::StoreError;
use crate::settings_repo::set_setting;

/// Hat jedes Todo eine `sync_id`? Wie in Go bekommen ältere Todos erst beim Sync eine.
pub(crate) fn assign_missing_sync_ids(
    conn: &Connection,
    mut new_id: impl FnMut() -> String,
) -> Result<(), StoreError> {
    let ids: Vec<i64> = conn
        .prepare("SELECT id FROM todos WHERE sync_id IS NULL OR sync_id = ''")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    for id in ids {
        conn.execute(
            "UPDATE todos SET sync_id = ?1 WHERE id = ?2",
            params![new_id(), id],
        )?;
    }
    Ok(())
}

fn non_empty(text: Option<String>) -> Option<String> {
    text.filter(|t| !t.is_empty())
}

pub(crate) fn sync_session(row: &Row<'_>, device_id: &str) -> rusqlite::Result<SyncSession> {
    let minutes = |index: usize| -> rusqlite::Result<u32> {
        Ok(row.get::<_, i64>(index)?.try_into().unwrap_or(0))
    };
    let origin: String = row.get(0)?;
    Ok(SyncSession {
        device_id: if origin.is_empty() {
            device_id.to_owned()
        } else {
            origin
        },
        mode: row.get(1)?,
        session_type: row.get(2)?,
        status: row.get(3)?,
        started_at: row.get(4)?,
        ended_at: non_empty(row.get(5)?),
        planned_min: minutes(6)?,
        actual_min: minutes(7)?,
        break_min: minutes(8)?,
    })
}

/// Was [`apply_state`](crate::apply_state) in der Datenbank verändert hat.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Applied {
    /// Neu angelegte Todos (bereits gelöschte Todos des Servers werden nicht angelegt).
    pub todos_added: u32,
    /// Vorhandene Todos, die durch die Server-Version ersetzt oder gelöscht wurden.
    pub todos_replaced: u32,
    /// Neu angefügte Sessions.
    pub sessions_added: u32,
    /// Vorhandene Sessions, deren Felder durch den zusammengeführten Stand ersetzt wurden (Bearbeitung, Löschen).
    pub sessions_replaced: u32,
}

const URL_KEY: &str = "sync.webdav.url";
const AUTH_KEY: &str = "sync.webdav.auth_method";
const USER_KEY: &str = "sync.webdav.username";
const PATH_KEY: &str = "sync.webdav.remote_path";
const LAST_SYNC_KEY: &str = "sync.webdav.last_sync_at";

fn setting(conn: &Connection, key: &str) -> Result<Option<String>, StoreError> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()?)
}

/// Die vier WebDAV-Schlüssel der Go-App; fehlende oder leere Werte ergeben die Standardwerte
/// (`basic`, `/nunc/`).
pub fn load_webdav_config(conn: &Connection) -> Result<WebDavConfig, StoreError> {
    let defaults = WebDavConfig::default();
    let text = |key: &str| setting(conn, key).map(Option::unwrap_or_default);
    let auth = text(AUTH_KEY)?;
    let remote_path = text(PATH_KEY)?;
    Ok(WebDavConfig {
        url: text(URL_KEY)?,
        auth_method: auth.parse::<WebDavAuth>().unwrap_or(defaults.auth_method),
        username: text(USER_KEY)?,
        remote_path: if remote_path.is_empty() {
            defaults.remote_path
        } else {
            remote_path
        },
    })
}

/// Schreibt die vier Schlüssel in einer Transaktion. Das Passwort steht nie in der Datenbank.
pub fn save_webdav_config(conn: &mut Connection, config: &WebDavConfig) -> Result<(), StoreError> {
    let tx = conn.transaction()?;
    set_setting(&tx, URL_KEY, &config.url)?;
    set_setting(&tx, AUTH_KEY, config.auth_method.as_str())?;
    set_setting(&tx, USER_KEY, &config.username)?;
    set_setting(&tx, PATH_KEY, &config.remote_path)?;
    tx.commit()?;
    Ok(())
}

/// Zeitpunkt des letzten erfolgreichen WebDAV-Syncs (RFC 3339), falls es einen gab.
pub fn last_sync_at(conn: &Connection) -> Result<Option<String>, StoreError> {
    Ok(setting(conn, LAST_SYNC_KEY)?.filter(|text| !text.is_empty()))
}

pub fn set_last_sync_at(conn: &Connection, at: DateTime<Utc>) -> Result<(), StoreError> {
    set_setting(conn, LAST_SYNC_KEY, &crate::time::to_text(at))
}

fn time_setting(conn: &Connection, key: &str) -> Result<Option<String>, StoreError> {
    Ok(setting(conn, key)?.filter(|text| !text.is_empty()))
}

/// Zeitpunkt des letzten erfolgreichen Syncs des Anbieters; `None` für „kein Sync“.
pub fn last_sync_of(
    conn: &Connection,
    provider: SyncProvider,
) -> Result<Option<String>, StoreError> {
    match provider {
        SyncProvider::None => Ok(None),
        SyncProvider::Webdav => time_setting(conn, LAST_SYNC_KEY),
    }
}

/// Merkt sich den Zeitpunkt des letzten erfolgreichen Syncs für den Anbieter.
pub fn set_last_sync_of(
    conn: &Connection,
    provider: SyncProvider,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    let key = match provider {
        SyncProvider::None => return Ok(()),
        SyncProvider::Webdav => LAST_SYNC_KEY,
    };
    set_setting(conn, key, &crate::time::to_text(at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{add_todo, delete_todo, open_in_memory};

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn counter() -> impl FnMut() -> String {
        let mut n = 0;
        move || {
            n += 1;
            format!("gen-{n}")
        }
    }

    #[test]
    fn the_local_state_contains_deleted_todos_completed_sessions_and_assigns_missing_ids() {
        let conn = open_in_memory().unwrap();
        let kept = add_todo(&conn, "(B) bleibt +x", "s-1", at("2026-10-01T08:00:00Z")).unwrap();
        let gone = add_todo(&conn, "weg", "s-2", at("2026-10-01T08:00:00Z")).unwrap();
        delete_todo(&conn, gone.id, at("2026-10-01T09:00:00Z")).unwrap();
        conn.execute("UPDATE todos SET sync_id = NULL WHERE id = ?1", [kept.id])
            .unwrap();
        conn.execute(
            "INSERT INTO sessions (mode, session_type, status, started_at, ended_at, planned_min, actual_min)
             VALUES ('pomodoro','work','completed','2026-10-01T08:00:00Z','2026-10-01T08:25:00Z',25,25),
                    ('pomodoro','work','aborted','2026-10-01T09:00:00Z',NULL,25,3)",
            [],
        )
        .unwrap();

        let state = crate::local_state(&conn, counter()).unwrap();
        assert_eq!(state.todos.len(), 2);
        let first = &state.todos["gen-1"];
        assert_eq!(first.priority.value.as_deref(), Some("B"));
        assert_eq!(first.project_tags(), ["+x"]);
        assert!(!first.is_deleted() && state.todos["s-2"].is_deleted());
        // nur abgeschlossene Sessions, ohne Herkunftsgerät gilt das eigene
        assert_eq!(state.sessions.len(), 1);
        let session = state.sessions.values().next().unwrap();
        assert_eq!(session.device_id, "local");
        assert_eq!(session.planned_min, 25);
        // die vergebene Kennung ist gespeichert
        let stored: String = conn
            .query_row("SELECT sync_id FROM todos WHERE id = ?1", [kept.id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(stored, "gen-1");
    }

    #[test]
    fn the_webdav_config_roundtrips_and_falls_back_to_defaults() {
        let mut conn = open_in_memory().unwrap();
        assert_eq!(load_webdav_config(&conn).unwrap(), WebDavConfig::default());
        let config = WebDavConfig {
            url: "https://cloud.example.org/dav".into(),
            auth_method: WebDavAuth::Digest,
            username: "me".into(),
            remote_path: "/apps/nunc/".into(),
        };
        save_webdav_config(&mut conn, &config).unwrap();
        assert_eq!(load_webdav_config(&conn).unwrap(), config);
        // unlesbare Werte fallen zurück
        conn.execute(
            "UPDATE settings SET value = 'kerberos' WHERE key = 'sync.webdav.auth_method'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE settings SET value = '' WHERE key = 'sync.webdav.remote_path'",
            [],
        )
        .unwrap();
        let loaded = load_webdav_config(&conn).unwrap();
        assert_eq!(loaded.auth_method, WebDavAuth::Basic);
        assert_eq!(loaded.remote_path, "/nunc/");
    }

    #[test]
    fn the_last_sync_time_is_kept() {
        let conn = open_in_memory().unwrap();
        assert_eq!(last_sync_at(&conn).unwrap(), None);
        set_last_sync_at(&conn, at("2026-10-04T09:30:00Z")).unwrap();
        assert_eq!(
            last_sync_at(&conn).unwrap().as_deref(),
            Some("2026-10-04T09:30:00Z")
        );
    }
    #[test]
    fn the_last_sync_time_of_the_provider_is_kept() {
        let conn = open_in_memory().unwrap();
        let first = at("2026-10-04T09:30:00Z");
        let second = at("2026-10-04T11:00:00Z");
        set_last_sync_of(&conn, SyncProvider::Webdav, first).unwrap();
        set_last_sync_of(&conn, SyncProvider::None, second).unwrap();
        assert_eq!(
            last_sync_of(&conn, SyncProvider::Webdav)
                .unwrap()
                .as_deref(),
            Some("2026-10-04T09:30:00Z")
        );
        assert_eq!(last_sync_of(&conn, SyncProvider::None).unwrap(), None);
    }
}
