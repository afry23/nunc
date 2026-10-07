//! Datenbankseite der Synchronisierung (Go: `services/sync.go`): lokalen Stand als [`SyncFile`] lesen,
//! fremde Änderungen einspielen und die WebDAV-Konfiguration ablegen. Das Zusammenführen selbst ist pur
//! (`nunc_core::sync_merge`).

use chrono::{DateTime, Utc};
use nunc_core::model::{SyncFile, SyncProvider, SyncSession, SyncTodo, WebDavAuth, WebDavConfig};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::StoreError;
use crate::settings_repo::set_setting;

const SCHEMA_VERSION: u32 = 1;

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

fn sync_todo(row: &Row<'_>, device_id: &str) -> rusqlite::Result<SyncTodo> {
    let tags = |index: usize| -> rusqlite::Result<Vec<String>> {
        let json: String = row.get(index)?;
        Ok(serde_json::from_str(&json).unwrap_or_default())
    };
    Ok(SyncTodo {
        sync_id: row.get(0)?,
        device_id: device_id.to_owned(),
        raw_text: row.get(1)?,
        priority: non_empty(row.get(2)?),
        completed: row.get(3)?,
        completed_at: non_empty(row.get(4)?),
        created_at: non_empty(row.get(5)?),
        description: row.get(6)?,
        project_tags: tags(7)?,
        context_tags: tags(8)?,
        pomodoro_count: row.get::<_, i64>(9)?.try_into().unwrap_or(0),
        position: row.get::<_, i64>(10)?.try_into().unwrap_or(0),
        updated_at: row.get(11)?,
        deleted: row.get(12)?,
    })
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

/// Der lokale Stand für den Austausch: alle Todos (auch gelöschte) und alle abgeschlossenen Sessions.
/// `new_sync_id` vergibt Kennungen an Todos, die noch keine haben.
pub fn build_sync_file(
    conn: &Connection,
    device_id: &str,
    now: DateTime<Utc>,
    new_sync_id: impl FnMut() -> String,
) -> Result<SyncFile, StoreError> {
    assign_missing_sync_ids(conn, new_sync_id)?;
    let todos = conn
        .prepare(
            "SELECT sync_id, raw_text, priority, completed, completed_at, created_at, description,
                    project_tags, context_tags, pomodoro_count, position, updated_at, deleted
             FROM todos ORDER BY id",
        )?
        .query_map([], |row| sync_todo(row, device_id))?
        .collect::<Result<_, _>>()?;
    let sessions = conn
        .prepare(
            "SELECT COALESCE(sync_device_id, ''), mode, session_type, status, started_at,
                    COALESCE(ended_at, ''), COALESCE(planned_min, 0), COALESCE(actual_min, 0),
                    COALESCE(break_min, 0)
             FROM sessions WHERE status = 'completed' ORDER BY started_at, id",
        )?
        .query_map([], |row| sync_session(row, device_id))?
        .collect::<Result<_, _>>()?;
    Ok(SyncFile {
        schema_version: SCHEMA_VERSION,
        synced_at: now,
        device_id: device_id.to_owned(),
        todos,
        sessions,
    })
}

/// Was [`apply_remote`] in der Datenbank verändert hat.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Applied {
    /// Neu angelegte Todos (bereits gelöschte Todos des Servers werden nicht angelegt).
    pub todos_added: u32,
    /// Vorhandene Todos, die durch die Server-Version ersetzt oder gelöscht wurden.
    pub todos_replaced: u32,
    /// Neu angefügte Sessions (bekannte werden übersprungen).
    pub sessions_added: u32,
}

