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

    /// Wie [`Vault::load`], aber ein nicht erreichbarer Tresor (z. B. Linux ohne Secret-Service) zählt als „nicht
    /// verbunden“. Für Anzeige und Auslöser: sie dürfen beim Start keinen Fehler melden; Verbinden und Abgleich
    /// sagen es dem Nutzer, wenn er sie benutzt.
    pub fn load_lenient(&self) -> Option<Connection> {
        match self.load() {
            Ok(connection) => connection,
            Err(error) => {
                log::warn!("Tresor nicht lesbar ({}): {error}", self.account);
                None
            }
        }
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
    service: &'static str,
    /// Ohne Client (persönliches Token, ADR 0020) gibt es nichts zu erneuern.
    app: Option<OAuthApp>,
    http: reqwest::Client,
    clock: Arc<dyn Clock>,
    renewal: tokio::sync::Mutex<()>,
}

impl TokenSource {
    pub fn new(vault: Vault, app: OAuthApp, http: reqwest::Client, clock: Arc<dyn Clock>) -> Self {
        Self {
            vault,
            service: app.service,
            app: Some(app),
            http,
            clock,
            renewal: tokio::sync::Mutex::new(()),
        }
    }

    /// Für ein Token ohne OAuth-Client (persönliches API-Token): wird es abgelehnt oder ist es abgelaufen,
    /// meldet der Lieferant [`SyncError::AuthExpired`], statt zu erneuern.
    pub fn fixed(
        vault: Vault,
        service: &'static str,
        http: reqwest::Client,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            vault,
            service,
            app: None,
            http,
            clock,
            renewal: tokio::sync::Mutex::new(()),
        }
    }

    fn connection(&self) -> Result<Connection, SyncError> {
        self.vault
            .load()?
            .ok_or(SyncError::NotConnected(self.service))
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
        let app = self
            .app
            .as_ref()
            .ok_or(SyncError::AuthExpired(self.service))?;
        let refresh_token = connection
            .tokens
            .refresh_token
            .clone()
            .ok_or(SyncError::AuthExpired(self.service))?;
        connection.tokens =
            refresh_tokens(&self.http, app, &refresh_token, self.clock.now()).await?;
        self.vault.save(&connection)?;
        Ok(connection.tokens.access_token)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use nunc_platform::{ManualClock, MemorySecretStore};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn start() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 4, 9, 0, 0).unwrap()
    }

    fn connection(access: &str, refresh: Option<&str>, expires_in: Option<i64>) -> Connection {
        Connection {
            tokens: Tokens {
                access_token: access.into(),
                refresh_token: refresh.map(str::to_owned),
                expires_at: expires_in.map(|s| start() + Duration::seconds(s)),
            },
            email: Some("me@example.org".into()),
            avatar_url: None,
        }
    }

    fn app(token_url: String) -> OAuthApp {
        OAuthApp {
            service: "Test",
            client_id: "id".into(),
            client_secret: None,
            auth_url: String::new(),
            token_url,
            scope: String::new(),
            auth_params: vec![],
        }
    }

    fn source(vault: &Vault, token_url: String) -> TokenSource {
        TokenSource::new(
            vault.clone(),
            app(token_url),
            reqwest::Client::new(),
            Arc::new(ManualClock::new(start())),
        )
    }

    fn vault() -> Vault {
        Vault::new(Arc::new(MemorySecretStore::default()), "konto")
    }

    #[test]
    fn the_vault_saves_loads_and_deletes_and_ignores_unreadable_entries() {
        let secrets = Arc::new(MemorySecretStore::default());
        let vault = Vault::new(secrets.clone(), "konto");
        assert_eq!(vault.load().unwrap(), None);
        let saved = connection("a", Some("r"), Some(60));
        vault.save(&saved).unwrap();
        assert_eq!(vault.load().unwrap(), Some(saved));
        secrets.set(SECRET_SERVICE, "konto", "kein JSON").unwrap();
        assert_eq!(vault.load().unwrap(), None);
        vault.save(&connection("b", None, None)).unwrap();
        vault.delete().unwrap();
        assert_eq!(vault.load().unwrap(), None);
        assert!(secrets.get(SECRET_SERVICE, "konto").unwrap().is_none());
    }

    #[tokio::test]
    async fn a_valid_token_is_used_as_it_is() {
        let vault = vault();
        vault.save(&connection("gültig", None, Some(3600))).unwrap();
        // kein Server nötig: es wird nichts erneuert
        let source = source(&vault, "http://127.0.0.1:1/token".into());
        assert_eq!(source.access_token().await.unwrap(), "gültig");
    }

    #[tokio::test]
    async fn a_missing_connection_is_reported() {
        let source = source(&vault(), "http://127.0.0.1:1/token".into());
        assert!(matches!(
            source.access_token().await,
            Err(SyncError::NotConnected("Test"))
        ));
    }

    #[tokio::test]
    async fn an_expired_token_is_renewed_and_the_rotated_refresh_token_is_kept() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "neu", "refresh_token": "r2", "expires_in": 3600
            })))
            .expect(1)
            .mount(&server)
            .await;
        let vault = vault();
        vault
            .save(&connection("alt", Some("r1"), Some(-10)))
            .unwrap();
        let source = source(&vault, format!("{}/token", server.uri()));
        assert_eq!(source.access_token().await.unwrap(), "neu");
        let stored = vault.load().unwrap().unwrap();
        assert_eq!(stored.tokens.access_token, "neu");
        assert_eq!(stored.tokens.refresh_token.as_deref(), Some("r2"));
        assert_eq!(stored.email.as_deref(), Some("me@example.org"));
    }

    #[tokio::test]
    async fn renewal_after_someone_else_renewed_uses_their_token() {
        let vault = vault();
        vault
            .save(&connection("schon-neu", Some("r"), Some(3600)))
            .unwrap();
        let source = source(&vault, "http://127.0.0.1:1/token".into());
        // abgelehnt wurde das alte Token, gespeichert ist schon ein neues: kein Netzwerkzugriff
        assert_eq!(source.renew("alt").await.unwrap(), "schon-neu");
    }

    #[tokio::test]
    async fn a_fixed_token_is_never_renewed() {
        let vault = vault();
        vault
            .save(&connection("fest", Some("r"), Some(-10)))
            .unwrap();
        let source = TokenSource::fixed(
            vault.clone(),
            "Fest",
            reqwest::Client::new(),
            Arc::new(ManualClock::new(start())),
        );
        assert!(matches!(
            source.access_token().await,
            Err(SyncError::AuthExpired("Fest"))
        ));
        assert!(matches!(
            source.renew("fest").await,
            Err(SyncError::AuthExpired("Fest"))
        ));
        assert!(matches!(
            TokenSource::fixed(
                self::vault(),
                "Fest",
                reqwest::Client::new(),
                Arc::new(ManualClock::new(start())),
            )
            .access_token()
            .await,
            Err(SyncError::NotConnected("Fest"))
        ));
    }

    #[tokio::test]
    async fn without_a_refresh_token_an_expired_connection_needs_a_new_login() {
        let vault = vault();
        vault.save(&connection("alt", None, Some(-10))).unwrap();
        let source = source(&vault, "http://127.0.0.1:1/token".into());
        assert!(matches!(
            source.access_token().await,
            Err(SyncError::AuthExpired("Test"))
        ));
    }
}
