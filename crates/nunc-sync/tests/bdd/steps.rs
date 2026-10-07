use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use nunc_core::crdt::SyncState;
use nunc_core::model::{SyncFile, SyncTodo, WebDavAuth, WebDavConfig};
use nunc_core::sync_document::SyncDocument;
use nunc_sync::{WebDavClient, load_credentials, save_webdav, sync_webdav};

use crate::SyncWorld;
use crate::dav::{Auth, Dav, FILE_NAME, LEGACY_FILE_NAME};

fn rows(step: &Step) -> Vec<Vec<String>> {
    step.table.as_ref().expect("Tabelle fehlt").rows.clone()
}

fn folder_path(folder: &str) -> String {
    folder.to_owned()
}

fn config_for(world: &SyncWorld, user: &str, auth: WebDavAuth, url: String) -> WebDavConfig {
    WebDavConfig {
        url,
        auth_method: auth,
        username: user.to_owned(),
        remote_path: world.remote_path.clone(),
    }
}

fn server_url(world: &SyncWorld) -> String {
    match (&world.dav, &world.dead_url) {
        (Some(dav), _) => dav.server.uri(),
        (None, Some(url)) => url.clone(),
        (None, None) => panic!("kein Server"),
    }
}

// ---------------------------------------------------------------------------------------------
// Server

#[given(expr = "a WebDAV server with the folder {string}")]
async fn server_with_folder(world: &mut SyncWorld, folder: String) {
    let dav = Dav::start().await;
    dav.with(|state| state.folders.insert(folder_path(&folder)));
    world.dav = Some(dav);
}

#[given("a WebDAV server without folders")]
async fn server_without_folders(world: &mut SyncWorld) {
    world.dav = Some(Dav::start().await);
}

#[given("the server hides the existing folder from lookups")]
fn server_blind(world: &mut SyncWorld) {
    world.dav().with(|state| state.propfind_blind = true);
}

#[then(expr = "the server received {int} requests")]
fn request_count(world: &mut SyncWorld, count: usize) {
    assert_eq!(world.dav().with(|state| state.log.len()), count);
}

#[given("the server does not support ETags")]
fn server_without_etags(world: &mut SyncWorld) {
    world.dav().with(|state| state.etags = false);
}

#[given(expr = "a WebDAV server that answers every request with {int}")]
async fn server_failing(world: &mut SyncWorld, status: u16) {
    let dav = Dav::start().await;
    dav.with(|state| state.fail_all = Some(status));
    world.dav = Some(dav);
}

#[given("a WebDAV server that is not running")]
async fn server_dead(world: &mut SyncWorld) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    world.dead_url = Some(format!("http://{}", listener.local_addr().unwrap()));
    drop(listener);
}

#[given(
    expr = "a WebDAV server with the folder {string} that requires {word} authentication for {string} with password {string}"
)]
async fn server_with_auth(
    world: &mut SyncWorld,
    folder: String,
    method: String,
    user: String,
    password: String,
) {
    server_with_folder(world, folder).await;
    let auth = match method.as_str() {
        "basic" => Auth::Basic { user, password },
        "digest" => Auth::Digest { user, password },
        other => panic!("unbekanntes Verfahren {other}"),
    };
    world.dav().with(|state| state.auth = auth);
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(SyncWorld::fixture(name)).expect("Fixture lesen")
}

/// Die Fixture (Format `nunc-sync.json`) als Datei im aktuellen Format.
pub fn current_bytes(legacy: &[u8]) -> Vec<u8> {
    let file: SyncFile = serde_json::from_slice(legacy).expect("Fixture ist eine Sync-Datei");
    let state = SyncState::from_legacy(&file);
    serde_json::to_vec_pretty(&SyncDocument::from_state(&state, file.synced_at)).unwrap()
}

