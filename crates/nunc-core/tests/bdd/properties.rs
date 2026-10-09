//! Eigenschaftstests (`@property`-Szenarien): zufällige Eingaben statt Beispiele. Die Szenarien nennen die
//! Eigenschaft und die Zahl der Fälle (`Then the property "…" holds for 200 random cases`); die Regeln stehen
//! hier. Bei einem Fehler liefert proptest das verkleinerte Gegenbeispiel in der Fehlermeldung.
//!
//! Der Zufall ist nicht festgelegt (`failure_persistence` aus): ein Gegenbeispiel, das einmal auftritt, steht im
//! Testlauf und sollte als Beispiel in das passende Feature übernommen werden.

use std::collections::BTreeSet;

use chrono::{Duration, NaiveDate, TimeZone, Utc};
use cucumber::then;
use nunc_core::backup_policy::{BackupFile, backup_file_name, files_to_delete};
use nunc_core::crdt::{Register, SessionCrdt, SyncState, TodoCrdt};
use nunc_core::export::{ExportRow, to_csv};
use nunc_core::hlc::Hlc;
use nunc_core::model::{
    Session, SessionStatus, SessionType, Settings, StatsPeriod, StatsRequest, TimerMode,
};
use nunc_core::position::position_between;
use nunc_core::stats::{calculate_streaks, compute_stats, utc_date};
use nunc_core::timer::{FinishKind, Timer, TimerError};
use nunc_core::todotxt::{ParsedTodo, build_raw_text, parse_line};
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};

use crate::DomainWorld;

type Outcome = Result<(), TestCaseError>;

fn run<S: Strategy>(
    cases: u32,
    strategy: S,
    test: impl Fn(S::Value) -> Outcome,
) -> Result<(), String> {
    let config = Config {
        cases,
        failure_persistence: None,
        ..Config::default()
    };
    TestRunner::new(config)
        .run(&strategy, test)
        .map_err(|error| error.to_string())
}

#[then(expr = "the property {string} holds for {int} random cases")]
fn property_holds(_world: &mut DomainWorld, name: String, cases: u32) {
    let result = match name.as_str() {
        "any well-formed todo survives serializing and parsing" => todo_roundtrip(cases),
        "parsing and serializing is stable for any text" => parse_is_stable(cases),
        "merging sync states is commutative" => crdt_commutative(cases),
        "merging sync states is associative" => crdt_associative(cases),
        "merging sync states is idempotent" => crdt_idempotent(cases),
        "a position between two others sorts between them" => position_sorts_between(cases),
        "garbage collection is idempotent and only drops deleted todos" => gc_properties(cases),
        "merging session states is commutative, associative and idempotent" => {
            session_merge_laws(cases)
        }
        "streaks follow from the set of days" => streak_invariants(cases),
        "rotation keeps exactly the newest backups" => rotation_keeps_newest(cases),
        "random commands keep the timer consistent" => timer_consistent(cases),
        "the CSV export can be read back for any text" => csv_roundtrip(cases),
        "the statistics do not depend on the order of the sessions" => {
            stats_order_independent(cases)
        }
        other => panic!("unbekannte Eigenschaft {other:?}"),
    };
    if let Err(message) = result {
        panic!("Eigenschaft {name:?} verletzt:\n{message}");
    }
}

// ---- Strategien ------------------------------------------------------------

fn date_strategy() -> impl Strategy<Value = NaiveDate> {
    (2000i32..2100, 1u32..=12, 1u32..=28)
        .prop_map(|(y, m, d)| NaiveDate::from_ymd_opt(y, m, d).unwrap())
}

/// Beschreibung aus Wörtern und Tags. Das erste Wort beginnt mit einem Buchstaben und hat mindestens zwei Zeichen,
/// damit es weder Datum noch Priorität noch das Erledigt-Zeichen `x` sein kann.
fn description_strategy() -> impl Strategy<Value = String> {
    let first = "[a-zA-Z][a-zA-Z0-9]{1,6}";
    let word = prop_oneof![
        "[a-zA-Z0-9_.,!?-]{1,7}",
        "[+@][a-zA-Z0-9_]{1,5}",
        Just("+".to_owned()),
        Just("@".to_owned()),
    ];
    prop_oneof![
        Just(String::new()),
        (first, prop::collection::vec(word, 0..5)).prop_map(|(first, rest)| {
            let mut words = vec![first];
            words.extend(rest);
            words.join(" ")
        }),
    ]
}

