//! OAuth 2.0 mit PKCE für Desktop-Apps (RFC 7636, RFC 8252): Browser öffnen, den Rückruf auf `127.0.0.1`
//! abwarten, den Code gegen Tokens tauschen und Tokens erneuern. Gemeinsam für Google Drive und Todoist.

mod loopback;
mod vault;

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use nunc_core::clock::Clock;
use nunc_platform::UrlOpener;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use loopback::{Loopback, Page};
pub use vault::{Connection, SECRET_SERVICE, TokenSource, Vault};

use crate::SyncError;

/// Wie lange vor Ablauf ein Token schon als abgelaufen gilt.
const EXPIRY_MARGIN_SECONDS: i64 = 60;

/// Die App-Registrierung beim Dienst.
#[derive(Debug, Clone)]
pub struct OAuthApp {
    /// Name für Fehlermeldungen (`Google Drive`).
    pub service: &'static str,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: String,
    pub token_url: String,
    /// Durch Leerzeichen getrennt.
    pub scope: String,
    /// Weitere Parameter der Autorisierungsanfrage (`access_type=offline`).
    pub auth_params: Vec<(String, String)>,
}

/// Anmeldedaten, wie sie im Tresor liegen.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// `None`: läuft nicht ab (ältere Todoist-Apps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

// Tokens gehören nicht in Logs und Testausgaben
impl std::fmt::Debug for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tokens")
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

impl Tokens {
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at
            .is_some_and(|at| at - ChronoDuration::seconds(EXPIRY_MARGIN_SECONDS) <= now)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

/// HTTP-Client für Anmeldung, Dienst-APIs und WebDAV.
///
/// Vertraut neben den eingebauten Wurzeln auch dem Zertifikatsspeicher des Systems (Firmen-Proxys wie
/// Zscaler signieren mit einer CA, die nur dort liegt). Der Client wird einmal gebaut und danach
/// geklont, weil das Laden des Speichers ~100 ms dauert.
pub fn http_client() -> Result<reqwest::Client, SyncError> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    CLIENT
        .get_or_init(build_client)
        .clone()
        .map_err(SyncError::Transport)
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .use_preconfigured_tls(Arc::unwrap_or_clone(tls_config()?))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| error.to_string())
}

