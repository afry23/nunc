//! Steps für die Todoist-Szenarien: Anmeldung, Tresor und Sync-Lauf gegen das Test-„Todoist“.

use std::sync::Arc;
use std::time::Duration;

use chrono::Duration as ChronoDuration;
use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use nunc_core::clock::Clock;
use nunc_core::model::TodoistStatus;
use nunc_platform::{MemorySecretStore, SecretStore};
use nunc_sync::oauth::{Connection, Tokens, Vault};
use nunc_sync::{SyncRunner, TodoistConfig, TodoistRunner, TodoistService};

use crate::SyncWorld;
use crate::todoist_server::Todoist;

const ACCOUNT: &str = "todoist";

pub struct State {
    pub server: Option<Todoist>,
    pub secrets: Arc<MemorySecretStore>,
    pub status: Option<TodoistStatus>,
    /// Tokens, die das Test-Todoist ablehnt (sonst nimmt es jedes eingegebene persönliche Token an).
    pub rejected: Vec<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            server: None,
            secrets: Arc::new(MemorySecretStore::default()),
            status: None,
            rejected: Vec::new(),
        }
    }
}

fn server(world: &SyncWorld) -> &Todoist {
    world
        .todoist
        .server
        .as_ref()
        .expect("kein Todoist gestartet")
}

pub fn service(world: &SyncWorld) -> Arc<TodoistService> {
    let uri = server(world).server.uri();
    let mut config = TodoistConfig::todoist();
    config.api_base = format!("{uri}/api/v1");
    Arc::new(
        TodoistService::new(config, world.todoist.secrets.clone(), world.clock.clone())
            .expect("Todoist-Dienst"),
    )
}

fn vault(world: &SyncWorld) -> Vault {
    Vault::new(world.todoist.secrets.clone(), ACCOUNT)
}

fn rows(step: &Step) -> Vec<std::collections::HashMap<String, String>> {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    let header: Vec<String> = table.rows[0].iter().map(|c| c.trim().to_owned()).collect();
    table.rows[1..]
        .iter()
        .map(|row| {
            header
                .iter()
                .cloned()
                .zip(row.iter().map(|c| c.trim().to_owned()))
                .collect()
        })
        .collect()
}