fn tags_of(description: &str) -> (Vec<String>, Vec<String>) {
    let tag = |prefix: char| {
        description
            .split_whitespace()
            .filter(|w| w.len() > 1 && w.starts_with(prefix))
            .map(str::to_owned)
            .collect()
    };
    (tag('+'), tag('@'))
}

fn parsed_todo_strategy() -> impl Strategy<Value = ParsedTodo> {
    let open = (
        prop::option::of(prop::sample::select(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect::<Vec<_>>(),
        )),
        prop::option::of(date_strategy()),
        description_strategy(),
    )
        .prop_map(|(priority, created_at, description)| {
            (false, priority, None, created_at, description)
        });
    let done = (
        prop::option::of((date_strategy(), prop::option::of(date_strategy()))),
        description_strategy(),
    )
        .prop_map(|(dates, description)| {
            let (completed_at, created_at) = dates.map_or((None, None), |(c, o)| (Some(c), o));
            (true, None, completed_at, created_at, description)
        });
    prop_oneof![open, done].prop_map(
        |(completed, priority, completed_at, created_at, description)| {
            let (project_tags, context_tags) = tags_of(&description);
            ParsedTodo {
                priority,
                completed,
                completed_at,
                created_at,
                description,
                project_tags,
                context_tags,
            }
        },
    )
}

fn text_strategy() -> impl Strategy<Value = String> {
    "[ -~äöüß€()x0-9+@-]{0,40}"
}

// ---- todo.txt --------------------------------------------------------------

fn todo_roundtrip(cases: u32) -> Result<(), String> {
    run(cases, parsed_todo_strategy(), |todo| {
        let line = build_raw_text(&todo);
        prop_assert_eq!(parse_line(&line), todo, "Zeile {:?}", line);
        Ok(())
    })
}

fn parse_is_stable(cases: u32) -> Result<(), String> {
    run(cases, text_strategy(), |line| {
        let first = parse_line(&line);
        let rebuilt = build_raw_text(&first);
        prop_assert_eq!(
            parse_line(&rebuilt),
            first,
            "Zeile {:?} -> {:?}",
            line,
            rebuilt
        );
        Ok(())
    })
}

// ---- Serien ----------------------------------------------------------------

fn streak_invariants(cases: u32) -> Result<(), String> {
    let base = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let days = prop::collection::vec(0i64..40, 0..25).prop_map(move |offsets| {
        offsets
            .into_iter()
            .map(|o| base + Duration::days(o))
            .collect::<Vec<_>>()
    });
    run(cases, (days, 0i64..45), move |(days, today_offset)| {
        let today = base + Duration::days(today_offset);
        let (current, longest) = calculate_streaks(&days, today);
        let distinct: BTreeSet<_> = days.iter().copied().collect();

        prop_assert!(current <= longest);
        prop_assert!(longest as usize <= distinct.len());
        prop_assert_eq!(longest == 0, distinct.is_empty());

        let mut reversed = days.clone();
        reversed.reverse();
        prop_assert_eq!(calculate_streaks(&reversed, today), (current, longest));

        // aktuelle Serie = Folge, die am letzten Tag endet, nur wenn dieser heute oder gestern ist
        let last = distinct.iter().next_back().copied();
        let expected_current = match last {
            Some(last) if last == today || last == today - Duration::days(1) => {
                let mut run = 0;
                let mut day = last;
                while distinct.contains(&day) {
                    run += 1;
                    day -= Duration::days(1);
                }
                run
            }
            _ => 0,
        };
        prop_assert_eq!(current, expected_current);

        // ein weiterer Tag macht die längste Serie nie kürzer
        let mut more = days.clone();
        more.push(base + Duration::days(20));
        prop_assert!(calculate_streaks(&more, today).1 >= longest);
        Ok(())
    })
}

// ---- Rotation --------------------------------------------------------------

