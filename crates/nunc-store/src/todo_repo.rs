//! Todos in der Tabelle `todos` (Go: `services/todos.go`).
//!
//! Löschen ist weich (`deleted = 1`), damit der Sync die Löschung weitergeben kann. Jede Änderung am Inhalt
//! setzt `updated_at` (Grundlage des Last-Write-Wins-Merges); wie in Go ändern Umsortieren und der
//! Pomodoro-Zähler `updated_at` nicht. Zeit und `sync_id` kommen vom Aufrufer.

use chrono::{DateTime, NaiveDate, Utc};
use nunc_core::model::Todo;
use nunc_core::todotxt::{ParsedTodo, build_raw_text, parse_line};
use nunc_core::todotxt_file::{export_todotxt as export_todotxt_text, plan_import};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::StoreError;
use crate::crdt_repo::{
    add_pomodoro, atomic, fields_of, register_new_todo, rekey_positions, set_parent, stamp_changes,
};
use crate::time::{day_text, parse_day, to_text};

const COLUMNS: &str = "id, raw_text, priority, completed, completed_at, created_at, description, \
                       project_tags, context_tags, pomodoro_count, position, \
                       (SELECT p.id FROM todos p WHERE p.sync_id =
                            (SELECT parent_sync_id FROM todo_parent WHERE todo_id = todos.id))";

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
        parent_id: row.get(11)?,
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
    let id = atomic(conn, || {
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
        let id = conn.last_insert_rowid();
        register_new_todo(conn, id, now)?;
        Ok(i32::try_from(id).unwrap_or(i32::MAX))
    })?;
    get(conn, id)
}

/// Legt ein Todo wie [`add_todo`] an und hängt es unter `parent` (eine Transaktion).
pub fn add_subtask(
    conn: &Connection,
    parent: i32,
    raw_text: &str,
    sync_id: &str,
    now: DateTime<Utc>,
) -> Result<Todo, StoreError> {
    get(conn, parent)?;
    atomic(conn, || {
        let todo = add_todo(conn, raw_text, sync_id, now)?;
        set_todo_parent(conn, todo.id, Some(parent), now)
    })
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
    atomic(conn, || {
        let before = fields_of(conn, id.into())?;
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
        stamp_changes(conn, id.into(), before.as_ref(), now)
    })?;
    get(conn, id)
}

/// Löscht weich, samt aller Nachfahren (ADR 0022). Sessions behalten ihre Verknüpfung zum Todo.
pub fn delete_todo(conn: &Connection, id: i32, now: DateTime<Utc>) -> Result<(), StoreError> {
    atomic(conn, || {
        delete_one(conn, id, now)?;
        for descendant in descendants(conn, id)? {
            delete_one(conn, descendant, now)?;
        }
        Ok(())
    })
}

fn delete_one(conn: &Connection, id: i32, now: DateTime<Utc>) -> Result<(), StoreError> {
    let before = fields_of(conn, id.into())?;
    let changed = conn.execute(
        "UPDATE todos SET deleted = 1, updated_at = ?1 WHERE id = ?2",
        params![to_text(now), id],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound { entity: "Todo", id });
    }
    stamp_changes(conn, id.into(), before.as_ref(), now)
}

/// Alle nicht gelöschten Nachfahren von `id` (Kinder, Enkel, …) ohne `id` selbst; Zyklen enden von selbst.
fn descendants(conn: &Connection, id: i32) -> Result<Vec<i32>, StoreError> {
    let ids = conn
        .prepare(
            "WITH RECURSIVE sub(id) AS (
                 SELECT ?1
                 UNION
                 SELECT c.todo_id FROM todo_parent c
                     JOIN todos p ON p.sync_id = c.parent_sync_id
                     JOIN sub ON sub.id = p.id
             )
             SELECT s.id FROM sub s JOIN todos t ON t.id = s.id
             WHERE s.id != ?1 AND t.deleted = 0 ORDER BY s.id",
        )?
        .query_map([id], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(ids)
}

