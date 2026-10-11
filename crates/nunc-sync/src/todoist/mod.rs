//! Synchronisierung der Todos mit Todoist (Go: `services/todoist/sync.go`): Anmeldung mit dem persönlichen
//! API-Token (ADR 0020), Lauf und Verbindung. Die Merge-Regeln sind pur (`nunc_core::todoist_mapping`); die
//! Datenbank sieht dieses Modul nur über [`TodoistStore`], damit nie eine Sperre über einen Netzwerkzugriff hinweg
//! gehalten wird.

mod client;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};
use nunc_core::clock::Clock;
use nunc_core::model::{SyncResult, TodoistStatus};
use nunc_core::todoist_mapping::{
    LocalChange, LocalTodo, TodoistMapping, map_todo_to_task, merge_todoist, same_on_todoist,
};
use nunc_core::todotxt::{ParsedTodo, build_raw_text};
use nunc_platform::{MemorySecretStore, SecretStore, to_local_date};
use nunc_store::{StoreError, TodoistApplied, TodoistWrite};

pub use client::{Account, Named, TaskPayload, TodoistClient};

use crate::SyncError;
use crate::coordinator::{RunFuture, SyncRunner};
use crate::oauth::{Connection, TokenSource, Tokens, Vault, http_client};

/// Eintrag im Tresor.
pub const SECRET_ACCOUNT: &str = "todoist";
const SERVICE: &str = "Todoist";

/// Adresse der API; Tests setzen sie auf einen Testserver.
#[derive(Debug, Clone)]
pub struct TodoistConfig {
    pub api_base: String,
}

impl TodoistConfig {
    pub fn todoist() -> Self {
        Self {
            api_base: "https://api.todoist.com/api/v1".to_owned(),
        }
    }
}

/// Die lokale Datenbank aus Sicht des Todoist-Syncs. Die Methoden sind synchron und halten Sperren nur kurz.
pub trait TodoistStore: Send + Sync {
    fn local_todos(&self) -> Result<Vec<LocalTodo>, StoreError>;
    fn mappings(&self) -> Result<Vec<TodoistMapping>, StoreError>;
    /// Verknüpft ein Todo sofort mit der neu angelegten Aufgabe (auch wenn der Lauf danach scheitert, entsteht so
    /// beim nächsten Lauf kein Duplikat).
    fn add_mapping(&self, todo_id: i32, task_id: &str, at: DateTime<Utc>)
    -> Result<(), StoreError>;
    fn remove_mappings(&self, task_ids: &[String]) -> Result<(), StoreError>;
    /// Spielt Änderungen aus Todoist atomar ein. `sync_id` für neue Todos vergibt der Aufrufer.
    fn apply(
        &self,
        writes: &[TodoistWrite],
        today: NaiveDate,
        now: DateTime<Utc>,
    ) -> Result<TodoistApplied, StoreError>;
    fn record_sync(&self, at: DateTime<Utc>) -> Result<(), StoreError>;
    /// Neue Kennung für ein importiertes Todo.
    fn new_sync_id(&self) -> String;
}

/// Verbindung zu Todoist: anmelden, abmelden, Zustand, Sync-Läufe.
pub struct TodoistService {
    config: TodoistConfig,
    vault: Vault,
    clock: Arc<dyn Clock>,
    http: reqwest::Client,
}