fn rotation_keeps_newest(cases: u32) -> Result<(), String> {
    let base = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let names = (
        prop::collection::btree_set(0i64..400, 0..14),
        prop::collection::vec(
            prop::sample::select(vec![
                "notes.txt",
                "nunc-backup-manual.db",
                "before-restore-2025-01-01.db",
                "nunc-backup-2025-13-40.db",
            ]),
            0..3,
        ),
        0usize..10,
    );
    run(cases, names, move |(offsets, noise, max)| {
        let mut files: Vec<BackupFile> = offsets
            .iter()
            .map(|o| backup(&backup_file_name(base + Duration::days(*o))))
            .collect();
        files.extend(noise.iter().map(|name| backup(name)));
        // Reihenfolge der Eingabe darf nichts ändern
        files.reverse();

        let deleted = files_to_delete(&files, max);
        prop_assert_eq!(deleted.len(), offsets.len().saturating_sub(max));

        let valid: Vec<String> = offsets
            .iter()
            .map(|o| backup_file_name(base + Duration::days(*o)))
            .collect();
        prop_assert!(
            deleted.iter().all(|name| valid.contains(name)),
            "nur gültige Backups"
        );

        // gelöscht wird nur, was älter ist als alles, was bleibt
        let remaining: Vec<&String> = valid.iter().filter(|n| !deleted.contains(n)).collect();
        if let (Some(newest_deleted), Some(oldest_kept)) =
            (deleted.iter().max(), remaining.iter().min())
        {
            prop_assert!(newest_deleted < *oldest_kept);
        }

        // nach dem Löschen ist die Rotation fertig
        let rest: Vec<BackupFile> = files
            .into_iter()
            .filter(|f| !deleted.contains(&f.name))
            .collect();
        prop_assert!(files_to_delete(&rest, max).is_empty());
        Ok(())
    })
}

fn backup(name: &str) -> BackupFile {
    BackupFile {
        name: name.to_owned(),
        path: format!("/b/{name}"),
        size_bytes: 1,
        modified_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
    }
}

// ---- Timer -----------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Command {
    StartPomodoro(Option<i32>),
    StartFlowtime,
    StartBreak,
    Pause,
    Resume,
    Stop,
    Tick,
    Advance(i64),
}

fn command_strategy() -> impl Strategy<Value = Command> {
    prop_oneof![
        prop::option::of(1i32..4).prop_map(Command::StartPomodoro),
        Just(Command::StartFlowtime),
        Just(Command::StartBreak),
        Just(Command::Pause),
        Just(Command::Resume),
        Just(Command::Stop),
        Just(Command::Tick),
        (0i64..4000).prop_map(Command::Advance),
    ]
}

fn timer_consistent(cases: u32) -> Result<(), String> {
    run(
        cases,
        prop::collection::vec(command_strategy(), 1..40),
        |commands| {
            let settings = Settings::default();
            let mut now = Utc.with_ymd_and_hms(2026, 10, 3, 9, 0, 0).unwrap();
            let mut timer = Timer::default();
            for command in commands {
                let before = timer.state();
                let outcome: Result<Option<nunc_core::timer::Finish>, TimerError> = match command {
                    Command::StartPomodoro(todo) => timer
                        .start(now, TimerMode::Pomodoro, todo, &settings)
                        .map(|()| None),
                    Command::StartFlowtime => timer
                        .start(now, TimerMode::Flowtime, None, &settings)
                        .map(|()| None),
                    Command::StartBreak => timer.start_break(now).map(|()| None),
                    Command::Pause => timer.pause(now).map(|()| None),
                    Command::Resume => timer.resume(now).map(|()| None),
                    Command::Stop => timer.stop(now, &settings).map(Some),
                    Command::Tick => Ok(timer.tick(now, &settings)),
                    Command::Advance(seconds) => {
                        now += Duration::seconds(seconds);
                        Ok(None)
                    }
                };

                // Fehler passen zum Zustand davor
                if let Err(error) = outcome {
                    use nunc_core::model::TimerState::{Idle, Paused, Running};
                    let allowed = match (command, before) {
                        (
                            Command::StartPomodoro(_)
                            | Command::StartFlowtime
                            | Command::StartBreak,
                            Running | Paused,
                        ) => {
                            matches!(error, TimerError::AlreadyActive)
                        }
                        (Command::StartBreak, Idle) => {
                            matches!(error, TimerError::NoBreakSuggested)
                        }
                        (Command::Pause | Command::Resume | Command::Stop, Idle) => {
                            matches!(error, TimerError::NotActive)
                        }
                        (Command::Pause, Paused) => matches!(error, TimerError::NotRunning),
                        (Command::Resume, Running) => matches!(error, TimerError::NotPaused),
                        _ => false,
                    };
                    prop_assert!(
                        allowed,
                        "{:?} im Zustand {:?} lieferte {:?}",
                        command,
                        before,
                        error
                    );
                }

                // Zustand lässt sich speichern und wiederherstellen
                let json = serde_json::to_string(&timer).unwrap();
                prop_assert_eq!(serde_json::from_str::<Timer>(&json).unwrap(), timer.clone());

                // Anzeige ist stimmig
                let snapshot = timer.snapshot(now);
                prop_assert_eq!(snapshot.state, timer.state());
                if snapshot.total > 0 {
                    prop_assert!(snapshot.remaining <= snapshot.total);
                }
                if timer.state() == nunc_core::model::TimerState::Idle {
                    prop_assert_eq!((snapshot.remaining, snapshot.total), (0, 0));
                }

                // beendete Sessions sind in sich schlüssig
                if let Ok(Some(finish)) = outcome {
                    let session = &finish.session;
                    prop_assert!(session.ended_at >= session.started_at);
                    let wall_seconds = (session.ended_at - session.started_at).num_seconds();
                    prop_assert!(
                        i64::from(session.actual_min) * 60 <= wall_seconds,
                        "mehr gearbeitet als vergangen"
                    );
                    if finish.kind == FinishKind::Completed && session.planned_min.is_some() {
                        prop_assert_eq!(Some(session.actual_min), session.planned_min);
                    }
                    prop_assert_eq!(timer.state(), nunc_core::model::TimerState::Idle);
                    if session.session_type != SessionType::Work {
                        prop_assert!(
                            finish.increment_todo.is_none() && finish.suggested_break.is_none()
                        );
                    }
                }
            }
            Ok(())
        },
    )
}

