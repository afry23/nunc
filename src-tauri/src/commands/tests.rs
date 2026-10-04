//! Command-Integrationstests (M3-09): echte Commands über die IPC-Schicht von Tauri (`mock_builder`), echte
//! SQLite-Datei, feste Uhr. Geprüft werden Argumentnamen (camelCase wie aus JavaScript), Ergebnisse, Fehlerform
//! `{ code, message }` und die Reihenfolge der Ereignisse.

use std::sync::Arc;

use chrono::{DateTime, Duration, TimeZone, Utc};
use nunc_platform::{ManualClock, MemorySecretStore, RecordingEventSink, RecordingNotifier};
use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, mock_builder, mock_context, noop_assets};
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindow, WebviewWindowBuilder, generate_handler};
use tempfile::TempDir;

use crate::state::{AppPaths, AppState, Platform};

use super::{app, backup, notifications, settings, stats, sync, timer, todos};

struct Harness {
    _dir: TempDir,
    app: tauri::App<tauri::test::MockRuntime>,
    webview: WebviewWindow<tauri::test::MockRuntime>,
    clock: Arc<ManualClock>,
    events: Arc<RecordingEventSink>,
}

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 4, hour, minute, 0).unwrap()
}

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let clock = Arc::new(ManualClock::new(at(9, 0)));
        let events = Arc::new(RecordingEventSink::default());
        let state = AppState::open(
            AppPaths {
                db: dir.path().join("nunc.db"),
                default_backup_dir: dir.path().join("backups"),
                legacy_db: None,
            },
            Platform {
                clock: clock.clone(),
                events: events.clone(),
                notifier: Arc::new(RecordingNotifier::new(false)),
                secrets: Arc::new(MemorySecretStore::default()),
            },
        )
        .unwrap();

        let app = mock_builder()
            .plugin(tauri_plugin_notification::init())
            .invoke_handler(generate_handler![
                app::app_info,
                settings::get_settings,
                settings::save_settings,
                todos::get_todos,
                todos::add_todo,
                todos::update_todo,
                todos::delete_todo,
                todos::set_todo_completed,
                todos::reorder_todos,
                timer::start_timer,
                timer::start_break,
                timer::pause_timer,
                timer::resume_timer,
                timer::stop_timer,
                timer::get_timer_tick,
                timer::get_suggested_break,
                stats::get_stats,
                stats::export_stats,
                backup::get_backup_status,
                backup::trigger_backup,
                backup::list_backups,
                backup::restore_from_backup,
                backup::export_backup,
                sync::set_sync_provider,
                sync::get_webdav_config,
                sync::save_webdav_config,
                sync::test_webdav_connection,
                notifications::get_notification_status,
                notifications::request_notification_permission,
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        app.manage(state);
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        Self {
            _dir: dir,
            app,
            webview,
            clock,
            events,
        }
    }

    /// Ruft einen Command auf wie die Oberfläche: `Ok(Wert)` oder `Err({ code, message })`.
    fn call(&self, cmd: &str, args: Value) -> Result<Value, Value> {
        let request = InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        };
        tauri::test::get_ipc_response(&self.webview, request)
            .map(|body| body.deserialize::<Value>().unwrap())
    }

    fn ok(&self, cmd: &str, args: Value) -> Value {
        self.call(cmd, args)
            .unwrap_or_else(|error| panic!("{cmd} schlug fehl: {error}"))
    }

    fn err_code(&self, cmd: &str, args: Value) -> String {
        let error = self
            .call(cmd, args)
            .expect_err(&format!("{cmd} hätte fehlschlagen müssen"));
        assert!(error["message"].as_str().is_some_and(|m| !m.is_empty()));
        error["code"].as_str().unwrap().to_owned()
    }

    fn advance_min(&self, minutes: i64) {
        self.clock.advance(Duration::minutes(minutes));
    }

    fn state(&self) -> tauri::State<'_, AppState> {
        self.app.state::<AppState>()
    }
}

fn texts(todos: &Value) -> Vec<&str> {
    todos
        .as_array()
        .unwrap()
        .iter()
        .map(|todo| todo["rawText"].as_str().unwrap())
        .collect()
}

/// Auf dem Desktop ist die Berechtigung immer erteilt; die Abfrage fragt niemanden.
#[test]
fn notification_permission_is_granted_on_desktop() {
    let h = Harness::new();
    assert_eq!(
        h.ok("get_notification_status", json!({})),
        json!({ "permission": "granted" })
    );
    assert_eq!(
        h.ok("request_notification_permission", json!({})),
        json!({ "permission": "granted" })
    );
}