impl TodoistService {
    pub fn new(
        config: TodoistConfig,
        secrets: Arc<dyn SecretStore>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, SyncError> {
        Ok(Self {
            config,
            vault: Vault::new(secrets, SECRET_ACCOUNT),
            clock,
            http: http_client()?,
        })
    }

    pub fn is_connected(&self) -> Result<bool, SyncError> {
        Ok(self.vault.load_lenient().is_some())
    }

    /// Zustand der Verbindung (ohne Netzwerkzugriff).
    pub fn status(&self) -> Result<TodoistStatus, SyncError> {
        let connection = self.vault.load_lenient();
        Ok(TodoistStatus {
            connected: connection.is_some(),
            email: connection.as_ref().and_then(|c| c.email.clone()),
            avatar_url: connection.and_then(|c| c.avatar_url),
        })
    }

    /// Ein Client für das Token in `vault`. Todoist erneuert persönliche Tokens nicht; eine alte OAuth-Anmeldung
    /// (mit Refresh-Token) wird ebenfalls nicht mehr erneuert, es gibt keinen Client dafür.
    fn client_for(&self, vault: Vault) -> TodoistClient {
        let tokens = Arc::new(TokenSource::fixed(
            vault,
            SERVICE,
            self.http.clone(),
            self.clock.clone(),
        ));
        TodoistClient::new(self.http.clone(), &self.config, tokens)
    }

    fn client(&self) -> Result<TodoistClient, SyncError> {
        if !self.is_connected()? {
            return Err(SyncError::NotConnected(SERVICE));
        }
        Ok(self.client_for(self.vault.clone()))
    }

    /// Verbindet mit dem persönlichen API-Token: prüft es über das Konto (E-Mail, Avatar) und legt es erst danach
    /// im Tresor ab. Ein abgelehntes Token ändert nichts an einer bestehenden Verbindung.
    pub async fn connect_token(&self, token: &str) -> Result<TodoistStatus, SyncError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(SyncError::TodoistTokenRejected);
        }
        let mut connection = Connection {
            tokens: Tokens {
                access_token: token.to_owned(),
                refresh_token: None,
                expires_at: None,
            },
            email: None,
            avatar_url: None,
        };
        let trial = Vault::new(Arc::new(MemorySecretStore::default()), SECRET_ACCOUNT);
        trial.save(&connection)?;
        let account = match self.client_for(trial).account().await {
            Ok(account) => account,
            Err(SyncError::AuthExpired(_) | SyncError::Forbidden) => {
                return Err(SyncError::TodoistTokenRejected);
            }
            Err(error) => return Err(error),
        };
        connection.email = account.email;
        connection.avatar_url = account.avatar_url;
        self.vault.save(&connection)?;
        self.status()
    }

    /// Löscht die Tokens lokal; lokale Todos und Zuordnungen bleiben.
    pub fn disconnect(&self) -> Result<(), SyncError> {
        self.vault.delete()
    }

    pub async fn sync(&self, store: &dyn TodoistStore) -> Result<SyncResult, SyncError> {
        let client = self.client()?;
        sync_todoist(&client, store, &*self.clock)
            .await
            .map_err(|error| match error {
                SyncError::AuthExpired(_) => SyncError::TodoistTokenInvalid,
                other => other,
            })
    }
}

/// Die Todoist-Version als Todo (für den Vergleich).
fn as_todo(id: i32, parsed: &ParsedTodo) -> nunc_core::model::Todo {
    nunc_core::model::Todo {
        id,
        raw_text: build_raw_text(parsed),
        priority: parsed.priority,
        completed: parsed.completed,
        completed_at: parsed.completed_at,
        created_at: parsed.created_at,
        description: parsed.description.clone(),
        project_tags: parsed.project_tags.clone(),
        context_tags: parsed.context_tags.clone(),
        pomodoro_count: 0,
        position: 0,
    }
}

fn instant_or(time: Option<DateTime<Utc>>, fallback: DateTime<Utc>) -> DateTime<Utc> {
    time.unwrap_or(fallback)
}

/// Projekt- oder Label-Kennung zu einem Namen, angelegt, wenn es fehlt. Der Vergleich ignoriert die
/// Großschreibung.
struct Lookup {
    by_name: HashMap<String, Named>,
}

impl Lookup {
    fn new(items: Vec<Named>) -> Self {
        Self {
            by_name: items
                .into_iter()
                .map(|item| (item.name.to_lowercase(), item))
                .collect(),
        }
    }