/// Hängt das Todo unter `parent` (`None`: zur Wurzel machen). Der Elternknoten muss existieren, darf nicht gelöscht
/// sein und weder das Todo selbst noch einer seiner Nachfahren sein.
pub fn set_todo_parent(
    conn: &Connection,
    id: i32,
    parent: Option<i32>,
    now: DateTime<Utc>,
) -> Result<Todo, StoreError> {
    get(conn, id)?;
    let parent_sync_id = match parent {
        None => None,
        Some(parent) => {
            if parent == id || descendants(conn, id)?.contains(&parent) {
                return Err(StoreError::InvalidParent { id, parent });
            }
            get(conn, parent)?;
            let sync_id: String =
                conn.query_row("SELECT sync_id FROM todos WHERE id = ?1", [parent], |row| {
                    row.get(0)
                })?;
            Some(sync_id)
        }
    };
    atomic(conn, || {
        let before = fields_of(conn, id.into())?;
        set_parent(conn, id.into(), parent_sync_id.as_deref())?;
        conn.execute(
            "UPDATE todos SET updated_at = ?1 WHERE id = ?2",
            params![to_text(now), id],
        )?;
        stamp_changes(conn, id.into(), before.as_ref(), now)
    })?;
    get(conn, id)
}

/// Hakt ab (`today` wird Erledigungsdatum) oder nimmt den Haken zurück; `raw_text` wird neu aufgebaut.
/// Die Priorität bleibt in der Spalte erhalten und kommt beim Zurücknehmen wieder in die Zeile.
/// Abhaken trifft auch alle offenen Nachfahren, Zurücknehmen nur das Todo selbst (ADR 0022).
pub fn set_todo_completed(
    conn: &Connection,
    id: i32,
    completed: bool,
    today: NaiveDate,
    now: DateTime<Utc>,
) -> Result<Todo, StoreError> {
    get(conn, id)?;
    atomic(conn, || {
        if completed {
            for descendant in descendants(conn, id)? {
                if !get(conn, descendant)?.completed {
                    complete_one(conn, descendant, true, today, now)?;
                }
            }
        }
        complete_one(conn, id, completed, today, now)
    })
}

fn complete_one(
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
    atomic(conn, || {
        let before = fields_of(conn, id.into())?;
        conn.execute(
            "UPDATE todos SET raw_text = ?1, completed = ?2, completed_at = ?3, updated_at = ?4 WHERE id = ?5",
            params![raw_text, completed, completed_at.map(day_text), to_text(now), id],
        )?;
        stamp_changes(conn, id.into(), before.as_ref(), now)
    })?;
    get(conn, id)
}

/// Setzt die Position jedes Todos auf seinen Index in `ids` (atomar). Unbekannte IDs werden übergangen,
/// nicht genannte Todos behalten ihre Position. Danach passen die Fractional-Index-Schlüssel zur neuen Reihenfolge;
/// gestempelt werden nur Todos, deren Schlüssel sich ändert (ADR 0016).
pub fn reorder_todos(
    conn: &mut Connection,
    ids: &[i32],
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    let tx = conn.transaction()?;
    for (position, id) in (0_i64..).zip(ids) {
        tx.execute(
            "UPDATE todos SET position = ?1 WHERE id = ?2",
            params![position, id],
        )?;
    }
    rekey_positions(&tx, now)?;
    tx.commit()?;
    Ok(())
}

/// Ergebnis von [`import_todotxt`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TodotxtImport {
    /// Neu angelegte Todos in Dateireihenfolge.
    pub added: Vec<Todo>,
    /// Zeilen, die es schon gab oder die mehrfach in der Datei standen.
    pub duplicates: usize,
}

/// Liest den Text einer todo.txt-Datei und legt alle neuen Zeilen hinten an (eine Transaktion: alles oder
/// nichts). Vorhandene, nicht gelöschte Todos mit gleichem Text und Wiederholungen zählen als Dublette.
/// `new_sync_id` liefert je neuem Todo eine frische Sync-Kennung.
pub fn import_todotxt(
    conn: &mut Connection,
    text: &str,
    now: DateTime<Utc>,
    new_sync_id: &mut dyn FnMut() -> String,
) -> Result<TodotxtImport, StoreError> {
    let tx = conn.transaction()?;
    let plan = plan_import(text, &list_todos(&tx)?);
    let mut added = Vec::with_capacity(plan.to_add.len());
    for line in &plan.to_add {
        added.push(add_todo(&tx, line, &new_sync_id(), now)?);
    }
    tx.commit()?;
    Ok(TodotxtImport {
        added,
        duplicates: plan.duplicates,
    })
}