#[test]
fn app_info_names_the_app() {
    let h = Harness::new();
    let info = h.ok("app_info", json!({}));
    assert_eq!(info["name"], "nunc");
    assert!(info["version"].is_string());
}

#[test]
fn settings_default_roundtrip_and_validation() {
    let h = Harness::new();
    let mut current = h.ok("get_settings", json!({}));
    assert_eq!(current["pomodoroMin"], 25);
    assert_eq!(current["flowtimeRatio"], 0.2);

    current["pomodoroMin"] = json!(30);
    current["autoStartBreaks"] = json!(true);
    h.ok("save_settings", json!({ "settings": current }));
    let saved = h.ok("get_settings", json!({}));
    assert_eq!(saved["pomodoroMin"], 30);
    assert_eq!(saved["autoStartBreaks"], true);

    // ungültig: alle Verstöße in einer Meldung, nichts wird gespeichert
    let mut invalid = saved.clone();
    invalid["pomodoroMin"] = json!(0);
    invalid["soundVolume"] = json!(150);
    let error = h
        .call("save_settings", json!({ "settings": invalid }))
        .unwrap_err();
    assert_eq!(error["code"], "invalid_settings");
    let message = error["message"].as_str().unwrap();
    assert!(message.contains("pomodoroMin") && message.contains("soundVolume"));
    assert_eq!(h.ok("get_settings", json!({}))["pomodoroMin"], 30);
}

#[test]
fn settings_keep_backend_owned_fields() {
    let h = Harness::new();
    h.ok("trigger_backup", json!({}));
    let mut current = h.ok("get_settings", json!({}));
    let stored_date = current["lastBackupDate"].clone();
    assert_eq!(stored_date, "2026-10-04");

    current["lastBackupDate"] = json!("1999-01-01");
    current["backupDir"] = json!("/irgendwo/anders");
    h.ok("save_settings", json!({ "settings": current }));

    let after = h.ok("get_settings", json!({}));
    assert_eq!(after["lastBackupDate"], "2026-10-04");
    assert_eq!(after["backupDir"], "");
}

#[test]
fn the_sync_provider_is_chosen_by_its_own_command_and_not_by_save_settings() {
    let h = Harness::new();
    assert_eq!(h.ok("get_settings", json!({}))["syncProvider"], "none");
    h.ok("set_sync_provider", json!({ "provider": "webdav" }));
    assert_eq!(h.ok("get_settings", json!({}))["syncProvider"], "webdav");

    // ein veralteter Entwurf im Formular überschreibt die Wahl nicht
    let mut stale = h.ok("get_settings", json!({}));
    stale["syncProvider"] = json!("none");
    h.ok("save_settings", json!({ "settings": stale }));
    assert_eq!(h.ok("get_settings", json!({}))["syncProvider"], "webdav");

    h.ok("set_sync_provider", json!({ "provider": "none" }));
    assert_eq!(h.ok("get_settings", json!({}))["syncProvider"], "none");
    assert!(
        h.call("set_sync_provider", json!({ "provider": "dropbox" }))
            .is_err()
    );
}

#[test]
fn webdav_settings_are_saved_without_ever_returning_the_password() {
    let h = Harness::new();
    let fresh = h.ok("get_webdav_config", json!({}));
    assert_eq!(fresh["hasPassword"], false);
    assert_eq!(fresh["config"]["remotePath"], "/nunc/");
    assert_eq!(fresh["config"]["authMethod"], "basic");

    let config = json!({
        "url": " https://dav.example.org/dav ",
        "authMethod": "digest",
        "username": "me",
        "remotePath": "/apps/nunc/"
    });
    h.ok(
        "save_webdav_config",
        json!({ "config": config, "password": "s3cret" }),
    );
    let saved = h.ok("get_webdav_config", json!({}));
    assert_eq!(saved["hasPassword"], true);
    assert_eq!(saved["config"]["url"], "https://dav.example.org/dav");
    assert_eq!(saved["config"]["authMethod"], "digest");
    assert!(!saved.to_string().contains("s3cret"));

    // ohne Passwort bleibt das gespeicherte erhalten
    h.ok(
        "save_webdav_config",
        json!({ "config": config, "password": null }),
    );
    assert_eq!(h.ok("get_webdav_config", json!({}))["hasPassword"], true);
}

