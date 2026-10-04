//! Steps für `features/timer/*.feature` (Pomodoro, Flowtime, Persistenz).

use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use cucumber::{given, then, when};
use nunc_core::model::{SessionStatus, SessionType, Settings, TimerMode, TimerState};
use nunc_core::timer::{AutoStart, Finish, FinishKind, Timer, TimerError};

use crate::DomainWorld;

#[derive(Debug)]
pub struct State {
    settings: Settings,
    timer: Timer,
    now: DateTime<Utc>,
    finish: Option<Finish>,
    error: Option<TimerError>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            timer: Timer::default(),
            now: Utc.with_ymd_and_hms(2026, 10, 3, 9, 0, 0).unwrap(),
            finish: None,
            error: None,
        }
    }
}

impl State {
    fn record(&mut self, result: Result<(), TimerError>) {
        self.error = result.err();
    }

    fn finish(&self) -> &Finish {
        self.assert_no_error();
        self.finish.as_ref().expect("keine Session beendet")
    }

    fn assert_no_error(&self) {
        assert_eq!(self.error, None, "unerwarteter Fehler");
    }

    /// Lässt Zeit vergehen und führt danach einen Tick aus, wie es der Dienst im Sekundentakt täte.
    fn advance(&mut self, delta: Duration) {
        self.now += delta;
    }

    fn tick(&mut self) {
        if let Some(finish) = self.timer.tick(self.now, &self.settings) {
            self.finish = Some(finish);
        }
    }

    fn start(&mut self, mode: TimerMode, todo_id: Option<i32>) {
        let result = self.timer.start(self.now, mode, todo_id, &self.settings);
        self.record(result);
    }

    fn start_break(&mut self) {
        let result = self.timer.start_break(self.now);
        self.record(result);
    }
}

fn clock_time(text: &str) -> DateTime<Utc> {
    let time = NaiveTime::parse_from_str(text, "%H:%M:%S").expect("Uhrzeit HH:MM:SS erwartet");
    Utc.with_ymd_and_hms(2026, 10, 3, 0, 0, 0).unwrap()
        + Duration::seconds(i64::from(chrono::Timelike::num_seconds_from_midnight(
            &time,
        )))
}

fn clock_text(time: DateTime<Utc>) -> String {
    time.format("%H:%M:%S").to_string()
}

fn session_type(name: &str) -> SessionType {
    name.parse()
        .unwrap_or_else(|_| panic!("unbekannter Session-Typ {name:?}"))
}

// ---- Given -----------------------------------------------------------------

#[given(expr = "it is {word}")]
fn it_is(world: &mut DomainWorld, time: String) {
    world.timer.now = clock_time(&time);
}

#[given(expr = "the pomodoro lasts {int} minutes")]
fn pomodoro_minutes(world: &mut DomainWorld, minutes: u32) {
    world.timer.settings.pomodoro_min = minutes;
}

#[given(expr = "the short break lasts {int} minutes")]
fn short_break_minutes(world: &mut DomainWorld, minutes: u32) {
    world.timer.settings.short_break_min = minutes;
}

#[given(expr = "the long break lasts {int} minutes and comes after every {int} pomodoros")]
fn long_break(world: &mut DomainWorld, minutes: u32, after: u32) {
    world.timer.settings.long_break_min = minutes;
    world.timer.settings.long_break_after = after;
}

#[given("automatic start of breaks is on")]
fn auto_breaks(world: &mut DomainWorld) {
    world.timer.settings.auto_start_breaks = true;
}

#[given("automatic start of the next pomodoro is on")]
fn auto_pomodoro(world: &mut DomainWorld) {
    world.timer.settings.auto_start_next_pomodoro = true;
}

#[given(expr = "the flowtime ratio is {float}")]
fn flowtime_ratio(world: &mut DomainWorld, ratio: f64) {
    world.timer.settings.flowtime_ratio = ratio;
}

#[given(expr = "the minimum flow state is {int} minutes")]
fn min_flow_state(world: &mut DomainWorld, minutes: u32) {
    world.timer.settings.flowtime_min_flow_state_min = minutes;
}

// ---- When ------------------------------------------------------------------

#[when(expr = "{int} minutes pass")]
fn minutes_pass(world: &mut DomainWorld, minutes: i64) {
    world.timer.advance(Duration::minutes(minutes));
}

/// Die App war im Hintergrund: es gab keinen `timer.tick`, nur die Uhr ist weitergelaufen.
#[when(expr = "{int} minutes pass without a tick")]
fn minutes_pass_without_tick(world: &mut DomainWorld, minutes: i64) {
    world.timer.advance(Duration::minutes(minutes));
}

#[when(expr = "{int} hours pass without a tick")]
fn hours_pass_without_tick(world: &mut DomainWorld, hours: i64) {
    world.timer.advance(Duration::hours(hours));
}

#[when(expr = "{int} hours pass")]
fn hours_pass(world: &mut DomainWorld, hours: i64) {
    world.timer.advance(Duration::hours(hours));
}