#[given(expr = "a WebDAV server with the folder {string} holding the sync file {string}")]
async fn server_holding_file(world: &mut SyncWorld, folder: String, fixture: String) {
    server_with_folder(world, folder.clone()).await;
    let bytes = current_bytes(&fixture_bytes(&fixture));
    world
        .dav()
        .with(|state| state.put(&format!("{folder}{FILE_NAME}"), bytes));
}

#[given(expr = "a WebDAV server with the folder {string} holding the legacy sync file {string}")]
async fn server_holding_legacy_file(world: &mut SyncWorld, folder: String, fixture: String) {
    server_with_folder(world, folder.clone()).await;
    let bytes = fixture_bytes(&fixture);
    world
        .dav()
        .with(|state| state.put(&format!("{folder}{LEGACY_FILE_NAME}"), bytes));
}

#[given(expr = "a WebDAV server with the folder {string} holding a sync file of format {int}")]
async fn server_holding_other_format(world: &mut SyncWorld, folder: String, format: u32) {
    server_with_folder(world, folder.clone()).await;
    let body = serde_json::json!({
        "schema_version": format,
        "synced_at": "2026-10-01T12:00:00Z",
        "todos": [],
        "sessions": [],
    })
    .to_string()
    .into_bytes();
    world
        .dav()
        .with(|state| state.put(&format!("{folder}{FILE_NAME}"), body));
}

#[given(
    expr = "a WebDAV server with the folder {string} holding the todos {string} deleted on {string}"
)]
async fn server_holding_deleted(world: &mut SyncWorld, folder: String, ids: String, day: String) {
    server_with_folder(world, folder.clone()).await;
    let todos = ids
        .split(',')
        .map(|id| {
            let mut todo = todo_json(id.trim(), &format!("Deleted {}", id.trim()));
            todo.deleted = true;
            todo.updated_at = format!("{day}T12:00:00Z");
            todo
        })
        .collect();
    let file = SyncFile {
        schema_version: 1,
        synced_at: "2026-10-01T12:00:00Z".parse().unwrap(),
        device_id: "fixture-device-c".to_owned(),
        todos,
        sessions: vec![],
    };
    let state = SyncState::from_legacy(&file);
    let body =
        serde_json::to_vec_pretty(&SyncDocument::from_state(&state, file.synced_at)).unwrap();
    world
        .dav()
        .with(|state| state.put(&format!("{folder}{FILE_NAME}"), body));
}

#[given(expr = "a WebDAV server with the folder {string} holding a broken sync file")]
async fn server_holding_broken_file(world: &mut SyncWorld, folder: String) {
    server_with_folder(world, folder.clone()).await;
    world.dav().with(|state| {
        state.put(
            &format!("{folder}{FILE_NAME}"),
            b"{ das ist kein JSON".to_vec(),
        );
    });
}

pub fn todo_json(sync_id: &str, text: &str) -> SyncTodo {
    SyncTodo {
        sync_id: sync_id.to_owned(),
        device_id: "fixture-device-c".to_owned(),
        raw_text: text.to_owned(),
        priority: None,
        completed: false,
        completed_at: None,
        created_at: None,
        description: text.to_owned(),
        project_tags: vec![],
        context_tags: vec![],
        pomodoro_count: 0,
        position: 0,
        updated_at: "2026-10-01T12:00:00Z".to_owned(),
        deleted: false,
    }
}

/// Die Datei auf dem Server plus ein Todo eines weiteren Geräts.
fn file_with_extra_todo(world: &SyncWorld, text: &str) -> Vec<u8> {
    let path = format!("{}{FILE_NAME}", world.remote_path);
    let current = world
        .dav()
        .with(|state| state.files.get(&path).map(|(body, _)| body.clone()))
        .expect("keine Sync-Datei auf dem Server");
    with_extra_todo(&current, text)
}

