//! todo.txt als Datei: Export aller Todos und Planung eines Imports (pur, ohne I/O).
//!
//! Das Dateiformat ist schlicht todo.txt, eine Zeile je Todo. Pomodoro-Zähler gibt es im Format nicht und
//! gehen beim Export verloren; Position folgt aus der Zeilenreihenfolge.

use std::collections::HashSet;

use crate::model::Todo;

/// Ergebnis der Planung eines Imports.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportPlan {
    /// Zeilen, die als neue Todos angelegt werden, in Dateireihenfolge.
    pub to_add: Vec<String>,
    /// Zeilen, die schon als Todo vorhanden sind oder mehrfach in der Datei stehen.
    pub duplicates: usize,
}

/// Die Datei zu den Todos: `raw_text` je Zeile in der manuellen Reihenfolge (`position`, dann `id`), mit
/// Zeilenende `\n` hinter jeder Zeile. Subtasks (`parent_id`) fehlen, todo.txt kennt keine Hierarchie. Leer, wenn es nichts zu schreiben gibt.
pub fn export_todotxt(todos: &[Todo], include_completed: bool) -> String {
    let mut ordered: Vec<&Todo> = todos
        .iter()
        .filter(|todo| include_completed || !todo.completed)
        .filter(|todo| !todo.raw_text.trim().is_empty())
        // Subtasks gibt es nur in nunc (ADR 0022)
        .filter(|todo| todo.parent_id.is_none())
        .collect();
    ordered.sort_by_key(|todo| (todo.position, todo.id));
    ordered
        .iter()
        .map(|todo| format!("{}\n", todo.raw_text.trim_end()))
        .collect()
}

/// Liest eine todo.txt-Datei und bestimmt, was davon neu ist. Leere Zeilen und Leerraum am Rand fallen weg,
/// ein Byte-Order-Mark am Dateianfang und `\r\n` sind erlaubt. Eine Zeile gilt als Dublette, wenn ein
/// vorhandenes Todo denselben Text hat oder sie weiter oben in der Datei schon stand.
pub fn plan_import(text: &str, existing: &[Todo]) -> ImportPlan {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut seen: HashSet<&str> = existing.iter().map(|todo| todo.raw_text.trim()).collect();
    let mut plan = ImportPlan::default();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if seen.insert(line) {
            plan.to_add.push(line.to_owned());
        } else {
            plan.duplicates += 1;
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todotxt::parse_line;

    fn todo(id: i32, line: &str, position: u32) -> Todo {
        let parsed = parse_line(line);
        Todo {
            id,
            raw_text: line.to_owned(),
            priority: parsed.priority,
            completed: parsed.completed,
            completed_at: parsed.completed_at,
            created_at: parsed.created_at,
            description: parsed.description,
            project_tags: parsed.project_tags,
            context_tags: parsed.context_tags,
            pomodoro_count: 0,
            position,
            parent_id: None,
        }
    }

    #[test]
    fn equal_positions_fall_back_to_the_id() {
        let todos = [todo(2, "b", 0), todo(1, "a", 0)];
        assert_eq!(export_todotxt(&todos, true), "a\nb\n");
    }

    #[test]
    fn trailing_whitespace_is_not_exported() {
        assert_eq!(export_todotxt(&[todo(1, "a  ", 0)], true), "a\n");
    }

    #[test]
    fn blank_todos_are_not_exported() {
        assert_eq!(export_todotxt(&[todo(1, "  ", 0)], true), "");
    }

    #[test]
    fn crlf_line_endings_are_trimmed() {
        let plan = plan_import("a\r\nb\r\n", &[]);
        assert_eq!(plan.to_add, ["a", "b"]);
    }

    #[test]
    fn existing_text_is_compared_trimmed() {
        let plan = plan_import("a\n", &[todo(1, "a  ", 0)]);
        assert!(plan.to_add.is_empty());
        assert_eq!(plan.duplicates, 1);
    }
}
