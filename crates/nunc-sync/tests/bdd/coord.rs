//! Steps für die Steuerung des Syncs (`SyncCoordinator`): Auslöser, Entprellen, Serialisierung, Beenden,
//! Statusmeldungen. Die Läufe selbst sind hier ein Skript (`ScriptedRunner`); echte Läufe gegen den
//! WebDAV-Server prüfen die Szenarien in `steps.rs`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cucumber::{given, then, when};
use nunc_core::model::{SyncResult, SyncState, SyncStatus};
use nunc_store::Connection;
use nunc_sync::{
    RunFuture, SyncCoordinator, SyncError, SyncObserver, SyncRunner, Timing, Trigger, WebDavRunner,
};

use crate::SyncWorld;

#[derive(Default)]
pub struct Counters {
    runs: AtomicUsize,
    active: AtomicUsize,
    max_active: AtomicUsize,
    delay: Mutex<Duration>,
    fail_with: Mutex<Option<String>>,
    result: Mutex<SyncResult>,
}

#[derive(Default)]
pub struct ScriptedRunner {
    pub c: Arc<Counters>,
}

impl SyncRunner for ScriptedRunner {
    fn run(&self) -> RunFuture {
        let c = self.c.clone();
        let delay = *c.delay.lock().unwrap();
        let fail = c.fail_with.lock().unwrap().take();
        let result = *c.result.lock().unwrap();
        let active = c.active.fetch_add(1, Ordering::SeqCst) + 1;
        c.max_active.fetch_max(active, Ordering::SeqCst);
        Box::pin(async move {
            tokio::time::sleep(delay).await;
            c.active.fetch_sub(1, Ordering::SeqCst);
            c.runs.fetch_add(1, Ordering::SeqCst);
            match fail {
                Some(message) => Err(SyncError::Transport(message)),
                None => Ok(Some((result, "2026-10-04T09:00:00Z".to_owned()))),
            }
        })
    }
}

#[derive(Default)]
pub struct Recorder {
    log: Mutex<Vec<String>>,
}

impl SyncObserver for Recorder {
    fn started(&self) {
        self.log.lock().unwrap().push("start".to_owned());
    }

    fn status(&self, status: &SyncStatus) {
        self.log
            .lock()
            .unwrap()
            .push(format!("status {}", state_name(status.state)));
    }

    fn completed(&self, result: &SyncResult) {
        self.log.lock().unwrap().push(format!(
            "complete {} {} {}",
            result.imported_todos, result.imported_sessions, result.conflicted_todos
        ));
    }
}

fn state_name(state: SyncState) -> &'static str {
    match state {
        SyncState::Idle => "idle",
        SyncState::Syncing => "syncing",
        SyncState::Error => "error",
    }
}

pub struct State {
    pub runner: Arc<ScriptedRunner>,
    pub observer: Arc<Recorder>,
    pub debounce: Duration,
    pub quit_limit: Duration,
    pub mobile: bool,
    pub coordinator: Option<Arc<SyncCoordinator>>,
    pub quit_took: Option<Duration>,
    pub last_error: Option<String>,
    pub runner_outcome: Option<nunc_sync::RunOutcome>,
    /// Statt des Skripts: ein echter Läufer.
    pub runner_override: Option<Arc<dyn SyncRunner>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            runner: Arc::new(ScriptedRunner::default()),
            observer: Arc::new(Recorder::default()),
            debounce: Duration::from_millis(100),
            quit_limit: Duration::from_millis(1000),
            mobile: false,
            coordinator: None,
            quit_took: None,
            last_error: None,
            runner_outcome: None,
            runner_override: None,
        }
    }
}

impl State {
    pub fn coordinator(&mut self) -> Arc<SyncCoordinator> {
        self.coordinator
            .get_or_insert_with(|| {
                SyncCoordinator::new(
                    self.runner_override
                        .clone()
                        .unwrap_or_else(|| self.runner.clone()),
                    self.observer.clone(),
                    tokio::runtime::Handle::current(),
                    self.mobile,
                    Timing {
                        debounce: self.debounce,
                        quit_limit: self.quit_limit,
                    },
                    None,
                )
            })
            .clone()
    }
}