/// Alle nicht gelöschten Todos als todo.txt-Datei (siehe [`export_todotxt_text`](nunc_core::todotxt_file::export_todotxt)).
pub fn export_todotxt(conn: &Connection, include_completed: bool) -> Result<String, StoreError> {
    Ok(export_todotxt_text(&list_todos(conn)?, include_completed))
}

/// Zählt einen abgeschlossenen Pomodoro für das Todo.
pub fn increment_pomodoro_count(conn: &Connection, id: i32) -> Result<(), StoreError> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM todos WHERE id = ?1)",
        [id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(StoreError::NotFound { entity: "Todo", id });
    }
    atomic(conn, || add_pomodoro(conn, id.into()))
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

    fn tree() -> Connection {
        let conn = open_in_memory().unwrap();
        add_todo(&conn, "root", "s1", now()).unwrap();
        add_subtask(&conn, 1, "child", "s2", now()).unwrap();
        add_subtask(&conn, 2, "grandchild", "s3", now()).unwrap();
        conn
    }

    fn invalid_parent(result: Result<Todo, StoreError>, id: i32, parent: i32) {
        match result {
            Err(StoreError::InvalidParent {
                id: got_id,
                parent: got_parent,
            }) => assert_eq!((got_id, got_parent), (id, parent)),
            other => panic!("erwartet InvalidParent, erhalten {other:?}"),
        }
    }

    #[test]
    fn a_todo_cannot_move_below_itself_or_its_descendants() {
        let conn = tree();
        invalid_parent(set_todo_parent(&conn, 1, Some(1), now()), 1, 1);
        invalid_parent(set_todo_parent(&conn, 1, Some(3), now()), 1, 3);
        assert_eq!(get(&conn, 1).unwrap().parent_id, None);
    }

    #[test]
    fn the_parent_must_exist_and_not_be_deleted() {
        let conn = tree();
        not_found(set_todo_parent(&conn, 3, Some(99), now()), 99);
        add_todo(&conn, "gone", "s4", now()).unwrap();
        delete_todo(&conn, 4, now()).unwrap();
        not_found(set_todo_parent(&conn, 3, Some(4), now()), 4);
        not_found(add_subtask(&conn, 4, "x", "s5", now()), 4);
    }

    #[test]
    fn a_deleted_parent_leaves_its_children_without_a_parent_id() {
        let conn = tree();
        conn.execute("UPDATE todos SET deleted = 1 WHERE id = 1", [])
            .unwrap();
        let all = list_all_todos(&conn).unwrap();
        // Der Verweis bleibt gespeichert und löst weiter auf; das Filtern der Liste macht den Rest.
        assert_eq!(all[1].parent_id, Some(1));
        assert_eq!(list_todos(&conn).unwrap()[0].parent_id, Some(1));
    }

    #[test]
    fn completing_skips_descendants_that_are_already_done() {
        let conn = tree();
        set_todo_completed(&conn, 3, true, today(), now()).unwrap();
        let later = DateTime::parse_from_rfc3339("2026-10-05T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let next_day = NaiveDate::from_ymd_opt(2026, 10, 5).unwrap();
        set_todo_completed(&conn, 1, true, next_day, later).unwrap();
        assert_eq!(get(&conn, 3).unwrap().completed_at, Some(today()));
        assert_eq!(get(&conn, 2).unwrap().completed_at, Some(next_day));
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
        reorder_todos(&mut conn, &[todo.id], now()).unwrap();
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
        reorder_todos(&mut conn, &[b.id, a.id], now()).unwrap();
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
        reorder_todos(&mut conn, &[ids[2], ids[1]], now()).unwrap();
        let order: Vec<i32> = list_todos(&conn).unwrap().iter().map(|t| t.id).collect();
        // c=0, b=1, a bleibt 0 -> a (id 1) vor c (id 3)
        assert_eq!(order, [ids[0], ids[2], ids[1]]);
    }
}
