//! Todoist-REST-Client (API v1, ADR 0009; Go: `services/todoist/client.go`).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use nunc_core::todoist_mapping::TodoistTask;
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

use super::TodoistConfig;
use crate::SyncError;
use crate::oauth::TokenSource;

const SERVICE: &str = "Todoist";

/// Aufgabe, wie die API sie liefert. Namen und Formen schwanken zwischen API-Versionen: `checked` oder
/// `is_completed`, `added_at` oder `created_at`.
#[derive(Debug, Deserialize)]
struct RawTask {
    id: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    priority: u8,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    checked: Option<bool>,
    #[serde(default)]
    is_completed: Option<bool>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    added_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

fn instant(text: &Option<String>) -> Option<DateTime<Utc>> {
    text.as_deref()
        .and_then(|text| DateTime::parse_from_rfc3339(text).ok())
        .map(|time| time.with_timezone(&Utc))
}

impl From<RawTask> for TodoistTask {
    fn from(raw: RawTask) -> Self {
        Self {
            created_at: instant(&raw.created_at).or_else(|| instant(&raw.added_at)),
            updated_at: instant(&raw.updated_at),
            is_completed: raw.checked.or(raw.is_completed).unwrap_or(false),
            id: raw.id,
            content: raw.content,
            priority: raw.priority,
            project_id: raw.project_id.unwrap_or_default(),
            labels: raw.labels,
        }
    }
}

/// Projekt oder Label (Id und Name).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Named {
    pub id: String,
    pub name: String,
}

/// Konto des angemeldeten Nutzers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// Inhalt einer Anlege- oder Änderungsanfrage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskPayload {
    pub content: String,
    pub priority: u8,
    pub project_id: Option<String>,
    pub labels: Vec<String>,
}

impl TaskPayload {
    /// Beim Anlegen entfallen leere Angaben; beim Ändern wird eine leere Label-Liste gesendet, damit Labels
    /// auch verschwinden können.
    fn to_json(&self, for_update: bool) -> Value {
        let mut body = json!({ "content": self.content, "priority": self.priority });
        if let Some(project) = &self.project_id {
            body["project_id"] = json!(project);
        }
        if for_update || !self.labels.is_empty() {
            body["labels"] = json!(self.labels);
        }
        body
    }
}

pub struct TodoistClient {
    http: reqwest::Client,
    base: String,
    tokens: Arc<TokenSource>,
}

#[derive(Deserialize)]
struct Page<T> {
    #[serde(default = "Vec::new", alias = "items")]
    results: Vec<T>,
    #[serde(default)]
    next_cursor: Option<String>,
}

impl TodoistClient {
    pub fn new(http: reqwest::Client, config: &TodoistConfig, tokens: Arc<TokenSource>) -> Self {
        Self {
            http,
            base: config.api_base.trim_end_matches('/').to_owned(),
            tokens,
        }
    }

    async fn dispatch(
        &self,
        build: &impl Fn(&reqwest::Client) -> RequestBuilder,
        token: &str,
    ) -> Result<Response, SyncError> {
        build(&self.http)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|error| SyncError::Transport(crate::webdav::describe_transport_error(&error)))
    }

    /// Sendet mit dem aktuellen Token; lehnt Todoist es ab (`401`), wird es einmal erneuert.
    async fn send(
        &self,
        action: &'static str,
        build: impl Fn(&reqwest::Client) -> RequestBuilder,
    ) -> Result<Response, SyncError> {
        let token = self.tokens.access_token().await?;
        let mut response = self.dispatch(&build, &token).await?;
        if response.status() == StatusCode::UNAUTHORIZED {
            let renewed = self.tokens.renew(&token).await?;
            response = self.dispatch(&build, &renewed).await?;
        }
        match response.status() {
            status if status.is_success() => Ok(response),
            StatusCode::UNAUTHORIZED => Err(SyncError::AuthExpired(SERVICE)),
            StatusCode::FORBIDDEN => Err(SyncError::Forbidden),
            status => Err(SyncError::Status {
                status: status.as_u16(),
                action,
            }),
        }
    }

    async fn json<T: for<'de> Deserialize<'de>>(
        response: Response,
        action: &'static str,
    ) -> Result<T, SyncError> {
        response.json().await.map_err(|_| SyncError::Status {
            status: 200,
            action,
        })
    }

    /// Alle Seiten einer Liste.
    async fn list<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        action: &'static str,
    ) -> Result<Vec<T>, SyncError> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let url = format!("{}{path}", self.base);
            let response = self
                .send(action, |http| {
                    let request = http.get(&url);
                    match &cursor {
                        Some(cursor) => request.query(&[("cursor", cursor.as_str())]),
                        None => request,
                    }
                })
                .await?;
            let page: Page<T> = Self::json(response, action).await?;
            all.extend(page.results);
            match page.next_cursor.filter(|cursor| !cursor.is_empty()) {
                Some(next) => cursor = Some(next),
                None => return Ok(all),
            }
        }
    }

    /// Alle aktiven (nicht erledigten) Aufgaben.
    pub async fn tasks(&self) -> Result<Vec<TodoistTask>, SyncError> {
        let raw: Vec<RawTask> = self.list("/tasks", "Laden der Aufgaben").await?;
        Ok(raw.into_iter().map(TodoistTask::from).collect())
    }

    pub async fn projects(&self) -> Result<Vec<Named>, SyncError> {
        self.list("/projects", "Laden der Projekte").await
    }

    pub async fn labels(&self) -> Result<Vec<Named>, SyncError> {
        self.list("/labels", "Laden der Labels").await
    }

    pub async fn create_project(&self, name: &str) -> Result<Named, SyncError> {
        self.create_named("/projects", name, "Anlegen des Projekts")
            .await
    }

    pub async fn create_label(&self, name: &str) -> Result<Named, SyncError> {
        self.create_named("/labels", name, "Anlegen des Labels")
            .await
    }

    async fn create_named(
        &self,
        path: &str,
        name: &str,
        action: &'static str,
    ) -> Result<Named, SyncError> {
        let url = format!("{}{path}", self.base);
        let body = json!({ "name": name });
        let response = self
            .send(action, |http| http.post(&url).json(&body))
            .await?;
        Self::json(response, action).await
    }

    /// Legt eine Aufgabe an und liefert ihre Kennung.
    pub async fn create_task(&self, payload: &TaskPayload) -> Result<String, SyncError> {
        let url = format!("{}/tasks", self.base);
        let body = payload.to_json(false);
        let response = self
            .send("Anlegen der Aufgabe", |http| http.post(&url).json(&body))
            .await?;
        let task: Created = Self::json(response, "Anlegen der Aufgabe").await?;
        Ok(task.id)
    }

    pub async fn update_task(&self, id: &str, payload: &TaskPayload) -> Result<(), SyncError> {
        let url = format!("{}/tasks/{}", self.base, encode(id));
        let body = payload.to_json(true);
        self.send("Ändern der Aufgabe", |http| http.post(&url).json(&body))
            .await?;
        Ok(())
    }

    /// Schließt eine Aufgabe. Gibt es sie nicht mehr (`404`), ist das Ziel erreicht.
    pub async fn close_task(&self, id: &str) -> Result<(), SyncError> {
        let url = format!("{}/tasks/{}/close", self.base, encode(id));
        match self
            .send("Schließen der Aufgabe", |http| http.post(&url))
            .await
        {
            Err(SyncError::Status { status: 404, .. }) => Ok(()),
            other => other.map(|_| ()),
        }
    }

    /// E-Mail-Adresse und Avatar des Kontos.
    pub async fn account(&self) -> Result<Account, SyncError> {
        #[derive(Deserialize)]
        struct User {
            email: Option<String>,
            avatar_small: Option<String>,
            avatar_url: Option<String>,
        }
        let url = format!("{}/user", self.base);
        let response = self.send("Lesen des Kontos", |http| http.get(&url)).await?;
        let user: User = Self::json(response, "Lesen des Kontos").await?;
        Ok(Account {
            email: user.email.filter(|text| !text.is_empty()),
            avatar_url: user
                .avatar_small
                .or(user.avatar_url)
                .filter(|text| !text.is_empty()),
        })
    }
}

