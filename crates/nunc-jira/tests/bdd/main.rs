//! BDD-Runner für die `@jira`-Szenarien (`@integration`): Jira-Client gegen wiremock, Zugang mit echter SQLite.
//!
//! Aufruf: `cargo test -p nunc-jira --test bdd` · Szenarien mit `@wip` werden übersprungen.

use cucumber::World;
use nunc_core::jira::{JiraConfig, JiraIssue, JiraKind};
use nunc_jira::{JiraCredentials, JiraError};
use nunc_platform::MemorySecretStore;
use nunc_store::Connection;
use wiremock::MockServer;

mod steps;

#[derive(World)]
#[world(init = Self::new)]
pub struct JiraWorld {
    pub conn: Connection,
    pub secrets: MemorySecretStore,
    pub server: Option<MockServer>,
    /// Zugang, mit dem der Client arbeitet (Szenarien können Teile verfälschen).
    pub credentials: Option<JiraCredentials>,
    pub dead: bool,
    pub trailing_slash: bool,
    pub hits: Option<Vec<JiraIssue>>,
    pub error: Option<JiraError>,
    pub test_ok: bool,
}

impl std::fmt::Debug for JiraWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JiraWorld")
    }
}

impl JiraWorld {
    fn new() -> Result<Self, std::convert::Infallible> {
        Ok(Self {
            conn: nunc_store::open_in_memory().unwrap(),
            secrets: MemorySecretStore::default(),
            server: None,
            credentials: None,
            dead: false,
            trailing_slash: false,
            hits: None,
            error: None,
            test_ok: false,
        })
    }
}

pub fn config(kind: JiraKind, base_url: String, email: &str) -> JiraConfig {
    JiraConfig {
        base_url,
        kind,
        email: email.to_owned(),
    }
}

const FEATURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../features");

/// Nur `@jira`-Szenarien, `@wip` wird übersprungen.
fn selected(
    feature: &cucumber::gherkin::Feature,
    _rule: Option<&cucumber::gherkin::Rule>,
    scenario: &cucumber::gherkin::Scenario,
) -> bool {
    let has = |tag: &str| feature.tags.iter().chain(&scenario.tags).any(|t| t == tag);
    has("integration") && has("jira") && !has("wip")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Der gemeinsame HTTP-Client lädt beim ersten Aufruf den Zertifikatsspeicher (~100 ms)
    nunc_platform::http_client().expect("HTTP-Client");
    JiraWorld::cucumber()
        .fail_on_skipped()
        .filter_run_and_exit(FEATURES, selected)
        .await;
}