/// Die Datei (Format v2) plus ein Todo eines weiteren Geräts.
pub fn with_extra_todo(current: &[u8], text: &str) -> Vec<u8> {
    let document: SyncDocument = serde_json::from_slice(current).unwrap();
    let synced_at = document.synced_at;
    let state = document.into_state().unwrap();
    let id = format!("c-{}", state.todos.len() + 1);
    let extra = SyncState::from_legacy(&SyncFile {
        schema_version: 1,
        synced_at,
        device_id: "fixture-device-c".to_owned(),
        todos: vec![todo_json(&id, text)],
        sessions: vec![],
    });
    serde_json::to_vec_pretty(&SyncDocument::from_state(&state.merge(&extra), synced_at)).unwrap()
}

#[given(expr = "another device adds the todo {string} to the server during the first upload")]
fn another_device_once(world: &mut SyncWorld, text: String) {
    let body = file_with_extra_todo(world, &text);
    world.dav().with(|state| state.interfere = vec![body]);
}

#[given("another device changes the file on the server during every upload")]
fn another_device_always(world: &mut SyncWorld) {
    let body = file_with_extra_todo(world, "Dauerbaustelle");
    world.dav().with(|state| {
        state.interfere = vec![body];
        state.interfere_always = true;
    });
}

// ---------------------------------------------------------------------------------------------
// Datenbank

#[given("no database exists yet")]
fn no_database(_world: &mut SyncWorld) {}

#[given("the Go fixture database")]
fn go_fixture(world: &mut SyncWorld) {
    let path = world.dir.path().join("go-v5.db");
    std::fs::copy(SyncWorld::fixture("go-v5.db"), &path).unwrap();
    world.store = std::sync::Arc::new(crate::TestStore::new(nunc_store::open(&path).unwrap()));
}

#[given(expr = "the local todos {string}")]
fn local_todos(world: &mut SyncWorld, texts: String) {
    let conn = world.store.conn.lock().unwrap();
    for text in texts.split(',').map(str::trim) {
        let id = world.store.next_id("local");
        nunc_store::add_todo(
            &conn,
            text,
            &id,
            nunc_core::clock::Clock::now(&*world.clock),
        )
        .unwrap();
    }
}

fn local_texts(world: &SyncWorld) -> Vec<String> {
    let conn = world.store.conn.lock().unwrap();
    nunc_store::list_todos(&conn)
        .unwrap()
        .into_iter()
        .map(|todo| todo.raw_text)
        .collect()
}

#[when(expr = "I rename the local todo {string} to {string}")]
fn rename_local_todo(world: &mut SyncWorld, text: String, new_text: String) {
    let conn = world.store.conn.lock().unwrap();
    let id = nunc_store::list_todos(&conn)
        .unwrap()
        .into_iter()
        .find(|todo| todo.raw_text == text)
        .unwrap_or_else(|| panic!("kein lokales Todo {text:?}"))
        .id;
    nunc_store::update_todo(
        &conn,
        id,
        &new_text,
        nunc_core::clock::Clock::now(&*world.clock),
    )
    .unwrap();
}

/// Der lokale Stand als Datei auf dem Server, in der das Todo `text` ein anderes Gerät eine Stunde später
/// abgehakt hat.
#[given(expr = "another device has completed the todo {string} on the server")]
fn other_device_completed(world: &mut SyncWorld, text: String) {
    let now = nunc_core::clock::Clock::now(&*world.clock);
    let mut state = {
        let conn = world.store.conn.lock().unwrap();
        nunc_store::local_state(&conn, || "unused".to_owned()).unwrap()
    };
    let later = nunc_core::hlc::Hlc::new(now.timestamp_millis() + 3_600_000, 0, "other-device");
    let todo = state
        .todos
        .values_mut()
        .find(|todo| todo.raw_text() == text)
        .unwrap_or_else(|| panic!("kein lokales Todo {text:?}"));
    todo.completed.set(true, later.clone());
    todo.completed_at.set(Some("2026-10-04".to_owned()), later);
    let body = serde_json::to_vec_pretty(&SyncDocument::from_state(&state, now)).unwrap();
    let path = format!("{}{FILE_NAME}", world.remote_path);
    world.dav().with(|server| server.put(&path, body));
}