// ---- CSV -------------------------------------------------------------------

/// Minimaler RFC-4180-Leser für den Rücklesetest.
fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let (mut rows, mut row, mut field) = (Vec::new(), Vec::new(), String::new());
    let (mut quoted, mut chars) = (false, text.chars().peekable());
    while let Some(c) = chars.next() {
        match (quoted, c) {
            (true, '"') if chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            (true, '"') => quoted = false,
            (true, _) => field.push(c),
            (false, '"') => quoted = true,
            (false, ',') => row.push(std::mem::take(&mut field)),
            (false, '\n') => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            (false, _) => field.push(c),
        }
    }
    rows
}

fn csv_roundtrip(cases: u32) -> Result<(), String> {
    let free_text = prop_oneof!["[ -~äöü\n\r\t\"',=+@-]{0,30}", Just(String::new())];
    let row = (
        free_text.clone(),
        prop::collection::vec(free_text.clone(), 0..3),
        prop::collection::vec(free_text, 0..3),
    );
    run(cases, prop::collection::vec(row, 0..5), |rows| {
        let export: Vec<ExportRow> = rows
            .iter()
            .map(|(text, projects, contexts)| ExportRow {
                date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                start_time: chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                end_time: None,
                duration_min: Some(5),
                mode: TimerMode::Pomodoro,
                status: SessionStatus::Completed,
                todo_text: text.clone(),
                project_tags: projects.clone(),
                context_tags: contexts.clone(),
                session_type: SessionType::Work,
                note: String::new(),
                origin: "timer".to_owned(),
            })
            .collect();
        let parsed = parse_csv(&to_csv(&export));
        prop_assert_eq!(parsed.len(), export.len() + 1);
        for (row, (text, projects, contexts)) in parsed[1..].iter().zip(&rows) {
            prop_assert_eq!(row.len(), 12);
            for (cell, expected) in [
                (&row[6], text.clone()),
                (&row[7], projects.join(" ")),
                (&row[8], contexts.join(" ")),
            ] {
                let guarded = expected.starts_with(['=', '+', '-', '@', '\t', '\r', '\n']);
                let wanted = if guarded {
                    format!("'{expected}")
                } else {
                    expected
                };
                prop_assert_eq!(cell, &wanted);
            }
        }
        Ok(())
    })
}

// ---- Statistik -------------------------------------------------------------