fn words(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Given

#[given("a Todoist server")]
async fn start_server(world: &mut SyncWorld) {
    world.todoist.server = Some(Todoist::start().await);
}

#[given("Todoist is connected")]
fn connected(world: &mut SyncWorld) {
    server(world).with(|state| state.valid.insert("todoist-access-0".to_owned()));
    vault(world)
        .save(&Connection {
            tokens: Tokens {
                access_token: "todoist-access-0".to_owned(),
                refresh_token: Some("todoist-refresh-0".to_owned()),
                expires_at: Some(world.clock.now() + ChronoDuration::hours(1)),
            },
            email: Some("me@example.org".to_owned()),
            avatar_url: Some("https://avatars.example/me.png".to_owned()),
        })
        .unwrap();
}

#[given("the Todoist access token has expired")]
fn token_expired(world: &mut SyncWorld) {
    let vault = vault(world);
    let mut connection = vault.load().unwrap().expect("nicht verbunden");
    connection.tokens.expires_at = Some(world.clock.now() - ChronoDuration::minutes(1));
    vault.save(&connection).unwrap();
}

#[given(expr = "Todoist answers every request with {int}")]
fn fails(world: &mut SyncWorld, status: u16) {
    server(world).with(|state| state.fail_all = Some(status));
}

#[given(expr = "the Todoist clock is {string}")]
fn clock(world: &mut SyncWorld, now: String) {
    server(world).with(|state| state.now = now);
}

#[given(expr = "Todoist lists {int} items per page")]
fn page_size(world: &mut SyncWorld, size: usize) {
    server(world).with(|state| state.page_size = size);
}

#[given("Todoist holds the tasks:")]
fn holds_tasks(world: &mut SyncWorld, step: &Step) {
    for row in rows(step) {
        server(world).with(|state| {
            let project = row.get("project").filter(|p| !p.is_empty());
            state.add_task(
                &row["content"],
                row.get("priority")
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(1),
                project.map(String::as_str),
                words(row.get("labels").map_or("", String::as_str)),
                row.get("updated")
                    .filter(|u| !u.is_empty())
                    .map(String::as_str),
            );
        });
    }
}

#[given(expr = "Todoist knows the project {string} and the label {string}")]
fn knows(world: &mut SyncWorld, project: String, label: String) {
    server(world).with(|state| {
        state.project_id(&project);
        state.labels.push(("label-known".to_owned(), label));
    });
}

fn todo_id(world: &SyncWorld, text: &str) -> i32 {
    let conn = world.store.conn.lock().unwrap();
    nunc_store::list_local_todos(&conn)
        .unwrap()
        .into_iter()
        .find(|local| local.todo.raw_text == text)
        .unwrap_or_else(|| panic!("Todo {text:?} fehlt"))
        .todo
        .id
}

#[given(expr = "the todo {string} is mapped to the Todoist task {string}")]
fn mapped(world: &mut SyncWorld, text: String, content: String) {
    let id = todo_id(world, &text);
    let task = server(world).with(|state| {
        state
            .tasks
            .iter()
            .find(|t| t.content == content)
            .map(|t| t.id.clone())
            .unwrap_or_else(|| panic!("Aufgabe {content:?} fehlt"))
    });
    let conn = world.store.conn.lock().unwrap();
    nunc_store::add_todoist_mapping(&conn, id, &task, world.clock.now()).unwrap();
}

#[given(expr = "the todo {string} was deleted")]
fn deleted(world: &mut SyncWorld, text: String) {
    let id = todo_id(world, &text);
    let conn = world.store.conn.lock().unwrap();
    nunc_store::delete_todo(&conn, id, world.clock.now()).unwrap();
}

#[given("the coordinator runs the Todoist runner")]
fn coordinator_runs(world: &mut SyncWorld) {
    let runner = TodoistRunner::new(world.store.clone(), service(world), world.clock.clone());
    world.coord.runner_override = Some(Arc::new(runner));
}

// ---------------------------------------------------------------------------------------------
// When

#[given(expr = "Todoist rejects the token {string}")]
#[when(expr = "Todoist rejects the token {string}")]
fn rejects_token(world: &mut SyncWorld, token: String) {
    server(world).with(|state| state.valid.remove(&token));
    world.todoist.rejected.push(token);
}

#[when(expr = "I connect Todoist with the token {string}")]
async fn connect_token(world: &mut SyncWorld, token: String) {
    let trimmed = token.trim().to_owned();
    if !trimmed.is_empty() && !world.todoist.rejected.contains(&trimmed) {
        server(world).with(|state| state.valid.insert(trimmed));
    }
    let service = service(world);
    match service.connect_token(&token).await {
        Ok(status) => {
            world.todoist.status = Some(status);
            world.google.error = None;
        }
        Err(error) => world.google.error = Some(error.to_string()),
    }
}

#[when("I disconnect Todoist")]
fn disconnect(world: &mut SyncWorld) {
    service(world).disconnect().unwrap();
}

#[when("I sync with Todoist")]
async fn sync(world: &mut SyncWorld) {
    let service = service(world);
    world.outcome = Some(service.sync(&*world.store).await);
}

#[when("the Todoist runner runs")]
async fn runner_runs(world: &mut SyncWorld) {
    let runner = TodoistRunner::new(world.store.clone(), service(world), world.clock.clone());
    world.coord.runner_outcome = Some(runner.run().await);
}

#[when("the account has just been connected")]
fn just_connected(world: &mut SyncWorld) {
    world
        .coord
        .coordinator()
        .trigger(nunc_sync::Trigger::Connected);
}

// ---------------------------------------------------------------------------------------------
// Then

#[then(expr = "the Todoist status shows the account {string} with an avatar")]
fn status_account(world: &mut SyncWorld, email: String) {
    let status = world.todoist.status.as_ref().expect("keine Verbindung");
    assert!(status.connected);
    assert_eq!(status.email.as_deref(), Some(email.as_str()));
    assert_eq!(
        status.avatar_url.as_deref(),
        Some("https://avatars.example/me.png")
    );
}

#[then("the secret store holds a Todoist connection")]
fn holds(world: &mut SyncWorld) {
    assert!(
        world
            .todoist
            .secrets
            .get("nunc", ACCOUNT)
            .unwrap()
            .is_some()
    );
}

#[then("the secret store holds no Todoist connection")]
fn holds_nothing(world: &mut SyncWorld) {
    assert!(
        world
            .todoist
            .secrets
            .get("nunc", ACCOUNT)
            .unwrap()
            .is_none()
    );
}

#[then(expr = "Todoist issued {int} tokens by renewal")]
fn renewals(world: &mut SyncWorld, count: usize) {
    let renewals = server(world).with(|state| {
        state
            .token_requests
            .iter()
            .filter(|request| request["grant_type"] == "refresh_token")
            .count()
    });
    assert_eq!(renewals, count);
}

#[then("the secret store holds a Todoist connection that never expires")]
fn holds_fixed(world: &mut SyncWorld) {
    let connection = vault(world).load().unwrap().expect("nicht verbunden");
    assert_eq!(connection.tokens.expires_at, None);
    assert_eq!(connection.tokens.refresh_token, None);
}

#[then(expr = "Todoist received {string} with the bearer token {string}")]
fn received_with_bearer(world: &mut SyncWorld, request: String, token: String) {
    let (method, path) = request.split_once(' ').expect("METHODE PFAD");
    let found = server(world).with(|state| {
        state.log.iter().any(|logged| {
            logged.method == method
                && logged.path == path
                && logged.bearer.as_deref() == Some(token.as_str())
        })
    });
    assert!(found, "{request} mit {token} fehlt");
}

#[then(expr = "every Todoist request carried the bearer token {string}")]
fn every_bearer(world: &mut SyncWorld, token: String) {
    let bearers = server(world).with(|state| {
        state
            .log
            .iter()
            .map(|logged| logged.bearer.clone())
            .collect::<Vec<_>>()
    });
    assert!(!bearers.is_empty());
    assert!(
        bearers.iter().all(|b| b.as_deref() == Some(token.as_str())),
        "{bearers:?}"
    );
}

#[then("Todoist was not contacted")]
fn not_contacted(world: &mut SyncWorld) {
    assert!(server(world).with(|state| state.log.is_empty() && state.token_requests.is_empty()));
}

#[then("Todoist has the active tasks:")]
fn active_tasks(world: &mut SyncWorld, step: &Step) {
    let expected = rows(step);
    server(world).with(|state| {
        let actual: Vec<_> = state.tasks.iter().filter(|t| !t.closed).collect();
        assert_eq!(
            actual.len(),
            expected.len(),
            "aktive Aufgaben: {:?}",
            actual.iter().map(|t| &t.content).collect::<Vec<_>>()
        );
        for (task, row) in actual.iter().zip(&expected) {
            assert_eq!(task.content, row["content"]);
            if let Some(priority) = row.get("priority") {
                assert_eq!(task.priority.to_string(), *priority, "{}", task.content);
            }
            if let Some(project) = row.get("project") {
                assert_eq!(
                    state.project_name(&task.project_id),
                    *project,
                    "{}",
                    task.content
                );
            }
            if let Some(labels) = row.get("labels") {
                assert_eq!(task.labels, words(labels), "{}", task.content);
            }
        }
    });
}

#[then(expr = "Todoist closed the task {string}")]
fn closed(world: &mut SyncWorld, content: String) {
    assert!(
        server(world).with(|state| state.tasks.iter().any(|t| t.content == content && t.closed))
    );
}

#[given(expr = "the Todoist task {string} was closed")]
fn was_closed(world: &mut SyncWorld, content: String) {
    server(world).with(|state| {
        state
            .tasks
            .iter_mut()
            .find(|t| t.content == content)
            .unwrap_or_else(|| panic!("Aufgabe {content:?} fehlt"))
            .closed = true;
    });
}

#[then(expr = "Todoist deleted nothing")]
fn deleted_nothing(world: &mut SyncWorld) {
    assert_eq!(server(world).with(|state| state.count("DELETE", "")), 0);
}

#[then(expr = "Todoist received {int} create, {int} update and {int} close requests")]
fn request_counts(world: &mut SyncWorld, create: usize, update: usize, close: usize) {
    server(world).with(|state| {
        let creates = state.count("POST", "/tasks");
        let closes = state.count("POST", "/close");
        let updates = state
            .log
            .iter()
            .filter(|l| {
                l.method == "POST"
                    && l.path.starts_with("/api/v1/tasks/")
                    && !l.path.ends_with("/close")
            })
            .count();
        assert_eq!((creates, updates, closes), (create, update, close));
    });
}

#[then(expr = "Todoist knows the project {string} once")]
fn project_once(world: &mut SyncWorld, name: String) {
    assert_eq!(
        server(world).with(|state| state.projects.iter().filter(|(_, n)| *n == name).count()),
        1
    );
}

#[then(expr = "Todoist knows the label {string} once")]
fn label_once(world: &mut SyncWorld, name: String) {
    assert_eq!(
        server(world).with(|state| state.labels.iter().filter(|(_, n)| *n == name).count()),
        1
    );
}

#[then(expr = "Todoist received {int} requests for projects and labels")]
fn catalog_requests(world: &mut SyncWorld, count: usize) {
    let actual = server(world).with(|state| {
        state
            .log
            .iter()
            .filter(|l| l.path.ends_with("/projects") || l.path.ends_with("/labels"))
            .count()
    });
    assert_eq!(actual, count);
}

#[then("the local todos are mapped to the Todoist tasks:")]
fn local_mapped(world: &mut SyncWorld, step: &Step) {
    let conn = world.store.conn.lock().unwrap();
    let mappings = nunc_store::list_todoist_mappings(&conn).unwrap();
    let todos = nunc_store::list_local_todos(&conn).unwrap();
    let expected = rows(step);
    let task_content = |task_id: &str| {
        server(world).with(|state| {
            state
                .tasks
                .iter()
                .find(|t| t.id == task_id)
                .map(|t| t.content.clone())
                .unwrap_or_else(|| format!("?{task_id}"))
        })
    };
    let mut actual: Vec<(String, String)> = mappings
        .iter()
        .map(|m| {
            let text = todos
                .iter()
                .find(|l| l.todo.id == m.nunc_todo_id)
                .map(|l| l.todo.raw_text.clone())
                .unwrap_or_default();
            (text, task_content(&m.todoist_task_id))
        })
        .collect();
    actual.sort();
    let mut expected: Vec<(String, String)> = expected
        .iter()
        .map(|row| (row["todo"].clone(), row["task"].clone()))
        .collect();
    expected.sort();
    assert_eq!(actual, expected);
}

#[then(expr = "there are {int} Todoist mappings")]
fn mapping_count(world: &mut SyncWorld, count: usize) {
    let conn = world.store.conn.lock().unwrap();
    assert_eq!(
        nunc_store::list_todoist_mappings(&conn).unwrap().len(),
        count
    );
}

#[then(expr = "the local todo {string} has the change time {string}")]
fn change_time(world: &mut SyncWorld, text: String, time: String) {
    let conn = world.store.conn.lock().unwrap();
    let local = nunc_store::list_local_todos(&conn)
        .unwrap()
        .into_iter()
        .find(|l| l.todo.raw_text == text)
        .unwrap_or_else(|| panic!("Todo {text:?} fehlt"));
    assert_eq!(
        local
            .updated_at
            .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        Some(time)
    );
}

#[then(expr = "the last Todoist sync time in the settings is {string}")]
fn last_sync(world: &mut SyncWorld, time: String) {
    let conn = world.store.conn.lock().unwrap();
    assert_eq!(
        nunc_store::todoist_last_sync_at(&conn).unwrap().as_deref(),
        Some(time.as_str())
    );
}

#[then(expr = "within {int} ms Todoist holds the task {string}")]
async fn eventually(world: &mut SyncWorld, within: u64, content: String) {
    let deadline = std::time::Instant::now() + Duration::from_millis(within);
    loop {
        if server(world).with(|state| state.tasks.iter().any(|t| t.content == content)) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Aufgabe {content:?} erschien nicht"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