    fn find(&self, name: &str) -> Option<&Named> {
        self.by_name.get(&name.to_lowercase())
    }

    fn insert(&mut self, item: Named) {
        self.by_name.insert(item.name.to_lowercase(), item);
    }
}

struct Resolver<'a> {
    client: &'a TodoistClient,
    projects: Lookup,
    labels: Lookup,
}

impl Resolver<'_> {
    /// Die Anfrage an Todoist für ein Todo; fehlende Projekte und Labels werden angelegt.
    async fn payload(&mut self, todo: &nunc_core::model::Todo) -> Result<TaskPayload, SyncError> {
        let request = map_todo_to_task(todo);
        let project_id = match request
            .project_name
            .as_deref()
            .filter(|name| !name.is_empty())
        {
            Some(name) => Some(match self.projects.find(name) {
                Some(project) => project.id.clone(),
                None => {
                    let project = self.client.create_project(name).await?;
                    let id = project.id.clone();
                    self.projects.insert(project);
                    id
                }
            }),
            None => None,
        };
        let mut labels = Vec::new();
        for name in request.labels.iter().filter(|name| !name.is_empty()) {
            let label = match self.labels.find(name) {
                Some(label) => label.name.clone(),
                None => {
                    let label = self.client.create_label(name).await?;
                    let created = label.name.clone();
                    self.labels.insert(label);
                    created
                }
            };
            labels.push(label);
        }
        Ok(TaskPayload {
            content: request.content,
            priority: request.priority,
            project_id,
            labels,
        })
    }
}

/// Ein Lauf: laden, zusammenführen, nach Todoist schreiben, lokal einspielen. Gezählt wird, was lokal ankam
/// (`imported_todos` = neue Todos, `conflicted_todos` = durch die Todoist-Version ersetzte).
pub async fn sync_todoist(
    client: &TodoistClient,
    store: &dyn TodoistStore,
    clock: &dyn Clock,
) -> Result<SyncResult, SyncError> {
    let tasks = client.tasks().await?;
    let projects = client.projects().await?;
    let labels = client.labels().await?;
    let locals = store.local_todos()?;
    let mappings = store.mappings()?;
    let now = clock.now();
    let today = to_local_date(now);

    let projects_by_id: HashMap<String, String> = projects
        .iter()
        .map(|project| (project.id.clone(), project.name.clone()))
        .collect();
    let merge = merge_todoist(&locals, &tasks, &mappings, &projects_by_id, today);
    let mut resolver = Resolver {
        client,
        projects: Lookup::new(projects),
        labels: Lookup::new(labels),
    };

    for todo in &merge.to_create_in_todoist {
        let payload = resolver.payload(todo).await?;
        let task_id = client.create_task(&payload).await?;
        store.add_mapping(todo.id, &task_id, now)?;
    }
    let task_of: HashMap<i32, &str> = mappings
        .iter()
        .map(|m| (m.nunc_todo_id, m.todoist_task_id.as_str()))
        .collect();
    for todo in &merge.to_update_in_todoist {
        let Some(task_id) = task_of.get(&todo.id) else {
            continue;
        };
        let payload = resolver.payload(todo).await?;
        client.update_task(task_id, &payload).await?;
    }
    for task_id in &merge.to_close_in_todoist {
        client.close_task(task_id).await?;
    }
    // Aufgaben gelöschter Todos sind geschlossen und tauchen nicht mehr auf: die Zuordnung wird nicht mehr gebraucht
    let deleted_ids: Vec<&i32> = locals
        .iter()
        .filter(|local| local.deleted)
        .map(|local| &local.todo.id)
        .collect();
    let closed_for_deleted: Vec<String> = merge
        .to_close_in_todoist
        .iter()
        .filter(|task_id| {
            mappings
                .iter()
                .any(|m| m.todoist_task_id == **task_id && deleted_ids.contains(&&m.nunc_todo_id))
        })
        .cloned()
        .collect();
    store.remove_mappings(&closed_for_deleted)?;

    let task_time: HashMap<&str, DateTime<Utc>> = tasks
        .iter()
        .map(|task| {
            (
                task.id.as_str(),
                instant_or(task.updated_at.or(task.created_at), now),
            )
        })
        .collect();
    let mut writes: Vec<TodoistWrite> = merge
        .local_changes
        .iter()
        .map(|change| match change {
            LocalChange::Insert {
                todoist_task_id,
                todo,
            } => TodoistWrite::Insert {
                task_id: todoist_task_id.clone(),
                todo: todo.clone(),
                at: task_time
                    .get(todoist_task_id.as_str())
                    .copied()
                    .unwrap_or(now),
                sync_id: store.new_sync_id(),
            },
            LocalChange::Update { todo_id, todo } => {
                let at = task_of
                    .get(todo_id)
                    .and_then(|task_id| task_time.get(task_id))
                    .copied()
                    .unwrap_or(now);
                let unchanged = locals
                    .iter()
                    .find(|local| local.todo.id == *todo_id)
                    .is_some_and(|local| same_on_todoist(&local.todo, &as_todo(*todo_id, todo)));
                if unchanged {
                    TodoistWrite::Touch {
                        todo_id: *todo_id,
                        at,
                    }
                } else {
                    TodoistWrite::Update {
                        todo_id: *todo_id,
                        todo: todo.clone(),
                        at,
                    }
                }
            }
        })
        .collect();
    writes.extend(
        merge
            .to_complete_locally
            .iter()
            .map(|todo_id| TodoistWrite::Complete { todo_id: *todo_id }),
    );
    let applied = store.apply(&writes, today, now)?;
    Ok(SyncResult {
        imported_todos: applied.inserted,
        imported_sessions: 0,
        conflicted_todos: applied.updated + applied.completed,
    })
}