#[derive(Deserialize)]
struct Created {
    id: String,
}

fn encode(segment: &str) -> String {
    segment
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_task_dialects_are_understood() {
        let v1: RawTask = serde_json::from_value(json!({
            "id": "1", "content": "A", "priority": 4, "project_id": "p", "labels": ["x"],
            "checked": true, "added_at": "2026-10-01T08:00:00.123456Z", "updated_at": "2026-10-02T08:00:00Z"
        }))
        .unwrap();
        let task = TodoistTask::from(v1);
        assert!(task.is_completed);
        assert_eq!(
            task.created_at.map(|t| t.to_rfc3339()),
            Some("2026-10-01T08:00:00.123456+00:00".to_owned())
        );
        assert!(task.updated_at.is_some());
        assert_eq!(task.project_id, "p");

        let old: RawTask = serde_json::from_value(json!({
            "id": "2", "content": "B", "priority": 1, "is_completed": true,
            "created_at": "2026-10-01T08:00:00Z"
        }))
        .unwrap();
        let task = TodoistTask::from(old);
        assert!(task.is_completed && task.project_id.is_empty() && task.labels.is_empty());
        assert!(task.created_at.is_some() && task.updated_at.is_none());

        let open: RawTask =
            serde_json::from_value(json!({ "id": "3", "created_at": "kaputt" })).unwrap();
        let task = TodoistTask::from(open);
        assert!(!task.is_completed && task.created_at.is_none());
    }

    #[test]
    fn a_create_payload_drops_empty_values_and_an_update_clears_labels() {
        let payload = TaskPayload {
            content: "Text".into(),
            priority: 3,
            project_id: None,
            labels: vec![],
        };
        assert_eq!(
            payload.to_json(false),
            json!({ "content": "Text", "priority": 3 })
        );
        assert_eq!(
            payload.to_json(true),
            json!({ "content": "Text", "priority": 3, "labels": [] })
        );
        let full = TaskPayload {
            project_id: Some("p1".into()),
            labels: vec!["a".into()],
            ..payload
        };
        assert_eq!(
            full.to_json(false),
            json!({ "content": "Text", "priority": 3, "project_id": "p1", "labels": ["a"] })
        );
    }

    #[test]
    fn ids_are_escaped_in_paths() {
        assert_eq!(encode("abc-1_2.3~"), "abc-1_2.3~");
        assert_eq!(encode("a/b c"), "a%2Fb%20c");
    }

    #[test]
    fn pages_accept_results_or_items() {
        let a: Page<Named> = serde_json::from_value(
            json!({ "results": [{"id":"1","name":"x"}], "next_cursor": "c" }),
        )
        .unwrap();
        assert_eq!(a.results.len(), 1);
        let b: Page<Named> = serde_json::from_value(
            json!({ "items": [{"id":"1","name":"x"}], "next_cursor": null }),
        )
        .unwrap();
        assert_eq!(b.results.len(), 1);
        assert!(b.next_cursor.is_none());
    }
}