fn stats_session_strategy() -> impl Strategy<Value = Session> {
    (
        0i64..(20 * 24 * 60),
        prop::sample::select(vec![TimerMode::Pomodoro, TimerMode::Flowtime]),
        prop::sample::select(vec![
            SessionType::Work,
            SessionType::ShortBreak,
            SessionType::LongBreak,
        ]),
        prop::sample::select(vec![
            SessionStatus::Completed,
            SessionStatus::Aborted,
            SessionStatus::Skipped,
        ]),
        prop::option::of(0u32..90),
        prop::option::of(1i32..4),
    )
        .prop_map(
            |(minutes, mode, session_type, status, actual_min, todo_id)| Session {
                id: 0,
                mode,
                session_type,
                status,
                started_at: Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap()
                    + Duration::minutes(minutes),
                ended_at: None,
                planned_min: None,
                actual_min,
                todo_id,
                break_min: None,
            },
        )
}

fn stats_order_independent(cases: u32) -> Result<(), String> {
    run(
        cases,
        prop::collection::vec(stats_session_strategy(), 0..25),
        |sessions| {
            let todos = Vec::new();
            let day = |d: u32| NaiveDate::from_ymd_opt(2026, 3, d).unwrap();
            let request = StatsRequest {
                period: StatsPeriod::Month,
                start_date: day(3),
                end_date: day(18),
            };
            let today = day(21);
            let forward = compute_stats(&sessions, &todos, &request, today, &utc_date);
            let mut reversed = sessions.clone();
            reversed.reverse();
            prop_assert_eq!(
                &compute_stats(&reversed, &todos, &request, today, &utc_date),
                &forward
            );

            // die Tageswerte im Zeitraum können die Summen nicht übersteigen
            let day_pomodoros: u32 = forward.daily.iter().map(|d| d.pomodoro_count).sum();
            let day_work: u32 = forward.daily.iter().map(|d| d.work_minutes).sum();
            prop_assert!(
                day_pomodoros <= forward.total_pomodoros && day_work <= forward.total_work_min
            );
            prop_assert_eq!(
                forward
                    .weekdays
                    .iter()
                    .map(|w| w.pomodoro_count)
                    .sum::<u32>(),
                day_pomodoros
            );
            prop_assert_eq!(
                forward.monthly.iter().map(|m| m.work_minutes).sum::<u32>(),
                day_work
            );
            Ok(())
        },
    )
}

// ---- CRDT ------------------------------------------------------------------

fn hlc_strategy() -> impl Strategy<Value = Hlc> {
    (
        0i64..4,
        0u32..3,
        prop::sample::select(vec!["d1", "d2", "d3"]),
    )
        .prop_map(|(wall, counter, device)| Hlc::new(wall, counter, device))
}

fn register<T: Ord + Clone + std::fmt::Debug>(
    value: impl Strategy<Value = T>,
) -> impl Strategy<Value = Register<T>> {
    (value, hlc_strategy()).prop_map(|(value, clock)| Register::new(value, clock))
}

fn small_text() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["", "x", "y"]).prop_map(str::to_owned)
}

fn small_option() -> impl Strategy<Value = Option<String>> {
    prop::option::of(small_text())
}

fn todo_crdt(id: &'static str) -> impl Strategy<Value = TodoCrdt> {
    (
        (
            register(small_option()),
            register(prop::bool::ANY),
            register(small_option()),
            register(small_option()),
            register(small_text()),
        ),
        (
            register(small_text()),
            register(prop::bool::ANY),
            prop::collection::btree_map(
                prop::sample::select(vec!["d1".to_owned(), "d2".to_owned(), "d3".to_owned()]),
                0u32..4,
                0..3,
            ),
        ),
    )
        .prop_map(
            move |(
                (priority, completed, completed_at, created_at, description),
                (position, deleted, pomodoros),
            )| TodoCrdt {
                sync_id: id.to_owned(),
                priority,
                completed,
                completed_at,
                created_at,
                description,
                position,
                deleted,
                pomodoros,
            },
        )
}

fn session_crdt() -> impl Strategy<Value = SessionCrdt> {
    (
        (
            prop::sample::select(vec!["s1", "s2"]),
            prop::sample::select(vec!["d1", "d2"]),
            prop::sample::select(vec!["pomodoro", "flowtime"]),
            0u32..3,
        ),
        (
            register(prop::sample::select(vec![
                "2026-03-01T10:00:00Z".to_owned(),
                "2026-03-01T11:00:00Z".to_owned(),
            ])),
            register(prop::option::of(small_text())),
            register(0u32..3),
            register(prop::option::of(small_text())),
            register(small_text()),
            register(prop::bool::ANY),
        ),
    )
        .prop_map(
            |(
                (sync_id, device, mode, planned),
                (started_at, ended_at, actual_min, todo, note, deleted),
            )| SessionCrdt {
                sync_id: sync_id.to_owned(),
                device_id: device.to_owned(),
                mode: mode.to_owned(),
                session_type: "work".to_owned(),
                status: "completed".to_owned(),
                planned_min: planned,
                break_min: 5,
                origin: "timer".to_owned(),
                started_at,
                ended_at,
                actual_min,
                todo,
                note,
                deleted,
            },
        )
}

