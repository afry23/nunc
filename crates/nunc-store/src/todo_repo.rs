//! Todos in der Tabelle `todos` (Go: `services/todos.go`).
//!
//! Löschen ist weich (`deleted = 1`), damit der Sync die Löschung weitergeben kann. Jede Änderung am Inhalt
//! setzt `updated_at` (Grundlage des Last-Write-Wins-Merges); wie in Go ändern Umsortieren und der
//! Pomodoro-Zähler `updated_at` nicht. Zeit und `sync_id` kommen vom Aufrufer.

use chrono::{DateTime, NaiveDate, Utc};
use nunc_core::model::Todo;
use nunc_core::todotxt::{ParsedTodo, build_raw_text, parse_line};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::StoreError;
use crate::time::{day_text, parse_day, to_text};

const COLUMNS: &str = "id, raw_text, priority, completed, completed_at, created_at, description, \
                       project_tags, context_tags, pomodoro_count, position";

fn todo_from_row(row: &Row<'_>) -> rusqlite::Result<Todo> {
    let priority: Option<String> = row.get(2)?;
    let completed_at: Option<String> = row.get(4)?;
    let created_at: Option<String> = row.get(5)?;
    let tags = |index: usize| -> rusqlite::Result<Vec<String>> {
        let json: String = row.get(index)?;
        // wie Go: unlesbare Tag-Listen werden zu leeren Listen
        Ok(serde_json::from_str(&json).unwrap_or_default())
    };
    Ok(Todo {
        id: row.get(0)?,
        raw_text: row.get(1)?,
        priority: priority.and_then(|p| p.chars().next()),
        completed: row.get(3)?,
        completed_at: completed_at.as_deref().and_then(parse_day),
        created_at: created_at.as_deref().and_then(parse_day),
        description: row.get(6)?,
        project_tags: tags(7)?,
        context_tags: tags(8)?,
        pomodoro_count: row.get::<_, i64>(9)?.try_into().unwrap_or(0),
        position: row.get::<_, i64>(10)?.try_into().unwrap_or(0),
    })
}

fn query(conn: &Connection, filter: &str) -> Result<Vec<Todo>, StoreError> {
    let sql = format!("SELECT {COLUMNS} FROM todos {filter} ORDER BY position, id");
    let mut stmt = conn.prepare(&sql)?;
    let todos = stmt
        .query_map([], todo_from_row)?
        .collect::<Result<_, _>>()?;
    Ok(todos)
}

/// Alle nicht gelöschten Todos nach Position.
pub fn list_todos(conn: &Connection) -> Result<Vec<Todo>, StoreError> {
    query(conn, "WHERE deleted = 0")
}

/// Alle Todos inklusive weich gelöschter (Statistik und Export lösen damit auch Sessions gelöschter Todos auf).
pub fn list_all_todos(conn: &Connection) -> Result<Vec<Todo>, StoreError> {
    query(conn, "")
}

/// Nicht gelöschtes Todo oder `NotFound`.
fn get(conn: &Connection, id: i32) -> Result<Todo, StoreError> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM todos WHERE id = ?1 AND deleted = 0"),
        [id],
        todo_from_row,
    )
    .optional()?
    .ok_or(StoreError::NotFound { entity: "Todo", id })
}

fn tags_json(tags: &[String]) -> Result<String, StoreError> {
    Ok(serde_json::to_string(tags)?)
}

/// Parst `raw_text` als todo.txt-Zeile und hängt das Todo hinten an (Position = größte vorhandene + 1,
/// weich gelöschte zählen mit, damit Positionen im Sync eindeutig bleiben).
pub fn add_todo(
    conn: &Connection,
    raw_text: &str,
    sync_id: &str,
    now: DateTime<Utc>,
) -> Result<Todo, StoreError> {
    let parsed = parse_line(raw_text);
    conn.execute(
        "INSERT INTO todos (raw_text, priority, completed, completed_at, created_at, description,
                            project_tags, context_tags, pomodoro_count, position, updated_at, sync_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0,
                 (SELECT COALESCE(MAX(position), -1) + 1 FROM todos), ?9, ?10)",
        params![
            raw_text,
            parsed.priority.map(String::from),
            parsed.completed,
            parsed.completed_at.map(day_text),
            parsed.created_at.map(day_text),
            parsed.description,
            tags_json(&parsed.project_tags)?,
            tags_json(&parsed.context_tags)?,
            to_text(now),
            sync_id,
        ],
    )?;
    let id = i32::try_from(conn.last_insert_rowid()).unwrap_or(i32::MAX);
    get(conn, id)
}

