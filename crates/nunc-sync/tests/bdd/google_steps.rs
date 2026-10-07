//! Steps für die Google-Drive-Szenarien: Anmeldung, Tresor, Sync-Lauf gegen das Test-„Google“.

use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Duration as ChronoDuration;
use cucumber::{given, then, when};
use nunc_core::clock::Clock;
use nunc_core::model::{DriveStatus, SyncImportInfo};
use nunc_core::sync_document::SyncDocument;
use nunc_platform::{MemorySecretStore, SecretStore};
use nunc_sync::oauth::{Connection, Tokens, Vault};
use nunc_sync::{DriveConfig, DriveRunner, DriveService, SyncRunner};
use sha2::{Digest, Sha256};

use crate::SyncWorld;
use crate::google::{AutoBrowser, Behavior, Google};

const ACCOUNT: &str = "google-drive";
const CURRENT: &str = "nunc-sync.v2.json";
const LEGACY: &str = "nunc-sync.json";

pub struct State {
    pub server: Option<Google>,
    pub secrets: Arc<MemorySecretStore>,
    pub browser: Option<Arc<AutoBrowser>>,
    pub client_id: String,
    pub auth_timeout: Duration,
    pub error: Option<String>,
    pub status: Option<DriveStatus>,
    pub import: Option<SyncImportInfo>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            server: None,
            secrets: Arc::new(MemorySecretStore::default()),
            browser: None,
            client_id: "client-123".to_owned(),
            auth_timeout: Duration::from_secs(3),
            error: None,
            status: None,
            import: None,
        }
    }
}

fn google(world: &SyncWorld) -> &Google {
    world.google.server.as_ref().expect("kein Google gestartet")
}

fn browser(world: &SyncWorld) -> Arc<AutoBrowser> {
    world.google.browser.clone().expect("kein Browser")
}

pub fn service(world: &SyncWorld) -> Arc<DriveService> {
    let uri = google(world).server.uri();
    let mut config = DriveConfig::google(&world.google.client_id, "secret-456");
    config.app.auth_url = format!("{uri}/auth");
    config.app.token_url = format!("{uri}/token");
    config.api_base = format!("{uri}/drive/v3");
    config.upload_base = format!("{uri}/upload/drive/v3");
    config.auth_timeout = world.google.auth_timeout;
    Arc::new(
        DriveService::new(config, world.google.secrets.clone(), world.clock.clone())
            .expect("Drive-Dienst"),
    )
}

fn vault(world: &SyncWorld) -> Vault {
    Vault::new(world.google.secrets.clone(), ACCOUNT)
}

// ---------------------------------------------------------------------------------------------
// Given

#[given(expr = "a Google server for the account {string}")]
async fn server_for(world: &mut SyncWorld, email: String) {
    world.google.server = Some(Google::start(Some(&email)).await);
    world.google.browser = Some(Arc::new(
        AutoBrowser::new(tokio::runtime::Handle::current()),
    ));
}

#[given("a Google server that does not tell the account")]
async fn server_without_account(world: &mut SyncWorld) {
    server_for(world, String::new()).await;
    google(world).with(|state| {
        state.email = None;
        state.about_fails = true;
    });
}

#[given(expr = "Google Drive is connected as {string}")]
fn connected_as(world: &mut SyncWorld, email: String) {
    google(world).with(|state| state.valid.insert("access-0".to_owned()));
    vault(world)
        .save(&Connection {
            tokens: Tokens {
                access_token: "access-0".to_owned(),
                refresh_token: Some("refresh-0".to_owned()),
                expires_at: Some(world.clock.now() + ChronoDuration::hours(1)),
            },
            email: Some(email),
            avatar_url: None,
        })
        .unwrap();
}

#[given("the user denies the authorization")]
fn user_denies(world: &mut SyncWorld) {
    *browser(world).behavior.lock().unwrap() = Behavior::Deny;
}

#[given("the user ignores the browser")]
fn user_ignores(world: &mut SyncWorld) {
    *browser(world).behavior.lock().unwrap() = Behavior::Ignore;
}

#[given(expr = "the authorization times out after {int} second(s)")]
fn times_out(world: &mut SyncWorld, seconds: u64) {
    world.google.auth_timeout = Duration::from_secs(seconds);
}

