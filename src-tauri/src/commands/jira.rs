//! Commands für den Jira-Import (ADR 0038): Zugang verwalten, Tickets suchen, Ticket als Aufgabe übernehmen.

use nunc_core::jira::{JiraConfig, JiraIssue, is_imported, todo_line};
use nunc_core::model::{ConnectionTestResult, Todo, is_http_url};
use nunc_jira::{JiraClient, JiraCredentials};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Gespeicherter Zugang. Das Token verlässt den Tresor nicht: die Oberfläche erfährt nur, ob eins gespeichert ist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct JiraSettings {
    pub config: JiraConfig,
    pub has_token: bool,
}

/// Ein Treffer der Suche; `imported`, wenn es schon eine offene Aufgabe mit dem Ticket-Link gibt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueHit {
    pub issue: JiraIssue,
    pub imported: bool,
}

#[tauri::command]
#[specta::specta]
pub fn get_jira_config(state: State<'_, AppState>) -> Result<JiraSettings, AppError> {
    let config = state.db.with(|conn| nunc_store::load_jira_config(conn))?;
    let has_token = nunc_jira::has_jira_token(&*state.secrets)?;
    Ok(JiraSettings { config, has_token })
}

/// Eine leere Adresse ist erlaubt (Einstellungen leeren), sonst muss es `http(s)://host` sein.
fn validated(config: JiraConfig) -> Result<JiraConfig, AppError> {
    let base_url = config.base_url.trim().to_owned();
    if !base_url.is_empty() && !is_http_url(&base_url) {
        return Err(AppError::validation(
            "Die Jira-Adresse muss mit http:// oder https:// beginnen.",
        ));
    }
    Ok(JiraConfig {
        base_url,
        email: config.email.trim().to_owned(),
        ..config
    })
}

/// Speichert die Einstellungen in der Datenbank und das Token im Tresor. `token: None` lässt das gespeicherte
/// Token unverändert.
#[tauri::command]
#[specta::specta]
pub fn save_jira_config(
    state: State<'_, AppState>,
    config: JiraConfig,
    token: Option<String>,
) -> Result<(), AppError> {
    let config = validated(config)?;
    state
        .db
        .with(|conn| nunc_jira::save_jira(conn, &*state.secrets, &config, token.as_deref()))
}

/// Entfernt Einstellungen und Token.
#[tauri::command]
#[specta::specta]
pub fn clear_jira_config(state: State<'_, AppState>) -> Result<(), AppError> {
    state
        .db
        .with(|conn| nunc_jira::clear_jira(conn, &*state.secrets))
}

/// Prüft die Verbindung mit den übergebenen (noch nicht gespeicherten) Werten. Ohne Token gilt das gespeicherte.
/// Fehler kommen als Ergebnis zurück, nicht als `AppError` (wie beim WebDAV-Test).
#[tauri::command]
#[specta::specta]
pub async fn test_jira_connection(
    state: State<'_, AppState>,
    config: JiraConfig,
    token: Option<String>,
) -> Result<ConnectionTestResult, AppError> {
    let config = validated(config)?;
    let token = match token {
        Some(token) => token,
        None => state
            .secrets
            .get(nunc_jira::SECRET_SERVICE, nunc_jira::SECRET_ACCOUNT)?
            .unwrap_or_default(),
    };
    let outcome = if config.is_complete() && !token.is_empty() {
        match JiraClient::new(JiraCredentials { config, token }) {
            Ok(client) => client.test().await,
            Err(error) => Err(error),
        }
    } else {
        Err(nunc_jira::JiraError::NotConfigured)
    };
    Ok(match outcome {
        Ok(()) => ConnectionTestResult {
            success: true,
            message: String::new(),
        },
        Err(error) => ConnectionTestResult {
            success: false,
            message: error.to_string(),
        },
    })
}

/// Sucht Tickets (leer: eigene offene Tickets) und markiert bereits übernommene.
#[tauri::command]
#[specta::specta]
pub async fn search_jira_issues(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<JiraIssueHit>, AppError> {
    // die Sperre der Datenbank wird vor dem Netzwerkzugriff wieder freigegeben
    let credentials = state
        .db
        .with(|conn| nunc_jira::load_jira_credentials(conn, &*state.secrets))?;
    let issues = JiraClient::new(credentials)?.search(&query).await?;
    let todos = state.db.with(|conn| nunc_store::list_todos(conn))?;
    Ok(issues
        .into_iter()
        .map(|issue| JiraIssueHit {
            imported: is_imported(&todos, &issue),
            issue,
        })
        .collect())
}

/// Legt aus dem Ticket eine Aufgabe `[KEY](url) Titel` an.
#[tauri::command]
#[specta::specta]
pub fn add_todo_from_jira(state: State<'_, AppState>, issue: JiraIssue) -> Result<Todo, AppError> {
    if issue.key.trim().is_empty() || !is_http_url(&issue.url) {
        return Err(AppError::validation("Das Jira-Ticket ist unvollständig."));
    }
    let line = todo_line(&issue);
    let sync_id = uuid::Uuid::new_v4().to_string();
    let now = state.clock.now();
    let todo = state
        .db
        .with(|conn| nunc_store::add_todo(conn, &line, &sync_id, now))?;
    state.todos_changed();
    Ok(todo)
}