fn millis(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

#[given(expr = "a sync coordinator whose runs take {int} ms")]
fn runs_take(world: &mut SyncWorld, ms: u64) {
    *world.coord.runner.c.delay.lock().unwrap() = millis(ms);
}

#[given(expr = "the debounce time is {int} ms")]
fn debounce(world: &mut SyncWorld, ms: u64) {
    world.coord.debounce = millis(ms);
}

#[given(expr = "the quit limit is {int} ms")]
fn quit_limit(world: &mut SyncWorld, ms: u64) {
    world.coord.quit_limit = millis(ms);
}

#[given(expr = "the coordinator runs on {word}")]
fn set_platform(world: &mut SyncWorld, platform: String) {
    world.coord.mobile = match platform.as_str() {
        "mobile" => true,
        "desktop" => false,
        other => panic!("unbekannte Plattform {other}"),
    };
}

#[given(expr = "the next run imports {int} todos and {int} sessions with {int} conflicts")]
fn next_result(world: &mut SyncWorld, todos: u32, sessions: u32, conflicts: u32) {
    *world.coord.runner.c.result.lock().unwrap() = SyncResult {
        imported_todos: todos,
        imported_sessions: sessions,
        conflicted_todos: conflicts,
    };
}

#[given(expr = "the next run fails with {string}")]
#[when(expr = "the next run fails with {string}")]
fn next_fails(world: &mut SyncWorld, message: String) {
    *world.coord.runner.c.fail_with.lock().unwrap() = Some(message);
}

#[when(expr = "{int} syncs are requested at the same time")]
async fn concurrent(world: &mut SyncWorld, count: usize) {
    let coordinator = world.coord.coordinator();
    let calls = (0..count).map(|_| {
        let coordinator = coordinator.clone();
        async move { coordinator.sync_now().await }
    });
    for outcome in futures_join(calls).await {
        outcome.expect("Sync");
    }
}

/// `futures::future::join_all` ohne zusätzliche Abhängigkeit: nebenläufig über Tasks.
async fn futures_join<F>(calls: impl Iterator<Item = F>) -> Vec<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    let tasks: Vec<_> = calls.map(tokio::spawn).collect();
    let mut out = Vec::new();
    for task in tasks {
        out.push(task.await.expect("Task"));
    }
    out
}

#[when(expr = "I replace the last sync time with {string}")]
fn replace_last_sync(world: &mut SyncWorld, time: String) {
    world.coord.coordinator().set_last_sync_at(Some(time));
}

#[when("I replace the last sync time with nothing")]
fn clear_last_sync(world: &mut SyncWorld) {
    world.coord.coordinator().set_last_sync_at(None);
}

#[when("I sync now")]
async fn sync_now(world: &mut SyncWorld) {
    let coordinator = world.coord.coordinator();
    world.coord.last_error = coordinator.sync_now().await.err().map(|e| e.to_string());
}

fn trigger(world: &mut SyncWorld, trigger: Trigger) {
    world.coord.coordinator().trigger(trigger);
}

#[when("the app starts")]
fn app_starts(world: &mut SyncWorld) {
    trigger(world, Trigger::Start);
}

#[when(regex = r"^a session (?:finishes|is aborted)$")]
fn session_ended(world: &mut SyncWorld) {
    trigger(world, Trigger::SessionFinished);
}

#[when("the app resumes")]
fn resumes(world: &mut SyncWorld) {
    trigger(world, Trigger::Resume);
}

#[when("the app goes to the background")]
fn background(world: &mut SyncWorld) {
    trigger(world, Trigger::Background);
}

#[when("a todo changes")]
fn todo_changes(world: &mut SyncWorld) {
    trigger(world, Trigger::TodoChanged);
}

#[when(expr = "{int} todos change within {int} ms")]
async fn todos_change(world: &mut SyncWorld, count: u64, within: u64) {
    let gap = millis(within / count.max(1));
    for _ in 0..count {
        trigger(world, Trigger::TodoChanged);
        tokio::time::sleep(gap).await;
    }
}