#[when(expr = "{int} seconds pass")]
fn seconds_pass(world: &mut DomainWorld, seconds: i64) {
    world.timer.advance(Duration::seconds(seconds));
}

#[when(expr = "the clock jumps back {int} minutes")]
fn clock_jumps_back(world: &mut DomainWorld, minutes: i64) {
    world.timer.advance(Duration::minutes(-minutes));
}

#[when("I start a pomodoro")]
fn start_pomodoro(world: &mut DomainWorld) {
    world.timer.start(TimerMode::Pomodoro, None);
}

#[when(expr = "I start a pomodoro for todo {int}")]
fn start_pomodoro_for_todo(world: &mut DomainWorld, todo: i32) {
    world.timer.start(TimerMode::Pomodoro, Some(todo));
}

#[when("I start a flowtime session")]
fn start_flowtime(world: &mut DomainWorld) {
    world.timer.start(TimerMode::Flowtime, None);
}

#[when(expr = "I start a flowtime session for todo {int}")]
fn start_flowtime_for_todo(world: &mut DomainWorld, todo: i32) {
    world.timer.start(TimerMode::Flowtime, Some(todo));
}

#[when("I start the suggested break")]
fn start_suggested_break(world: &mut DomainWorld) {
    world.timer.start_break();
}

#[when("I pause the timer")]
fn pause(world: &mut DomainWorld) {
    let state = &mut world.timer;
    let result = state.timer.pause(state.now);
    state.record(result);
}

#[when("I resume the timer")]
fn resume(world: &mut DomainWorld) {
    let state = &mut world.timer;
    let result = state.timer.resume(state.now);
    state.record(result);
}

#[when("I stop the timer")]
fn stop(world: &mut DomainWorld) {
    let state = &mut world.timer;
    match state.timer.stop(state.now, &state.settings) {
        Ok(finish) => {
            state.error = None;
            state.finish = Some(finish);
        }
        Err(error) => state.error = Some(error),
    }
}

#[when("the timer ticks")]
fn ticks(world: &mut DomainWorld) {
    world.timer.tick();
}

#[when(expr = "I finish {int} pomodoros in a row")]
fn finish_pomodoros(world: &mut DomainWorld, count: u32) {
    let state = &mut world.timer;
    for _ in 0..count {
        state.start(TimerMode::Pomodoro, None);
        state.advance(Duration::minutes(i64::from(state.settings.pomodoro_min)));
        state.tick();
        state.assert_no_error();
    }
}

#[when("I finish the suggested break")]
fn finish_break(world: &mut DomainWorld) {
    let state = &mut world.timer;
    let minutes = state
        .timer
        .suggested_break()
        .expect("keine Pause vorgeschlagen")
        .minutes;
    state.start_break();
    state.advance(Duration::minutes(i64::from(minutes)));
    state.tick();
}

#[when("the timer state is saved and restored")]
fn save_and_restore(world: &mut DomainWorld) {
    let json = serde_json::to_string(&world.timer.timer).expect("Zustand serialisieren");
    world.timer.timer = serde_json::from_str(&json).expect("Zustand wiederherstellen");
}

// ---- Then ------------------------------------------------------------------

#[then(expr = "the timer is {word}")]
fn timer_is(world: &mut DomainWorld, expected: String) {
    world.timer.assert_no_error();
    let actual = match world.timer.timer.state() {
        TimerState::Idle => "idle",
        TimerState::Running => "running",
        TimerState::Paused => "paused",
    };
    assert_eq!(actual, expected);
}

#[then(expr = "{int} seconds remain")]
fn seconds_remain(world: &mut DomainWorld, expected: u32) {
    world.timer.assert_no_error();
    let snapshot = world.timer.timer.snapshot(world.timer.now);
    assert_eq!(snapshot.remaining, expected);
}

#[then(expr = "{int} seconds have elapsed")]
fn seconds_elapsed(world: &mut DomainWorld, expected: u32) {
    world.timer.assert_no_error();
    let snapshot = world.timer.timer.snapshot(world.timer.now);
    assert_eq!(snapshot.total, 0, "Stoppuhr hat keine Gesamtdauer");
    assert_eq!(snapshot.remaining, expected);
}

#[then(expr = "the total is {int} seconds")]
fn total_is(world: &mut DomainWorld, expected: u32) {
    let snapshot = world.timer.timer.snapshot(world.timer.now);
    assert_eq!(snapshot.total, expected);
}

#[then(expr = "the command fails with {word}")]
fn fails_with(world: &mut DomainWorld, code: String) {
    // Der erwartete Fehler gilt danach als behandelt, damit Folgeschritte den Zustand prüfen können.
    let error = world
        .timer
        .error
        .take()
        .expect("der Befehl hätte fehlschlagen müssen");
    assert_eq!(error.code(), code);
}