/// Der Todoist-Lauf für den [`SyncCoordinator`](crate::SyncCoordinator): läuft nur, wenn Todoist verbunden ist.
pub struct TodoistRunner {
    store: Arc<dyn TodoistStore>,
    service: Arc<TodoistService>,
    clock: Arc<dyn Clock>,
}

impl TodoistRunner {
    pub fn new(
        store: Arc<dyn TodoistStore>,
        service: Arc<TodoistService>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            store,
            service,
            clock,
        }
    }
}

impl SyncRunner for TodoistRunner {
    fn run(&self) -> RunFuture {
        let store = self.store.clone();
        let service = self.service.clone();
        let clock = self.clock.clone();
        Box::pin(async move {
            if !service.is_connected()? {
                return Ok(None);
            }
            let result = service.sync(&*store).await?;
            let finished = clock.now();
            store.record_sync(finished)?;
            Ok(Some((
                result,
                finished.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )))
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use nunc_platform::{ManualClock, MemorySecretStore, UnavailableSecretStore};

    use super::*;

    fn service(secrets: Arc<dyn SecretStore>) -> TodoistService {
        TodoistService::new(
            TodoistConfig::todoist(),
            secrets,
            Arc::new(ManualClock::new(
                Utc.with_ymd_and_hms(2026, 10, 4, 9, 0, 0).unwrap(),
            )),
        )
        .unwrap()
    }

    #[test]
    fn the_config_points_at_todoist() {
        assert_eq!(
            TodoistConfig::todoist().api_base,
            "https://api.todoist.com/api/v1"
        );
    }

    #[test]
    fn the_status_follows_the_vault() {
        let service = service(Arc::new(MemorySecretStore::default()));
        let status = service.status().unwrap();
        assert!(!status.connected);
        assert!(!service.is_connected().unwrap());
    }

    #[test]
    fn an_unreachable_secret_store_counts_as_not_connected() {
        let service = service(Arc::new(UnavailableSecretStore::new("kein Secret-Service")));
        assert!(!service.is_connected().unwrap());
        assert!(!service.status().unwrap().connected);
    }
}