#[when(expr = "the sync interval is set to {int} ms")]
fn set_interval(world: &mut SyncWorld, ms: u64) {
    world.coord.coordinator().set_interval(Some(millis(ms)));
}

#[when("the sync interval is switched off")]
fn interval_off(world: &mut SyncWorld) {
    world.coord.coordinator().set_interval(None);
}

#[when(expr = "I wait {int} ms")]
async fn wait(_world: &mut SyncWorld, ms: u64) {
    tokio::time::sleep(millis(ms)).await;
}

#[when("the app quits")]
async fn quits(world: &mut SyncWorld) {
    let coordinator = world.coord.coordinator();
    let started = Instant::now();
    coordinator.sync_on_quit().await;
    world.coord.quit_took = Some(started.elapsed());
}

#[then(expr = "within {int} ms {int} sync runs have happened")]
async fn runs_happened(world: &mut SyncWorld, within: u64, expected: usize) {
    let runs = || world.coord.runner.c.runs.load(Ordering::SeqCst);
    let deadline = Instant::now() + millis(within);
    while runs() < expected && Instant::now() < deadline {
        tokio::time::sleep(millis(10)).await;
    }
    // ein weiterer Lauf, der nicht hätte starten dürfen, hätte jetzt Zeit gehabt
    tokio::time::sleep(millis(150)).await;
    assert_eq!(runs(), expected);
}

#[then(expr = "within {int} ms at least {int} sync runs have happened")]
async fn at_least_runs(world: &mut SyncWorld, within: u64, expected: usize) {
    let runs = || world.coord.runner.c.runs.load(Ordering::SeqCst);
    let deadline = Instant::now() + millis(within);
    while runs() < expected && Instant::now() < deadline {
        tokio::time::sleep(millis(10)).await;
    }
    assert!(runs() >= expected, "nur {} Läufe", runs());
}

#[then(expr = "at most {int} sync run was active at a time")]
fn max_active(world: &mut SyncWorld, limit: usize) {
    assert!(world.coord.runner.c.max_active.load(Ordering::SeqCst) <= limit);
}

#[then(expr = "no sync run has happened yet")]
fn no_runs(world: &mut SyncWorld) {
    assert_eq!(world.coord.runner.c.runs.load(Ordering::SeqCst), 0);
}

#[then(expr = "{int} sync runs have happened one after the other")]
fn serialized(world: &mut SyncWorld, expected: usize) {
    assert_eq!(world.coord.runner.c.runs.load(Ordering::SeqCst), expected);
    assert_eq!(world.coord.runner.c.max_active.load(Ordering::SeqCst), 1);
}

#[then(expr = "closing waited for {int} finished sync run")]
fn closing_waited(world: &mut SyncWorld, expected: usize) {
    assert_eq!(world.coord.runner.c.runs.load(Ordering::SeqCst), expected);
}

#[then(expr = "closing was delayed by less than {int} ms")]
fn closing_delay(world: &mut SyncWorld, limit: u64) {
    let took = world.coord.quit_took.expect("nicht beendet");
    assert!(took < millis(limit), "{took:?}");
}

#[then(expr = "the observer saw: {string}")]
fn observer_saw(world: &mut SyncWorld, expected: String) {
    let log = world.coord.observer.log.lock().unwrap().join(", ");
    assert_eq!(log, expected);
}

#[then(expr = "the sync status is {string}")]
fn status_is(world: &mut SyncWorld, expected: String) {
    let status = world.coord.coordinator().status();
    let actual = format!(
        "{} {} {}",
        state_name(status.state),
        status.last_sync_at.as_deref().unwrap_or("-"),
        status.error_message.as_deref().unwrap_or("-")
    );
    assert_eq!(actual, expected);
}

#[then(expr = "the sync call failed with {string}")]
fn call_failed(world: &mut SyncWorld, message: String) {
    assert_eq!(world.coord.last_error.as_deref(), Some(message.as_str()));
}

// ---------------------------------------------------------------------------------------------
// Der WebDAV-Lauf (Runner) mit Einstellungen aus der Datenbank

