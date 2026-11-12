use cucumber::{given, then, when};
use nunc_core::jira::JiraKind;
use nunc_jira::{
    JiraClient, JiraCredentials, JiraError, SECRET_ACCOUNT, SECRET_SERVICE, clear_jira,
    load_jira_credentials, save_jira,
};
use nunc_platform::SecretStore;
use wiremock::matchers::{basic_auth, bearer_token, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::{JiraWorld, config};

const ISSUES: &str = r#"{"issues":[{"key":"PROJ-1","fields":{"summary":"First"}},{"key":"PROJ-2","fields":{"summary":"Second"}}]}"#;

async fn serve(world: &mut JiraWorld, kind: JiraKind, email: &str, token: &str) {
    let server = MockServer::start().await;
    let (search, myself) = match kind {
        JiraKind::Cloud => ("/rest/api/3/search/jql", "/rest/api/3/myself"),
        JiraKind::DataCenter => ("/rest/api/2/search", "/rest/api/2/myself"),
    };
    for (endpoint, body) in [(search, ISSUES), (myself, r#"{"name":"me"}"#)] {
        let mock = Mock::given(method("GET")).and(path(endpoint));
        let mock = match kind {
            JiraKind::Cloud => mock.and(basic_auth(email, token)),
            JiraKind::DataCenter => mock.and(bearer_token(token)),
        };
        mock.respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;
    }
    // Wie Jira: alles, was nicht zu den Zugangsdaten passt, ist 401
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401))
        .with_priority(10)
        .mount(&server)
        .await;
    world.credentials = Some(JiraCredentials {
        config: config(kind, server.uri(), email),
        token: token.to_owned(),
    });
    world.server = Some(server);
}

#[given(expr = "a Jira Cloud server for {string} with the token {string}")]
async fn cloud_server(world: &mut JiraWorld, email: String, token: String) {
    serve(world, JiraKind::Cloud, &email, &token).await;
}

#[given(expr = "a Jira Data Center server with the token {string}")]
async fn dc_server(world: &mut JiraWorld, token: String) {
    serve(world, JiraKind::DataCenter, "", &token).await;
}

async fn catch_all(world: &mut JiraWorld, response: ResponseTemplate) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(response)
        .mount(&server)
        .await;
    world.credentials = Some(JiraCredentials {
        config: config(JiraKind::DataCenter, server.uri(), ""),
        token: "pat".to_owned(),
    });
    world.server = Some(server);
}

#[given(expr = "a Jira server that answers with status {int}")]
async fn status_server(world: &mut JiraWorld, status: u16) {
    catch_all(world, ResponseTemplate::new(status)).await;
}

#[given(expr = "a Jira server that answers with the body {string}")]
async fn body_server(world: &mut JiraWorld, body: String) {
    catch_all(world, ResponseTemplate::new(200).set_body_string(body)).await;
}

#[given("no Jira server is running")]
fn no_server(world: &mut JiraWorld) {
    world.credentials = Some(JiraCredentials {
        // Port 1 ist nicht belegt: die Verbindung wird abgelehnt
        config: config(JiraKind::DataCenter, "http://127.0.0.1:1".into(), ""),
        token: "pat".to_owned(),
    });
}

#[given(expr = "the client uses the token {string}")]
fn other_token(world: &mut JiraWorld, token: String) {
    world.credentials.as_mut().expect("kein Zugang").token = token;
}

#[given(expr = "the client uses the e-mail {string}")]
fn other_email(world: &mut JiraWorld, email: String) {
    world
        .credentials
        .as_mut()
        .expect("kein Zugang")
        .config
        .email = email;
}

#[given("the base URL has a trailing slash")]
fn trailing_slash(world: &mut JiraWorld) {
    world
        .credentials
        .as_mut()
        .expect("kein Zugang")
        .config
        .base_url += "/";
}

fn client(world: &JiraWorld) -> JiraClient {
    JiraClient::new(world.credentials.clone().expect("kein Zugang")).expect("Client")
}

#[when(expr = "I search Jira for {string}")]
async fn search(world: &mut JiraWorld, query: String) {
    match client(world).search(&query).await {
        Ok(hits) => world.hits = Some(hits),
        Err(error) => world.error = Some(error),
    }
}

#[when("I test the Jira connection")]
async fn test_connection(world: &mut JiraWorld) {
    match client(world).test().await {
        Ok(()) => world.test_ok = true,
        Err(error) => world.error = Some(error),
    }
}

#[then(expr = "I get {int} Jira hits")]
fn hit_count(world: &mut JiraWorld, count: usize) {
    let hits = world.hits.as_ref().unwrap_or_else(|| {
        panic!(
            "keine Treffer, Fehler: {:?}",
            world.error.as_ref().map(|e| e.to_string())
        )
    });
    assert_eq!(hits.len(), count);
}

#[then(expr = "ticket {int} of the Jira hits is {string} {string}")]
fn hit_is(world: &mut JiraWorld, n: usize, key: String, summary: String) {
    let hit = &world.hits.as_ref().expect("keine Treffer")[n - 1];
    let base = world.server.as_ref().expect("kein Server").uri();
    assert_eq!(hit.key, key);
    assert_eq!(hit.summary, summary);
    assert_eq!(hit.url, format!("{base}/browse/{key}"));
}