#[then(expr = "the legacy file was requested {int} time(s)")]
fn legacy_requests(world: &mut SyncWorld, count: usize) {
    let path = format!("{}{LEGACY_FILE_NAME}", world.remote_path);
    let actual = world.dav().with(|state| {
        state
            .log
            .iter()
            .filter(|entry| entry.method == "GET" && entry.path == path)
            .count()
    });
    assert_eq!(actual, count);
}

#[then(expr = "the local todos are {string}")]
fn local_todos_are(world: &mut SyncWorld, texts: String) {
    let expected: Vec<&str> = texts.split(',').map(str::trim).collect();
    assert_eq!(local_texts(world), expected);
}

#[then(expr = "the local todos include {string}")]
fn local_todos_include(world: &mut SyncWorld, text: String) {
    assert!(
        local_texts(world).contains(&text),
        "{:?}",
        local_texts(world)
    );
}

fn local_row(world: &SyncWorld, sync_id: &str) -> (String, bool) {
    let conn = world.store.conn.lock().unwrap();
    conn.query_row(
        "SELECT raw_text, deleted FROM todos WHERE sync_id = ?1",
        [sync_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .unwrap_or_else(|_| panic!("Todo {sync_id} fehlt"))
}

#[then(expr = "the local todo {string} says {string}")]
fn local_todo_says(world: &mut SyncWorld, sync_id: String, text: String) {
    assert_eq!(local_row(world, &sync_id).0, text);
}

#[then(expr = "the local todo {string} is deleted")]
fn local_todo_deleted(world: &mut SyncWorld, sync_id: String) {
    assert!(local_row(world, &sync_id).1);
}

#[then("the local database has the todos:")]
fn local_database_todos(world: &mut SyncWorld, step: &Step) {
    let expected = rows(step);
    let conn = world.store.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT sync_id, raw_text, deleted FROM todos ORDER BY sync_id")
        .unwrap();
    let actual: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?.to_string(),
            ])
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let expected: Vec<Vec<String>> = expected[1..]
        .iter()
        .map(|row| row.iter().map(|c| c.trim().to_owned()).collect())
        .collect();
    assert_eq!(actual, expected);
}

#[then(expr = "the local database has {int} sessions")]
fn local_database_sessions(world: &mut SyncWorld, count: i64) {
    let conn = world.store.conn.lock().unwrap();
    let actual: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(actual, count);
}

// ---------------------------------------------------------------------------------------------
// Konfiguration und Tresor

fn config_from_table(step: &Step) -> WebDavConfig {
    let mut config = WebDavConfig::default();
    for row in rows(step) {
        let value = row[1].trim().to_owned();
        match row[0].as_str() {
            "url" => config.url = value,
            "authMethod" => config.auth_method = value.parse().expect("Verfahren"),
            "username" => config.username = value,
            "remotePath" => config.remote_path = value,
            other => panic!("unbekanntes Feld {other}"),
        }
    }
    config
}

#[when(expr = "I save the WebDAV settings with the password {string}:")]
fn save_with_password(world: &mut SyncWorld, password: String, step: &Step) {
    let config = config_from_table(step);
    let mut conn = world.store.conn.lock().unwrap();
    save_webdav(&mut conn, &world.secrets, &config, Some(&password)).unwrap();
}

#[when("I save the WebDAV settings without a password:")]
fn save_without_password(world: &mut SyncWorld, step: &Step) {
    let config = config_from_table(step);
    let mut conn = world.store.conn.lock().unwrap();
    save_webdav(&mut conn, &world.secrets, &config, None).unwrap();
}

#[then(expr = "the secret store holds the password {string}")]
fn secret_holds(world: &mut SyncWorld, password: String) {
    use nunc_platform::SecretStore;
    let stored = world
        .secrets
        .get(nunc_sync::SECRET_SERVICE, nunc_sync::SECRET_ACCOUNT)
        .unwrap();
    assert_eq!(stored.as_deref(), Some(password.as_str()));
}