#[given(expr = "the sync provider is {string}")]
fn set_provider(world: &mut SyncWorld, provider: String) {
    let conn = world.store.conn.lock().unwrap();
    nunc_store::set_sync_provider(&conn, provider.parse().expect("Anbieter")).unwrap();
}

#[given("the sync is paused")]
fn paused(world: &mut SyncWorld) {
    let mut conn = world.store.conn.lock().unwrap();
    pause(&mut conn);
}

fn pause(conn: &mut Connection) {
    let mut settings = nunc_store::load_settings(conn).unwrap();
    settings.sync_enabled = false;
    nunc_store::save_settings(conn, &settings).unwrap();
}

#[given(expr = "the WebDAV settings point to the server for {string} with password {string}")]
fn settings_point_to_server(world: &mut SyncWorld, user: String, password: String) {
    use nunc_core::model::{WebDavAuth, WebDavConfig};
    let config = WebDavConfig {
        url: world.dav().server.uri(),
        auth_method: WebDavAuth::Basic,
        username: user,
        remote_path: world.remote_path.clone(),
    };
    let mut conn = world.store.conn.lock().unwrap();
    nunc_sync::save_webdav(&mut conn, &world.secrets, &config, Some(&password)).unwrap();
}

#[given("the WebDAV settings are incomplete")]
fn settings_incomplete(world: &mut SyncWorld) {
    let mut conn = world.store.conn.lock().unwrap();
    nunc_sync::save_webdav(
        &mut conn,
        &world.secrets,
        &nunc_core::model::WebDavConfig::default(),
        None,
    )
    .unwrap();
}

#[when("the WebDAV runner runs")]
async fn runner_runs(world: &mut SyncWorld) {
    use nunc_platform::SecretStore;
    // der Tresor gehört der Welt; der Runner bekommt eine Kopie des Passworts im eigenen Tresor
    let secrets = nunc_platform::MemorySecretStore::default();
    if let Some(password) = world
        .secrets
        .get(nunc_sync::SECRET_SERVICE, nunc_sync::SECRET_ACCOUNT)
        .unwrap()
    {
        secrets
            .set(
                nunc_sync::SECRET_SERVICE,
                nunc_sync::SECRET_ACCOUNT,
                &password,
            )
            .unwrap();
    }
    let runner = WebDavRunner::new(world.store.clone(), Arc::new(secrets), world.clock.clone());
    world.coord.runner_outcome = Some(runner.run().await);
}

#[then("the runner did nothing")]
fn did_nothing(world: &mut SyncWorld) {
    assert!(matches!(world.coord.runner_outcome, Some(Ok(None))));
}

#[then("the server did not receive any request")]
fn no_requests(world: &mut SyncWorld) {
    assert!(world.dav().with(|state| state.log.is_empty()));
}

#[then(expr = "the runner failed with {string}")]
fn runner_failed(world: &mut SyncWorld, message: String) {
    match &world.coord.runner_outcome {
        Some(Err(error)) => assert_eq!(error.to_string(), message),
        other => panic!(
            "Fehler erwartet, erhalten {:?}",
            other.as_ref().map(|r| r.is_ok())
        ),
    }
}

#[then(expr = "the runner reports the finish time {string}")]
fn runner_time(world: &mut SyncWorld, time: String) {
    match &world.coord.runner_outcome {
        Some(Ok(Some((_, at)))) => assert_eq!(at, &time),
        _ => panic!("kein Ergebnis"),
    }
}

#[then(expr = "the last sync time in the settings is {string}")]
fn last_sync_setting(world: &mut SyncWorld, time: String) {
    let conn = world.store.conn.lock().unwrap();
    assert_eq!(
        nunc_store::last_sync_at(&conn).unwrap().as_deref(),
        Some(time.as_str())
    );
}

#[then("the settings have no last sync time")]
fn no_last_sync(world: &mut SyncWorld) {
    let conn = world.store.conn.lock().unwrap();
    assert_eq!(nunc_store::last_sync_at(&conn).unwrap(), None);
}