#[given(expr = "Google rejects the authorization code with {string}")]
fn rejects_code(world: &mut SyncWorld, error: String) {
    google(world).with(|state| state.reject_code = Some(error));
}

#[given("Google refuses to renew tokens")]
fn refuses_renewal(world: &mut SyncWorld) {
    google(world).with(|state| state.reject_refresh = true);
}

#[given("the access token has expired")]
fn token_expired(world: &mut SyncWorld) {
    let vault = vault(world);
    let mut connection = vault.load().unwrap().expect("nicht verbunden");
    connection.tokens.expires_at = Some(world.clock.now() - ChronoDuration::minutes(1));
    vault.save(&connection).unwrap();
}

#[given("Google no longer accepts the access token")]
fn token_rejected(world: &mut SyncWorld) {
    google(world).with(|state| state.valid.clear());
}

#[given("the app has no Google credentials")]
fn no_credentials(world: &mut SyncWorld) {
    world.google.client_id = String::new();
}

#[given(expr = "Google answers every request with {int}")]
#[when(expr = "Google answers every request with {int}")]
fn google_fails(world: &mut SyncWorld, status: u16) {
    google(world).with(|state| state.fail_all = Some(status));
}

#[given(expr = "the Drive folder {string} holds the sync file {string}")]
fn folder_holding(world: &mut SyncWorld, folder: String, fixture: String) {
    let bytes =
        crate::steps::current_bytes(&std::fs::read(SyncWorld::fixture(&fixture)).expect("Fixture"));
    google(world).with(|state| {
        let id = state.add_folder(&folder);
        state.add_file(CURRENT, &id, bytes);
    });
}

#[given(expr = "the Drive folder {string} holds the legacy sync file {string}")]
fn folder_holding_legacy(world: &mut SyncWorld, folder: String, fixture: String) {
    let bytes = std::fs::read(SyncWorld::fixture(&fixture)).expect("Fixture");
    google(world).with(|state| {
        let id = state.add_folder(&folder);
        state.add_file(LEGACY, &id, bytes);
    });
}

#[given(expr = "the Drive folder {string} holds a broken sync file")]
fn folder_broken(world: &mut SyncWorld, folder: String) {
    google(world).with(|state| {
        let id = state.add_folder(&folder);
        state.add_file(CURRENT, &id, b"{ kein JSON".to_vec());
    });
}

#[given(expr = "the Drive folder {string} exists without a sync file")]
fn folder_empty(world: &mut SyncWorld, folder: String) {
    google(world).with(|state| state.add_folder(&folder));
}

#[given(expr = "another device adds the todo {string} to the Drive file during the first upload")]
fn another_device(world: &mut SyncWorld, text: String) {
    google(world).with(|state| {
        let item = state.file_in("nunc", CURRENT).expect("keine Sync-Datei");
        state.interfere = Some(crate::steps::with_extra_todo(&item.content, &text));
    });
}

// ---------------------------------------------------------------------------------------------
// When

#[when("I connect Google Drive")]
async fn connect(world: &mut SyncWorld) {
    let service = service(world);
    let browser = browser(world);
    match service.connect(&*browser).await {
        Ok(status) => {
            world.google.status = Some(status);
            world.google.error = None;
        }
        Err(error) => world.google.error = Some(error.to_string()),
    }
}

#[when("I disconnect Google Drive")]
fn disconnect(world: &mut SyncWorld) {
    service(world).disconnect().unwrap();
}

#[when("I sync with Google Drive")]
async fn sync_drive(world: &mut SyncWorld) {
    let service = service(world);
    world.outcome = Some(service.sync(&*world.store).await.map(|(result, _)| result));
}

#[when("the Drive runner runs")]
async fn runner_runs(world: &mut SyncWorld) {
    let runner = DriveRunner::new(world.store.clone(), service(world), world.clock.clone());
    world.coord.runner_outcome = Some(runner.run().await);
}

#[when("I look for a sync file to import")]
async fn look_for_import(world: &mut SyncWorld) {
    let service = service(world);
    match service.check_import(&*world.store).await {
        Ok(info) => {
            world.google.import = Some(info);
            world.google.error = None;
        }
        Err(error) => world.google.error = Some(error.to_string()),
    }
}