#[then(expr = "the database does not contain {string}")]
fn database_does_not_contain(world: &mut SyncWorld, needle: String) {
    let conn = world.store.conn.lock().unwrap();
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    for table in tables {
        let columns: Vec<String> = conn
            .prepare(&format!("PRAGMA table_info(\"{table}\")"))
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for column in columns {
            let hits: i64 = conn
                .query_row(
                    &format!(
                        "SELECT COUNT(*) FROM \"{table}\" WHERE CAST(\"{column}\" AS TEXT) LIKE ?1"
                    ),
                    [format!("%{needle}%")],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(hits, 0, "{needle:?} steht in {table}.{column}");
        }
    }
}

#[then(expr = "the loaded WebDAV settings have the url {string}")]
fn loaded_url(world: &mut SyncWorld, url: String) {
    let conn = world.store.conn.lock().unwrap();
    assert_eq!(nunc_store::load_webdav_config(&conn).unwrap().url, url);
}

#[then(expr = "the settings table contains:")]
fn settings_table(world: &mut SyncWorld, step: &Step) {
    let conn = world.store.conn.lock().unwrap();
    for row in rows(step).iter().skip(1) {
        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                [row[0].trim()],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| panic!("Schlüssel {} fehlt", row[0]));
        assert_eq!(value, row[1].trim(), "{}", row[0]);
    }
}

#[then(expr = "the settings are usable for a sync: {word}")]
fn usable(world: &mut SyncWorld, answer: String) {
    let conn = world.store.conn.lock().unwrap();
    let loaded = load_credentials(&conn, &world.secrets);
    match answer.as_str() {
        "yes" => assert!(loaded.is_ok(), "{:?}", loaded.err()),
        "no" => assert!(matches!(loaded, Err(nunc_sync::SyncError::NotConfigured))),
        other => panic!("yes/no erwartet, nicht {other}"),
    }
}

// ---------------------------------------------------------------------------------------------
// Verbindungstest und Sync-Lauf

fn parse_auth(method: &str) -> WebDavAuth {
    method.parse().expect("Verfahren")
}

#[when(expr = "I test the connection as {string} with password {string}")]
async fn test_connection(world: &mut SyncWorld, user: String, password: String) {
    let url = server_url(world);
    test_connection_to(world, url, user, password).await;
}

#[when(expr = "I test the connection to {string} as {string} with password {string}")]
async fn test_connection_to(world: &mut SyncWorld, url: String, user: String, password: String) {
    let config = config_for(world, &user, WebDavAuth::Basic, url);
    world.test_result = Some(match WebDavClient::new(config, password) {
        Ok(client) => client.test_connection().await,
        Err(error) => nunc_core::model::ConnectionTestResult {
            success: false,
            message: error.to_string(),
        },
    });
}

#[then(expr = "the connection test succeeds with the message {string}")]
fn test_succeeds(world: &mut SyncWorld, message: String) {
    let result = world.test_result.as_ref().expect("kein Test gelaufen");
    assert!(result.success, "{}", result.message);
    assert_eq!(result.message, message);
}

#[then(expr = "the connection test fails with the message {string}")]
fn test_fails(world: &mut SyncWorld, message: String) {
    let result = world.test_result.as_ref().expect("kein Test gelaufen");
    assert!(!result.success, "Verbindungstest hätte scheitern müssen");
    assert_eq!(result.message, message);
}

async fn run_sync(world: &mut SyncWorld, user: &str, password: &str, auth: WebDavAuth) {
    let config = config_for(world, user, auth, server_url(world));
    let path = format!("{}{FILE_NAME}", config.remote_path);
    world.server_file_before = world
        .dav
        .as_ref()
        .and_then(|dav| dav.with(|state| state.files.get(&path).map(|(b, _)| b.clone())));
    world.outcome = Some(match WebDavClient::new(config, password.to_owned()) {
        Ok(client) => sync_webdav(&client, &*world.store, &*world.clock).await,
        Err(error) => Err(error),
    });
}

