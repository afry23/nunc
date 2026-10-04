use chrono::{Duration, TimeZone};
use nunc_core::model::{SessionStatus, SessionType};
use nunc_platform::{ManualClock, RecordingEventSink, RecordingNotifier, Reminder};
use nunc_store::{add_todo, list_sessions, list_todos, open_in_memory, save_settings};

use super::*;

fn start_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 4, 9, 0, 0).unwrap()
}

struct Harness {
    db: Arc<Db>,
    clock: Arc<ManualClock>,
    events: Arc<RecordingEventSink>,
    notifier: Arc<RecordingNotifier>,
    service: TimerService,
}

impl Harness {
    /// `can_schedule`: Mobil (geplante Benachrichtigung) oder Desktop (sofort zeigen).
    fn new(can_schedule: bool) -> Self {
        let db = Arc::new(Db::new(open_in_memory().unwrap()));
        let clock = Arc::new(ManualClock::new(start_time()));
        let events = Arc::new(RecordingEventSink::default());
        let notifier = Arc::new(RecordingNotifier::new(can_schedule));
        let service = Self::service_for(&db, &clock, &events, &notifier);
        Self {
            db,
            clock,
            events,
            notifier,
            service,
        }
    }

    fn service_for(
        db: &Arc<Db>,
        clock: &Arc<ManualClock>,
        events: &Arc<RecordingEventSink>,
        notifier: &Arc<RecordingNotifier>,
    ) -> TimerService {
        TimerService::new(
            db.clone(),
            clock.clone(),
            events.clone(),
            notifier.clone(),
            "dev-1".to_owned(),
        )
        .unwrap()
    }

    fn todo(&self, text: &str) -> i32 {
        self.db
            .with(|conn| add_todo(conn, text, "sync-1", start_time()))
            .unwrap()
            .id
    }

    fn set(&self, change: impl FnOnce(&mut Settings)) {
        let mut settings = Settings::default();
        change(&mut settings);
        self.db.with(|conn| save_settings(conn, &settings)).unwrap();
    }

    fn advance_min(&self, minutes: i64) {
        self.clock.advance(Duration::minutes(minutes));
    }

    fn sessions(&self) -> Vec<nunc_core::model::Session> {
        self.db.with(|conn| list_sessions(conn)).unwrap()
    }
}

#[test]
fn start_persists_the_timer_schedules_the_end_and_emits_a_tick() {
    let h = Harness::new(true);
    let snapshot = h.service.start(TimerMode::Pomodoro, None).unwrap();
    assert_eq!(snapshot.state, TimerState::Running);
    assert_eq!((snapshot.remaining, snapshot.total), (1500, 1500));

    // überlebt einen Neustart: neuer Dienst auf derselben Datenbank sieht den laufenden Timer
    h.advance_min(5);
    let restarted = Harness::service_for(&h.db, &h.clock, &h.events, &h.notifier);
    assert_eq!(restarted.snapshot().unwrap().remaining, 1200);

    assert_eq!(h.events.names(), ["timer:tick"]);
    assert_eq!(
        h.notifier.calls(),
        [Reminder::Scheduled {
            id: NOTIFICATION_ID,
            at: start_time() + Duration::minutes(25),
            title: "Pomodoro geschafft".into(),
            body: "Zeit für eine Pause.".into(),
        }]
    );
}

#[test]
fn pause_cancels_and_resume_reschedules_with_the_remaining_time() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.advance_min(10);
    let paused = h.service.pause().unwrap();
    assert_eq!(paused.state, TimerState::Paused);
    assert_eq!(paused.remaining, 900);

    h.advance_min(30);
    h.service.resume().unwrap();

    let calls = h.notifier.calls();
    assert_eq!(calls[1], Reminder::Cancelled(NOTIFICATION_ID));
    assert!(matches!(
        &calls[2],
        Reminder::Scheduled { at, .. } if *at == start_time() + Duration::minutes(40 + 15)
    ));
    assert_eq!(h.events.names(), ["timer:tick"; 3]);
}

#[test]
fn a_tick_before_the_end_only_updates_the_display() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.advance_min(1);
    h.service.tick().unwrap();
    let events = h.events.events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].1["remaining"], 1440);
    assert!(h.sessions().is_empty());
}

#[test]
fn a_paused_timer_sends_no_ticks() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.service.pause().unwrap();
    h.service.tick().unwrap();
    assert_eq!(h.events.names().len(), 2);
}