#[given("the coordinator runs the Drive runner")]
fn coordinator_runs_drive(world: &mut SyncWorld) {
    let runner = DriveRunner::new(world.store.clone(), service(world), world.clock.clone());
    world.coord.runner_override = Some(Arc::new(runner));
}

// ---------------------------------------------------------------------------------------------
// Then

fn opened_url(world: &SyncWorld) -> reqwest::Url {
    let urls = browser(world).opened();
    assert_eq!(urls.len(), 1, "Der Browser sollte einmal geöffnet werden");
    urls.into_iter().next().unwrap()
}

fn param(url: &reqwest::Url, name: &str) -> String {
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
        .unwrap_or_else(|| panic!("Parameter {name} fehlt in {url}"))
}

#[then("the browser was opened once with the Google authorization page")]
fn browser_opened(world: &mut SyncWorld) {
    let url = opened_url(world);
    assert!(
        url.as_str()
            .starts_with(&format!("{}/auth?", google(world).server.uri()))
    );
    assert_eq!(param(&url, "client_id"), "client-123");
    assert_eq!(param(&url, "response_type"), "code");
    assert!(param(&url, "redirect_uri").starts_with("http://127.0.0.1:"));
}

#[then(expr = "the authorization request uses the code challenge method {string}")]
fn challenge_method(world: &mut SyncWorld, method: String) {
    assert_eq!(param(&opened_url(world), "code_challenge_method"), method);
}

#[then("the token request carries the code verifier of the challenge")]
fn verifier_matches(world: &mut SyncWorld) {
    let challenge = param(&opened_url(world), "code_challenge");
    let request = google(world).with(|state| state.token_requests[0].clone());
    assert_eq!(request["grant_type"], "authorization_code");
    assert_eq!(request["code"], "auth-code");
    let verifier = &request["code_verifier"];
    assert_eq!(
        URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
        challenge
    );
    assert_eq!(
        request["redirect_uri"],
        param(&opened_url(world), "redirect_uri")
    );
}

#[then(expr = "the authorization request asks for the scope {string} only")]
fn only_scope(world: &mut SyncWorld, scope: String) {
    assert_eq!(param(&opened_url(world), "scope"), scope);
}

#[then(expr = "the Drive status shows the account {string}")]
fn status_account(world: &mut SyncWorld, email: String) {
    let status = world.google.status.as_ref().expect("keine Verbindung");
    assert!(status.connected);
    assert_eq!(status.email.as_deref(), Some(email.as_str()));
}

#[then("the Drive status shows no account")]
fn status_no_account(world: &mut SyncWorld) {
    let status = world.google.status.as_ref().expect("keine Verbindung");
    assert!(status.connected);
    assert_eq!(status.email, None);
}

#[then("the secret store holds a Google Drive connection")]
fn holds(world: &mut SyncWorld) {
    assert!(world.google.secrets.get("nunc", ACCOUNT).unwrap().is_some());
}

#[then("the secret store holds no Google Drive connection")]
fn holds_nothing(world: &mut SyncWorld) {
    assert!(world.google.secrets.get("nunc", ACCOUNT).unwrap().is_none());
}

#[then(expr = "the stored refresh token is {string}")]
fn stored_refresh(world: &mut SyncWorld, token: String) {
    let connection = vault(world).load().unwrap().expect("nicht verbunden");
    assert_eq!(
        connection.tokens.refresh_token.as_deref(),
        Some(token.as_str())
    );
}

#[then(expr = "Google issued {int} tokens by renewal")]
fn renewals(world: &mut SyncWorld, count: usize) {
    let renewals = google(world).with(|state| {
        state
            .token_requests
            .iter()
            .filter(|request| request["grant_type"] == "refresh_token")
            .count()
    });
    assert_eq!(renewals, count);
}

#[then(expr = "connecting failed with {string}")]
fn connecting_failed(world: &mut SyncWorld, message: String) {
    assert_eq!(world.google.error.as_deref(), Some(message.as_str()));
}

#[then("Google was not contacted")]
fn not_contacted(world: &mut SyncWorld) {
    assert!(google(world).with(|state| state.log.is_empty() && state.token_requests.is_empty()));
}