#[when(expr = "I sync as {string} with password {string} using {string}")]
async fn sync_as(world: &mut SyncWorld, user: String, password: String, method: String) {
    run_sync(world, &user, &password, parse_auth(&method)).await;
}

#[when("I sync")]
async fn sync_default(world: &mut SyncWorld) {
    run_sync(world, "me", "secret", WebDavAuth::Basic).await;
}

#[when(expr = "I sync with the remote path {string}")]
async fn sync_with_path(world: &mut SyncWorld, path: String) {
    world.remote_path = path;
    run_sync(world, "me", "secret", WebDavAuth::Basic).await;
}

#[then("the sync succeeds")]
fn sync_succeeds(world: &mut SyncWorld) {
    match world.outcome.as_ref().expect("kein Sync gelaufen") {
        Ok(_) => {}
        Err(error) => panic!("Sync fehlgeschlagen: {error}"),
    }
}

#[then(
    expr = "the sync succeeds with {int} imported todos, {int} imported sessions and {int} conflicts"
)]
fn sync_succeeds_with(world: &mut SyncWorld, todos: u32, sessions: u32, conflicts: u32) {
    let result = world
        .outcome
        .as_ref()
        .expect("kein Sync gelaufen")
        .as_ref()
        .unwrap_or_else(|error| panic!("Sync fehlgeschlagen: {error}"));
    assert_eq!(
        (
            result.imported_todos,
            result.imported_sessions,
            result.conflicted_todos
        ),
        (todos, sessions, conflicts)
    );
}

fn failure(world: &SyncWorld) -> String {
    match world.outcome.as_ref().expect("kein Sync gelaufen") {
        Ok(result) => panic!("Sync hätte scheitern müssen: {result:?}"),
        Err(error) => error.to_string(),
    }
}

#[then(expr = "the sync fails with {string}")]
fn sync_fails(world: &mut SyncWorld, message: String) {
    assert_eq!(failure(world), message);
}

#[then(expr = "the sync fails with a message starting {string}")]
fn sync_fails_starting(world: &mut SyncWorld, prefix: String) {
    let message = failure(world);
    assert!(message.starts_with(&prefix), "{message}");
}

// ---------------------------------------------------------------------------------------------
// Beobachtungen am Server

#[then(expr = "the server received a {string} request for {string} with depth {string}")]
fn server_received(world: &mut SyncWorld, method: String, path: String, depth: String) {
    let hit = world.dav().with(|state| {
        state.log.iter().any(|entry| {
            entry.method == method
                && entry.path == path
                && entry.headers.get("depth").map(String::as_str) == Some(depth.as_str())
        })
    });
    assert!(hit, "keine {method}-Anfrage für {path} mit Depth {depth}");
}

#[then("every request to the server was authenticated")]
fn all_authenticated(world: &mut SyncWorld) {
    world.dav().with(|state| {
        assert!(!state.log.is_empty());
        assert!(
            state.log.iter().all(|entry| entry.status != 401),
            "Anfragen mit 401: {:?}",
            state
                .log
                .iter()
                .filter(|e| e.status == 401)
                .collect::<Vec<_>>()
        );
    });
}

#[then("the server accepted a digest authorization")]
fn digest_accepted(world: &mut SyncWorld) {
    assert!(world.dav().with(|state| state.digest_accepted));
}

#[then("the server did not receive a basic authorization")]
fn no_basic(world: &mut SyncWorld) {
    world.dav().with(|state| {
        assert!(state.log.iter().all(|entry| {
            !entry
                .headers
                .get("authorization")
                .is_some_and(|value| value.starts_with("Basic"))
        }));
    });
}