fn tls_config() -> Result<Arc<rustls::ClientConfig>, String> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let native = rustls_native_certs::load_native_certs();
    for error in &native.errors {
        log::warn!("Systemzertifikate: {error}");
    }
    // Einzelne ungültige Systemzertifikate werden übersprungen (bei Firmen-Stores nicht selten).
    let (added, ignored) = roots.add_parsable_certificates(native.certs);
    log::debug!("Systemzertifikate: {added} geladen, {ignored} übersprungen");
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| error.to_string())?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// Zufällige Bytes als hexadezimaler Text (`state`).
pub fn random_hex(bytes: usize) -> Result<String, SyncError> {
    let mut buffer = vec![0u8; bytes];
    getrandom::fill(&mut buffer).map_err(|error| SyncError::OAuth(error.to_string()))?;
    Ok(buffer.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Code-Verifier (43 Zeichen) und seine S256-Prüfsumme.
pub fn pkce() -> Result<Pkce, SyncError> {
    let mut buffer = [0u8; 32];
    getrandom::fill(&mut buffer).map_err(|error| SyncError::OAuth(error.to_string()))?;
    Ok(pkce_from(&URL_SAFE_NO_PAD.encode(buffer)))
}

fn pkce_from(verifier: &str) -> Pkce {
    Pkce {
        verifier: verifier.to_owned(),
        challenge: URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
    }
}

/// Adresse der Autorisierungsseite.
pub fn authorize_url(
    app: &OAuthApp,
    state: &str,
    challenge: &str,
    redirect_uri: &str,
) -> Result<String, SyncError> {
    let mut url = Url::parse(&app.auth_url).map_err(|error| SyncError::OAuth(error.to_string()))?;
    {
        let mut query = url.query_pairs_mut();
        query
            .append_pair("client_id", &app.client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &app.scope)
            .append_pair("state", state)
            .append_pair("code_challenge", challenge)
            .append_pair("code_challenge_method", "S256");
        for (name, value) in &app.auth_params {
            query.append_pair(name, value);
        }
    }
    Ok(url.into())
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    error: Option<String>,
    error_description: Option<String>,
}

async fn token_request(
    http: &reqwest::Client,
    app: &OAuthApp,
    mut form: Vec<(&str, String)>,
    now: DateTime<Utc>,
) -> Result<Tokens, SyncError> {
    form.push(("client_id", app.client_id.clone()));
    if let Some(secret) = &app.client_secret {
        form.push(("client_secret", secret.clone()));
    }
    let response = http
        .post(&app.token_url)
        .header("Accept", "application/json")
        .form(&form)
        .send()
        .await
        .map_err(|error| SyncError::Transport(crate::webdav::describe_transport_error(&error)))?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let parsed: Option<TokenResponse> = serde_json::from_str(&body).ok();
    match parsed {
        Some(TokenResponse {
            access_token: Some(access_token),
            refresh_token,
            expires_in,
            ..
        }) if status.is_success() && !access_token.is_empty() => Ok(Tokens {
            access_token,
            refresh_token: refresh_token.filter(|token| !token.is_empty()),
            expires_at: expires_in
                .filter(|seconds| *seconds > 0)
                .map(|seconds| now + ChronoDuration::seconds(seconds)),
        }),
        other => {
            let detail = other
                .and_then(|r| r.error_description.or(r.error))
                .unwrap_or_else(|| format!("HTTP {}", status.as_u16()));
            Err(SyncError::OAuth(format!(
                "Token-Austausch mit {} fehlgeschlagen: {detail}",
                app.service
            )))
        }
    }
}

/// Tauscht den Code der Autorisierung gegen Tokens.
pub async fn exchange_code(
    http: &reqwest::Client,
    app: &OAuthApp,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
    now: DateTime<Utc>,
) -> Result<Tokens, SyncError> {
    token_request(
        http,
        app,
        vec![
            ("grant_type", "authorization_code".to_owned()),
            ("code", code.to_owned()),
            ("code_verifier", verifier.to_owned()),
            ("redirect_uri", redirect_uri.to_owned()),
        ],
        now,
    )
    .await
}

/// Erneuert das Access-Token. Der Dienst darf das Refresh-Token drehen; sonst bleibt das alte gültig.
pub async fn refresh_tokens(
    http: &reqwest::Client,
    app: &OAuthApp,
    refresh_token: &str,
    now: DateTime<Utc>,
) -> Result<Tokens, SyncError> {
    let mut tokens = token_request(
        http,
        app,
        vec![
            ("grant_type", "refresh_token".to_owned()),
            ("refresh_token", refresh_token.to_owned()),
        ],
        now,
    )
    .await
    .map_err(|error| match error {
        SyncError::OAuth(_) => SyncError::AuthExpired(app.service),
        other => other,
    })?;
    tokens
        .refresh_token
        .get_or_insert_with(|| refresh_token.to_owned());
    Ok(tokens)
}

/// Wie die Anmeldung abläuft.
#[derive(Debug, Clone)]
pub struct AuthorizeOptions {
    /// Ports für den Rückruf in der Reihenfolge der Versuche (`0` = beliebig).
    pub ports: Vec<u16>,
    pub callback_path: &'static str,
    pub timeout: Duration,
    pub success_page: Page,
}

/// Vollständige Anmeldung: Server starten, Browser öffnen, auf den Rückruf warten, Code tauschen.
pub async fn authorize(
    http: &reqwest::Client,
    app: &OAuthApp,
    options: &AuthorizeOptions,
    opener: &dyn UrlOpener,
    clock: &dyn Clock,
) -> Result<Tokens, SyncError> {
    let loopback = Loopback::bind(&options.ports, options.callback_path).await?;
    let redirect_uri = loopback.redirect_uri();
    let pkce = pkce()?;
    let state = random_hex(16)?;
    let url = authorize_url(app, &state, &pkce.challenge, &redirect_uri)?;
    opener.open(&url)?;
    let code = loopback
        .wait_for_code(&state, options.timeout, &options.success_page, app.service)
        .await?;
    exchange_code(http, app, &code, &pkce.verifier, &redirect_uri, clock.now()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_challenge_matches_the_rfc_example() {
        // RFC 7636, Anhang B
        let pkce = pkce_from("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk");
        assert_eq!(
            pkce.challenge,
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn generated_values_are_long_and_different() {
        let a = pkce().unwrap();
        let b = pkce().unwrap();
        assert_eq!(a.verifier.len(), 43);
        assert_ne!(a.verifier, b.verifier);
        assert_eq!(random_hex(16).unwrap().len(), 32);
    }

    #[test]
    fn tokens_expire_a_minute_early_and_never_without_a_date() {
        let now = DateTime::parse_from_rfc3339("2026-10-04T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let tokens = |seconds: Option<i64>| Tokens {
            access_token: "a".into(),
            refresh_token: None,
            expires_at: seconds.map(|s| now + ChronoDuration::seconds(s)),
        };
        assert!(!tokens(None).is_expired(now));
        assert!(!tokens(Some(61)).is_expired(now));
        assert!(tokens(Some(60)).is_expired(now));
        assert!(tokens(Some(-5)).is_expired(now));
    }

    #[test]
    fn tokens_are_not_printed() {
        let tokens = Tokens {
            access_token: "geheim".into(),
            refresh_token: Some("auch-geheim".into()),
            expires_at: None,
        };
        let text = format!("{tokens:?}");
        assert!(
            text.starts_with("Tokens") && !text.contains("geheim"),
            "{text}"
        );
    }

    #[test]
    fn the_authorize_url_carries_pkce_state_and_extras() {
        let app = OAuthApp {
            service: "Test",
            client_id: "id 1".into(),
            client_secret: None,
            auth_url: "https://auth.example/o/authorize".into(),
            token_url: String::new(),
            scope: "a b".into(),
            auth_params: vec![("access_type".into(), "offline".into())],
        };
        let url =
            Url::parse(&authorize_url(&app, "st", "ch", "http://127.0.0.1:1/cb").unwrap()).unwrap();
        let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["client_id"], "id 1");
        assert_eq!(query["scope"], "a b");
        assert_eq!(query["state"], "st");
        assert_eq!(query["code_challenge"], "ch");
        assert_eq!(query["code_challenge_method"], "S256");
        assert_eq!(query["response_type"], "code");
        assert_eq!(query["redirect_uri"], "http://127.0.0.1:1/cb");
        assert_eq!(query["access_type"], "offline");
    }

    mod token_endpoint {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        use super::super::*;

        fn app(server: &MockServer) -> OAuthApp {
            OAuthApp {
                service: "Test",
                client_id: "id".into(),
                client_secret: Some("secret".into()),
                auth_url: String::new(),
                token_url: format!("{}/token", server.uri()),
                scope: String::new(),
                auth_params: vec![],
            }
        }

        fn now() -> DateTime<Utc> {
            DateTime::parse_from_rfc3339("2026-10-04T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        }

        async fn answer(status: u16, body: serde_json::Value) -> (MockServer, OAuthApp) {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/token"))
                .respond_with(ResponseTemplate::new(status).set_body_json(body))
                .mount(&server)
                .await;
            let app = app(&server);
            (server, app)
        }

        async fn exchange(status: u16, body: serde_json::Value) -> Result<Tokens, SyncError> {
            let (_server, app) = answer(status, body).await;
            exchange_code(
                &http_client().unwrap(),
                &app,
                "code",
                "verifier",
                "http://127.0.0.1:1/cb",
                now(),
            )
            .await
        }

        #[tokio::test]
        async fn tokens_with_a_lifetime_expire_at_the_right_time() {
            let tokens = exchange(
                200,
                serde_json::json!({ "access_token": "a", "refresh_token": "r", "expires_in": 3600 }),
            )
            .await
            .unwrap();
            assert_eq!(tokens.access_token, "a");
            assert_eq!(tokens.refresh_token.as_deref(), Some("r"));
            assert_eq!(
                tokens.expires_at,
                Some(now() + ChronoDuration::seconds(3600))
            );
        }

        #[tokio::test]
        async fn tokens_without_a_positive_lifetime_never_expire() {
            for body in [
                serde_json::json!({ "access_token": "a" }),
                serde_json::json!({ "access_token": "a", "expires_in": 0 }),
                serde_json::json!({ "access_token": "a", "expires_in": -5, "refresh_token": "" }),
            ] {
                let tokens = exchange(200, body).await.unwrap();
                assert_eq!(tokens.expires_at, None);
                assert_eq!(tokens.refresh_token, None);
            }
        }

        #[tokio::test]
        async fn a_one_second_lifetime_counts() {
            let tokens = exchange(
                200,
                serde_json::json!({ "access_token": "a", "expires_in": 1 }),
            )
            .await
            .unwrap();
            assert_eq!(tokens.expires_at, Some(now() + ChronoDuration::seconds(1)));
        }

        #[tokio::test]
        async fn failures_name_the_reason() {
            // ein Token trotz Fehlerstatus gilt nicht
            let error = exchange(
                400,
                serde_json::json!({ "access_token": "a", "error": "invalid_grant" }),
            )
            .await
            .unwrap_err()
            .to_string();
            assert_eq!(
                error,
                "Token-Austausch mit Test fehlgeschlagen: invalid_grant"
            );
            // leeres Token trotz Erfolgsstatus
            let error = exchange(200, serde_json::json!({ "access_token": "" }))
                .await
                .unwrap_err()
                .to_string();
            assert_eq!(error, "Token-Austausch mit Test fehlgeschlagen: HTTP 200");
            let error = exchange(
                400,
                serde_json::json!({ "error": "x", "error_description": "Beschreibung" }),
            )
            .await
            .unwrap_err()
            .to_string();
            assert!(error.ends_with("Beschreibung"), "{error}");
        }

        #[tokio::test]
        async fn renewal_keeps_the_old_refresh_token_when_none_comes_back() {
            let (_server, app) = answer(
                200,
                serde_json::json!({ "access_token": "neu", "expires_in": 60 }),
            )
            .await;
            let tokens = refresh_tokens(&http_client().unwrap(), &app, "alt", now())
                .await
                .unwrap();
            assert_eq!(tokens.access_token, "neu");
            assert_eq!(tokens.refresh_token.as_deref(), Some("alt"));
        }

        #[tokio::test]
        async fn a_rejected_renewal_asks_for_a_new_login() {
            let (_server, app) = answer(400, serde_json::json!({ "error": "invalid_grant" })).await;
            let error = refresh_tokens(&http_client().unwrap(), &app, "alt", now())
                .await
                .unwrap_err();
            assert!(matches!(error, SyncError::AuthExpired("Test")), "{error}");
        }
    }
}
