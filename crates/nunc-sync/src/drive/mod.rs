//! Synchronisierung über Google Drive (ADR 0009): Anmeldung, Sync-Lauf und
//! Steuerung der Verbindung. Das Zusammenführen ist dasselbe wie bei WebDAV (`engine::merge_remote`).
//!
//! Ablage: `nunc/nunc-sync.v2.json` im Drive-Stamm, Scope `drive.file`. Drive kennt kein bedingtes Schreiben
//! (WebDAV: ETag); vor dem Überschreiben prüft der Lauf deshalb die Änderungszeit der Datei und setzt neu an,
//! wenn ein anderes Gerät dazwischen war. Ein Rest-Risiko bleibt (zwischen Prüfung und Schreiben).

mod client;

use std::sync::Arc;
use std::time::Duration;

use nunc_core::clock::Clock;
use nunc_core::crdt::SyncState;
use nunc_core::model::{DriveStatus, SyncFile, SyncImportInfo, SyncProvider, SyncResult};
use nunc_core::sync_document::SyncDocument;
use nunc_platform::{SecretStore, UrlOpener};

pub use client::{DriveClient, DriveFile};

use crate::SyncError;
use crate::coordinator::{RunFuture, SyncRunner};
use crate::engine::{MAX_ATTEMPTS, SyncStore, merge_remote, remote_state};
use crate::oauth::{
    AuthorizeOptions, Connection, OAuthApp, Page, TokenSource, Vault, authorize, http_client,
};

/// Ordner und Datei im Drive.
pub const FOLDER_NAME: &str = "nunc";
pub const FILE_NAME: &str = "nunc-sync.v2.json";
/// Alte Datei (Format vor ADR 0016), nur für den einmaligen Import.
pub const LEGACY_FILE_NAME: &str = "nunc-sync.json";
/// Eintrag im Tresor.
pub const SECRET_ACCOUNT: &str = "google-drive";
const SERVICE: &str = "Google Drive";
const CALLBACK_PATH: &str = "/callback";

/// Adressen und Zugangsdaten; Tests setzen die Adressen auf einen Testserver.
#[derive(Debug, Clone)]
pub struct DriveConfig {
    pub app: OAuthApp,
    pub api_base: String,
    pub upload_base: String,
    pub auth_timeout: Duration,
}