#[test]
fn webdav_settings_with_a_bad_address_are_refused() {
    let h = Harness::new();
    let config = json!({
        "url": "cloud.example.org", "authMethod": "basic", "username": "me", "remotePath": "/nunc/"
    });
    assert_eq!(
        h.err_code(
            "save_webdav_config",
            json!({ "config": config, "password": "x" })
        ),
        "invalid_input"
    );
    assert_eq!(h.ok("get_webdav_config", json!({}))["config"]["url"], "");
    // leeren ist erlaubt
    let empty = json!({
        "url": "", "authMethod": "basic", "username": "", "remotePath": "/nunc/"
    });
    h.ok(
        "save_webdav_config",
        json!({ "config": empty, "password": null }),
    );
}

#[test]
fn the_connection_test_reports_failures_as_a_result_not_an_error() {
    let h = Harness::new();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let config = json!({
        "url": url, "authMethod": "basic", "username": "me", "remotePath": "/nunc/"
    });
    let result = h.ok(
        "test_webdav_connection",
        json!({ "config": config, "password": "x" }),
    );
    assert_eq!(result["success"], false);
    assert_eq!(
        result["message"],
        "Verbindung abgelehnt – Server nicht erreichbar"
    );

    let bad = json!({
        "url": "kein-url", "authMethod": "basic", "username": "me", "remotePath": "/"
    });
    let result = h.ok(
        "test_webdav_connection",
        json!({ "config": bad, "password": null }),
    );
    assert_eq!(result["success"], false);
    assert_eq!(result["message"], "Ungültige Server-Adresse: kein-url");
}

#[test]
fn todos_crud_filter_sort_and_reorder() {
    let h = Harness::new();
    let a = h.ok("add_todo", json!({ "rawText": "(B) Bericht +work @desk" }));
    let b = h.ok("add_todo", json!({ "rawText": "(A) Steuer +privat" }));
    let c = h.ok("add_todo", json!({ "rawText": "Einkaufen @street" }));
    assert_eq!(a["priority"], "B");
    assert_eq!(a["projectTags"], json!(["+work"]));
    assert_eq!(a["contextTags"], json!(["@desk"]));
    assert_eq!(a["pomodoroCount"], 0);

    // ohne Angaben: manuelle Reihenfolge
    let all = h.ok("get_todos", json!({}));
    assert_eq!(
        texts(&all),
        [
            "(B) Bericht +work @desk",
            "(A) Steuer +privat",
            "Einkaufen @street"
        ]
    );

    let by_priority = h.ok("get_todos", json!({ "sort": "priority" }));
    assert_eq!(
        texts(&by_priority),
        [
            "(A) Steuer +privat",
            "(B) Bericht +work @desk",
            "Einkaufen @street"
        ]
    );
    let filtered = h.ok(
        "get_todos",
        json!({ "filter": { "projects": ["+work"] }, "sort": "manual", "direction": "descending" }),
    );
    assert_eq!(texts(&filtered), ["(B) Bericht +work @desk"]);

    // bearbeiten, abhaken, umsortieren
    let updated = h.ok(
        "update_todo",
        json!({ "id": c["id"], "rawText": "Einkaufen @street +haushalt" }),
    );
    assert_eq!(updated["projectTags"], json!(["+haushalt"]));
    let done = h.ok(
        "set_todo_completed",
        json!({ "id": b["id"], "completed": true }),
    );
    assert_eq!(done["completed"], true);
    assert_eq!(done["completedAt"], "2026-10-04");

    h.ok(
        "reorder_todos",
        json!({ "ids": [c["id"], a["id"], b["id"]] }),
    );
    let reordered = h.ok("get_todos", json!({}));
    assert_eq!(reordered[0]["id"], c["id"]);
    assert_eq!(reordered[2]["id"], b["id"]);

    // löschen entfernt es aus der Liste
    h.ok("delete_todo", json!({ "id": a["id"] }));
    assert_eq!(h.ok("get_todos", json!({})).as_array().unwrap().len(), 2);
}

