//! HTTP-Zugriff auf Jira Cloud (Basic, API v3) und Data Center (Bearer, API v2).

use nunc_core::jira::{
    JiraEndpoint, JiraIssue, JiraKind, MAX_HITS, api_path, parse_search, search_jql,
};
use reqwest::{RequestBuilder, Response, StatusCode};

use crate::{JiraCredentials, JiraError};

#[derive(Debug, Clone)]
pub struct JiraClient {
    http: reqwest::Client,
    credentials: JiraCredentials,
}

impl JiraClient {
    pub fn new(credentials: JiraCredentials) -> Result<Self, JiraError> {
        let http = nunc_platform::http_client().map_err(JiraError::Transport)?;
        Ok(Self { http, credentials })
    }

    fn url(&self, endpoint: JiraEndpoint) -> String {
        let base = self
            .credentials
            .config
            .base_url
            .trim()
            .trim_end_matches('/');
        format!("{base}{}", api_path(self.credentials.config.kind, endpoint))
    }

    fn get(&self, endpoint: JiraEndpoint) -> RequestBuilder {
        let request = self.http.get(self.url(endpoint));
        let JiraCredentials { config, token } = &self.credentials;
        match config.kind {
            JiraKind::Cloud => request.basic_auth(&config.email, Some(token)),
            JiraKind::DataCenter => request.bearer_auth(token),
        }
    }

    async fn send(request: RequestBuilder) -> Result<Response, JiraError> {
        let response = request
            .send()
            .await
            .map_err(|e| JiraError::Transport(e.without_url().to_string()))?;
        match response.status() {
            status if status.is_success() => Ok(response),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(JiraError::Unauthorized),
            status => Err(JiraError::Http(status.as_u16())),
        }
    }

    /// Sucht Tickets (`query` wie im Suchfeld, siehe [`search_jql`]).
    pub async fn search(&self, query: &str) -> Result<Vec<JiraIssue>, JiraError> {
        let request = self.get(JiraEndpoint::Search).query(&[
            ("jql", search_jql(query)),
            ("fields", "summary".to_owned()),
            ("maxResults", MAX_HITS.to_string()),
        ]);
        let body = Self::send(request)
            .await?
            .text()
            .await
            .map_err(|e| JiraError::Transport(e.without_url().to_string()))?;
        parse_search(&body, &self.credentials.config.base_url)
            .map_err(|e| JiraError::BadResponse(e.to_string()))
    }

    /// Verbindungstest: fragt den angemeldeten Benutzer ab.
    pub async fn test(&self) -> Result<(), JiraError> {
        Self::send(self.get(JiraEndpoint::Myself)).await?;
        Ok(())
    }
}
