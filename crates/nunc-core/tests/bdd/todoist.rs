//! Steps für `features/todoist/mapping.feature`. Den Bezugstag liefert `today is …` aus dem Modul `stats`.

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use cucumber::{gherkin::Step, given, then, when};
use nunc_core::model::Todo;
use nunc_core::todoist_mapping::{
    LocalChange, LocalTodo, TodoistMapping, TodoistMerge, TodoistTask, TodoistTaskRequest,
    map_task_to_todo, map_todo_to_task, merge_todoist,
};
use nunc_core::todotxt::{ParsedTodo, build_raw_text, parse_line};

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    task: Option<TodoistTask>,
    project_name: String,
    mapped: Option<ParsedTodo>,
    todo: Option<Todo>,
    request: Option<TodoistTaskRequest>,
    locals: Vec<LocalTodo>,
    remotes: Vec<TodoistTask>,
    mappings: Vec<TodoistMapping>,
    projects: HashMap<String, String>,
    merge: Option<TodoistMerge>,
}

fn rows(step: &Step) -> (Vec<String>, Vec<Vec<String>>) {
    let rows = &step.table.as_ref().expect("Tabelle fehlt").rows;
    (rows[0].clone(), rows[1..].to_vec())
}

fn cell<'a>(header: &[String], row: &'a [String], name: &str) -> &'a str {
    header
        .iter()
        .position(|h| h == name)
        .map_or("", |i| row[i].as_str())
}

fn instant(text: &str) -> Option<DateTime<Utc>> {
    (!text.is_empty()).then(|| {
        DateTime::parse_from_rfc3339(text)
            .unwrap_or_else(|_| panic!("Zeit {text:?} ungültig"))
            .with_timezone(&Utc)
    })
}

/// Spalten: `id`, `content`, `priority`, `project` (Name, nur für den Einzel-Step), `labels` (Komma-getrennt),
/// `completed` (`yes`), `created`, `updated` (RFC 3339), `project_id` (nur Merge).
fn tasks(step: &Step) -> Vec<(TodoistTask, String)> {
    let (header, data) = rows(step);
    data.iter()
        .map(|row| {
            let get = |name: &str| cell(&header, row, name);
            let task = TodoistTask {
                id: get("id").to_owned(),
                content: get("content").to_owned(),
                priority: get("priority").parse().unwrap_or(1),
                project_id: get("project_id").to_owned(),
                labels: get("labels")
                    .split(',')
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(str::to_owned)
                    .collect(),
                is_completed: get("completed") == "yes",
                created_at: instant(get("created")),
                updated_at: instant(get("updated")),
            };
            (task, get("project").to_owned())
        })
        .collect()
}

fn ids<T: Copy + ToString>(values: impl IntoIterator<Item = T>) -> Vec<String> {
    values.into_iter().map(|v| v.to_string()).collect()
}

fn words(list: &str) -> Vec<String> {
    list.split_whitespace().map(str::to_owned).collect()
}

fn to_todo(id: i32, line: &str) -> Todo {
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
        parent_id: None,
    }
}

fn mapped(world: &DomainWorld) -> &ParsedTodo {
    world.todoist.mapped.as_ref().expect("nicht abgebildet")
}

fn request(world: &DomainWorld) -> &TodoistTaskRequest {
    world.todoist.request.as_ref().expect("nicht abgebildet")
}

fn merge(world: &DomainWorld) -> &TodoistMerge {
    world.todoist.merge.as_ref().expect("nicht zusammengeführt")
}

// ---- Todoist → nunc --------------------------------------------------------

#[given("the Todoist task:")]
fn single_task(world: &mut DomainWorld, step: &Step) {
    let (task, project) = tasks(step).remove(0);
    world.todoist.task = Some(task);
    world.todoist.project_name = project;
}

#[when("I map the task to a todo")]
fn map_task(world: &mut DomainWorld) {
    let state = &world.todoist;
    let task = state.task.as_ref().expect("keine Aufgabe");
    world.todoist.mapped = Some(map_task_to_todo(
        task,
        &state.project_name,
        &task.labels,
        world.stats.today,
    ));
}

#[then(expr = "the mapped priority is {word}")]
fn mapped_priority(world: &mut DomainWorld, expected: String) {
    let actual = mapped(world)
        .priority
        .map_or("none".to_owned(), String::from);
    assert_eq!(actual, expected);
}

#[then(expr = "the mapped project tags are {string}")]
fn mapped_projects(world: &mut DomainWorld, expected: String) {
    assert_eq!(mapped(world).project_tags, words(&expected));
}

#[then(expr = "the mapped context tags are {string}")]
fn mapped_contexts(world: &mut DomainWorld, expected: String) {
    assert_eq!(mapped(world).context_tags, words(&expected));
}

#[then(expr = "the mapped description is {string}")]
fn mapped_description(world: &mut DomainWorld, expected: String) {
    assert_eq!(mapped(world).description, expected);
}

#[then("the mapped todo is open")]
fn mapped_open(world: &mut DomainWorld) {
    let todo = mapped(world);
    assert!(!todo.completed && todo.completed_at.is_none());
}

#[then(expr = "the mapped todo is completed on {word}")]
fn mapped_completed(world: &mut DomainWorld, date: String) {
    let todo = mapped(world);
    assert!(todo.completed);
    assert_eq!(
        todo.completed_at,
        Some(NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap())
    );
}

