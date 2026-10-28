//! Todoist ↔ todo.txt: Feld-Mapping und Last-Write-Wins-Merge (pur, `services/todoist/mapping.go` der Go-App).
//!
//! Nicht in diesem Modul: HTTP, Authentifizierung, die Mapping-Tabelle und das Anlegen fehlender Projekte oder
//! Labels in Todoist (`nunc-sync`, M7). Die Typen hier sind die fachliche Sicht auf eine Todoist-Aufgabe; der
//! Client wandelt die API-Antwort hinein (Welche API-Version, klärt M7-04).
//!
//! Abweichungen von der Go-App:
//! - Importierte Todos sind **kanonische todo.txt-Zeilen**: Die Beschreibung enthält wie beim Parser die Tags, die
//!   Zeile baut der Aufrufer mit `todotxt::build_raw_text` (erledigt: `x Erledigt Erstellt Text`, ohne Priorität).
//! - Der Export (`content`) enthält die Beschreibung **ohne** `+`/`@`-Tags (sie werden Projekt und Labels), Go
//!   schickte die Beschreibung samt Tags.
//! - HTML-Auszeichnung wird aus dem Inhalt entfernt (Story US-Todoist-Sync, Go tat das nicht).
//! - Der heutige Tag für das Erledigungsdatum wird übergeben statt aus der Systemzeit gelesen.
//! - Lokal erledigte Todos mit Mapping schließen die Aufgabe in Todoist; erledigte Todos ohne Mapping werden nicht
//!   neu angelegt.

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};

use crate::model::Todo;
use crate::todotxt::ParsedTodo;

/// Eine Todoist-Aufgabe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoistTask {
    pub id: String,
    pub content: String,
    /// 1–4, 4 = höchste (`p1` in der Oberfläche).
    pub priority: u8,
    pub project_id: String,
    pub labels: Vec<String>,
    pub is_completed: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Ein lokales Todo mit Änderungszeit und Löschkennzeichen (aus der Datenbank).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTodo {
    pub todo: Todo,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted: bool,
}

/// Zuordnung eines lokalen Todos zu einer Todoist-Aufgabe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoistMapping {
    pub nunc_todo_id: i32,
    pub todoist_task_id: String,
}

/// Inhalt einer Anlege-/Änderungsanfrage an Todoist. Die Projekt-ID löst der Aufrufer über den Namen auf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoistTaskRequest {
    pub content: String,
    /// 1–4.
    pub priority: u8,
    /// Name des Projekts (erster `+`-Tag) oder `None`.
    pub project_name: Option<String>,
    /// Weitere `+`-Tags und alle `@`-Tags als Label-Namen.
    pub labels: Vec<String>,
}

/// Lokale Änderung aus dem Merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalChange {
    /// Neue Aufgabe aus Todoist als Todo anlegen und mit `todoist_task_id` verknüpfen.
    Insert {
        todoist_task_id: String,
        todo: ParsedTodo,
    },
    /// Bestehendes Todo mit der Todoist-Version überschreiben.
    Update { todo_id: i32, todo: ParsedTodo },
}

/// Ergebnis des Merges; der Sync-Dienst führt die Teile getrennt aus.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TodoistMerge {
    pub to_create_in_todoist: Vec<Todo>,
    pub to_update_in_todoist: Vec<Todo>,
    /// Aufgaben-IDs, die in Todoist geschlossen (nicht gelöscht) werden.
    pub to_close_in_todoist: Vec<String>,
    pub local_changes: Vec<LocalChange>,
    /// Lokale Todos, deren Aufgabe in Todoist verschwunden ist: lokal als erledigt markieren.
    pub to_complete_locally: Vec<i32>,
}

/// Todoist-Priorität (4 = höchste) → todo.txt-Priorität; `1` und Unbekanntes ergeben keine.
fn priority_from_todoist(priority: u8) -> Option<char> {
    match priority {
        4 => Some('A'),
        3 => Some('B'),
        2 => Some('C'),
        _ => None,
    }
}

/// todo.txt-Priorität → Todoist-Priorität; ohne oder jenseits von `C` ist es `1`.
fn priority_to_todoist(priority: Option<char>) -> u8 {
    match priority {
        Some('A') => 4,
        Some('B') => 3,
        Some('C') => 2,
        _ => 1,
    }
}

/// Leerraum in Namen wird zu `_`, damit der Name ein einzelnes todo.txt-Wort bleibt.
fn name_to_tag_word(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_whitespace() { '_' } else { c })
        .collect()
}