impl DriveConfig {
    /// Google mit den Zugangsdaten des Builds (leer = nicht eingerichtet).
    pub fn google(client_id: &str, client_secret: &str) -> Self {
        Self {
            app: OAuthApp {
                service: SERVICE,
                client_id: client_id.to_owned(),
                client_secret: Some(client_secret.to_owned()).filter(|secret| !secret.is_empty()),
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_owned(),
                token_url: "https://oauth2.googleapis.com/token".to_owned(),
                scope: "https://www.googleapis.com/auth/drive.file".to_owned(),
                auth_params: vec![
                    ("access_type".to_owned(), "offline".to_owned()),
                    ("prompt".to_owned(), "consent".to_owned()),
                ],
            },
            api_base: "https://www.googleapis.com/drive/v3".to_owned(),
            upload_base: "https://www.googleapis.com/upload/drive/v3".to_owned(),
            auth_timeout: Duration::from_secs(120),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.app.client_id.is_empty()
    }
}

/// Link auf den Ordner im Drive.
pub fn folder_url(folder_id: &str) -> String {
    format!("https://drive.google.com/drive/folders/{folder_id}")
}

/// Verbindung zu Google Drive: anmelden, abmelden, Zustand, Sync-Läufe.
pub struct DriveService {
    config: DriveConfig,
    vault: Vault,
    clock: Arc<dyn Clock>,
    http: reqwest::Client,
}

impl DriveService {
    pub fn new(
        config: DriveConfig,
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

    /// Zustand der Verbindung (ohne Netzwerkzugriff). `folder_id`: der Ordner aus dem letzten Sync.
    pub fn status(&self, folder_id: Option<&str>) -> Result<DriveStatus, SyncError> {
        let connection = self.vault.load_lenient();
        Ok(DriveStatus {
            configured: self.config.is_configured(),
            connected: connection.is_some(),
            email: connection.and_then(|connection| connection.email),
            folder_url: folder_id.filter(|id| !id.is_empty()).map(folder_url),
        })
    }

    pub fn is_connected(&self) -> Result<bool, SyncError> {
        Ok(self.vault.load_lenient().is_some())
    }

    fn client(&self) -> Result<DriveClient, SyncError> {
        if !self.is_connected()? {
            return Err(SyncError::NotConnected(SERVICE));
        }
        let tokens = Arc::new(TokenSource::new(
            self.vault.clone(),
            self.config.app.clone(),
            self.http.clone(),
            self.clock.clone(),
        ));
        Ok(DriveClient::new(
            self.http.clone(),
            self.config.clone(),
            tokens,
        ))
    }

    /// Meldet im Browser an und legt die Tokens im Tresor ab. Die E-Mail-Adresse ist nur Anzeige: kann sie
    /// nicht gelesen werden, ist die Verbindung trotzdem hergestellt.
    pub async fn connect(&self, opener: &dyn UrlOpener) -> Result<DriveStatus, SyncError> {
        if !self.config.is_configured() {
            return Err(SyncError::OAuth(
                "In diesem Build sind keine Google-Zugangsdaten hinterlegt (NUNC_GOOGLE_CLIENT_ID, NUNC_GOOGLE_CLIENT_SECRET)."
                    .to_owned(),
            ));
        }
        let options = AuthorizeOptions {
            ports: vec![0],
            callback_path: CALLBACK_PATH,
            timeout: self.config.auth_timeout,
            success_page: Page {
                title: "Verbunden!",
                message: "Google Drive wurde erfolgreich mit nunc verbunden.",
            },
        };
        let tokens =
            authorize(&self.http, &self.config.app, &options, opener, &*self.clock).await?;
        self.vault.save(&Connection {
            tokens,
            email: None,
            avatar_url: None,
        })?;
        let email = match self.client() {
            Ok(client) => client.account_email().await.unwrap_or(None),
            Err(_) => None,
        };
        if let Some(mut connection) = self.vault.load()? {
            connection.email = email;
            self.vault.save(&connection)?;
        }
        self.status(None)
    }

    /// Löscht die Tokens lokal; die Sync-Datei im Drive bleibt.
    pub fn disconnect(&self) -> Result<(), SyncError> {
        self.vault.delete()
    }

    /// Gibt es auf Drive Stand, den ein Gerät ohne eigene Daten übernehmen kann?
    pub async fn check_import(&self, store: &dyn SyncStore) -> Result<SyncImportInfo, SyncError> {
        let local = store.local_state()?;
        let has_local_data =
            local.todos.values().any(|todo| !todo.is_deleted()) || !local.sessions.is_empty();
        if has_local_data {
            return Ok(SyncImportInfo::default());
        }
        let client = self.client()?;
        let folder = client.find_or_create_folder(FOLDER_NAME).await?;
        let Some((remote, _)) = fetch_remote(&client, &folder).await? else {
            return Ok(SyncImportInfo::default());
        };
        Ok(SyncImportInfo {
            available: true,
            todo_count: u32::try_from(
                remote
                    .todos
                    .values()
                    .filter(|todo| !todo.is_deleted())
                    .count(),
            )
            .unwrap_or(u32::MAX),
            session_count: u32::try_from(remote.sessions.len()).unwrap_or(u32::MAX),
        })
    }

    /// Ein Sync-Lauf; liefert, was übernommen wurde, und den Ordner im Drive.
    pub async fn sync(&self, store: &dyn SyncStore) -> Result<(SyncResult, String), SyncError> {
        let client = self.client()?;
        let (result, folder) = sync_drive(&client, store, &*self.clock).await?;
        store.record_drive_folder(&folder)?;
        Ok((result, folder))
    }
}

fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SyncError> {
    serde_json::from_slice(bytes).map_err(|error| SyncError::InvalidFile(error.to_string()))
}

fn encode(file: &SyncDocument) -> Result<Vec<u8>, SyncError> {
    serde_json::to_vec_pretty(file).map_err(|error| SyncError::InvalidFile(error.to_string()))
}

/// Der Stand auf Drive: die v2-Datei samt Eintrag oder, wenn es nur die alte `nunc-sync.json` gibt, deren
/// Inhalt (einmaliger Import; ohne Eintrag, denn geschrieben wird immer die v2-Datei).
async fn fetch_remote(
    client: &DriveClient,
    folder: &str,
) -> Result<Option<(SyncState, Option<DriveFile>)>, SyncError> {
    let current = client.find_file(FILE_NAME, folder).await?;
    let document: Option<SyncDocument> = match &current {
        Some(file) => Some(parse(&client.download(&file.id).await?)?),
        None => None,
    };
    let legacy: Option<SyncFile> =
        match (&current, client.find_file(LEGACY_FILE_NAME, folder).await?) {
            (None, Some(file)) => Some(parse(&client.download(&file.id).await?)?),
            _ => None,
        };
    Ok(remote_state(document.as_ref(), legacy.as_ref())?.map(|state| (state, current)))
}

/// Führt einen Sync-Lauf gegen Drive aus.
pub async fn sync_drive(
    client: &DriveClient,
    store: &dyn SyncStore,
    clock: &dyn Clock,
) -> Result<(SyncResult, String), SyncError> {
    let folder = client.find_or_create_folder(FOLDER_NAME).await?;
    let mut result = SyncResult::default();
    for _ in 0..MAX_ATTEMPTS {
        let remote = fetch_remote(client, &folder).await?;
        let (state, file) = match remote {
            Some((state, file)) => (Some(state), file),
            None => (None, None),
        };
        let merged = merge_remote(store, clock, state.as_ref(), &mut result)?;
        let data = encode(&merged)?;
        match file {
            Some(file) => {
                // ein anderes Gerät war schneller: neu laden und zusammenführen
                if client.modified_time(&file.id).await? != file.modified_time {
                    continue;
                }
                client.update(&file.id, &data).await?;
            }
            None => {
                client.create(FILE_NAME, &folder, &data).await?;
            }
        }
        return Ok((result, folder));
    }
    Err(SyncError::Conflict)
}

/// Der Sync-Lauf gegen Google Drive für den [`SyncCoordinator`](crate::SyncCoordinator): läuft nur, wenn Drive
/// gewählt und der Sync nicht pausiert ist.
pub struct DriveRunner {
    store: Arc<dyn SyncStore>,
    service: Arc<DriveService>,
    clock: Arc<dyn Clock>,
}

impl DriveRunner {
    pub fn new(
        store: Arc<dyn SyncStore>,
        service: Arc<DriveService>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            store,
            service,
            clock,
        }
    }
}

impl SyncRunner for DriveRunner {
    fn run(&self) -> RunFuture {
        let store = self.store.clone();
        let service = self.service.clone();
        let clock = self.clock.clone();
        Box::pin(async move {
            let setup = store.setup()?;
            if setup.provider != SyncProvider::GoogleDrive || !setup.enabled {
                return Ok(None);
            }
            let (result, _) = service.sync(&*store).await?;
            let finished = clock.now();
            store.record_sync(SyncProvider::GoogleDrive, finished)?;
            Ok(Some((
                result,
                finished.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )))
        })
    }
}

/// Mehrere Läufer hintereinander: der erste, der zuständig ist (Ergebnis `Some`), bestimmt den Lauf. Jeder
/// Läufer prüft selbst, ob sein Anbieter gewählt ist.
pub struct ChainRunner(pub Vec<Arc<dyn SyncRunner>>);

impl SyncRunner for ChainRunner {
    fn run(&self) -> RunFuture {
        let runners = self.0.clone();
        Box::pin(async move {
            for runner in runners {
                if let Some(outcome) = runner.run().await? {
                    return Ok(Some(outcome));
                }
            }
            Ok(None)
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use nunc_platform::{ManualClock, MemorySecretStore};

    use super::*;

    fn service(client_id: &str, client_secret: &str) -> DriveService {
        DriveService::new(
            DriveConfig::google(client_id, client_secret),
            Arc::new(MemorySecretStore::default()),
            Arc::new(ManualClock::new(
                Utc.with_ymd_and_hms(2026, 10, 4, 9, 0, 0).unwrap(),
            )),
        )
        .unwrap()
    }

    fn connect(service: &DriveService, email: Option<&str>) {
        service
            .vault
            .save(&Connection {
                tokens: crate::oauth::Tokens {
                    access_token: "a".into(),
                    refresh_token: None,
                    expires_at: None,
                },
                email: email.map(str::to_owned),
                avatar_url: None,
            })
            .unwrap();
    }

    #[test]
    fn google_needs_a_client_id_and_may_lack_a_secret() {
        let config = DriveConfig::google("id", "secret");
        assert!(config.is_configured());
        assert_eq!(config.app.client_secret.as_deref(), Some("secret"));
        assert_eq!(
            config.app.scope,
            "https://www.googleapis.com/auth/drive.file"
        );
        let without_secret = DriveConfig::google("id", "");
        assert_eq!(without_secret.app.client_secret, None);
        assert!(!DriveConfig::google("", "secret").is_configured());
    }

    #[test]
    fn the_folder_link_points_into_drive() {
        assert_eq!(
            folder_url("abc123"),
            "https://drive.google.com/drive/folders/abc123"
        );
    }

    #[test]
    fn the_status_reflects_the_vault_and_the_known_folder() {
        let service = service("id", "secret");
        let status = service.status(Some("folder-1")).unwrap();
        assert!(status.configured && !status.connected);
        assert_eq!(status.email, None);
        assert_eq!(
            status.folder_url.as_deref(),
            Some("https://drive.google.com/drive/folders/folder-1")
        );
        assert_eq!(service.status(Some("")).unwrap().folder_url, None);
        assert_eq!(service.status(None).unwrap().folder_url, None);
        assert!(!service.is_connected().unwrap());

        connect(&service, Some("me@example.org"));
        assert!(service.is_connected().unwrap());
        let status = service.status(None).unwrap();
        assert!(status.connected);
        assert_eq!(status.email.as_deref(), Some("me@example.org"));
        assert!(!self::service("", "").status(None).unwrap().configured);
    }

    #[test]
    fn an_unreachable_secret_store_counts_as_not_connected() {
        let service = DriveService::new(
            DriveConfig::google("id", "secret"),
            Arc::new(nunc_platform::UnavailableSecretStore::new(
                "kein Secret-Service",
            )),
            Arc::new(ManualClock::new(
                Utc.with_ymd_and_hms(2026, 10, 4, 9, 0, 0).unwrap(),
            )),
        )
        .unwrap();
        assert!(!service.is_connected().unwrap());
        assert!(!service.status(None).unwrap().connected);
    }

    #[test]
    fn disconnecting_removes_the_connection() {
        let service = service("id", "secret");
        connect(&service, None);
        service.disconnect().unwrap();
        assert!(!service.is_connected().unwrap());
        assert!(matches!(
            service.client(),
            Err(SyncError::NotConnected("Google Drive"))
        ));
    }
}
