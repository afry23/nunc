//! Ein kleines „Todoist“ für die Szenarien (wiremock): Token-Endpunkt, Aufgaben, Projekte, Labels, Konto,
//! Seitenweise Listen und ein Protokoll aller Anfragen.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use wiremock::{MockServer, Request, Respond, ResponseTemplate};

#[derive(Debug, Clone)]
pub struct TaskRec {
    pub id: String,
    pub content: String,
    pub priority: u8,
    pub project_id: Option<String>,
    pub labels: Vec<String>,
    pub closed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct Logged {
    pub method: String,
    pub path: String,
    pub body: Value,
}

pub struct TodoistState {
    pub tasks: Vec<TaskRec>,
    pub projects: Vec<(String, String)>,
    pub labels: Vec<(String, String)>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub valid: BTreeSet<String>,
    token_counter: u32,
    pub token_requests: Vec<BTreeMap<String, String>>,
    pub reject_code: Option<String>,
    pub reject_refresh: bool,
    /// `None`: Tokens ohne Ablauf (ältere Apps).
    pub expires_in: Option<i64>,
    pub fail_all: Option<u16>,
    pub page_size: usize,
    /// Zeitstempel, den Todoist bei Änderungen vergibt.
    pub now: String,
    next_id: u32,
    pub log: Vec<Logged>,
}

impl TodoistState {
    fn new_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}-{}", self.next_id)
    }

    pub fn project_id(&mut self, name: &str) -> String {
        if let Some((id, _)) = self.projects.iter().find(|(_, n)| n == name) {
            return id.clone();
        }
        let id = self.new_id("proj");
        self.projects.push((id.clone(), name.to_owned()));
        id
    }

    pub fn add_task(
        &mut self,
        content: &str,
        priority: u8,
        project: Option<&str>,
        labels: Vec<String>,
        updated_at: Option<&str>,
    ) -> String {
        let id = self.new_id("task");
        let project_id = project.map(|name| self.project_id(name));
        let now = self.now.clone();
        self.tasks.push(TaskRec {
            id: id.clone(),
            content: content.to_owned(),
            priority,
            project_id,
            labels,
            closed: false,
            created_at: now.clone(),
            updated_at: updated_at.map_or(now, str::to_owned),
        });
        id
    }

    pub fn project_name(&self, id: &Option<String>) -> String {
        id.as_ref()
            .and_then(|id| self.projects.iter().find(|(pid, _)| pid == id))
            .map(|(_, name)| name.clone())
            .unwrap_or_default()
    }

    pub fn count(&self, method: &str, path_suffix: &str) -> usize {
        self.log
            .iter()
            .filter(|l| l.method == method && l.path.ends_with(path_suffix))
            .count()
    }
}

pub struct Todoist {
    /// Wird absichtlich nie freigegeben: Das Freigeben eines `MockServer` wartet auf alle offenen Verbindungen und
    /// hängt, wenn ein Szenario mitten im Schritt scheitert (der Mutationstest sähe dann nur Zeitüberschreitungen).
    pub server: std::mem::ManuallyDrop<MockServer>,
    pub state: Arc<Mutex<TodoistState>>,
}

impl std::fmt::Debug for Todoist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Todoist")
    }
}

impl Todoist {
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        let state = Arc::new(Mutex::new(TodoistState {
            tasks: Vec::new(),
            projects: Vec::new(),
            labels: Vec::new(),
            email: Some("me@example.org".to_owned()),
            avatar: Some("https://avatars.example/me.png".to_owned()),
            valid: BTreeSet::new(),
            token_counter: 0,
            token_requests: Vec::new(),
            reject_code: None,
            reject_refresh: false,
            expires_in: Some(3600),
            fail_all: None,
            page_size: 100,
            now: "2026-10-04T09:30:00Z".to_owned(),
            next_id: 0,
            log: Vec::new(),
        }));
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(Responder(state.clone()))
            .mount(&server)
            .await;
        Self {
            server: std::mem::ManuallyDrop::new(server),
            state,
        }
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut TodoistState) -> T) -> T {
        f(&mut self.state.lock().unwrap())
    }
}

struct Responder(Arc<Mutex<TodoistState>>);

fn json_response(status: u16, value: Value) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(value)
}

fn task_json(task: &TaskRec) -> Value {
    json!({
        "id": task.id, "content": task.content, "priority": task.priority,
        "project_id": task.project_id, "labels": task.labels, "checked": task.closed,
        "added_at": task.created_at, "updated_at": task.updated_at,
    })
}

fn form_of(body: &[u8]) -> BTreeMap<String, String> {
    reqwest::Url::parse(&format!("http://x/?{}", String::from_utf8_lossy(body)))
        .map(|url| {
            url.query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect()
        })
        .unwrap_or_default()
}

impl Respond for Responder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let mut state = self.0.lock().unwrap();
        let method = request.method.to_string();
        let path = request.url.path().to_owned();
        let body: Value = serde_json::from_slice(&request.body).unwrap_or(Value::Null);
        let bearer = request
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::to_owned);
        let response = answer(
            &mut state,
            request,
            &method,
            &path,
            &body,
            bearer.as_deref(),
        );
        state.log.push(Logged { method, path, body });
        response
    }
}