async fn last_request(world: &JiraWorld) -> wiremock::Request {
    let requests = world
        .server
        .as_ref()
        .expect("kein Server")
        .received_requests()
        .await
        .expect("Aufzeichnung aus");
    requests.last().expect("keine Anfrage").clone()
}

fn param(request: &wiremock::Request, name: &str) -> Option<String> {
    request
        .url
        .query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
}

#[then(expr = "the Jira server received the JQL {string}")]
async fn received_jql(world: &mut JiraWorld, jql: String) {
    let request = last_request(world).await;
    assert_eq!(param(&request, "jql"), Some(jql.replace("\\\"", "\"")));
}

#[then(expr = "the Jira server was asked for {int} results with the field {string}")]
async fn received_paging(world: &mut JiraWorld, max: u32, field: String) {
    let request = last_request(world).await;
    assert_eq!(param(&request, "maxResults"), Some(max.to_string()));
    assert_eq!(param(&request, "fields"), Some(field));
}

#[then("the Jira connection test succeeds")]
fn test_succeeds(world: &mut JiraWorld) {
    assert!(world.test_ok, "Fehler: {:?}", world.error);
}

fn kind_name(error: &JiraError) -> String {
    match error {
        JiraError::NotConfigured => "not configured".into(),
        JiraError::Unauthorized => "unauthorized".into(),
        JiraError::Http(status) => format!("http {status}"),
        JiraError::Transport(_) => "transport".into(),
        JiraError::BadResponse(_) => "bad response".into(),
        JiraError::Store(_) => "store".into(),
        JiraError::Secret(_) => "secret".into(),
    }
}

#[then(expr = "the Jira call fails with {string}")]
fn fails_with(world: &mut JiraWorld, expected: String) {
    let error = world.error.as_ref().expect("kein Fehler");
    assert_eq!(kind_name(error), expected);
}

#[then(expr = "the Jira error text does not contain {string}")]
fn error_text_clean(world: &mut JiraWorld, secret: String) {
    let error = world.error.as_ref().expect("kein Fehler");
    assert!(!error.to_string().contains(&secret) && !format!("{error:?}").contains(&secret));
}

#[then(expr = "the Jira credentials debug output does not contain {string}")]
fn debug_clean(world: &mut JiraWorld, secret: String) {
    let credentials = world.credentials.as_ref().expect("kein Zugang");
    assert!(!format!("{credentials:?}").contains(&secret));
}

// ---- Zugang -----------------------------------------------------------------------------------

fn kind(name: &str) -> JiraKind {
    name.parse().expect("Jira-Art")
}

#[when(expr = "I save the Jira configuration {string} {string} {string} with the token {string}")]
#[given(
    expr = "the Jira configuration {string} {string} {string} with the token {string} is saved"
)]
fn save_with_token(world: &mut JiraWorld, k: String, url: String, email: String, token: String) {
    // Ein leeres Token im Beispiel steht für „nicht gespeichert“
    let token = (!token.is_empty()).then_some(token);
    save_jira(
        &mut world.conn,
        &world.secrets,
        &config(kind(&k), url, &email),
        token.as_deref(),
    )
    .expect("speichern");
}

#[when(expr = "I save the Jira configuration {string} {string} {string} without a token")]
fn save_without_token(world: &mut JiraWorld, k: String, url: String, email: String) {
    save_jira(
        &mut world.conn,
        &world.secrets,
        &config(kind(&k), url, &email),
        None,
    )
    .expect("speichern");
}

#[when("I clear the Jira configuration")]
fn clear(world: &mut JiraWorld) {
    clear_jira(&world.conn, &world.secrets).expect("entfernen");
}

#[when("I load the Jira credentials")]
fn load(world: &mut JiraWorld) {
    match load_jira_credentials(&world.conn, &world.secrets) {
        Ok(credentials) => world.credentials = Some(credentials),
        Err(error) => world.error = Some(error),
    }
}

#[then(expr = "the vault holds the Jira token {string}")]
fn vault_holds(world: &mut JiraWorld, token: String) {
    let stored = world.secrets.get(SECRET_SERVICE, SECRET_ACCOUNT).unwrap();
    assert_eq!(stored, Some(token));
}

#[then("the vault holds no Jira token")]
fn vault_empty(world: &mut JiraWorld) {
    assert_eq!(
        world.secrets.get(SECRET_SERVICE, SECRET_ACCOUNT).unwrap(),
        None
    );
}

#[then(expr = "the saved Jira configuration is {string} {string} {string}")]
fn saved_config(world: &mut JiraWorld, k: String, url: String, email: String) {
    let loaded = nunc_store::load_jira_config(&world.conn).unwrap();
    assert_eq!(loaded, config(kind(&k), url, &email));
}

#[then(expr = "the database does not contain {string}")]
fn db_clean(world: &mut JiraWorld, secret: String) {
    let hits: i64 = world
        .conn
        .query_row(
            "SELECT COUNT(*) FROM settings WHERE value LIKE '%' || ?1 || '%'",
            [&secret],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(hits, 0);
}

#[then("the Jira credentials can be loaded")]
fn loadable(world: &mut JiraWorld) {
    assert!(load_jira_credentials(&world.conn, &world.secrets).is_ok());
}

#[then(expr = "the Jira call fails when loading the credentials with {string}")]
fn load_fails(world: &mut JiraWorld, expected: String) {
    let error = load_jira_credentials(&world.conn, &world.secrets).expect_err("geladen");
    assert_eq!(kind_name(&error), expected);
}