#[test]
fn todo_errors_are_typed() {
    let h = Harness::new();
    assert_eq!(
        h.err_code("add_todo", json!({ "rawText": "   " })),
        "invalid_input"
    );
    assert_eq!(
        h.err_code("update_todo", json!({ "id": 1, "rawText": "" })),
        "invalid_input"
    );
    assert_eq!(
        h.err_code("update_todo", json!({ "id": 999, "rawText": "x" })),
        "not_found"
    );
    assert_eq!(h.err_code("delete_todo", json!({ "id": 999 })), "not_found");
    assert_eq!(
        h.err_code(
            "set_todo_completed",
            json!({ "id": 999, "completed": true })
        ),
        "not_found"
    );
}

#[test]
fn a_pomodoro_runs_through_events_in_order_and_lands_in_the_stats() {
    let h = Harness::new();
    let todo = h.ok("add_todo", json!({ "rawText": "Bericht +work" }));

    let started = h.ok(
        "start_timer",
        json!({ "mode": "pomodoro", "todoId": todo["id"] }),
    );
    assert_eq!(started["state"], "running");
    assert_eq!(started["remaining"], 1500);
    assert_eq!(started["total"], 1500);
    assert_eq!(started["todoId"], todo["id"]);

    h.advance_min(10);
    h.ok("pause_timer", json!({}));
    assert_eq!(h.ok("get_timer_tick", json!({}))["remaining"], 900);
    h.advance_min(30);
    h.ok("resume_timer", json!({}));
    h.advance_min(15);
    let after = h.ok("get_timer_tick", json!({}));
    assert_eq!(after["state"], "idle");

    assert_eq!(
        h.events.names(),
        [
            "timer:tick",     // start
            "timer:tick",     // pause
            "timer:tick",     // resume
            "timer:complete", // Ende, beim Abruf nach dem Hintergrund verbucht
            "timer:tick",
        ]
    );
    let complete = &h.events.events()[3].1;
    assert_eq!(complete["session"]["status"], "completed");
    assert_eq!(complete["session"]["actualMin"], 25);
    assert_eq!(complete["suggestedBreak"]["minutes"], 5);
    assert_eq!(
        h.ok("get_suggested_break", json!({}))["sessionType"],
        "short_break"
    );

    // Pomodoro-Zähler und Statistik stammen aus der gespeicherten Session
    assert_eq!(h.ok("get_todos", json!({}))[0]["pomodoroCount"], 1);
    let stats = h.ok(
        "get_stats",
        json!({ "request": { "period": "day", "startDate": "2026-10-04", "endDate": "2026-10-04" } }),
    );
    assert_eq!(stats["totalPomodoros"], 1);
    assert_eq!(stats["totalWorkMin"], 25);
    assert_eq!(stats["topTodos"][0]["todoId"], todo["id"]);
}

#[test]
fn stopping_and_breaks_through_commands() {
    let h = Harness::new();
    h.ok("start_timer", json!({ "mode": "flowtime", "todoId": null }));
    h.advance_min(50);
    let stopped = h.ok("stop_timer", json!({}));
    assert_eq!(stopped["state"], "idle");
    assert_eq!(
        h.events.names(),
        ["timer:tick", "timer:complete", "timer:tick"]
    );

    let on_break = h.ok("start_break", json!({}));
    assert_eq!(on_break["sessionType"], "short_break");
    assert_eq!(on_break["remaining"], 600);
    h.advance_min(2);
    h.ok("stop_timer", json!({}));
    assert_eq!(
        h.events.names()[3..],
        ["timer:tick", "timer:stopped", "timer:tick"]
    );
    let skipped = &h.events.events()[4].1;
    assert_eq!(skipped["session"]["status"], "skipped");
}

#[test]
fn timer_errors_are_typed_and_leave_the_state_alone() {
    let h = Harness::new();
    assert_eq!(h.err_code("pause_timer", json!({})), "not_active");
    assert_eq!(h.err_code("stop_timer", json!({})), "not_active");
    assert_eq!(h.err_code("start_break", json!({})), "no_break_suggested");

    h.ok("start_timer", json!({ "mode": "pomodoro", "todoId": null }));
    assert_eq!(
        h.err_code("start_timer", json!({ "mode": "flowtime", "todoId": null })),
        "already_active"
    );
    assert_eq!(h.err_code("resume_timer", json!({})), "not_paused");
    assert_eq!(h.ok("get_timer_tick", json!({}))["mode"], "pomodoro");
    assert_eq!(h.events.names().len(), 1);
}