#[then(expr = "the server created the folders {string} in this order")]
fn created_in_order(world: &mut SyncWorld, folders: String) {
    let created: Vec<String> = world.dav().with(|state| {
        state
            .log
            .iter()
            .filter(|entry| entry.method == "MKCOL" && entry.status == 201)
            .map(|entry| entry.path.clone())
            .collect()
    });
    assert_eq!(created, folders.split_whitespace().collect::<Vec<_>>());
}

#[then(expr = "the server holds the sync file {string}")]
fn server_holds(world: &mut SyncWorld, path: String) {
    assert!(world.dav().with(|state| state.files.contains_key(&path)));
}

fn server_file(world: &SyncWorld) -> SyncDocument {
    let path = format!("{}{FILE_NAME}", world.remote_path);
    let bytes = world
        .dav()
        .with(|state| state.files.get(&path).map(|(body, _)| body.clone()))
        .expect("keine Sync-Datei auf dem Server");
    serde_json::from_slice(&bytes).expect("Sync-Datei auf dem Server")
}

#[then(expr = "the sync file on the server has the todos {string}")]
fn server_todos(world: &mut SyncWorld, texts: String) {
    let mut actual: Vec<String> = server_file(world)
        .todos
        .into_iter()
        .map(|todo| todo.raw_text())
        .collect();
    let mut expected: Vec<String> = texts.split(',').map(|t| t.trim().to_owned()).collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected);
}

#[then(expr = "the sync file on the server has the todo {string}")]
fn server_has_todo(world: &mut SyncWorld, text: String) {
    assert!(
        server_file(world)
            .todos
            .iter()
            .any(|todo| todo.raw_text() == text)
    );
}

#[then(expr = "the sync file on the server has {int} todos and {int} sessions")]
fn server_counts(world: &mut SyncWorld, todos: usize, sessions: usize) {
    let file = server_file(world);
    assert_eq!((file.todos.len(), file.sessions.len()), (todos, sessions));
}

#[then("the sync file on the server is unchanged")]
fn server_unchanged(world: &mut SyncWorld) {
    let path = format!("{}{FILE_NAME}", world.remote_path);
    let now = world
        .dav()
        .with(|state| state.files.get(&path).map(|(body, _)| body.clone()));
    assert_eq!(now, world.server_file_before);
}

fn uploads(world: &SyncWorld) -> Vec<crate::dav::Logged> {
    world.dav().with(|state| {
        state
            .log
            .iter()
            .filter(|e| e.method == "PUT")
            .cloned()
            .collect()
    })
}

#[then("the upload required that the file does not exist yet")]
fn upload_none_exists(world: &mut SyncWorld) {
    let puts = uploads(world);
    assert_eq!(puts.len(), 1);
    assert_eq!(
        puts[0].headers.get("if-none-match").map(String::as_str),
        Some("*")
    );
    assert!(!puts[0].headers.contains_key("if-match"));
}

#[then("the upload carried the ETag of the downloaded file")]
fn upload_etag(world: &mut SyncWorld) {
    let puts = uploads(world);
    assert_eq!(puts.len(), 1);
    assert_eq!(
        puts[0].headers.get("if-match").map(String::as_str),
        Some("\"v1\"")
    );
}

#[then(expr = "the uploads carried the ETags {string}")]
fn uploads_etags(world: &mut SyncWorld, etags: String) {
    let actual: Vec<String> = uploads(world)
        .iter()
        .map(|put| put.headers.get("if-match").cloned().unwrap_or_default())
        .collect();
    assert_eq!(actual, etags.split_whitespace().collect::<Vec<_>>());
}

#[then("no upload carried a condition")]
fn no_condition(world: &mut SyncWorld) {
    let puts = uploads(world);
    assert!(!puts.is_empty());
    assert!(
        puts.iter().all(
            |p| !p.headers.contains_key("if-match") && !p.headers.contains_key("if-none-match")
        )
    );
}

#[then(expr = "the server received {int} uploads")]
fn upload_count(world: &mut SyncWorld, count: usize) {
    assert_eq!(uploads(world).len(), count);
}