#[test]
fn the_end_is_booked_at_the_calculated_time_with_counter_and_break() {
    let h = Harness::new(false);
    let todo = h.todo("Bericht +work");
    h.service.start(TimerMode::Pomodoro, Some(todo)).unwrap();
    h.advance_min(40); // der Prozess war 15 Minuten über das Ende hinaus beschäftigt
    h.service.tick().unwrap();

    let sessions = h.sessions();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].status, SessionStatus::Completed);
    assert_eq!(sessions[0].session_type, SessionType::Work);
    assert_eq!(
        sessions[0].ended_at,
        Some(start_time() + Duration::minutes(25))
    );
    assert_eq!(sessions[0].actual_min, Some(25));
    let todos = h.db.with(|conn| list_todos(conn)).unwrap();
    assert_eq!(todos[0].pomodoro_count, 1);

    // Reihenfolge: Start-Tick, dann Ende (timer:complete) und der Ruhezustand
    assert_eq!(
        h.events.names(),
        ["timer:tick", "timer:complete", "timer:tick"]
    );
    let complete = &h.events.events()[1].1;
    assert_eq!(complete["session"]["status"], "completed");
    assert_eq!(complete["suggestedBreak"]["sessionType"], "short_break");
    assert_eq!(complete["suggestedBreak"]["minutes"], 5);
    assert_eq!(h.events.events()[2].1["state"], "idle");

    // Desktop: keine geplante Benachrichtigung, das Ende wird sofort gezeigt
    assert_eq!(
        h.notifier.calls(),
        [
            Reminder::Cancelled(NOTIFICATION_ID),
            Reminder::Shown {
                title: "Pomodoro geschafft".into(),
                body: "Zeit für eine Pause.".into()
            },
            Reminder::Cancelled(NOTIFICATION_ID),
        ]
    );
    assert_eq!(h.service.suggested_break().unwrap().unwrap().minutes, 5);
}

#[test]
fn mobile_relies_on_the_scheduled_notification_at_the_end() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.advance_min(25);
    h.service.tick().unwrap();
    assert!(
        !h.notifier
            .calls()
            .iter()
            .any(|call| matches!(call, Reminder::Shown { .. }))
    );
}

#[test]
fn disabled_notifications_neither_schedule_nor_show() {
    let h = Harness::new(false);
    h.set(|s| s.notifications_enabled = false);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.advance_min(25);
    h.service.tick().unwrap();
    assert!(
        h.notifier
            .calls()
            .iter()
            .all(|call| matches!(call, Reminder::Cancelled(_)))
    );

    let mobile = Harness::new(true);
    mobile.set(|s| s.notifications_enabled = false);
    mobile.service.start(TimerMode::Pomodoro, None).unwrap();
    assert_eq!(
        mobile.notifier.calls(),
        [Reminder::Cancelled(NOTIFICATION_ID)]
    );
}

#[test]
fn stopping_early_books_an_aborted_session() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.advance_min(5);
    let snapshot = h.service.stop().unwrap();
    assert_eq!(snapshot.state, TimerState::Idle);

    let sessions = h.sessions();
    assert_eq!(sessions[0].status, SessionStatus::Aborted);
    assert_eq!(sessions[0].actual_min, Some(5));
    assert_eq!(
        h.events.names(),
        ["timer:tick", "timer:stopped", "timer:tick"]
    );
    assert_eq!(h.events.events()[1].1["session"]["status"], "aborted");
    assert_eq!(
        h.notifier.calls().last(),
        Some(&Reminder::Cancelled(NOTIFICATION_ID))
    );
}

#[test]
fn stopping_flowtime_completes_it_and_suggests_a_break() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Flowtime, None).unwrap();
    h.advance_min(50);
    h.service.stop().unwrap();

    assert_eq!(h.sessions()[0].status, SessionStatus::Completed);
    assert_eq!(
        h.events.names(),
        ["timer:tick", "timer:complete", "timer:tick"]
    );
    let suggestion = h.service.suggested_break().unwrap().unwrap();
    assert_eq!(suggestion.session_type, SessionType::ShortBreak);
    assert_eq!(suggestion.minutes, 10);
}

