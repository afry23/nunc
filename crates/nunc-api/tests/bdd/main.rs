//! BDD-Runner für `features/api/local_api.feature` (Tag `@api`): `@domain` prüft das pure Routing gegen ein
//! Fake-Backend, `@integration` den Server über einen echten Socket.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use cucumber::{World, WriterExt as _, writer};
use nunc_api::server::ApiServer;
use nunc_api::{ApiError, ImportOutcome, Response, TodoBackend};
use nunc_core::model::Todo;
use nunc_core::todotxt::parse_line;
use nunc_core::todotxt_file::plan_import;

mod routing;
mod server;

/// Todos im Speicher; `fail` lässt jeden Zugriff scheitern.
#[derive(Debug, Default)]
pub struct FakeBackend {
    todos: Mutex<Vec<Todo>>,
    fail: Mutex<Option<String>>,
}

impl FakeBackend {
    fn check(&self) -> Result<(), ApiError> {
        match &*self.fail.lock().unwrap() {
            Some(message) => Err(ApiError::Internal(message.clone())),
            None => Ok(()),
        }
    }

    pub fn lines(&self) -> Vec<String> {
        self.todos
            .lock()
            .unwrap()
            .iter()
            .map(|todo| todo.raw_text.clone())
            .collect()
    }
}

pub fn make_todo(id: i32, line: &str, pomodoros: u32, position: u32) -> Todo {
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
        pomodoro_count: pomodoros,
        position,
    }
}

impl TodoBackend for FakeBackend {
    fn list(&self) -> Result<Vec<Todo>, ApiError> {
        self.check()?;
        Ok(self.todos.lock().unwrap().clone())
    }

    fn import(&self, text: &str) -> Result<ImportOutcome, ApiError> {
        self.check()?;
        let mut todos = self.todos.lock().unwrap();
        let plan = plan_import(text, &todos);
        let mut added = Vec::new();
        for line in plan.to_add {
            let id = todos.iter().map(|todo| todo.id).max().unwrap_or(0) + 1;
            let position = u32::try_from(todos.len()).unwrap_or(0) + 1;
            let todo = make_todo(id, &line, 0, position);
            todos.push(todo.clone());
            added.push(todo);
        }
        Ok(ImportOutcome {
            added,
            duplicates: plan.duplicates,
        })
    }

    fn set_completed(&self, id: i32, completed: bool) -> Result<Todo, ApiError> {
        self.check()?;
        let mut todos = self.todos.lock().unwrap();
        let todo = todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or(ApiError::NotFound(id))?;
        todo.completed = completed;
        Ok(todo.clone())
    }
}

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct ApiWorld {
    pub token: String,
    pub port: u16,
    pub backend: Arc<FakeBackend>,
    /// Kopfzeilen der nächsten Anfrage; `None` = entfernt. Voreinstellung: Host und Token passen.
    pub headers: BTreeMap<String, Option<String>>,
    pub response: Option<Response>,
    pub server: server::State,
}

impl ApiWorld {
    fn new() -> Self {
        Self {
            token: String::new(),
            port: 0,
            backend: Arc::new(FakeBackend::default()),
            headers: BTreeMap::new(),
            response: None,
            server: server::State::default(),
        }
    }

    pub fn running(&mut self) -> &ApiServer {
        self.server.running.as_ref().expect("Server läuft nicht")
    }
}

const FEATURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../features");

/// Nur `@api`-Szenarien, `@wip` wird übersprungen.
fn selected(
    feature: &cucumber::gherkin::Feature,
    _rule: Option<&cucumber::gherkin::Rule>,
    scenario: &cucumber::gherkin::Scenario,
) -> bool {
    let has = |tag: &str| feature.tags.iter().chain(&scenario.tags).any(|t| t == tag);
    has("api") && !has("wip")
}

/// Mit `NUNC_BDD_REPORT=<datei>` wird zusätzlich ein Cucumber-JSON-Report geschrieben.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cucumber = ApiWorld::cucumber().fail_on_skipped();
    match std::env::var_os("NUNC_BDD_REPORT") {
        Some(path) => {
            let path = std::path::PathBuf::from(path).with_file_name("api.json");
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).expect("Report-Verzeichnis anlegen");
            }
            let file = std::fs::File::create(path).expect("Report-Datei anlegen");
            cucumber
                .with_writer(
                    writer::Basic::stdout()
                        .summarized()
                        .tee::<ApiWorld, _>(writer::Json::for_tee(file))
                        .normalized(),
                )
                .filter_run_and_exit(FEATURES, selected)
                .await;
        }
        None => cucumber.filter_run_and_exit(FEATURES, selected).await,
    }
}
