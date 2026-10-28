//! Filtern und Sortieren der Todo-Liste (pur, ohne Zugriff auf Datenbank oder Uhr).
//!
//! Verhalten der Go-App (`TodoList.svelte`): innerhalb einer Kategorie genügt **ein** Treffer (ODER), über
//! Kategorien hinweg müssen **alle** zutreffen (UND). Erweitert um Priorität, Freitext, Erledigt-Status
//! und mehrere Sortierungen (REQUIREMENTS 4.2).

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::Todo;

/// Filter für die Todo-Liste. Leere Felder filtern nicht.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct TodoFilter {
    /// Projekt-Tags mit Präfix (`+work`); ein Todo muss mindestens einen davon tragen.
    pub projects: Vec<String>,
    /// Kontext-Tags mit Präfix (`@desk`); ein Todo muss mindestens einen davon tragen.
    pub contexts: Vec<String>,
    /// Prioritäten `A`–`Z`; ein Todo muss eine davon haben (Todos ohne Priorität passen dann nicht).
    pub priorities: Vec<char>,
    /// Freitext: jedes durch Leerraum getrennte Wort muss in der Beschreibung vorkommen
    /// (Groß-/Kleinschreibung egal). Die Beschreibung enthält die Tags, `+work` lässt sich also suchen.
    pub text: String,
    /// `Some(false)` nur offene, `Some(true)` nur erledigte, `None` alle.
    pub completed: Option<bool>,
}

/// Sortierkriterium.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum TodoSort {
    /// Manuelle Reihenfolge (`position`).
    Manual,
    /// Priorität `A` vor `B` … `Z`, ohne Priorität zuletzt.
    Priority,
    /// Erstellungsdatum, ohne Datum zuletzt.
    CreatedAt,
    /// Anzahl abgeschlossener Pomodoros.
    PomodoroCount,
    /// Beschreibung alphabetisch (Groß-/Kleinschreibung egal, Unicode-Kleinschreibung statt Locale-Vergleich).
    Alphabetical,
}

/// Sortierrichtung. Fehlende Werte (keine Priorität, kein Datum) stehen in beiden Richtungen zuletzt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Gibt die Todos zurück, auf die der Filter zutrifft; die Reihenfolge bleibt erhalten. Wer trifft, bringt seine
/// Vorfahren als Kontext mit (Subtasks, ADR 0022). Ohne Filter bleibt alles erhalten.
pub fn filter_todos(todos: &[Todo], filter: &TodoFilter) -> Vec<Todo> {
    let terms: Vec<String> = filter
        .text
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    let mut keep: HashSet<i32> = todos
        .iter()
        .filter(|todo| matches(todo, filter, &terms))
        .map(|todo| todo.id)
        .collect();
    let parents: HashMap<i32, Option<i32>> = todos.iter().map(|t| (t.id, t.parent_id)).collect();
    for id in keep.clone() {
        let mut seen = HashSet::from([id]);
        let mut next = parents.get(&id).copied().flatten();
        while let Some(parent) = next.filter(|parent| seen.insert(*parent)) {
            keep.insert(parent);
            next = parents.get(&parent).copied().flatten();
        }
    }
    todos
        .iter()
        .filter(|todo| keep.contains(&todo.id))
        .cloned()
        .collect()
}

/// Macht aus Verweisen ohne Elternknoten in `todos` (gelöscht, nicht geladen) und aus Zyklen (gleichzeitiges
/// Umhängen auf zwei Geräten) Wurzeln. Alle anderen Verweise bleiben.
pub fn normalize_parents(todos: &mut [Todo]) {
    let parents: HashMap<i32, Option<i32>> = todos.iter().map(|t| (t.id, t.parent_id)).collect();
    for todo in todos.iter_mut() {
        let Some(first) = todo.parent_id else {
            continue;
        };
        let mut seen = HashSet::from([todo.id]);
        let mut next = Some(first);
        let mut valid = true;
        while let Some(id) = next {
            if !parents.contains_key(&id) || !seen.insert(id) {
                valid = false;
                break;
            }
            next = parents[&id];
        }
        if !valid {
            todo.parent_id = None;
        }
    }
}