/// Ersetzt den Inhalt durch die neu geparste Zeile; Position, Pomodoro-Zähler und `sync_id` bleiben.
pub fn update_todo(
    conn: &Connection,
    id: i32,
    raw_text: &str,
    now: DateTime<Utc>,
) -> Result<Todo, StoreError> {
    get(conn, id)?;
    let parsed = parse_line(raw_text);
    conn.execute(
        "UPDATE todos SET raw_text = ?1, priority = ?2, completed = ?3, completed_at = ?4,
                created_at = ?5, description = ?6, project_tags = ?7, context_tags = ?8, updated_at = ?9
         WHERE id = ?10",
        params![
            raw_text,
            parsed.priority.map(String::from),
            parsed.completed,
            parsed.completed_at.map(day_text),
            parsed.created_at.map(day_text),
            parsed.description,
            tags_json(&parsed.project_tags)?,
            tags_json(&parsed.context_tags)?,
            to_text(now),
            id,
        ],
    )?;
    get(conn, id)
}

/// Löscht weich. Sessions behalten ihre Verknüpfung zum Todo.
pub fn delete_todo(conn: &Connection, id: i32, now: DateTime<Utc>) -> Result<(), StoreError> {
    let changed = conn.execute(
        "UPDATE todos SET deleted = 1, updated_at = ?1 WHERE id = ?2",
        params![to_text(now), id],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound { entity: "Todo", id });
    }
    Ok(())
}

/// Hakt ab (`today` wird Erledigungsdatum) oder nimmt den Haken zurück; `raw_text` wird neu aufgebaut.
/// Die Priorität bleibt in der Spalte erhalten und kommt beim Zurücknehmen wieder in die Zeile.
pub fn set_todo_completed(
    conn: &Connection,
    id: i32,
    completed: bool,
    today: NaiveDate,
    now: DateTime<Utc>,
) -> Result<Todo, StoreError> {
    let todo = get(conn, id)?;
    let completed_at = completed.then_some(today);
    let raw_text = build_raw_text(&ParsedTodo {
        priority: todo.priority,
        completed,
        completed_at,
        created_at: todo.created_at,
        description: todo.description,
        ..ParsedTodo::default()
    });
    conn.execute(
        "UPDATE todos SET raw_text = ?1, completed = ?2, completed_at = ?3, updated_at = ?4 WHERE id = ?5",
        params![raw_text, completed, completed_at.map(day_text), to_text(now), id],
    )?;
    get(conn, id)
}

