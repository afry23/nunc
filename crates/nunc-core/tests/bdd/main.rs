//! BDD-Runner für `@domain`-Szenarien (reine Fachlogik, keine I/O).
//!
//! Aufruf: `cargo test -p nunc-core --test bdd`
//! Step-Definitions: ein Modul pro Themenbereich neben dieser Datei (`tests/bdd/*.rs`).
//! Szenarien mit `@wip` werden übersprungen.

use cucumber::{World, WriterExt as _, given, then, when, writer};

mod timer;
mod todo_query;
mod todotxt;

/// Gemeinsamer Zustand aller `@domain`-Szenarien. Jeder Themenbereich legt seine Felder hier ab und
/// definiert seine Steps in einem eigenen Modul.
#[derive(Debug, Default, World)]
pub struct DomainWorld {
    value: i64,
    timer: timer::State,
    todo_query: todo_query::State,
    todotxt: todotxt::State,
}

#[given(expr = "the number {int}")]
fn given_number(world: &mut DomainWorld, n: i64) {
    world.value = n;
}

#[when(expr = "I add {int}")]
fn add(world: &mut DomainWorld, n: i64) {
    world.value += n;
}

#[then(expr = "the result is {int}")]
fn result_is(world: &mut DomainWorld, expected: i64) {
    assert_eq!(world.value, expected);
}

const FEATURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../features");

/// Nur `@domain`-Szenarien, `@wip` wird übersprungen.
fn selected(
    feature: &cucumber::gherkin::Feature,
    _rule: Option<&cucumber::gherkin::Rule>,
    scenario: &cucumber::gherkin::Scenario,
) -> bool {
    let has = |tag: &str| feature.tags.iter().chain(&scenario.tags).any(|t| t == tag);
    has("domain") && !has("wip")
}

/// Mit `NUNC_BDD_REPORT=<datei>` wird zusätzlich ein Cucumber-JSON-Report geschrieben (CI-Artefakt, absoluter Pfad empfohlen: `cargo test` läuft im Crate-Verzeichnis).
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cucumber = DomainWorld::cucumber().fail_on_skipped();
    match std::env::var_os("NUNC_BDD_REPORT") {
        Some(path) => {
            let path = std::path::PathBuf::from(path);
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).expect("Report-Verzeichnis anlegen");
            }
            let file = std::fs::File::create(path).expect("Report-Datei anlegen");
            cucumber
                .with_writer(
                    writer::Basic::stdout()
                        .summarized()
                        .tee::<DomainWorld, _>(writer::Json::for_tee(file))
                        .normalized(),
                )
                .filter_run_and_exit(FEATURES, selected)
                .await;
        }
        None => cucumber.filter_run_and_exit(FEATURES, selected).await,
    }
}
