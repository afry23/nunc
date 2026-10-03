//! BDD-Runner für `@domain`-Szenarien (reine Fachlogik, keine I/O).
//!
//! Aufruf: `cargo test -p nunc-core --test bdd`
//! Szenarien mit `@wip` werden übersprungen.

use cucumber::{World, given, then, when};

#[derive(Debug, Default, World)]
struct DomainWorld {
    value: i64,
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    DomainWorld::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../features"),
            |feature, _, scenario| {
                let has = |tag: &str| feature.tags.iter().chain(&scenario.tags).any(|t| t == tag);
                has("domain") && !has("wip")
            },
        )
        .await;
}