fn page(state: &TodoistState, request: &Request, items: Vec<Value>) -> ResponseTemplate {
    let start: usize = request
        .url
        .query_pairs()
        .find(|(k, _)| k == "cursor")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);
    let end = (start + state.page_size).min(items.len());
    let next = (end < items.len()).then(|| end.to_string());
    json_response(
        200,
        json!({ "results": items[start..end], "next_cursor": next }),
    )
}

fn answer(
    state: &mut TodoistState,
    request: &Request,
    method: &str,
    path: &str,
    body: &Value,
    bearer: Option<&str>,
) -> ResponseTemplate {
    if path == "/oauth/access_token" {
        return token(state, request);
    }
    if let Some(status) = state.fail_all {
        return ResponseTemplate::new(status);
    }
    if !bearer.is_some_and(|token| state.valid.contains(token)) {
        return ResponseTemplate::new(401);
    }
    let Some(rest) = path.strip_prefix("/api/v1") else {
        return ResponseTemplate::new(404);
    };
    match (method, rest) {
        ("GET", "/user") => json_response(
            200,
            json!({ "email": state.email, "avatar_small": state.avatar }),
        ),
        ("GET", "/tasks") => {
            let items = state
                .tasks
                .iter()
                .filter(|t| !t.closed)
                .map(task_json)
                .collect();
            page(state, request, items)
        }
        ("POST", "/tasks") => {
            let labels = body["labels"]
                .as_array()
                .map(|l| {
                    l.iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            let id = state.add_task(
                body["content"].as_str().unwrap_or_default(),
                u8::try_from(body["priority"].as_u64().unwrap_or(1)).unwrap_or(1),
                None,
                labels,
                None,
            );
            if let Some(task) = state.tasks.iter_mut().find(|t| t.id == id) {
                task.project_id = body["project_id"].as_str().map(str::to_owned);
            }
            json_response(200, json!({ "id": id }))
        }
        ("GET", "/projects") => {
            let items = state
                .projects
                .iter()
                .map(|(id, name)| json!({ "id": id, "name": name }))
                .collect();
            page(state, request, items)
        }
        ("POST", "/projects") => {
            let name = body["name"].as_str().unwrap_or_default().to_owned();
            let id = state.project_id(&name);
            json_response(200, json!({ "id": id, "name": name }))
        }
        ("GET", "/labels") => {
            let items = state
                .labels
                .iter()
                .map(|(id, name)| json!({ "id": id, "name": name }))
                .collect();
            page(state, request, items)
        }
        ("POST", "/labels") => {
            let name = body["name"].as_str().unwrap_or_default().to_owned();
            let id = state.new_id("label");
            state.labels.push((id.clone(), name.clone()));
            json_response(200, json!({ "id": id, "name": name }))
        }
        ("POST", p) if p.starts_with("/tasks/") => {
            let rest = p.trim_start_matches("/tasks/");
            let now = state.now.clone();
            if let Some(id) = rest.strip_suffix("/close") {
                return match state.tasks.iter_mut().find(|t| t.id == id) {
                    Some(task) => {
                        task.closed = true;
                        task.updated_at = now;
                        ResponseTemplate::new(204)
                    }
                    None => ResponseTemplate::new(404),
                };
            }
            match state.tasks.iter_mut().find(|t| t.id == rest) {
                Some(task) => {
                    if let Some(content) = body["content"].as_str() {
                        content.clone_into(&mut task.content);
                    }
                    if let Some(priority) = body["priority"].as_u64() {
                        task.priority = u8::try_from(priority).unwrap_or(1);
                    }
                    if let Some(project) = body["project_id"].as_str() {
                        task.project_id = Some(project.to_owned());
                    }
                    if let Some(labels) = body["labels"].as_array() {
                        task.labels = labels
                            .iter()
                            .filter_map(|v| v.as_str().map(str::to_owned))
                            .collect();
                    }
                    task.updated_at = now;
                    json_response(200, task_json(task))
                }
                None => ResponseTemplate::new(404),
            }
        }
        _ => ResponseTemplate::new(404),
    }
}

fn token(state: &mut TodoistState, request: &Request) -> ResponseTemplate {
    let form = form_of(&request.body);
    state.token_requests.push(form.clone());
    let rejection = match form.get("grant_type").map(String::as_str) {
        Some("authorization_code") => state.reject_code.clone(),
        Some("refresh_token") => state.reject_refresh.then(|| "invalid_grant".to_owned()),
        _ => Some("unsupported_grant_type".to_owned()),
    };
    if let Some(error) = rejection {
        return json_response(400, json!({ "error": error }));
    }
    state.token_counter += 1;
    let access = format!("todoist-access-{}", state.token_counter);
    state.valid.insert(access.clone());
    let mut body = json!({
        "access_token": access,
        "refresh_token": format!("todoist-refresh-{}", state.token_counter),
        "token_type": "Bearer",
    });
    if let Some(expires_in) = state.expires_in {
        body["expires_in"] = json!(expires_in);
    }
    json_response(200, body)
}