/// Setzt die Position jedes Todos auf seinen Index in `ids` (atomar). Unbekannte IDs werden übergangen,
/// nicht genannte Todos behalten ihre Position.
pub fn reorder_todos(conn: &mut Connection, ids: &[i32]) -> Result<(), StoreError> {
    let tx = conn.transaction()?;
    for (position, id) in (0_i64..).zip(ids) {
        tx.execute(
            "UPDATE todos SET position = ?1 WHERE id = ?2",
            params![position, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Zählt einen abgeschlossenen Pomodoro für das Todo.
pub fn increment_pomodoro_count(conn: &Connection, id: i32) -> Result<(), StoreError> {
    let changed = conn.execute(
        "UPDATE todos SET pomodoro_count = pomodoro_count + 1 WHERE id = ?1",
        [id],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound { entity: "Todo", id });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-04T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 4).unwrap()
    }

    fn not_found(result: Result<impl std::fmt::Debug, StoreError>, id: i32) {
        match result {
            Err(StoreError::NotFound {
                entity: "Todo",
                id: got,
            }) => assert_eq!(got, id),
            other => panic!("erwartet NotFound, erhalten {other:?}"),
        }
    }

    #[test]
    fn missing_or_deleted_todos_are_not_found() {
        let mut conn = open_in_memory().unwrap();
        not_found(update_todo(&conn, 7, "x", now()), 7);
        not_found(set_todo_completed(&conn, 7, true, today(), now()), 7);
        not_found(delete_todo(&conn, 7, now()), 7);
        not_found(increment_pomodoro_count(&conn, 7), 7);
        let todo = add_todo(&conn, "a", "s1", now()).unwrap();
        delete_todo(&conn, todo.id, now()).unwrap();
        not_found(update_todo(&conn, todo.id, "b", now()), todo.id);
        not_found(
            set_todo_completed(&conn, todo.id, true, today(), now()),
            todo.id,
        );
        reorder_todos(&mut conn, &[todo.id]).unwrap();
    }

    #[test]
    fn writes_sync_fields() {
        let conn = open_in_memory().unwrap();
        let todo = add_todo(&conn, "task", "uuid-1", now()).unwrap();
        let (sync_id, updated_at, deleted): (String, String, i64) = conn
            .query_row(
                "SELECT sync_id, updated_at, deleted FROM todos WHERE id = ?1",
                [todo.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            (sync_id.as_str(), updated_at.as_str(), deleted),
            ("uuid-1", "2026-10-04T10:00:00Z", 0)
        );

        let later = now() + chrono::Duration::hours(1);
        update_todo(&conn, todo.id, "task 2", later).unwrap();
        delete_todo(&conn, todo.id, later + chrono::Duration::hours(1)).unwrap();
        let (sync_id, updated_at, deleted): (String, String, i64) = conn
            .query_row(
                "SELECT sync_id, updated_at, deleted FROM todos WHERE id = ?1",
                [todo.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            (sync_id.as_str(), updated_at.as_str(), deleted),
            ("uuid-1", "2026-10-04T12:00:00Z", 1)
        );
    }

    #[test]
    fn reorder_and_pomodoro_do_not_touch_updated_at() {
        let mut conn = open_in_memory().unwrap();
        let a = add_todo(&conn, "a", "s1", now()).unwrap();
        let b = add_todo(&conn, "b", "s2", now()).unwrap();
        reorder_todos(&mut conn, &[b.id, a.id]).unwrap();
        increment_pomodoro_count(&conn, a.id).unwrap();
        let stamps: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM todos WHERE updated_at = '2026-10-04T10:00:00Z'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stamps, 2);
    }

    #[test]
    fn unreadable_tags_become_empty_and_rfc3339_days_are_read() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO todos (raw_text, description, project_tags, context_tags, created_at, completed_at, position)
             VALUES ('t', 't', 'kaputt', '[\"@a\"]', '2026-09-18T00:00:00Z', 'gestern', 0)",
            [],
        )
        .unwrap();
        let todo = list_todos(&conn).unwrap().remove(0);
        assert!(todo.project_tags.is_empty());
        assert_eq!(todo.context_tags, ["@a"]);
        assert_eq!(todo.created_at, NaiveDate::from_ymd_opt(2026, 9, 18));
        assert_eq!(todo.completed_at, None);
    }

    #[test]
    fn completing_a_todo_without_priority_or_dates() {
        let conn = open_in_memory().unwrap();
        let todo = add_todo(&conn, "plain", "s1", now()).unwrap();
        let done = set_todo_completed(&conn, todo.id, true, today(), now()).unwrap();
        assert_eq!(done.raw_text, "x 2026-10-04 plain");
        let back = set_todo_completed(&conn, todo.id, false, today(), now()).unwrap();
        assert_eq!(back.raw_text, "plain");
        assert_eq!(back.completed_at, None);
    }

    #[test]
    fn completion_keeps_the_creation_date_in_the_line() {
        let conn = open_in_memory().unwrap();
        let todo = add_todo(&conn, "(B) 2026-09-20 Report +work", "s1", now()).unwrap();
        let done = set_todo_completed(&conn, todo.id, true, today(), now()).unwrap();
        assert_eq!(done.raw_text, "x 2026-10-04 2026-09-20 Report +work");
        assert_eq!(done.created_at, NaiveDate::from_ymd_opt(2026, 9, 20));
        let back = set_todo_completed(&conn, todo.id, false, today(), now()).unwrap();
        assert_eq!(back.raw_text, "(B) 2026-09-20 Report +work");
    }

    #[test]
    fn list_orders_by_position_then_id() {
        let mut conn = open_in_memory().unwrap();
        let ids: Vec<i32> = ["a", "b", "c"]
            .iter()
            .map(|t| add_todo(&conn, t, t, now()).unwrap().id)
            .collect();
        reorder_todos(&mut conn, &[ids[2], ids[1]]).unwrap();
        let order: Vec<i32> = list_todos(&conn).unwrap().iter().map(|t| t.id).collect();
        // c=0, b=1, a bleibt 0 -> a (id 1) vor c (id 3)
        assert_eq!(order, [ids[0], ids[2], ids[1]]);
    }
}
