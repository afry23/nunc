//! Schreibt eine Beispiel-Datenbank mit nunc-store (Todos, Sessions, Einstellungen, Timer-Zustand) für den
//! Interop-Check mit der Go-App (`tests/interop/go-reads-nunc-db.sh`).
//!
//! Aufruf: `cargo run -p nunc-store --example write_sample -- <ziel.db>`

use chrono::{DateTime, NaiveDate, Utc};
use nunc_core::model::{Settings, SyncProvider, TimerMode};
use nunc_core::timer::Timer;

fn at(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .unwrap()
        .with_timezone(&Utc)
}

fn main() {
    let path = std::env::args().nth(1).expect("Zielpfad fehlt");
    let _ = std::fs::remove_file(&path);
    let mut conn = nunc_store::open(path.as_ref()).expect("öffnen");
    let t0 = at("2026-10-04T08:00:00Z");

    let report = nunc_store::add_todo(
        &conn,
        "(A) 2026-10-01 Write report +work @desk",
        "n-0001",
        t0,
    )
    .unwrap();
    let call = nunc_store::add_todo(&conn, "Call dentist @phone", "n-0002", t0).unwrap();
    let old = nunc_store::add_todo(&conn, "Old idea", "n-0003", t0).unwrap();
    nunc_store::set_todo_completed(
        &conn,
        call.id,
        true,
        NaiveDate::from_ymd_opt(2026, 10, 4).unwrap(),
        t0,
    )
    .unwrap();
    nunc_store::delete_todo(&conn, old.id, t0).unwrap();
    nunc_store::reorder_todos(&mut conn, &[call.id, report.id]).unwrap();

    let settings = Settings {
        pomodoro_min: 30,
        flowtime_ratio: 0.25,
        sync_provider: SyncProvider::Webdav,
        ..Settings::default()
    };
    nunc_store::save_settings(&mut conn, &settings).unwrap();

    // ein Pomodoro für `report`, danach läuft ein weiterer Timer (active_timer ist Go unbekannt)
    let mut timer = Timer::default();
    timer
        .start(
            at("2026-10-04T09:00:00Z"),
            TimerMode::Pomodoro,
            Some(report.id),
            &settings,
        )
        .unwrap();
    let finish = timer.tick(at("2026-10-04T09:40:00Z"), &settings).unwrap();
    nunc_store::commit_finish(
        &mut conn,
        &finish,
        "nunc-device",
        &timer,
        at("2026-10-04T09:40:00Z"),
    )
    .unwrap();
    timer.start_break(at("2026-10-04T09:45:00Z")).unwrap();
    nunc_store::save_timer(&conn, &timer, at("2026-10-04T09:45:00Z")).unwrap();
    let dir = std::path::Path::new(&path).with_file_name("backups");
    nunc_store::run_backup(&conn, &dir, NaiveDate::from_ymd_opt(2026, 10, 4).unwrap()).unwrap();
    println!("geschrieben: {path}");
}
