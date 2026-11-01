//! Steps für `features/timetracking/*.feature` auf Ebene `@integration`: nachgetragene, bearbeitete und gelöschte
//! Sessions in der Datenbank und der Austausch zweier Geräte (ADR 0017). Die Geräte stammen aus dem Modul `crdt`.

use chrono::NaiveDate;
use cucumber::{given, then, when};
use nunc_core::model::{StatsPeriod, StatsRequest};
use nunc_core::stats::compute_stats;
use nunc_store::{Connection, ManualSession, SessionEdit};

use crate::{StoreWorld, at};

fn start_edit(conn: &Connection) -> (i32, SessionEdit) {
    let session = nunc_store::list_sessions(conn)
        .expect("list")
        .into_iter()
        .next()
        .expect("keine Session");
    let note: String = conn
        .query_row(
            "SELECT note FROM session_meta WHERE session_id = ?1",
            [session.id],
            |row| row.get(0),
        )
        .unwrap_or_default();
    (
        session.id,
        SessionEdit {
            started_at: session.started_at,
            ended_at: session.ended_at,
            actual_min: session.actual_min.unwrap_or(0),
            todo_id: session.todo_id,
            note,
            project_tags: session.project_tags,
            context_tags: session.context_tags,
        },
    )
}

fn add_manual(world: &mut StoreWorld, device: &str, start: &str, end: &str, todo_id: Option<i32>) {
    let sync_id = format!("manual-{device}-{start}");
    let entry = ManualSession {
        sync_id,
        started_at: at(start),
        ended_at: at(end),
        todo_id,
        note: String::new(),
        project_tags: Vec::new(),
        context_tags: Vec::new(),
    };
    let device_id = format!("dev-{}", device.to_lowercase());
    nunc_store::insert_manual(world.crdt.conn(device), &entry, &device_id, at(end))
        .expect("manual");
}

#[given(expr = "device {word} adds a manual session from {word} to {word}")]
#[when(expr = "device {word} adds a manual session from {word} to {word}")]
fn adds_manual(world: &mut StoreWorld, device: String, start: String, end: String) {
    add_manual(world, &device, &start, &end, None);
}

fn tag_list(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_owned).collect()
}

#[given(
    expr = "device {word} adds a manual session from {word} to {word} with projects {string} and contexts {string}"
)]
fn adds_manual_labelled(
    world: &mut StoreWorld,
    device: String,
    start: String,
    end: String,
    projects: String,
    contexts: String,
) {
    let entry = ManualSession {
        sync_id: format!("manual-{device}-{start}"),
        started_at: at(&start),
        ended_at: at(&end),
        todo_id: None,
        note: String::new(),
        project_tags: tag_list(&projects),
        context_tags: tag_list(&contexts),
    };
    let device_id = format!("dev-{}", device.to_lowercase());
    nunc_store::insert_manual(world.crdt.conn(&device), &entry, &device_id, at(&end))
        .expect("manual");
}

#[when(expr = "device {word} changes the session projects to {string} at {word}")]
fn changes_projects(world: &mut StoreWorld, device: String, projects: String, time: String) {
    let conn = world.crdt.conn(&device);
    let (id, mut edit) = start_edit(conn);
    edit.project_tags = tag_list(&projects);
    nunc_store::update_session(conn, id, &edit, at(&time)).expect("update");
}

#[then(expr = "on device {word} the session has projects {string} and contexts {string}")]
fn session_labels(world: &mut StoreWorld, device: String, projects: String, contexts: String) {
    let (_, edit) = start_edit(world.crdt.conn(&device));
    assert_eq!(edit.project_tags, tag_list(&projects));
    assert_eq!(edit.context_tags, tag_list(&contexts));
}

#[given(expr = "device {word} adds a manual session from {word} to {word} for the todo")]
#[when(expr = "device {word} adds a manual session from {word} to {word} for the todo")]
fn adds_manual_for_todo(world: &mut StoreWorld, device: String, start: String, end: String) {
    add_manual(world, &device, &start, &end, Some(1));
}

