//! Steps für `features/todos/filter_sort.feature`.

use cucumber::{gherkin::Step, given, then, when};
use nunc_core::model::Todo;
use nunc_core::todo_query::{
    SortDirection, TodoFilter, TodoSort, filter_todos, query_todos, sort_todos,
};
use nunc_core::todotxt::parse_line;

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    pub(crate) todos: Vec<Todo>,
    filter: TodoFilter,
    result: Vec<Todo>,
}

fn words(list: &str) -> Vec<String> {
    list.split_whitespace().map(str::to_owned).collect()
}

fn sort_from(name: &str) -> TodoSort {
    match name {
        "manual" => TodoSort::Manual,
        "priority" => TodoSort::Priority,
        "created" => TodoSort::CreatedAt,
        "pomodoros" => TodoSort::PomodoroCount,
        "alphabetical" => TodoSort::Alphabetical,
        other => panic!("unbekannte Sortierung {other:?}"),
    }
}

fn direction_from(name: &str) -> SortDirection {
    match name {
        "ascending" => SortDirection::Ascending,
        "descending" => SortDirection::Descending,
        other => panic!("unbekannte Richtung {other:?}"),
    }
}

/// Tabelle mit den Spalten `id`, `line` (todo.txt-Zeile), `pomodoros`, `position`.
#[given("the todos:")]
fn todos(world: &mut DomainWorld, step: &Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    let header = &table.rows[0];
    let column = |name: &str| {
        header
            .iter()
            .position(|h| h == name)
            .unwrap_or_else(|| panic!("Spalte {name} fehlt"))
    };
    let (id, line, pomodoros, position) = (
        column("id"),
        column("line"),
        column("pomodoros"),
        column("position"),
    );

    world.todo_query.todos = table.rows[1..]
        .iter()
        .map(|row| {
            let parsed = parse_line(&row[line]);
            Todo {
                id: row[id].parse().expect("id"),
                raw_text: row[line].clone(),
                priority: parsed.priority,
                completed: parsed.completed,
                completed_at: parsed.completed_at,
                created_at: parsed.created_at,
                description: parsed.description,
                project_tags: parsed.project_tags,
                context_tags: parsed.context_tags,
                pomodoro_count: row[pomodoros].parse().expect("pomodoros"),
                position: row[position].parse().expect("position"),
            }
        })
        .collect();
}

#[given(expr = "the filter projects are {string}")]
fn filter_projects(world: &mut DomainWorld, list: String) {
    world.todo_query.filter.projects = words(&list);
}

#[given(expr = "the filter contexts are {string}")]
fn filter_contexts(world: &mut DomainWorld, list: String) {
    world.todo_query.filter.contexts = words(&list);
}

#[given(expr = "the filter priorities are {string}")]
fn filter_priorities(world: &mut DomainWorld, list: String) {
    world.todo_query.filter.priorities = list.chars().filter(|c| !c.is_whitespace()).collect();
}

#[given(expr = "the filter text is {string}")]
fn filter_text(world: &mut DomainWorld, text: String) {
    world.todo_query.filter.text = text;
}

#[given(expr = "the filter shows only {word} todos")]
fn filter_completion(world: &mut DomainWorld, which: String) {
    world.todo_query.filter.completed = match which.as_str() {
        "open" => Some(false),
        "completed" => Some(true),
        other => panic!("unbekannter Status {other:?}"),
    };
}

#[when("I apply the filter")]
fn apply_filter(world: &mut DomainWorld) {
    let state = &mut world.todo_query;
    state.result = filter_todos(&state.todos, &state.filter);
}

#[when(expr = "I sort by {word} in {word} order")]
fn sort_todos_step(world: &mut DomainWorld, sort: String, direction: String) {
    let state = &mut world.todo_query;
    state.result = state.todos.clone();
    sort_todos(
        &mut state.result,
        sort_from(&sort),
        direction_from(&direction),
    );
}

#[when(expr = "I apply the filter and sort by {word} in {word} order")]
fn filter_and_sort(world: &mut DomainWorld, sort: String, direction: String) {
    let state = &mut world.todo_query;
    state.result = query_todos(
        &state.todos,
        &state.filter,
        sort_from(&sort),
        direction_from(&direction),
    );
}

#[then(expr = "the result ids are {string}")]
fn result_ids(world: &mut DomainWorld, expected: String) {
    let actual: Vec<String> = world
        .todo_query
        .result
        .iter()
        .map(|t| t.id.to_string())
        .collect();
    assert_eq!(actual, words(&expected));
}
