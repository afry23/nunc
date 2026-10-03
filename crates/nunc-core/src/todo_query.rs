//! Filtern und Sortieren der Todo-Liste (pur, ohne Zugriff auf Datenbank oder Uhr).
//!
//! Verhalten der Go-App (`TodoList.svelte`): innerhalb einer Kategorie genügt **ein** Treffer (ODER), über
//! Kategorien hinweg müssen **alle** zutreffen (UND). Erweitert um Priorität, Freitext, Erledigt-Status
//! und mehrere Sortierungen (REQUIREMENTS 4.2).

use std::cmp::Ordering;

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

/// Gibt die Todos zurück, auf die der Filter zutrifft; die Reihenfolge bleibt erhalten.
pub fn filter_todos(todos: &[Todo], filter: &TodoFilter) -> Vec<Todo> {
    let terms: Vec<String> = filter
        .text
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    todos
        .iter()
        .filter(|todo| matches(todo, filter, &terms))
        .cloned()
        .collect()
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