/// Spielt Änderungen vom Server in **einer Transaktion** ein: neue Todos werden angehängt (außer bereits
/// gelöschte), vorhandene durch die übergebene (neuere) Version ersetzt, neue Sessions angefügt.
pub fn apply_remote(
    conn: &mut Connection,
    todos: &[SyncTodo],
    sessions: &[SyncSession],
) -> Result<Applied, StoreError> {
    let mut applied = Applied::default();
    if todos.is_empty() && sessions.is_empty() {
        return Ok(applied);
    }
    let tx = conn.transaction()?;
    for todo in todos {
        let local: Option<i64> = tx
            .query_row(
                "SELECT id FROM todos WHERE sync_id = ?1",
                [&todo.sync_id],
                |row| row.get(0),
            )
            .optional()?;
        let project_tags = serde_json::to_string(&todo.project_tags)?;
        let context_tags = serde_json::to_string(&todo.context_tags)?;
        match local {
            None if todo.deleted => {}
            None => {
                applied.todos_added += 1;
                tx.execute(
                    "INSERT INTO todos (raw_text, priority, completed, completed_at, created_at, description,
                                        project_tags, context_tags, pomodoro_count, position, updated_at,
                                        deleted, sync_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                             (SELECT COALESCE(MAX(position), -1) + 1 FROM todos), ?10, 0, ?11)",
                    params![
                        todo.raw_text,
                        todo.priority,
                        todo.completed,
                        todo.completed_at,
                        todo.created_at,
                        todo.description,
                        project_tags,
                        context_tags,
                        todo.pomodoro_count,
                        todo.updated_at,
                        todo.sync_id,
                    ],
                )?;
            }
            Some(id) if todo.deleted => {
                applied.todos_replaced += 1;
                tx.execute(
                    "UPDATE todos SET deleted = 1, updated_at = ?1 WHERE id = ?2",
                    params![todo.updated_at, id],
                )?;
            }
            Some(id) => {
                applied.todos_replaced += 1;
                tx.execute(
                    "UPDATE todos SET raw_text = ?1, priority = ?2, completed = ?3, completed_at = ?4,
                            created_at = ?5, description = ?6, project_tags = ?7, context_tags = ?8,
                            pomodoro_count = ?9, updated_at = ?10, deleted = 0
                     WHERE id = ?11",
                    params![
                        todo.raw_text,
                        todo.priority,
                        todo.completed,
                        todo.completed_at,
                        todo.created_at,
                        todo.description,
                        project_tags,
                        context_tags,
                        todo.pomodoro_count,
                        todo.updated_at,
                        id,
                    ],
                )?;
            }
        }
    }
    for session in sessions {
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
    tx.commit()?;
    Ok(applied)
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

/// Schlüssel der Go-App für Google Drive.
const DRIVE_LAST_SYNC_KEY: &str = "last_synced_at";
const DRIVE_FOLDER_KEY: &str = "drive_folder_id";
const TODOIST_LAST_SYNC_KEY: &str = "todoist.last_sync_at";

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
        SyncProvider::GoogleDrive => time_setting(conn, DRIVE_LAST_SYNC_KEY),
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
        SyncProvider::GoogleDrive => DRIVE_LAST_SYNC_KEY,
    };
    set_setting(conn, key, &crate::time::to_text(at))
}

/// Kennung des Drive-Ordners `nunc` (für den Link „In Drive öffnen“).
pub fn drive_folder_id(conn: &Connection) -> Result<Option<String>, StoreError> {
    time_setting(conn, DRIVE_FOLDER_KEY)
}

pub fn set_drive_folder_id(conn: &Connection, id: &str) -> Result<(), StoreError> {
    set_setting(conn, DRIVE_FOLDER_KEY, id)
}

/// Zeitpunkt des letzten erfolgreichen Todoist-Syncs.
pub fn todoist_last_sync_at(conn: &Connection) -> Result<Option<String>, StoreError> {
    time_setting(conn, TODOIST_LAST_SYNC_KEY)
}