fn tag_word_to_name(word: &str) -> String {
    word[1..].replace('_', " ")
}

/// Entfernt HTML-Auszeichnung (`<b>`, `</p>`, `<br/>`, …) und löst die gängigen Zeichenverweise auf.
/// Ein `<` beginnt nur dann ein Tag, wenn ein Buchstabe, `/` oder `!` folgt; `1 < 2` und `a > b` bleiben Text.
fn strip_html(content: &str) -> String {
    let mut text = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_tag = false;
    while let Some(c) = chars.next() {
        if in_tag {
            in_tag = c != '>';
        } else if c == '<'
            && chars
                .peek()
                .is_some_and(|n| n.is_ascii_alphabetic() || matches!(n, '/' | '!'))
        {
            in_tag = true;
        } else {
            text.push(c);
        }
    }
    text.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// Wandelt eine Todoist-Aufgabe in ein Todo (Todoist → nunc).
///
/// `project_name` ist der aufgelöste Projektname; leer und `Inbox` (ohne Beachtung der Großschreibung) ergeben
/// kein Projekt. `label_names` sind die Label-Namen ohne Präfix. `today` ist das Erledigungsdatum erledigter
/// Aufgaben.
pub fn map_task_to_todo(
    task: &TodoistTask,
    project_name: &str,
    label_names: &[String],
    today: NaiveDate,
) -> ParsedTodo {
    let project_tags: Vec<String> = Some(project_name)
        .filter(|name| !name.is_empty() && !name.eq_ignore_ascii_case("inbox"))
        .map(|name| format!("+{}", name_to_tag_word(name)))
        .into_iter()
        .collect();
    let context_tags: Vec<String> = label_names
        .iter()
        .filter(|label| !label.is_empty())
        .map(|label| format!("@{}", name_to_tag_word(label)))
        .collect();

    let mut words = vec![strip_html(&task.content).trim().to_owned()];
    words.extend(project_tags.iter().cloned());
    words.extend(context_tags.iter().cloned());
    let description = words.join(" ");

    let completed = task.is_completed;
    ParsedTodo {
        priority: if completed {
            None
        } else {
            priority_from_todoist(task.priority)
        },
        completed,
        completed_at: completed.then_some(today),
        created_at: task.created_at.map(|created| created.date_naive()),
        description,
        project_tags,
        context_tags,
    }
}

/// Wandelt ein Todo in eine Todoist-Anfrage (nunc → Todoist).
///
/// Der erste `+`-Tag ist das Projekt, alle weiteren `+`-Tags und alle `@`-Tags werden Labels. Die Beschreibung
/// geht ohne ihre Tag-Wörter als Inhalt hinaus.
pub fn map_todo_to_task(todo: &Todo) -> TodoistTaskRequest {
    let content = todo
        .description
        .split_whitespace()
        .filter(|word| !is_tag_word(word))
        .collect::<Vec<_>>()
        .join(" ");
    let labels = todo
        .project_tags
        .iter()
        .skip(1)
        .chain(&todo.context_tags)
        .map(|tag| tag_word_to_name(tag))
        .collect();
    TodoistTaskRequest {
        content,
        priority: priority_to_todoist(todo.priority),
        project_name: todo.project_tags.first().map(|tag| tag_word_to_name(tag)),
        labels,
    }
}

fn is_tag_word(word: &str) -> bool {
    word.len() > 1 && (word.starts_with('+') || word.starts_with('@'))
}

/// Sagen beide Todos für Todoist dasselbe? Verglichen wird, was Todoist kennt: Inhalt, Priorität, Projekt,
/// Labels (ohne Reihenfolge, Groß-/Kleinschreibung egal) und ob erledigt. Die Schreibweise der Zeile (etwa das
/// Erstellungsdatum, das Todoist liefert) spielt keine Rolle.
pub fn same_on_todoist(a: &Todo, b: &Todo) -> bool {
    let fold = |todo: &Todo| {
        let request = map_todo_to_task(todo);
        let mut labels: Vec<String> = request.labels.iter().map(|l| l.to_lowercase()).collect();
        labels.sort();
        (
            request.content,
            request.priority,
            request.project_name.map(|name| name.to_lowercase()),
            labels,
            todo.completed,
        )
    };
    fold(a) == fold(b)
}

/// Last-Write-Wins zwischen lokalen Todos und Todoist-Aufgaben.
///
/// - lokal gelöscht + Mapping → in Todoist schließen
/// - lokal neu (kein Mapping) → in Todoist anlegen (erledigte nicht)
/// - beide vorhanden, lokal **strikt** neuer → nach Todoist übertragen (erledigt: Aufgabe schließen)
/// - beide vorhanden, Todoist neuer oder gleich (Gleichstand: Todoist) → lokal überschreiben
/// - Aufgabe in Todoist verschwunden (Mapping vorhanden) → lokal erledigen
/// - Aufgabe nur in Todoist (kein Mapping) → lokal anlegen, erledigte werden nicht importiert
///
/// Der Zeitpunkt der Todoist-Seite ist `updated_at`, ersatzweise `created_at`.
pub fn merge_todoist(
    locals: &[LocalTodo],
    remotes: &[TodoistTask],
    mappings: &[TodoistMapping],
    projects_by_id: &HashMap<String, String>,
    today: NaiveDate,
) -> TodoistMerge {
    let mapping_by_todo: HashMap<i32, &TodoistMapping> =
        mappings.iter().map(|m| (m.nunc_todo_id, m)).collect();
    let mapped_task_ids: std::collections::HashSet<&str> = mappings
        .iter()
        .map(|m| m.todoist_task_id.as_str())
        .collect();
    let remote_by_id: HashMap<&str, &TodoistTask> = remotes
        .iter()
        .map(|task| (task.id.as_str(), task))
        .collect();

    let mut merge = TodoistMerge::default();
    let map_remote = |task: &TodoistTask| {
        let project = projects_by_id
            .get(&task.project_id)
            .map_or("", String::as_str);
        map_task_to_todo(task, project, &task.labels, today)
    };

    for local in locals {
        let mapping = mapping_by_todo.get(&local.todo.id);

        if local.deleted {
            if let Some(mapping) = mapping {
                merge
                    .to_close_in_todoist
                    .push(mapping.todoist_task_id.clone());
            }
            continue;
        }
        let Some(mapping) = mapping else {
            if !local.todo.completed {
                merge.to_create_in_todoist.push(local.todo.clone());
            }
            continue;
        };
        let Some(remote) = remote_by_id.get(mapping.todoist_task_id.as_str()) else {
            merge.to_complete_locally.push(local.todo.id);
            continue;
        };

        let remote_time = remote.updated_at.or(remote.created_at);
        if local.updated_at > remote_time {
            if local.todo.completed {
                merge
                    .to_close_in_todoist
                    .push(mapping.todoist_task_id.clone());
            } else {
                merge.to_update_in_todoist.push(local.todo.clone());
            }
        } else {
            merge.local_changes.push(LocalChange::Update {
                todo_id: local.todo.id,
                todo: map_remote(remote),
            });
        }
    }

    for task in remotes {
        if task.is_completed || mapped_task_ids.contains(task.id.as_str()) {
            continue;
        }
        merge.local_changes.push(LocalChange::Insert {
            todoist_task_id: task.id.clone(),
            todo: map_remote(task),
        });
    }
    merge
}

#[cfg(test)]
mod same_on_todoist_tests {
    use super::*;
    use crate::todotxt::parse_line;

    fn todo(raw: &str) -> Todo {
        let parsed = parse_line(raw);
        Todo {
            id: 1,
            raw_text: raw.to_owned(),
            priority: parsed.priority,
            completed: parsed.completed,
            completed_at: parsed.completed_at,
            created_at: parsed.created_at,
            description: parsed.description,
            project_tags: parsed.project_tags,
            context_tags: parsed.context_tags,
            pomodoro_count: 0,
            position: 0,
            parent_id: None,
        }
    }

    #[test]
    fn the_creation_date_label_order_and_case_do_not_matter() {
        let local = todo("(A) Report +Work @desk @new");
        assert!(same_on_todoist(
            &local,
            &todo("(A) 2026-10-04 Report +work @NEW @desk")
        ));
    }

    #[test]
    fn every_field_todoist_knows_makes_a_difference() {
        let base = todo("(A) Report +Work @desk");
        for other in [
            "(A) Other +Work @desk",
            "(B) Report +Work @desk",
            "(A) Report +Home @desk",
            "(A) Report +Work @desk @more",
            "(A) Report +Work",
            "x 2026-10-04 Report +Work @desk",
        ] {
            assert!(!same_on_todoist(&base, &todo(other)), "{other}");
        }
    }
}