#[when(expr = "device {word} changes the session note to {string} at {word}")]
fn changes_note(world: &mut StoreWorld, device: String, note: String, time: String) {
    let conn = world.crdt.conn(&device);
    let (id, mut edit) = start_edit(conn);
    edit.note = note;
    nunc_store::update_session(conn, id, &edit, at(&time)).expect("update");
}

#[when(expr = "device {word} changes the session minutes to {int} at {word}")]
fn changes_minutes(world: &mut StoreWorld, device: String, minutes: u32, time: String) {
    let conn = world.crdt.conn(&device);
    let (id, mut edit) = start_edit(conn);
    edit.actual_min = minutes;
    nunc_store::update_session(conn, id, &edit, at(&time)).expect("update");
}

#[when(expr = "device {word} deletes the session at {word}")]
fn deletes_session(world: &mut StoreWorld, device: String, time: String) {
    let conn = world.crdt.conn(&device);
    let (id, _) = start_edit(conn);
    nunc_store::delete_session(conn, id, at(&time)).expect("delete");
}

#[given(expr = "both devices know a completed session of device A started at {word}")]
fn both_know_session(world: &mut StoreWorld, time: String) {
    let started = at(&time).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    for device in ["A", "B"] {
        world
            .crdt
            .conn(device)
            .execute(
                "INSERT INTO sessions (mode, session_type, status, started_at, ended_at, planned_min,
                                       actual_min, break_min, sync_device_id)
                 VALUES ('pomodoro', 'work', 'completed', ?1, ?1, 25, 25, 5, 'dev-a')",
                [&started],
            )
            .expect("session");
    }
}

#[then(expr = "on device {word} the field {string} of the session was last written at {word}")]
fn session_field_at(world: &mut StoreWorld, device: String, field: String, time: String) {
    let ms: i64 = world
        .crdt
        .conn(&device)
        .query_row(
            "SELECT hlc_ms FROM session_field_clock WHERE session_id = 1 AND field = ?1",
            [&field],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| panic!("kein Stempel für {field}"));
    assert_eq!(ms, at(&time).timestamp_millis(), "{field}");
}

#[then(expr = "on device {word} the session note is {string}")]
fn session_note(world: &mut StoreWorld, device: String, note: String) {
    let (_, edit) = start_edit(world.crdt.conn(&device));
    assert_eq!(edit.note, note);
}

#[then(expr = "on device {word} the session has {int} minutes")]
fn session_minutes(world: &mut StoreWorld, device: String, minutes: u32) {
    let (_, edit) = start_edit(world.crdt.conn(&device));
    assert_eq!(edit.actual_min, minutes);
}

#[then(expr = "on device {word} {int} sessions are listed")]
fn sessions_listed(world: &mut StoreWorld, device: String, expected: usize) {
    let listed = nunc_store::list_sessions(world.crdt.conn(&device)).expect("list");
    assert_eq!(listed.len(), expected);
}