pub fn set_todoist_last_sync_at(conn: &Connection, at: DateTime<Utc>) -> Result<(), StoreError> {
    set_setting(conn, TODOIST_LAST_SYNC_KEY, &crate::time::to_text(at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{add_todo, delete_todo, list_all_todos, list_sessions, open_in_memory};

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

    fn remote_todo(sync_id: &str, text: &str, updated: &str, deleted: bool) -> SyncTodo {
        SyncTodo {
            sync_id: sync_id.into(),
            device_id: "other".into(),
            raw_text: text.into(),
            priority: Some("A".into()),
            completed: false,
            completed_at: None,
            created_at: Some("2026-09-01".into()),
            description: text.into(),
            project_tags: vec!["+p".into()],
            context_tags: vec![],
            pomodoro_count: 2,
            position: 0,
            updated_at: updated.into(),
            deleted,
        }
    }

    fn remote_session(start: &str, device: &str) -> SyncSession {
        SyncSession {
            device_id: device.into(),
            mode: "pomodoro".into(),
            session_type: "work".into(),
            status: "completed".into(),
            started_at: start.into(),
            ended_at: Some("2026-10-01T09:25:00Z".into()),
            planned_min: 25,
            actual_min: 25,
            break_min: 0,
        }
    }

    #[test]
    fn the_local_file_contains_deleted_todos_completed_sessions_and_assigns_missing_ids() {
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

        let file = build_sync_file(&conn, "dev-1", at("2026-10-02T00:00:00Z"), counter()).unwrap();
        assert_eq!((file.schema_version, file.device_id.as_str()), (1, "dev-1"));
        assert_eq!(file.todos.len(), 2);
        assert_eq!(file.todos[0].sync_id, "gen-1");
        assert_eq!(file.todos[0].priority.as_deref(), Some("B"));
        assert_eq!(file.todos[0].project_tags, ["+x"]);
        assert_eq!(file.todos[0].device_id, "dev-1");
        assert!(!file.todos[0].deleted && file.todos[1].deleted);
        assert_eq!(file.todos[1].updated_at, "2026-10-01T09:00:00Z");
        // nur abgeschlossene Sessions, ohne Herkunftsgerät gilt das eigene
        assert_eq!(file.sessions.len(), 1);
        assert_eq!(file.sessions[0].device_id, "dev-1");
        assert_eq!(file.sessions[0].planned_min, 25);
        // die vergebene Kennung ist gespeichert
        let stored: String = conn
            .query_row("SELECT sync_id FROM todos WHERE id = ?1", [kept.id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(stored, "gen-1");
    }

    #[test]
    fn applying_adds_replaces_and_deletes_todos_in_one_go() {
        let mut conn = open_in_memory().unwrap();
        add_todo(&conn, "alt", "s-1", at("2026-10-01T08:00:00Z")).unwrap();
        let doomed = add_todo(&conn, "löschen", "s-2", at("2026-10-01T08:00:00Z")).unwrap();
        let applied = apply_remote(
            &mut conn,
            &[
                remote_todo("s-1", "neu vom Server", "2026-10-01T10:00:00Z", false),
                remote_todo("s-2", "löschen", "2026-10-01T11:00:00Z", true),
                remote_todo("s-3", "ganz neu", "2026-10-01T10:00:00Z", false),
                remote_todo("s-4", "schon gelöscht", "2026-10-01T10:00:00Z", true),
            ],
            &[],
        )
        .unwrap();
        assert_eq!(
            applied,
            Applied {
                todos_added: 1,
                todos_replaced: 2,
                sessions_added: 0
            }
        );

        let all = list_all_todos(&conn).unwrap();
        let texts: Vec<_> = all.iter().map(|t| t.raw_text.as_str()).collect();
        assert_eq!(texts, ["neu vom Server", "löschen", "ganz neu"]);
        assert_eq!(all[0].priority, Some('A'));
        assert_eq!(all[0].pomodoro_count, 2);
        // angehängt hinter der größten vorhandenen Position
        assert_eq!(all[2].position, 2);
        let (deleted, updated): (bool, String) = conn
            .query_row(
                "SELECT deleted, updated_at FROM todos WHERE id = ?1",
                [doomed.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(deleted);
        assert_eq!(updated, "2026-10-01T11:00:00Z");
        // eine neuere Version ohne Löschkennzeichen belebt das Todo wieder
        apply_remote(
            &mut conn,
            &[remote_todo("s-2", "zurück", "2026-10-01T12:00:00Z", false)],
            &[],
        )
        .unwrap();
        assert_eq!(crate::list_todos(&conn).unwrap().len(), 3);
    }

    #[test]
    fn applying_sessions_skips_known_ones_and_keeps_the_origin() {
        let mut conn = open_in_memory().unwrap();
        let session = remote_session("2026-10-01T09:00:00Z", "other");
        let first = apply_remote(&mut conn, &[], &[session.clone(), session.clone()]).unwrap();
        assert_eq!(first.sessions_added, 1);
        assert_eq!(
            apply_remote(&mut conn, &[], &[session])
                .unwrap()
                .sessions_added,
            0
        );
        let mut zero = remote_session("2026-10-01T10:00:00Z", "other");
        zero.planned_min = 0;
        zero.ended_at = None;
        apply_remote(&mut conn, &[], &[zero]).unwrap();

        let sessions = list_sessions(&conn).unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].planned_min, Some(25));
        assert_eq!(sessions[1].planned_min, None);
        assert_eq!(sessions[1].ended_at, None);
        let origin: String = conn
            .query_row("SELECT sync_device_id FROM sessions LIMIT 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(origin, "other");
    }

    #[test]
    fn nothing_to_apply_touches_nothing() {
        let mut conn = open_in_memory().unwrap();
        assert_eq!(
            apply_remote(&mut conn, &[], &[]).unwrap(),
            Applied::default()
        );
        assert!(list_all_todos(&conn).unwrap().is_empty());
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
    fn each_provider_keeps_its_own_last_sync_time() {
        let conn = open_in_memory().unwrap();
        let first = at("2026-10-04T09:30:00Z");
        let second = at("2026-10-04T11:00:00Z");
        set_last_sync_of(&conn, SyncProvider::Webdav, first).unwrap();
        set_last_sync_of(&conn, SyncProvider::GoogleDrive, second).unwrap();
        set_last_sync_of(&conn, SyncProvider::None, second).unwrap();
        assert_eq!(
            last_sync_of(&conn, SyncProvider::Webdav)
                .unwrap()
                .as_deref(),
            Some("2026-10-04T09:30:00Z")
        );
        assert_eq!(
            last_sync_of(&conn, SyncProvider::GoogleDrive)
                .unwrap()
                .as_deref(),
            Some("2026-10-04T11:00:00Z")
        );
        assert_eq!(last_sync_of(&conn, SyncProvider::None).unwrap(), None);
    }

    #[test]
    fn the_drive_folder_and_the_todoist_time_are_kept() {
        let conn = open_in_memory().unwrap();
        assert_eq!(drive_folder_id(&conn).unwrap(), None);
        set_drive_folder_id(&conn, "folder-1").unwrap();
        assert_eq!(drive_folder_id(&conn).unwrap().as_deref(), Some("folder-1"));
        assert_eq!(todoist_last_sync_at(&conn).unwrap(), None);
        set_todoist_last_sync_at(&conn, at("2026-10-04T09:30:00Z")).unwrap();
        assert_eq!(
            todoist_last_sync_at(&conn).unwrap().as_deref(),
            Some("2026-10-04T09:30:00Z")
        );
    }
}
