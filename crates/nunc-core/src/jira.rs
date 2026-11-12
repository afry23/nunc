//! Jira-Import (ADR 0038): pure Funktionen für Suche, Zeilenformat und Antwort-Auswertung.
//!
//! Kein Netz, keine Konfiguration im Speicher: der HTTP-Teil liegt in `nunc-jira`.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::{Todo, text_enum};

/// Höchstzahl der Treffer einer Suche.
pub const MAX_HITS: usize = 50;

const OPEN_TICKETS: &str = "statusCategory != Done ORDER BY updated DESC";

/// Jira-Art: Cloud (E-Mail + API-Token, API v3) oder Data Center/Server (PAT, API v2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum JiraKind {
    Cloud,
    DataCenter,
}

text_enum!(JiraKind, "JiraKind", {
    Cloud => "cloud",
    DataCenter => "data_center",
});

/// Zugang ohne Token (das liegt im Tresor).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct JiraConfig {
    pub base_url: String,
    pub kind: JiraKind,
    pub email: String,
}

impl JiraConfig {
    /// Die URL ist gesetzt; Cloud braucht zusätzlich die E-Mail-Adresse.
    pub fn is_complete(&self) -> bool {
        !self.base_url.trim().is_empty()
            && (self.kind == JiraKind::DataCenter || !self.email.trim().is_empty())
    }
}

/// Ein Ticket aus der Suche.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssue {
    pub key: String,
    pub summary: String,
    pub url: String,
}

/// Antwort nicht auswertbar.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Jira-Antwort nicht lesbar: {0}")]
pub struct JiraParseError(pub String);

/// Endpunkte, die nunc nutzt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JiraEndpoint {
    Search,
    Myself,
}

/// REST-Pfad des Endpunkts je Jira-Art.
pub fn api_path(kind: JiraKind, endpoint: JiraEndpoint) -> &'static str {
    match (kind, endpoint) {
        (JiraKind::Cloud, JiraEndpoint::Search) => "/rest/api/3/search/jql",
        (JiraKind::Cloud, JiraEndpoint::Myself) => "/rest/api/3/myself",
        (JiraKind::DataCenter, JiraEndpoint::Search) => "/rest/api/2/search",
        (JiraKind::DataCenter, JiraEndpoint::Myself) => "/rest/api/2/myself",
    }
}

/// Maskiert `\` und `"` für ein JQL-Zeichenkettenliteral.
pub fn escape_jql(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c == '\\' || c == '"' {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// `PROJ-123`: Projekt (Buchstabe, dann Buchstaben/Ziffern/`_`, mindestens zwei Zeichen), `-`, Ziffern.
fn is_issue_key(s: &str) -> bool {
    let Some((project, number)) = s.rsplit_once('-') else {
        return false;
    };
    let mut chars = project.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && project.chars().count() >= 2
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !number.is_empty()
        && number.chars().all(|c| c.is_ascii_digit())
}

/// JQL für die Eingabe im Suchfeld: leer → eigene offene Tickets, Ticket-Key → genau dieses Ticket,
/// sonst Textsuche in offenen Tickets.
pub fn search_jql(query: &str) -> String {
    let query = query.trim();
    if query.is_empty() {
        return format!("assignee = currentUser() AND {OPEN_TICKETS}");
    }
    if is_issue_key(query) {
        return format!("key = \"{}\"", query.to_ascii_uppercase());
    }
    format!("text ~ \"{}\" AND {OPEN_TICKETS}", escape_jql(query))
}

/// Link auf das Ticket im Browser.
pub fn issue_url(base: &str, key: &str) -> String {
    format!("{}/browse/{key}", base.trim().trim_end_matches('/'))
}

/// todo.txt-Zeile `[KEY](url) Titel`; Zeilenumbrüche und Tabs im Titel werden zu Leerzeichen.
pub fn todo_line(issue: &JiraIssue) -> String {
    let summary: String = issue
        .summary
        .chars()
        .map(|c| {
            if matches!(c, '\n' | '\r' | '\t') {
                ' '
            } else {
                c
            }
        })
        .collect();
    let summary = summary.trim();
    let link = format!("[{}]({})", issue.key, issue.url);
    if summary.is_empty() {
        link
    } else {
        format!("{link} {summary}")
    }
}

#[derive(Deserialize)]
struct SearchResponse {
    issues: Vec<RawIssue>,
}

#[derive(Deserialize)]
struct RawIssue {
    key: String,
    #[serde(default)]
    fields: RawFields,
}

#[derive(Deserialize, Default)]
struct RawFields {
    #[serde(default)]
    summary: Option<String>,
}

/// Wertet `{ issues: [{ key, fields: { summary } }] }` aus (v2 und v3 gleich), höchstens [`MAX_HITS`].
pub fn parse_search(json: &str, base: &str) -> Result<Vec<JiraIssue>, JiraParseError> {
    let response: SearchResponse =
        serde_json::from_str(json).map_err(|e| JiraParseError(e.to_string()))?;
    Ok(response
        .issues
        .into_iter()
        .take(MAX_HITS)
        .map(|raw| JiraIssue {
            url: issue_url(base, &raw.key),
            summary: raw.fields.summary.unwrap_or_default(),
            key: raw.key,
        })
        .collect())
}

/// Gibt es eine offene Aufgabe, deren Text auf `/browse/KEY` verweist (Key endet an einer Wortgrenze)?
pub fn is_imported(todos: &[Todo], issue: &JiraIssue) -> bool {
    let needle = format!("/browse/{}", issue.key);
    todos.iter().filter(|t| !t.completed).any(|t| {
        t.raw_text.match_indices(&needle).any(|(at, _)| {
            t.raw_text[at + needle.len()..]
                .chars()
                .next()
                .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_round_trips_as_text() {
        assert_eq!("data_center".parse::<JiraKind>(), Ok(JiraKind::DataCenter));
        assert_eq!(JiraKind::Cloud.as_str(), "cloud");
    }

    #[test]
    fn config_completeness() {
        let mut cfg = JiraConfig {
            base_url: "https://x".into(),
            kind: JiraKind::Cloud,
            email: String::new(),
        };
        assert!(!cfg.is_complete());
        cfg.email = "a@b".into();
        assert!(cfg.is_complete());
        cfg.kind = JiraKind::DataCenter;
        cfg.email.clear();
        assert!(cfg.is_complete());
        cfg.base_url = "  ".into();
        assert!(!cfg.is_complete());
    }
}