fn sync_state() -> impl Strategy<Value = SyncState> {
    (
        prop::option::of(todo_crdt("a")),
        prop::option::of(todo_crdt("b")),
        prop::collection::vec(session_crdt(), 0..4),
    )
        .prop_map(|(a, b, sessions)| SyncState {
            todos: a
                .into_iter()
                .chain(b)
                .map(|todo| (todo.sync_id.clone(), todo))
                .collect(),
            sessions: sessions
                .into_iter()
                .map(|session| (session.sync_id.clone(), session))
                .collect(),
        })
}

fn session_merge_laws(cases: u32) -> Result<(), String> {
    run(
        cases,
        (session_crdt(), session_crdt(), session_crdt()),
        |(a, b, c)| {
            let (a, b, c) = (
                SessionCrdt {
                    sync_id: "s".to_owned(),
                    ..a
                },
                SessionCrdt {
                    sync_id: "s".to_owned(),
                    ..b
                },
                SessionCrdt {
                    sync_id: "s".to_owned(),
                    ..c
                },
            );
            prop_assert_eq!(a.merge(&b), b.merge(&a));
            prop_assert_eq!(a.merge(&b.merge(&c)), a.merge(&b).merge(&c));
            prop_assert_eq!(a.merge(&a), a.clone());
            let merged = a.merge(&b);
            prop_assert_eq!(merged.merge(&b), merged.clone());
            Ok(())
        },
    )
}

fn crdt_commutative(cases: u32) -> Result<(), String> {
    run(cases, (sync_state(), sync_state()), |(a, b)| {
        prop_assert_eq!(a.merge(&b), b.merge(&a));
        Ok(())
    })
}

fn crdt_associative(cases: u32) -> Result<(), String> {
    run(
        cases,
        (sync_state(), sync_state(), sync_state()),
        |(a, b, c)| {
            prop_assert_eq!(a.merge(&b.merge(&c)), a.merge(&b).merge(&c));
            Ok(())
        },
    )
}

fn crdt_idempotent(cases: u32) -> Result<(), String> {
    run(cases, (sync_state(), sync_state()), |(a, b)| {
        prop_assert_eq!(a.merge(&a), a.clone());
        let merged = a.merge(&b);
        prop_assert_eq!(merged.merge(&b), merged.clone());
        prop_assert_eq!(merged.merge(&a), merged);
        Ok(())
    })
}

fn position_sorts_between(cases: u32) -> Result<(), String> {
    let key = prop::collection::vec(
        prop::sample::select(b"0123456789abcdefghijklmnopqrstuvwxyz".to_vec()),
        1..5,
    )
    .prop_map(|digits| {
        let mut text = String::from_utf8(digits).unwrap_or_default();
        while text.ends_with('0') {
            text.pop();
        }
        text
    })
    .prop_filter("Schlüssel endet nie auf 0", |text| !text.is_empty());
    run(cases, (key.clone(), key), |(first, second)| {
        prop_assume!(first != second);
        let (low, high) = if first < second {
            (first, second)
        } else {
            (second, first)
        };
        let between = position_between(Some(&low), Some(&high));
        prop_assert!(low < between && between < high);
        prop_assert!(!between.ends_with('0'));
        let before = position_between(None, Some(&low));
        prop_assert!(before < low);
        let after = position_between(Some(&high), None);
        prop_assert!(high < after);
        Ok(())
    })
}

fn gc_properties(cases: u32) -> Result<(), String> {
    run(cases, (sync_state(), 0i64..200), |(state, days)| {
        let now = days * 24 * 3_600_000;
        let once = state.collect_garbage(now);
        prop_assert_eq!(once.collect_garbage(now), once.clone());
        for (id, todo) in &state.todos {
            if !once.todos.contains_key(id) {
                prop_assert!(todo.is_deleted());
            }
        }
        for (id, session) in &state.sessions {
            if !once.sessions.contains_key(id) {
                prop_assert!(session.is_deleted());
            }
        }
        Ok(())
    })
}
