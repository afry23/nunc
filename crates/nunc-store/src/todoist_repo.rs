//! Datenbankseite des Todoist-Syncs (Go: `services/todoist/sync.go`): lokale Todos samt Änderungszeit lesen,
//! die Zuordnung Todo ↔ Aufgabe (`todoist_mappings`) pflegen und Änderungen aus Todoist in **einer Transaktion**
//! einspielen. Die Regeln selbst sind pur (`nunc_core::todoist_mapping`).

use chrono::{DateTime, NaiveDate, Utc};
use nunc_core::model::Todo;
use nunc_core::todoist_mapping::{LocalTodo, TodoistMapping};
use nunc_core::todotxt::{ParsedTodo, build_raw_text};
use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;
use crate::time::{day_text, parse_day, parse_instant, to_text};
use crate::todo_repo::set_todo_completed;

const COLUMNS: &str = "id, raw_text, priority, completed, completed_at, created_at, description, \
                       project_tags, context_tags, pomodoro_count, position, updated_at, deleted";

/// Alle Todos – auch gelöschte – mit Änderungszeit, in Datenbankreihenfolge.
pub fn list_local_todos(conn: &Connection) -> Result<Vec<LocalTodo>, StoreError> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM todos ORDER BY id"))?;
    let todos = stmt
        .query_map([], |row| {
            let tags = |index: usize| -> rusqlite::Result<Vec<String>> {
                let json: String = row.get(index)?;
                Ok(serde_json::from_str(&json).unwrap_or_default())
            };
            let priority: Option<String> = row.get(2)?;
            let completed_at: Option<String> = row.get(4)?;
            let created_at: Option<String> = row.get(5)?;
            let updated_at: Option<String> = row.get(11)?;
            Ok(LocalTodo {
                todo: Todo {
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
                },
                updated_at: updated_at.as_deref().and_then(parse_instant),
                deleted: row.get(12)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(todos)
}

pub fn list_todoist_mappings(conn: &Connection) -> Result<Vec<TodoistMapping>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT nunc_todo_id, todoist_task_id FROM todoist_mappings ORDER BY nunc_todo_id",
    )?;
    let mappings = stmt
        .query_map([], |row| {
            Ok(TodoistMapping {
                nunc_todo_id: row.get(0)?,
                todoist_task_id: row.get(1)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(mappings)
}

/// Verknüpft ein Todo mit einer Aufgabe (ersetzt eine vorhandene Verknüpfung des Todos).
pub fn add_todoist_mapping(
    conn: &Connection,
    todo_id: i32,
    task_id: &str,
    at: DateTime<Utc>,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR REPLACE INTO todoist_mappings (nunc_todo_id, todoist_task_id, last_synced_at)
         VALUES (?1, ?2, ?3)",
        params![todo_id, task_id, to_text(at)],
    )?;
    Ok(())
}

/// Löst die Verknüpfungen zu den genannten Aufgaben.
pub fn remove_todoist_mappings(conn: &Connection, task_ids: &[String]) -> Result<(), StoreError> {
    for task_id in task_ids {
        conn.execute(
            "DELETE FROM todoist_mappings WHERE todoist_task_id = ?1",
            [task_id],
        )?;
    }
    Ok(())
}

/// Eine Änderung aus Todoist am lokalen Bestand. `at` ist die Änderungszeit der Aufgabe: Das Todo trägt sie
/// danach als `updated_at`, damit der nächste Abgleich Gleichstand sieht und nichts zurückschiebt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodoistWrite {
    /// Neue Aufgabe als Todo anlegen und verknüpfen.
    Insert {
        task_id: String,
        todo: ParsedTodo,
        at: DateTime<Utc>,
        sync_id: String,
    },
    /// Vorhandenes Todo mit der Todoist-Version überschreiben.
    Update {
        todo_id: i32,
        todo: ParsedTodo,
        at: DateTime<Utc>,
    },
    /// Todoist und das Todo sagen dasselbe (nur die Schreibweise der Zeile weicht ab, etwa das Erstellungsdatum):
    /// nur die Änderungszeit angleichen, damit der nächste Abgleich Gleichstand sieht. Zählt nicht als Änderung.
    Touch { todo_id: i32, at: DateTime<Utc> },
    /// Die Aufgabe ist in Todoist verschwunden: Todo als erledigt markieren (ein erledigtes bleibt unberührt).
    Complete { todo_id: i32 },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TodoistApplied {
    pub inserted: u32,
    pub updated: u32,
    pub completed: u32,
}

fn tags_json(tags: &[String]) -> Result<String, StoreError> {
    Ok(serde_json::to_string(tags)?)
}

/// Spielt die Änderungen in einer Transaktion ein.
pub fn apply_todoist_writes(
    conn: &mut Connection,
    writes: &[TodoistWrite],
    today: NaiveDate,
    now: DateTime<Utc>,
) -> Result<TodoistApplied, StoreError> {
    let mut applied = TodoistApplied::default();
    if writes.is_empty() {
        return Ok(applied);
    }
    let tx = conn.transaction()?;
    for write in writes {
        match write {
            TodoistWrite::Insert {
                task_id,
                todo,
                at,
                sync_id,
            } => {
                tx.execute(
                    "INSERT INTO todos (raw_text, priority, completed, completed_at, created_at, description,
                                        project_tags, context_tags, pomodoro_count, position, updated_at, deleted, sync_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0,
                             (SELECT COALESCE(MAX(position), -1) + 1 FROM todos), ?9, 0, ?10)",
                    params![
                        build_raw_text(todo),
                        todo.priority.map(String::from),
                        todo.completed,
                        todo.completed_at.map(day_text),
                        todo.created_at.map(day_text),
                        todo.description,
                        tags_json(&todo.project_tags)?,
                        tags_json(&todo.context_tags)?,
                        to_text(*at),
                        sync_id,
                    ],
                )?;
                let id = i32::try_from(tx.last_insert_rowid()).unwrap_or(i32::MAX);
                add_todoist_mapping(&tx, id, task_id, now)?;
                applied.inserted += 1;
            }
            TodoistWrite::Update { todo_id, todo, at } => {
                let changed = tx.execute(
                    "UPDATE todos SET raw_text = ?1, priority = ?2, completed = ?3, completed_at = ?4,
                            description = ?5, project_tags = ?6, context_tags = ?7, updated_at = ?8, deleted = 0
                     WHERE id = ?9",
                    params![
                        build_raw_text(todo),
                        todo.priority.map(String::from),
                        todo.completed,
                        todo.completed_at.map(day_text),
                        todo.description,
                        tags_json(&todo.project_tags)?,
                        tags_json(&todo.context_tags)?,
                        to_text(*at),
                        todo_id,
                    ],
                )?;
                applied.updated += u32::try_from(changed).unwrap_or(0);
            }
            TodoistWrite::Touch { todo_id, at } => {
                tx.execute(
                    "UPDATE todos SET updated_at = ?1 WHERE id = ?2 AND COALESCE(updated_at, '') != ?1",
                    params![to_text(*at), todo_id],
                )?;
            }
            TodoistWrite::Complete { todo_id } => {
                let open: Option<bool> = tx
                    .query_row(
                        "SELECT completed FROM todos WHERE id = ?1 AND deleted = 0",
                        [todo_id],
                        |row| row.get(0),
                    )
                    .optional()?;
                if open == Some(false) {
                    set_todo_completed(&tx, *todo_id, true, today, now)?;
                    applied.completed += 1;
                }
            }
        }
    }
    tx.commit()?;
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use nunc_core::todotxt::parse_line;

    use super::*;
    use crate::{add_todo, delete_todo, list_todos, open_in_memory};

    fn at(text: &str) -> DateTime<Utc> {
        parse_instant(text).unwrap()
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 4).unwrap()
    }

    #[test]
    fn local_todos_include_deleted_ones_with_their_change_time() {
        let conn = open_in_memory().unwrap();
        let kept = add_todo(&conn, "(A) Keep +p @c", "s-1", at("2026-10-04T08:00:00Z")).unwrap();
        let gone = add_todo(&conn, "Gone", "s-2", at("2026-10-04T08:30:00Z")).unwrap();
        delete_todo(&conn, gone.id, at("2026-10-04T09:00:00Z")).unwrap();
        let all = list_local_todos(&conn).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].todo, kept);
        assert_eq!(all[0].updated_at, Some(at("2026-10-04T08:00:00Z")));
        assert!(!all[0].deleted);
        assert!(all[1].deleted);
        assert_eq!(all[1].updated_at, Some(at("2026-10-04T09:00:00Z")));
    }

    #[test]
    fn mappings_are_added_replaced_and_removed() {
        let conn = open_in_memory().unwrap();
        let a = add_todo(&conn, "A", "s-1", at("2026-10-04T08:00:00Z")).unwrap();
        let b = add_todo(&conn, "B", "s-2", at("2026-10-04T08:00:00Z")).unwrap();
        add_todoist_mapping(&conn, a.id, "t-1", at("2026-10-04T09:00:00Z")).unwrap();
        add_todoist_mapping(&conn, b.id, "t-2", at("2026-10-04T09:00:00Z")).unwrap();
        add_todoist_mapping(&conn, a.id, "t-9", at("2026-10-04T09:30:00Z")).unwrap();
        let ids: Vec<_> = list_todoist_mappings(&conn)
            .unwrap()
            .into_iter()
            .map(|m| (m.nunc_todo_id, m.todoist_task_id))
            .collect();
        assert_eq!(ids, [(a.id, "t-9".to_owned()), (b.id, "t-2".to_owned())]);
        remove_todoist_mappings(&conn, &["t-9".to_owned(), "unbekannt".to_owned()]).unwrap();
        assert_eq!(list_todoist_mappings(&conn).unwrap().len(), 1);
    }

    #[test]
    fn nothing_to_write_touches_nothing() {
        let mut conn = open_in_memory().unwrap();
        let applied =
            apply_todoist_writes(&mut conn, &[], today(), at("2026-10-04T10:00:00Z")).unwrap();
        assert_eq!(applied, TodoistApplied::default());
    }

    #[test]
    fn an_inserted_task_becomes_a_mapped_todo_at_the_end_with_the_task_time() {
        let mut conn = open_in_memory().unwrap();
        add_todo(&conn, "Erstes", "s-1", at("2026-10-04T08:00:00Z")).unwrap();
        let write = TodoistWrite::Insert {
            task_id: "t-1".into(),
            todo: parse_line("(B) Neu +Work @desk"),
            at: at("2026-10-03T12:00:00Z"),
            sync_id: "sync-new".into(),
        };
        let applied =
            apply_todoist_writes(&mut conn, &[write], today(), at("2026-10-04T10:00:00Z")).unwrap();
        assert_eq!(applied.inserted, 1);
        let todos = list_local_todos(&conn).unwrap();
        let new = &todos[1];
        assert_eq!(new.todo.raw_text, "(B) Neu +Work @desk");
        assert_eq!(new.todo.position, 1);
        assert_eq!(new.todo.project_tags, ["+Work"]);
        assert_eq!(new.updated_at, Some(at("2026-10-03T12:00:00Z")));
        let sync_id: String = conn
            .query_row(
                "SELECT sync_id FROM todos WHERE id = ?1",
                [new.todo.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(sync_id, "sync-new");
        assert_eq!(
            list_todoist_mappings(&conn).unwrap(),
            [TodoistMapping {
                nunc_todo_id: new.todo.id,
                todoist_task_id: "t-1".into()
            }]
        );
    }

    #[test]
    fn an_update_overwrites_the_content_but_keeps_position_and_counter() {
        let mut conn = open_in_memory().unwrap();
        let todo = add_todo(&conn, "Alt", "s-1", at("2026-10-04T08:00:00Z")).unwrap();
        crate::increment_pomodoro_count(&conn, todo.id).unwrap();
        let write = TodoistWrite::Update {
            todo_id: todo.id,
            todo: parse_line("(A) Neu @desk"),
            at: at("2026-10-04T09:00:00Z"),
        };
        let applied =
            apply_todoist_writes(&mut conn, &[write], today(), at("2026-10-04T10:00:00Z")).unwrap();
        assert_eq!(applied.updated, 1);
        let after = &list_local_todos(&conn).unwrap()[0];
        assert_eq!(after.todo.raw_text, "(A) Neu @desk");
        assert_eq!(after.todo.priority, Some('A'));
        assert_eq!(after.todo.pomodoro_count, 1);
        assert_eq!(after.updated_at, Some(at("2026-10-04T09:00:00Z")));
    }

    #[test]
    fn a_touch_only_aligns_the_time_and_is_not_counted() {
        let mut conn = open_in_memory().unwrap();
        let todo = add_todo(&conn, "(A) Gleich +p", "s-1", at("2026-10-04T08:00:00Z")).unwrap();
        let touch = TodoistWrite::Touch {
            todo_id: todo.id,
            at: at("2026-10-04T09:00:00Z"),
        };
        let applied =
            apply_todoist_writes(&mut conn, &[touch], today(), at("2026-10-04T10:00:00Z")).unwrap();
        assert_eq!(applied, TodoistApplied::default());
        let after = &list_local_todos(&conn).unwrap()[0];
        assert_eq!(after.updated_at, Some(at("2026-10-04T09:00:00Z")));
        assert_eq!(after.todo.raw_text, "(A) Gleich +p");
    }

    #[test]
    fn an_update_of_an_unknown_todo_counts_for_nothing() {
        let mut conn = open_in_memory().unwrap();
        let write = TodoistWrite::Update {
            todo_id: 99,
            todo: parse_line("X"),
            at: at("2026-10-04T09:00:00Z"),
        };
        let applied =
            apply_todoist_writes(&mut conn, &[write], today(), at("2026-10-04T10:00:00Z")).unwrap();
        assert_eq!(applied.updated, 0);
    }

    #[test]
    fn completing_marks_open_todos_once_and_leaves_done_and_deleted_ones() {
        let mut conn = open_in_memory().unwrap();
        let open = add_todo(&conn, "Offen", "s-1", at("2026-10-04T08:00:00Z")).unwrap();
        let done = add_todo(
            &conn,
            "x 2026-10-01 Fertig",
            "s-2",
            at("2026-10-04T08:00:00Z"),
        )
        .unwrap();
        let gone = add_todo(&conn, "Weg", "s-3", at("2026-10-04T08:00:00Z")).unwrap();
        delete_todo(&conn, gone.id, at("2026-10-04T08:30:00Z")).unwrap();
        let writes: Vec<_> = [open.id, done.id, gone.id, 99]
            .into_iter()
            .map(|todo_id| TodoistWrite::Complete { todo_id })
            .collect();
        let applied =
            apply_todoist_writes(&mut conn, &writes, today(), at("2026-10-04T10:00:00Z")).unwrap();
        assert_eq!(applied.completed, 1);
        let todos = list_todos(&conn).unwrap();
        assert_eq!(todos[0].raw_text, "x 2026-10-04 Offen");
        assert_eq!(todos[1].raw_text, "x 2026-10-01 Fertig");
    }
}
