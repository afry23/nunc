//! Steps für `features/todos/import_export.feature` (`@domain`). Die Todos kommen aus `todo_query`.

use cucumber::{gherkin::Step, then, when};
use nunc_core::todotxt_file::{ImportPlan, export_todotxt, plan_import};

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    file: String,
    plan: ImportPlan,
}

fn docstring(step: &Step) -> String {
    step.docstring
        .as_deref()
        .expect("Docstring fehlt")
        .trim_matches('\n')
        .to_owned()
}

#[when("I export the todos")]
fn export(world: &mut DomainWorld) {
    world.todotxt_file.file = export_todotxt(&world.todo_query.todos, true);
}

#[when("I export the todos without completed ones")]
fn export_open(world: &mut DomainWorld) {
    world.todotxt_file.file = export_todotxt(&world.todo_query.todos, false);
}

#[then("the todo.txt file is:")]
fn file_is(world: &mut DomainWorld, step: &Step) {
    assert_eq!(world.todotxt_file.file, format!("{}\n", docstring(step)));
}

#[then("the todo.txt file is empty")]
fn file_is_empty(world: &mut DomainWorld) {
    assert_eq!(world.todotxt_file.file, "");
}

#[when("I import the todo.txt file:")]
fn import(world: &mut DomainWorld, step: &Step) {
    world.todotxt_file.plan = plan_import(&docstring(step), &world.todo_query.todos);
}

#[when("I import the todo.txt file with Windows line endings:")]
fn import_crlf(world: &mut DomainWorld, step: &Step) {
    let text = docstring(step).replace('\n', "\r\n");
    world.todotxt_file.plan = plan_import(&text, &world.todo_query.todos);
}

#[when("I import the todo.txt file with a byte order mark:")]
fn import_bom(world: &mut DomainWorld, step: &Step) {
    let text = format!("\u{feff}{}", docstring(step));
    world.todotxt_file.plan = plan_import(&text, &world.todo_query.todos);
}

#[when("I import the exported todos")]
fn import_export(world: &mut DomainWorld) {
    let file = export_todotxt(&world.todo_query.todos, true);
    world.todotxt_file.plan = plan_import(&file, &world.todo_query.todos);
}

#[then("the lines to import are:")]
fn lines_are(world: &mut DomainWorld, step: &Step) {
    assert_eq!(world.todotxt_file.plan.to_add.join("\n"), docstring(step));
}

#[then("there are no lines to import")]
fn no_lines(world: &mut DomainWorld) {
    assert!(world.todotxt_file.plan.to_add.is_empty());
}

#[then(expr = "{int} duplicates are skipped")]
fn duplicates(world: &mut DomainWorld, expected: usize) {
    assert_eq!(world.todotxt_file.plan.duplicates, expected);
}