#[test]
fn a_suggested_break_can_be_started() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.advance_min(25);
    h.service.tick().unwrap();

    let snapshot = h.service.start_break().unwrap();
    assert_eq!(snapshot.session_type, SessionType::ShortBreak);
    assert_eq!((snapshot.remaining, snapshot.total), (300, 300));

    // die Pause vorzeitig zu beenden verwirft den Vorschlag
    h.advance_min(1);
    h.service.stop().unwrap();
    assert_eq!(h.sessions()[1].status, SessionStatus::Skipped);
    assert_eq!(h.service.suggested_break().unwrap(), None);
}

#[test]
fn auto_start_breaks_begins_the_break_right_after_the_end() {
    let h = Harness::new(true);
    h.set(|s| s.auto_start_breaks = true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.advance_min(25);
    h.service.tick().unwrap();

    let snapshot = h.service.snapshot().unwrap();
    assert_eq!(snapshot.state, TimerState::Running);
    assert_eq!(snapshot.session_type, SessionType::ShortBreak);
    // der Folgestart ist persistiert
    let restarted = Harness::service_for(&h.db, &h.clock, &h.events, &h.notifier);
    assert_eq!(
        restarted.snapshot().unwrap().session_type,
        SessionType::ShortBreak
    );
}

#[test]
fn recovery_books_a_countdown_that_ended_while_the_app_was_closed() {
    let h = Harness::new(false);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.advance_min(120);
    let restarted = Harness::service_for(&h.db, &h.clock, &h.events, &h.notifier);
    let snapshot = restarted.recover().unwrap();

    assert_eq!(snapshot.state, TimerState::Idle);
    let sessions = h.sessions();
    assert_eq!(sessions.len(), 1);
    assert_eq!(
        sessions[0].ended_at,
        Some(start_time() + Duration::minutes(25))
    );
    // nicht live erlebt: keine Benachrichtigung
    assert!(
        !h.notifier
            .calls()
            .iter()
            .any(|call| matches!(call, Reminder::Shown { .. }))
    );
}

#[test]
fn recovery_reschedules_a_running_timer() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    h.advance_min(10);
    let restarted = Harness::service_for(&h.db, &h.clock, &h.events, &h.notifier);
    restarted.recover().unwrap();
    assert!(matches!(
        h.notifier.calls().last(),
        Some(Reminder::Scheduled { at, .. }) if *at == start_time() + Duration::minutes(25)
    ));
}

#[test]
fn an_invalid_transition_changes_nothing() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    let before = (h.events.names().len(), h.notifier.calls().len());

    let error = h.service.start(TimerMode::Flowtime, None).unwrap_err();
    assert_eq!(error.code, "already_active");
    assert_eq!(h.service.resume().unwrap_err().code, "not_paused");

    assert_eq!((h.events.names().len(), h.notifier.calls().len()), before);
    assert_eq!(h.service.snapshot().unwrap().mode, TimerMode::Pomodoro);
}

#[test]
fn idle_commands_report_typed_errors() {
    let h = Harness::new(true);
    assert_eq!(h.service.pause().unwrap_err().code, "not_active");
    assert_eq!(h.service.stop().unwrap_err().code, "not_active");
    assert_eq!(
        h.service.start_break().unwrap_err().code,
        "no_break_suggested"
    );
    assert!(h.events.names().is_empty());
}

#[test]
fn a_failed_booking_keeps_the_timer_so_nothing_is_lost() {
    let h = Harness::new(true);
    // Todo existiert nicht: Verbuchen schlägt fehl (Fremdschlüssel)
    h.service.start(TimerMode::Pomodoro, Some(999)).unwrap();
    h.advance_min(25);

    assert!(h.service.tick().is_err());
    assert!(h.sessions().is_empty());
    assert_eq!(h.service.lock().unwrap().state(), TimerState::Running);
    assert_eq!(h.events.names(), ["timer:tick"]);
}

#[test]
fn exclusive_reloads_the_timer_afterwards() {
    let h = Harness::new(true);
    h.service.start(TimerMode::Pomodoro, None).unwrap();
    let swapped = h
        .service
        .exclusive(|| {
            h.db.with(|conn| nunc_store::save_timer(conn, &Timer::default(), start_time()))
        })
        .is_ok();
    assert!(swapped);
    assert_eq!(h.service.snapshot().unwrap().state, TimerState::Idle);
    assert_eq!(
        h.notifier.calls().last(),
        Some(&Reminder::Cancelled(NOTIFICATION_ID))
    );
}