#[then(expr = "the session finished as {word}")]
fn finished_as(world: &mut DomainWorld, kind: String) {
    let expected = match kind.as_str() {
        "completed" => FinishKind::Completed,
        "stopped" => FinishKind::Stopped,
        other => panic!("unbekannte Art {other:?}"),
    };
    assert_eq!(world.timer.finish().kind, expected);
}

#[then("no session finished")]
fn nothing_finished(world: &mut DomainWorld) {
    world.timer.assert_no_error();
    assert!(
        world.timer.finish.is_none(),
        "unerwartet beendet: {:?}",
        world.timer.finish
    );
}

#[then(expr = "the recorded session is {word}")]
fn recorded_status(world: &mut DomainWorld, expected: String) {
    let status = world.timer.finish().session.status;
    assert_eq!(status.as_str(), expected);
    assert_eq!(status, expected.parse::<SessionStatus>().unwrap());
}

#[then(expr = "the recorded session is a {word} session")]
fn recorded_type(world: &mut DomainWorld, expected: String) {
    assert_eq!(
        world.timer.finish().session.session_type,
        session_type(&expected)
    );
}

#[then(expr = "the recorded session has {int} planned minutes")]
fn recorded_planned(world: &mut DomainWorld, expected: u32) {
    assert_eq!(world.timer.finish().session.planned_min, Some(expected));
}

#[then("the recorded session has no planned minutes")]
fn recorded_no_planned(world: &mut DomainWorld) {
    assert_eq!(world.timer.finish().session.planned_min, None);
}

#[then(expr = "the recorded session has {int} actual minutes")]
fn recorded_actual(world: &mut DomainWorld, expected: u32) {
    assert_eq!(world.timer.finish().session.actual_min, expected);
}

#[then(expr = "the recorded session started at {word}")]
fn recorded_started(world: &mut DomainWorld, expected: String) {
    assert_eq!(
        clock_text(world.timer.finish().session.started_at),
        expected
    );
}

#[then(expr = "the recorded session ended at {word}")]
fn recorded_ended(world: &mut DomainWorld, expected: String) {
    assert_eq!(clock_text(world.timer.finish().session.ended_at), expected);
}

#[then(expr = "the recorded session belongs to todo {int}")]
fn recorded_todo(world: &mut DomainWorld, expected: i32) {
    assert_eq!(world.timer.finish().session.todo_id, Some(expected));
}

#[then("the recorded session belongs to no todo")]
fn recorded_no_todo(world: &mut DomainWorld) {
    assert_eq!(world.timer.finish().session.todo_id, None);
}

#[then(expr = "the recorded session mode is {word}")]
fn recorded_mode(world: &mut DomainWorld, mode: String) {
    let expected: TimerMode = mode.parse().expect("Modus");
    assert_eq!(world.timer.finish().session.mode, expected);
}

#[then(expr = "the suggested break is {int} minutes")]
fn suggested_minutes(world: &mut DomainWorld, minutes: u32) {
    let finish = world.timer.finish();
    match (minutes, finish.suggested_break) {
        (0, None) => {}
        (expected, Some(suggestion)) => assert_eq!(suggestion.minutes, expected),
        (expected, None) => panic!("{expected} Minuten Pause erwartet, keine vorgeschlagen"),
    }
    assert_eq!(world.timer.timer.suggested_break(), finish.suggested_break);
}

#[then(expr = "the suggested break is a {word} of {int} minutes")]
fn suggested_break(world: &mut DomainWorld, kind: String, minutes: u32) {
    let suggestion = world
        .timer
        .finish()
        .suggested_break
        .expect("keine Pause vorgeschlagen");
    assert_eq!(suggestion.session_type, session_type(&kind));
    assert_eq!(suggestion.minutes, minutes);
    assert_eq!(world.timer.timer.suggested_break(), Some(suggestion));
}

#[then("no break is suggested")]
fn no_break(world: &mut DomainWorld) {
    assert_eq!(world.timer.finish().suggested_break, None);
    assert_eq!(world.timer.timer.suggested_break(), None);
}

#[then(expr = "todo {int} gets a pomodoro")]
fn todo_gets_pomodoro(world: &mut DomainWorld, expected: i32) {
    assert_eq!(world.timer.finish().increment_todo, Some(expected));
}

#[then("no todo gets a pomodoro")]
fn no_todo_gets_pomodoro(world: &mut DomainWorld) {
    assert_eq!(world.timer.finish().increment_todo, None);
}

#[then("a break starts automatically")]
fn auto_break(world: &mut DomainWorld) {
    assert_eq!(world.timer.finish().auto_start, Some(AutoStart::Break));
}

#[then(expr = "a {word} session starts automatically for todo {int}")]
fn auto_work(world: &mut DomainWorld, mode: String, todo: i32) {
    let expected = AutoStart::Work {
        mode: mode.parse().expect("Modus"),
        todo_id: Some(todo),
    };
    assert_eq!(world.timer.finish().auto_start, Some(expected));
}

#[then("nothing starts automatically")]
fn nothing_automatic(world: &mut DomainWorld) {
    assert_eq!(world.timer.finish().auto_start, None);
}
