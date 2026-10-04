//! Google-Drive-REST-Client (Go: `services/sync_drive.go`). Nur Dateien, die nunc selbst angelegt hat
//! (Scope `drive.file`).

use std::sync::Arc;

use reqwest::{RequestBuilder, Response, StatusCode, header};
use serde::Deserialize;

use super::DriveConfig;
use crate::SyncError;
use crate::oauth::{TokenSource, random_hex};

const FOLDER_MIME: &str = "application/vnd.google-apps.folder";

/// Eine Datei im Drive.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFile {
    pub id: String,
    /// Änderungszeit; Drive kennt kein bedingtes Schreiben, daran erkennen wir gleichzeitige Änderungen.
    #[serde(default)]
    pub modified_time: String,
}

#[derive(Deserialize)]
struct FileList {
    #[serde(default)]
    files: Vec<DriveFile>,
}

pub struct DriveClient {
    http: reqwest::Client,
    config: DriveConfig,
    tokens: Arc<TokenSource>,
}

/// Maskiert `\` und `'` in einem Wert der Suchanfrage (`q`).
fn escape_query(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

impl DriveClient {
    pub fn new(http: reqwest::Client, config: DriveConfig, tokens: Arc<TokenSource>) -> Self {
        Self {
            http,
            config,
            tokens,
        }
    }

    /// Sendet mit dem aktuellen Token; lehnt Drive es ab (`401`), wird es einmal erneuert.
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
            StatusCode::UNAUTHORIZED => Err(SyncError::AuthExpired("Google Drive")),
            StatusCode::FORBIDDEN => Err(SyncError::Forbidden),
            status => Err(SyncError::Status {
                status: status.as_u16(),
                action,
            }),
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

    async fn json<T: for<'de> Deserialize<'de>>(
        response: Response,
        action: &'static str,
    ) -> Result<T, SyncError> {
        response.json().await.map_err(|_| SyncError::Status {
            status: 200,
            action,
        })
    }

    async fn find(
        &self,
        query: String,
        action: &'static str,
    ) -> Result<Option<DriveFile>, SyncError> {
        let url = format!("{}/files", self.config.api_base);
        let response = self
            .send(action, |http| {
                http.get(&url).query(&[
                    ("q", query.as_str()),
                    ("spaces", "drive"),
                    ("fields", "files(id,modifiedTime)"),
                ])
            })
            .await?;
        let list: FileList = Self::json(response, action).await?;
        Ok(list.files.into_iter().next())
    }

    /// Kennung des Ordners `name` im Drive-Stamm; legt ihn an, wenn er fehlt.
    pub async fn find_or_create_folder(&self, name: &str) -> Result<String, SyncError> {
        let query = format!(
            "mimeType='{FOLDER_MIME}' and name='{}' and 'root' in parents and trashed=false",
            escape_query(name)
        );
        if let Some(folder) = self.find(query, "Suchen des Ordners").await? {
            return Ok(folder.id);
        }
        let url = format!("{}/files", self.config.api_base);
        let body = serde_json::json!({ "name": name, "mimeType": FOLDER_MIME });
        let response = self
            .send("Anlegen des Ordners", |http| {
                http.post(&url).query(&[("fields", "id")]).json(&body)
            })
            .await?;
        let created: DriveFile = Self::json(response, "Anlegen des Ordners").await?;
        Ok(created.id)
    }

    pub async fn find_file(
        &self,
        name: &str,
        folder_id: &str,
    ) -> Result<Option<DriveFile>, SyncError> {
        let query = format!(
            "name='{}' and '{}' in parents and trashed=false",
            escape_query(name),
            escape_query(folder_id)
        );
        self.find(query, "Suchen der Sync-Datei").await
    }

    pub async fn download(&self, file_id: &str) -> Result<Vec<u8>, SyncError> {
        let url = format!("{}/files/{file_id}", self.config.api_base);
        let response = self
            .send("Herunterladen", |http| {
                http.get(&url).query(&[("alt", "media")])
            })
            .await?;
        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|error| SyncError::Transport(crate::webdav::describe_transport_error(&error)))
    }

    /// Aktuelle Änderungszeit der Datei.
    pub async fn modified_time(&self, file_id: &str) -> Result<String, SyncError> {
        let url = format!("{}/files/{file_id}", self.config.api_base);
        let response = self
            .send("Prüfen der Sync-Datei", |http| {
                http.get(&url).query(&[("fields", "id,modifiedTime")])
            })
            .await?;
        let file: DriveFile = Self::json(response, "Prüfen der Sync-Datei").await?;
        Ok(file.modified_time)
    }

    pub async fn create(
        &self,
        name: &str,
        folder_id: &str,
        data: &[u8],
    ) -> Result<String, SyncError> {
        let boundary = format!("nunc{}", random_hex(8)?);
        let meta = serde_json::json!({ "name": name, "parents": [folder_id] });
        let mut body = Vec::new();
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{meta}\r\n--{boundary}\r\nContent-Type: application/json\r\n\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(data);
        body.extend_from_slice(format!("\r\n--{boundary}--").as_bytes());
        let url = format!("{}/files", self.config.upload_base);
        let content_type = format!("multipart/related; boundary={boundary}");
        let response = self
            .send("Anlegen der Sync-Datei", |http| {
                http.post(&url)
                    .query(&[("uploadType", "multipart"), ("fields", "id")])
                    .header(header::CONTENT_TYPE, &content_type)
                    .body(body.clone())
            })
            .await?;
        let created: DriveFile = Self::json(response, "Anlegen der Sync-Datei").await?;
        Ok(created.id)
    }

    pub async fn update(&self, file_id: &str, data: &[u8]) -> Result<(), SyncError> {
        let url = format!("{}/files/{file_id}", self.config.upload_base);
        self.send("Hochladen der Sync-Datei", |http| {
            http.patch(&url)
                .query(&[("uploadType", "media")])
                .header(header::CONTENT_TYPE, "application/json")
                .body(data.to_vec())
        })
        .await?;
        Ok(())
    }

    /// E-Mail-Adresse des angemeldeten Kontos (geht mit `drive.file`; ein zusätzlicher Scope wäre nötig, wenn
    /// man sie über den Anmeldedienst läse).
    pub async fn account_email(&self) -> Result<Option<String>, SyncError> {
        #[derive(Deserialize)]
        struct About {
            user: Option<User>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct User {
            email_address: Option<String>,
        }
        let url = format!("{}/about", self.config.api_base);
        let response = self
            .send("Lesen des Kontos", |http| {
                http.get(&url).query(&[("fields", "user(emailAddress)")])
            })
            .await?;
        let about: About = Self::json(response, "Lesen des Kontos").await?;
        Ok(about.user.and_then(|user| user.email_address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_and_backslashes_are_escaped_in_queries() {
        assert_eq!(escape_query(r"a'b\c"), r"a\'b\\c");
        assert_eq!(escape_query("nunc"), "nunc");
    }
}