#[then(
    expr = "on device {word} the session is a completed flowtime work session with origin {string}"
)]
fn session_kind(world: &mut StoreWorld, device: String, origin: String) {
    let row: (String, String, String, String) = world
        .crdt
        .conn(&device)
        .query_row(
            "SELECT s.mode, s.session_type, s.status, m.origin FROM sessions s
             JOIN session_meta m ON m.session_id = s.id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("Session mit Metadaten");
    assert_eq!(
        row,
        ("flowtime".into(), "work".into(), "completed".into(), origin)
    );
}

#[then(expr = "on device {word} the project {string} has {int} minutes")]
fn project_minutes(world: &mut StoreWorld, device: String, project: String, minutes: u32) {
    let conn = world.crdt.conn(&device);
    let sessions = nunc_store::list_sessions(conn).expect("sessions");
    let todos = nunc_store::list_all_todos(conn).expect("todos");
    let day = NaiveDate::from_ymd_opt(2026, 10, 4).unwrap();
    let request = StatsRequest {
        period: StatsPeriod::Day,
        start_date: day,
        end_date: day,
    };
    let stats = compute_stats(&sessions, &todos, &request, day, &|time| time.date_naive());
    let found = stats
        .projects
        .iter()
        .find(|entry| entry.project == project)
        .map_or(0, |entry| entry.work_minutes);
    assert_eq!(found, minutes);
}

#[then(expr = "the sync state of the database holds {int} sessions with stable ids")]
fn state_sessions(world: &mut StoreWorld, expected: usize) {
    let state = nunc_store::local_state(world.conn(), || "gen".to_owned()).expect("state");
    assert_eq!(state.sessions.len(), expected);
    assert!(state.sessions.keys().all(|id| id.starts_with("legacy:")));
}

#[given(expr = "device {word} records a stopwatch session from {word} to {word}")]
fn records_stopwatch(world: &mut StoreWorld, device: String, start: String, end: String) {
    let settings = nunc_core::model::Settings::default();
    let mut timer = nunc_core::timer::Timer::default();
    timer
        .start(
            at(&start),
            nunc_core::model::TimerMode::Stopwatch,
            None,
            &settings,
        )
        .expect("start");
    let finish = timer.stop(at(&end), &settings).expect("stop");
    let device_id = format!("dev-{}", device.to_lowercase());
    nunc_store::commit_finish(
        world.crdt.conn(&device),
        &finish,
        &device_id,
        &timer,
        at(&end),
    )
    .expect("commit");
}

#[given(
    expr = "device {word} records a stopwatch session from {word} to {word} with projects {string} contexts {string} and note {string}"
)]
fn records_labelled_stopwatch(
    world: &mut StoreWorld,
    device: String,
    start: String,
    end: String,
    projects: String,
    contexts: String,
    note: String,
) {
    let settings = nunc_core::model::Settings::default();
    let mut timer = nunc_core::timer::Timer::default();
    let labels = nunc_core::model::Labels {
        project_tags: tag_list(&projects),
        context_tags: tag_list(&contexts),
        note,
    };
    timer
        .start_labelled(
            at(&start),
            nunc_core::model::TimerMode::Stopwatch,
            None,
            &labels,
            &settings,
        )
        .expect("start");
    let finish = timer.stop(at(&end), &settings).expect("stop");
    let device_id = format!("dev-{}", device.to_lowercase());
    nunc_store::commit_finish(
        world.crdt.conn(&device),
        &finish,
        &device_id,
        &timer,
        at(&end),
    )
    .expect("commit");
}

#[then(expr = "on device {word} the session is stored as mode {string} with kind {string}")]
fn stored_with_kind(world: &mut StoreWorld, device: String, mode: String, kind: String) {
    // Die Metadaten von Sessions anderer Geräte entstehen beim Einspielen, die eigenen beim Sync-Zugriff.
    let conn = world.crdt.conn(&device);
    let row: (String, String) = conn
        .query_row(
            "SELECT s.mode, m.kind FROM sessions s JOIN session_meta m ON m.session_id = s.id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("Session mit Metadaten");
    assert_eq!(row, (mode, kind));
}

#[then(expr = "on device {word} the listed session is a stopwatch session of {int} minutes")]
fn listed_stopwatch(world: &mut StoreWorld, device: String, minutes: u32) {
    let sessions = nunc_store::list_sessions(world.crdt.conn(&device)).expect("list");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].mode, nunc_core::model::TimerMode::Stopwatch);
    assert_eq!(sessions[0].actual_min, Some(minutes));
}

#[then(expr = "on device {word} the day has {int} minutes of work and {int} pomodoros")]
fn day_work_and_pomodoros(world: &mut StoreWorld, device: String, minutes: u32, pomodoros: u32) {
    let conn = world.crdt.conn(&device);
    let sessions = nunc_store::list_sessions(conn).expect("sessions");
    let todos = nunc_store::list_all_todos(conn).expect("todos");
    let day = NaiveDate::from_ymd_opt(2026, 10, 4).unwrap();
    let request = StatsRequest {
        period: StatsPeriod::Day,
        start_date: day,
        end_date: day,
    };
    let stats = compute_stats(&sessions, &todos, &request, day, &|time| time.date_naive());
    let today = stats.daily.iter().find(|d| d.date == day).expect("Tag");
    assert_eq!(today.work_minutes, minutes);
    assert_eq!(today.pomodoro_count, pomodoros);
}