#[then(expr = "the sync file on Drive has the todos {string}")]
fn drive_todos(world: &mut SyncWorld, texts: String) {
    let mut expected: Vec<String> = texts
        .split(',')
        .map(|text| text.trim().to_owned())
        .collect();
    let mut actual: Vec<String> = drive_file(world)
        .todos
        .iter()
        .map(|todo| todo.raw_text())
        .collect();
    expected.sort();
    actual.sort();
    assert_eq!(actual, expected);
}

#[then(expr = "the sync file on Drive has {int} todos and {int} sessions")]
fn drive_counts(world: &mut SyncWorld, todos: usize, sessions: usize) {
    let file = drive_file(world);
    assert_eq!((file.todos.len(), file.sessions.len()), (todos, sessions));
}

fn drive_file(world: &SyncWorld) -> SyncDocument {
    let content = google(world).with(|state| {
        state
            .file_in("nunc", CURRENT)
            .map(|item| item.content.clone())
    });
    serde_json::from_slice(&content.expect("keine Sync-Datei im Ordner nunc")).unwrap()
}

#[then("the sync file on Drive still exists")]
fn drive_file_exists(world: &mut SyncWorld) {
    drive_file(world);
}

#[then(expr = "Drive holds {int} folder(s) named {string}")]
fn folder_count(world: &mut SyncWorld, count: usize, name: String) {
    let actual = google(world).with(|state| {
        state
            .items
            .iter()
            .filter(|i| i.folder && i.name == name)
            .count()
    });
    assert_eq!(actual, count);
}

#[then(expr = "Drive holds {int} sync file(s)")]
fn file_count(world: &mut SyncWorld, count: usize) {
    let actual = google(world).with(|state| state.items.iter().filter(|i| !i.folder).count());
    assert_eq!(actual, count);
}

#[then("the Drive folder is remembered in the settings")]
fn folder_remembered(world: &mut SyncWorld) {
    let expected = google(world).with(|state| state.folder("nunc").map(|f| f.id.clone()));
    let conn = world.store.conn.lock().unwrap();
    assert_eq!(nunc_store::drive_folder_id(&conn).unwrap(), expected);
}

#[then(expr = "the last Drive sync time in the settings is {string}")]
fn last_drive_sync(world: &mut SyncWorld, time: String) {
    let conn = world.store.conn.lock().unwrap();
    assert_eq!(
        nunc_store::last_sync_of(&conn, nunc_core::model::SyncProvider::GoogleDrive)
            .unwrap()
            .as_deref(),
        Some(time.as_str())
    );
}

#[then(expr = "the import offer shows {int} todos and {int} sessions")]
fn import_offer(world: &mut SyncWorld, todos: u32, sessions: u32) {
    let info = world.google.import.expect("keine Prüfung");
    assert!(info.available);
    assert_eq!((info.todo_count, info.session_count), (todos, sessions));
}

#[then("no import is offered")]
fn no_import(world: &mut SyncWorld) {
    assert!(!world.google.import.expect("keine Prüfung").available);
}

#[then(expr = "the lookup failed with {string}")]
fn lookup_failed(world: &mut SyncWorld, message: String) {
    assert_eq!(world.google.error.as_deref(), Some(message.as_str()));
}

#[then(expr = "within {int} ms the sync file appears on Drive")]
async fn appears(world: &mut SyncWorld, within: u64) {
    let deadline = std::time::Instant::now() + Duration::from_millis(within);
    loop {
        if google(world).with(|state| state.file_in("nunc", CURRENT).is_some()) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Sync-Datei erschien nicht"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[then("the browser was not opened")]
fn browser_not_opened(world: &mut SyncWorld) {
    assert!(browser(world).opened().is_empty());
}

#[then("the Drive sync file is unchanged")]
fn drive_unchanged(world: &mut SyncWorld) {
    let content = google(world).with(|state| {
        state
            .file_in("nunc", CURRENT)
            .map(|item| (item.content.clone(), item.version))
    });
    assert_eq!(content, Some((b"{ kein JSON".to_vec(), 1)));
}

#[then(expr = "the sync file on Drive has {int} sessions in the local database")]
fn sessions_match(world: &mut SyncWorld, count: i64) {
    let conn = world.store.conn.lock().unwrap();
    let actual: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE status = 'completed'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(actual, count);
}
