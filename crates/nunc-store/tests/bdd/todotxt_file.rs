//! Steps für den Import und Export in `features/todos/import_export.feature` (`@integration`).

use cucumber::{gherkin::Step, then, when};

use crate::StoreWorld;

#[derive(Debug, Default)]
pub struct State {
    exported: String,
    added: usize,
    duplicates: usize,
}

fn docstring(step: &Step) -> String {
    step.docstring
        .as_deref()
        .expect("Docstring fehlt")
        .trim_matches('\n')
        .to_owned()
}

fn import(world: &mut StoreWorld, text: &str) {
    let mut counter = world.sync_counter;
    let result =
        nunc_store::import_todotxt(world.conn_mut(), text, crate::at("10:00:00"), &mut || {
            counter += 1;
            format!("import-{counter}")
        })
        .expect("Import");
    world.sync_counter = counter;
    world.todotxt.added = result.added.len();
    world.todotxt.duplicates = result.duplicates;
}

#[when("I import the todo.txt lines:")]
fn import_lines(world: &mut StoreWorld, step: &Step) {
    import(world, &docstring(step));
}

#[when("I export the todos from the database")]
fn export(world: &mut StoreWorld) {
    world.todotxt.exported = nunc_store::export_todotxt(world.conn(), true).expect("Export");
}

#[when("I import the exported text")]
fn import_exported(world: &mut StoreWorld) {
    let text = world.todotxt.exported.clone();
    import(world, &text);
}

#[then("the exported text is:")]
fn exported_is(world: &mut StoreWorld, step: &Step) {
    assert_eq!(world.todotxt.exported, format!("{}\n", docstring(step)));
}

#[then(expr = "{int} todos were added and {int} duplicates were skipped")]
fn counts(world: &mut StoreWorld, added: usize, duplicates: usize) {
    assert_eq!(world.todotxt.added, added);
    assert_eq!(world.todotxt.duplicates, duplicates);
}