#[test]
fn export_contains_the_sessions_of_the_range() {
    let h = Harness::new();
    h.ok("start_timer", json!({ "mode": "pomodoro", "todoId": null }));
    h.advance_min(25);
    h.ok("get_timer_tick", json!({}));

    let request = json!({ "period": "day", "startDate": "2026-10-04", "endDate": "2026-10-04" });
    let csv = h.ok(
        "export_stats",
        json!({ "request": request.clone(), "format": "csv" }),
    );
    let lines: Vec<&str> = csv.as_str().unwrap().lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("date,start_time,end_time,duration_min,mode,status"));
    assert!(lines[1].contains("pomodoro,completed"));

    let json_export = h.ok(
        "export_stats",
        json!({ "request": request, "format": "json" }),
    );
    let rows: Value = serde_json::from_str(json_export.as_str().unwrap()).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert_eq!(rows[0]["duration_min"], 25);

    let empty = h.ok(
        "export_stats",
        json!({
            "request": { "period": "day", "startDate": "2026-01-01", "endDate": "2026-01-01" },
            "format": "json"
        }),
    );
    assert_eq!(empty.as_str().unwrap(), "[]");
}

#[test]
fn backup_and_restore_through_commands() {
    let h = Harness::new();
    assert_eq!(h.ok("get_backup_status", json!({}))["backupCount"], 0);
    assert_eq!(h.ok("get_backup_status", json!({}))["isOutdated"], true);

    h.ok("add_todo", json!({ "rawText": "vor dem Backup" }));
    let backup = h.ok("trigger_backup", json!({}));
    let path = backup["path"].as_str().unwrap().to_owned();
    assert!(std::path::Path::new(&path).is_file());
    assert!(backup["sizeBytes"].as_u64().unwrap() > 0);
    assert!(backup["deleted"].as_array().unwrap().is_empty());

    let status = h.ok("get_backup_status", json!({}));
    assert_eq!(status["backupCount"], 1);
    assert_eq!(status["isOutdated"], false);
    let list = h.ok("list_backups", json!({}));
    assert_eq!(list[0]["filename"], "nunc-backup-2026-10-04.db");
    assert_eq!(list[0]["path"], path);

    h.ok("add_todo", json!({ "rawText": "nach dem Backup" }));
    h.ok("start_timer", json!({ "mode": "pomodoro", "todoId": null }));
    h.ok("restore_from_backup", json!({ "sourcePath": path }));

    assert_eq!(texts(&h.ok("get_todos", json!({}))), ["vor dem Backup"]);
    assert_eq!(h.ok("get_timer_tick", json!({}))["state"], "idle");
    assert!(h.events.names().contains(&"app:reload".to_owned()));
    // Notfall-Backup des Zustands vor dem Restore liegt neben den Backups
    let emergency = h
        .state()
        .backup_dir()
        .unwrap()
        .join("before-restore-2026-10-04.db");
    assert!(emergency.is_file());
}

#[test]
fn restoring_a_bad_file_fails_with_a_typed_error_and_changes_nothing() {
    let h = Harness::new();
    h.ok("add_todo", json!({ "rawText": "bleibt" }));
    let bad = h._dir.path().join("kaputt.db");
    std::fs::write(&bad, b"kein SQLite").unwrap();

    assert_eq!(
        h.err_code("restore_from_backup", json!({ "sourcePath": bad })),
        "invalid_backup"
    );
    assert_eq!(
        h.err_code(
            "restore_from_backup",
            json!({ "sourcePath": h._dir.path().join("fehlt.db") })
        ),
        "invalid_backup"
    );
    assert_eq!(texts(&h.ok("get_todos", json!({}))), ["bleibt"]);
    assert!(!h.events.names().contains(&"app:reload".to_owned()));
}

#[test]
fn only_backups_from_the_backup_folder_can_be_exported() {
    let h = Harness::new();
    let outside = h._dir.path().join("nunc-backup-2026-10-04.db");
    std::fs::write(&outside, b"x").unwrap();
    assert_eq!(
        h.err_code("export_backup", json!({ "sourcePath": outside })),
        "invalid_input"
    );
    assert_eq!(
        h.err_code("export_backup", json!({ "sourcePath": "/etc/passwd" })),
        "invalid_input"
    );
}

#[test]
fn unknown_arguments_are_rejected_by_tauri_not_ignored() {
    // Schutz vor Tippfehlern im Frontend: `raw_text` statt `rawText` ist ein Fehler
    let h = Harness::new();
    assert!(h.call("add_todo", json!({ "raw_text": "x" })).is_err());
}
