//! Anmeldung im Tresor und ein Token-Lieferant, der abgelaufene Tokens selbst erneuert.

use std::sync::Arc;

use nunc_core::clock::Clock;
use nunc_platform::SecretStore;
use serde::{Deserialize, Serialize};

use super::{OAuthApp, Tokens, refresh_tokens};
use crate::SyncError;

/// Dienst für alle Einträge von nunc im Tresor.
pub const SECRET_SERVICE: &str = "nunc";

/// Was nach der Anmeldung im Tresor liegt: Tokens und das angezeigte Konto.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Connection {
    pub tokens: Tokens,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// Ein Eintrag im Tresor (`google-drive`, `todoist`).
#[derive(Clone)]
pub struct Vault {
    secrets: Arc<dyn SecretStore>,
    account: &'static str,
}

impl Vault {
    pub fn new(secrets: Arc<dyn SecretStore>, account: &'static str) -> Self {
        Self { secrets, account }
    }

    pub fn load(&self) -> Result<Option<Connection>, SyncError> {
        let Some(text) = self.secrets.get(SECRET_SERVICE, self.account)? else {
            return Ok(None);
        };
        // ein unlesbarer Eintrag gilt als nicht verbunden: neu verbinden überschreibt ihn
        Ok(serde_json::from_str(&text).ok())
    }

    pub fn save(&self, connection: &Connection) -> Result<(), SyncError> {
        let text = serde_json::to_string(connection)
            .map_err(|error| SyncError::OAuth(error.to_string()))?;
        Ok(self.secrets.set(SECRET_SERVICE, self.account, &text)?)
    }

    pub fn delete(&self) -> Result<(), SyncError> {
        Ok(self.secrets.delete(SECRET_SERVICE, self.account)?)
    }
}

/// Liefert ein gültiges Access-Token. Abgelaufene Tokens werden mit dem Refresh-Token erneuert und
/// (gedreht) wieder im Tresor abgelegt. Erneuerungen laufen nacheinander: Dienste drehen das Refresh-Token,
/// ein zweiter gleichzeitiger Versuch mit dem alten würde scheitern.
pub struct TokenSource {
    vault: Vault,
    app: OAuthApp,
    http: reqwest::Client,
    clock: Arc<dyn Clock>,
    renewal: tokio::sync::Mutex<()>,
}

impl TokenSource {
    pub fn new(vault: Vault, app: OAuthApp, http: reqwest::Client, clock: Arc<dyn Clock>) -> Self {
        Self {
            vault,
            app,
            http,
            clock,
            renewal: tokio::sync::Mutex::new(()),
        }
    }

    fn connection(&self) -> Result<Connection, SyncError> {
        self.vault
            .load()?
            .ok_or(SyncError::NotConnected(self.app.service))
    }

    pub async fn access_token(&self) -> Result<String, SyncError> {
        let connection = self.connection()?;
        if !connection.tokens.is_expired(self.clock.now()) {
            return Ok(connection.tokens.access_token);
        }
        self.renew(&connection.tokens.access_token).await
    }

    /// Erneuert das Token, das der Dienst abgelehnt hat (oder das abgelaufen ist). Hat inzwischen jemand anderes
    /// erneuert, gilt dessen Ergebnis.
    pub async fn renew(&self, rejected: &str) -> Result<String, SyncError> {
        let _turn = self.renewal.lock().await;
        let mut connection = self.connection()?;
        if connection.tokens.access_token != rejected {
            return Ok(connection.tokens.access_token);
        }
        let refresh_token = connection
            .tokens
            .refresh_token
            .clone()
            .ok_or(SyncError::AuthExpired(self.app.service))?;
        connection.tokens =
            refresh_tokens(&self.http, &self.app, &refresh_token, self.clock.now()).await?;
        self.vault.save(&connection)?;
        Ok(connection.tokens.access_token)
    }
}