fn matches(todo: &Todo, filter: &TodoFilter, terms: &[String]) -> bool {
    let any_of = |wanted: &[String], tags: &[String]| {
        wanted.is_empty() || wanted.iter().any(|tag| tags.contains(tag))
    };
    if !any_of(&filter.projects, &todo.project_tags)
        || !any_of(&filter.contexts, &todo.context_tags)
    {
        return false;
    }
    if !filter.priorities.is_empty()
        && !todo
            .priority
            .is_some_and(|p| filter.priorities.contains(&p))
    {
        return false;
    }
    if filter
        .completed
        .is_some_and(|wanted| wanted != todo.completed)
    {
        return false;
    }
    let description = todo.description.to_lowercase();
    terms.iter().all(|term| description.contains(term.as_str()))
}

/// Sortiert stabil. Gleiche Werte behalten ihre Reihenfolge nach `position`, danach nach `id`.
pub fn sort_todos(todos: &mut [Todo], sort: TodoSort, direction: SortDirection) {
    todos.sort_by(|a, b| {
        compare(a, b, sort, direction)
            .then_with(|| a.position.cmp(&b.position))
            .then_with(|| a.id.cmp(&b.id))
    });
}

/// Filtert und sortiert in einem Schritt.
pub fn query_todos(
    todos: &[Todo],
    filter: &TodoFilter,
    sort: TodoSort,
    direction: SortDirection,
) -> Vec<Todo> {
    let mut result = filter_todos(todos, filter);
    normalize_parents(&mut result);
    sort_todos(&mut result, sort, direction);
    result
}

fn compare(a: &Todo, b: &Todo, sort: TodoSort, direction: SortDirection) -> Ordering {
    match sort {
        TodoSort::Manual => directed(a.position.cmp(&b.position), direction),
        TodoSort::PomodoroCount => directed(a.pomodoro_count.cmp(&b.pomodoro_count), direction),
        TodoSort::Alphabetical => directed(
            a.description
                .to_lowercase()
                .cmp(&b.description.to_lowercase()),
            direction,
        ),
        TodoSort::Priority => compare_optional(a.priority, b.priority, direction),
        TodoSort::CreatedAt => compare_optional(a.created_at, b.created_at, direction),
    }
}

fn directed(ordering: Ordering, direction: SortDirection) -> Ordering {
    match direction {
        SortDirection::Ascending => ordering,
        SortDirection::Descending => ordering.reverse(),
    }
}

/// Fehlende Werte zuletzt, unabhängig von der Richtung.
fn compare_optional<T: Ord>(a: Option<T>, b: Option<T>, direction: SortDirection) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => directed(a.cmp(&b), direction),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todotxt::parse_line;

    fn todo(id: i32, line: &str, parent_id: Option<i32>) -> Todo {
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
            position: id as u32,
            parent_id,
        }
    }

    fn ids(todos: &[Todo]) -> Vec<i32> {
        todos.iter().map(|todo| todo.id).collect()
    }

    fn parents(todos: &[Todo]) -> Vec<Option<i32>> {
        todos.iter().map(|todo| todo.parent_id).collect()
    }

    #[test]
    fn a_filter_hit_brings_its_ancestors() {
        let todos = [
            todo(1, "root", None),
            todo(2, "child", Some(1)),
            todo(3, "grandchild needle", Some(2)),
            todo(4, "other", None),
        ];
        let filter = TodoFilter {
            text: "needle".into(),
            ..TodoFilter::default()
        };
        assert_eq!(ids(&filter_todos(&todos, &filter)), [1, 2, 3]);
    }

    #[test]
    fn ancestor_lookup_stops_at_a_cycle() {
        let todos = [todo(1, "a needle", Some(2)), todo(2, "b", Some(1))];
        let filter = TodoFilter {
            text: "needle".into(),
            ..TodoFilter::default()
        };
        assert_eq!(ids(&filter_todos(&todos, &filter)), [1, 2]);
    }

    #[test]
    fn missing_parents_and_cycles_become_roots() {
        let mut todos = [
            todo(1, "ok root", None),
            todo(2, "ok child", Some(1)),
            todo(3, "orphan", Some(99)),
            todo(4, "cycle a", Some(5)),
            todo(5, "cycle b", Some(4)),
            todo(6, "below cycle", Some(4)),
            todo(7, "self", Some(7)),
        ];
        normalize_parents(&mut todos);
        assert_eq!(
            parents(&todos),
            [None, Some(1), None, None, None, None, None]
        );
    }
}