#[then(expr = "the mapped creation date is {word}")]
fn mapped_created(world: &mut DomainWorld, date: String) {
    let expected = (date != "none").then(|| NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap());
    assert_eq!(mapped(world).created_at, expected);
}

#[then(expr = "the mapped raw text is {string}")]
fn mapped_raw(world: &mut DomainWorld, expected: String) {
    assert_eq!(build_raw_text(mapped(world)), expected);
}

// ---- nunc → Todoist --------------------------------------------------------

#[given(expr = "the todo {string}")]
fn the_todo(world: &mut DomainWorld, line: String) {
    world.todoist.todo = Some(to_todo(1, &line));
}

#[when("I map the todo to a Todoist task")]
fn map_todo(world: &mut DomainWorld) {
    let todo = world.todoist.todo.as_ref().expect("kein Todo");
    world.todoist.request = Some(map_todo_to_task(todo));
}

#[then(expr = "the Todoist content is {string}")]
fn todoist_content(world: &mut DomainWorld, expected: String) {
    assert_eq!(request(world).content, expected);
}

#[then(expr = "the Todoist priority is {int}")]
fn todoist_priority(world: &mut DomainWorld, expected: u8) {
    assert_eq!(request(world).priority, expected);
}

#[then(expr = "the Todoist project is {string}")]
fn todoist_project(world: &mut DomainWorld, expected: String) {
    let actual = request(world).project_name.clone().unwrap_or_default();
    assert_eq!(actual, expected);
}

#[then(expr = "the Todoist labels are {string}")]
fn todoist_labels(world: &mut DomainWorld, expected: String) {
    let actual = request(world).labels.join(",");
    assert_eq!(actual, expected);
}

// ---- Merge -----------------------------------------------------------------

/// Spalten: `id`, `line` (todo.txt-Zeile), `updated` (RFC 3339), `deleted` (`yes`).
#[given("the nunc todos:")]
fn nunc_todos(world: &mut DomainWorld, step: &Step) {
    let (header, data) = rows(step);
    world.todoist.locals = data
        .iter()
        .map(|row| LocalTodo {
            todo: to_todo(
                cell(&header, row, "id").parse().expect("id"),
                cell(&header, row, "line"),
            ),
            updated_at: instant(cell(&header, row, "updated")),
            deleted: cell(&header, row, "deleted") == "yes",
        })
        .collect();
}

#[given("the Todoist tasks:")]
fn todoist_tasks(world: &mut DomainWorld, step: &Step) {
    world.todoist.remotes = tasks(step).into_iter().map(|(task, _)| task).collect();
}

#[given("the mappings:")]
fn mappings(world: &mut DomainWorld, step: &Step) {
    let (header, data) = rows(step);
    world.todoist.mappings = data
        .iter()
        .map(|row| TodoistMapping {
            nunc_todo_id: cell(&header, row, "nunc_id").parse().expect("nunc_id"),
            todoist_task_id: cell(&header, row, "task_id").to_owned(),
        })
        .collect();
}

#[given("the Todoist projects:")]
fn projects(world: &mut DomainWorld, step: &Step) {
    let (header, data) = rows(step);
    world.todoist.projects = data
        .iter()
        .map(|row| {
            (
                cell(&header, row, "id").to_owned(),
                cell(&header, row, "name").to_owned(),
            )
        })
        .collect();
}

#[when("I merge the Todoist state")]
fn merge_state(world: &mut DomainWorld) {
    let state = &world.todoist;
    world.todoist.merge = Some(merge_todoist(
        &state.locals,
        &state.remotes,
        &state.mappings,
        &state.projects,
        world.stats.today,
    ));
}

#[then(expr = "the todos to create in Todoist are {string}")]
fn to_create(world: &mut DomainWorld, expected: String) {
    assert_eq!(
        ids(merge(world).to_create_in_todoist.iter().map(|t| t.id)),
        words(&expected)
    );
}

#[then(expr = "the todos to update in Todoist are {string}")]
fn to_update(world: &mut DomainWorld, expected: String) {
    assert_eq!(
        ids(merge(world).to_update_in_todoist.iter().map(|t| t.id)),
        words(&expected)
    );
}

#[then(expr = "the tasks to close in Todoist are {string}")]
fn to_close(world: &mut DomainWorld, expected: String) {
    assert_eq!(merge(world).to_close_in_todoist, words(&expected));
}

#[then(expr = "the todos to complete locally are {string}")]
fn to_complete(world: &mut DomainWorld, expected: String) {
    assert_eq!(
        ids(merge(world).to_complete_locally.iter().copied()),
        words(&expected)
    );
}

/// Zeilen `insert | <Aufgaben-ID> | <Beschreibung>` oder `update | <Todo-ID> | <Beschreibung>`.
#[then("the local changes are:")]
fn local_changes(world: &mut DomainWorld, step: &Step) {
    let expected: Vec<Vec<String>> = step
        .table
        .as_ref()
        .map_or_else(Vec::new, |t| t.rows.clone());
    let actual: Vec<Vec<String>> = merge(world)
        .local_changes
        .iter()
        .map(|change| match change {
            LocalChange::Insert {
                todoist_task_id,
                todo,
            } => vec![
                "insert".to_owned(),
                todoist_task_id.clone(),
                todo.description.clone(),
            ],
            LocalChange::Update { todo_id, todo } => vec![
                "update".to_owned(),
                todo_id.to_string(),
                todo.description.clone(),
            ],
        })
        .collect();
    assert_eq!(actual, expected);
}
