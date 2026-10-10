//! Timer-Zustandsautomat (pur, zeitstempelbasiert).
//!
//! Die Zeit ist nie ein hochgezählter Zähler: gearbeitet wurde, was in abgeschlossenen Laufabschnitten
//! aufgelaufen ist (`accumulated_ms`) plus der laufende Abschnitt (`now − running_since`). Damit überlebt
//! der Zustand Hintergrund und App-Neustart (ADR 0004): Der ganze [`Timer`] ist serialisierbar und wird bei
//! jeder Transition persistiert (`active_timer`, M2-02).
//!
//! Übergänge nehmen `now` als Parameter. Der Aufrufer ruft regelmäßig und beim Wiederöffnen [`Timer::tick`] auf;
//! läuft ein Countdown ab, endet er **zum rechnerischen Endzeitpunkt**, nicht zum Zeitpunkt des Aufrufs.
//!
//! Verhalten der Go-App (`services/timer.go`), mit bewussten Abweichungen:
//! - ein Break-Vorschlag gilt nur bis zum nächsten Start (Arbeit oder Pause) und verfällt, wenn die Pause
//!   endet (Go ließ ihn nach einer abgebrochenen Pause stehen);
//! - `auto_start_next_pomodoro` wird ausgewertet (Go speicherte die Einstellung nur).

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::flowtime::calculate_break_duration;
use crate::model::{SessionStatus, SessionType, Settings, TimerMode, TimerState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TimerError {
    #[error("es läuft bereits ein Timer")]
    AlreadyActive,
    #[error("es läuft kein Timer")]
    NotActive,
    #[error("der Timer läuft nicht")]
    NotRunning,
    #[error("der Timer ist nicht pausiert")]
    NotPaused,
    #[error("keine Pause vorgeschlagen: zuerst eine Arbeitssession abschließen")]
    NoBreakSuggested,
}

impl TimerError {
    /// Stabiler Code für die Schnittstelle (M8-02).
    pub fn code(self) -> &'static str {
        match self {
            Self::AlreadyActive => "already_active",
            Self::NotActive => "not_active",
            Self::NotRunning => "not_running",
            Self::NotPaused => "not_paused",
            Self::NoBreakSuggested => "no_break_suggested",
        }
    }
}

/// Vorgeschlagene Pause nach einer abgeschlossenen Arbeitssession.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BreakSuggestion {
    pub session_type: SessionType,
    pub minutes: u32,
}

/// Aktuell laufende oder pausierte Session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSession {
    pub mode: TimerMode,
    pub session_type: SessionType,
    pub todo_id: Option<i32>,
    /// Countdown-Ziel in Sekunden; `0` = Stoppuhr (Flowtime-Arbeit).
    pub planned_sec: u32,
    pub started_at: DateTime<Utc>,
    /// Arbeitszeit aus bereits beendeten Laufabschnitten.
    pub accumulated_ms: i64,
    /// Beginn des aktuellen Laufabschnitts; `None` solange pausiert.
    pub running_since: Option<DateTime<Utc>>,
}

impl ActiveSession {
    fn worked_ms(&self, now: DateTime<Utc>) -> i64 {
        let segment = self
            .running_since
            .map_or(0, |since| (now - since).num_milliseconds().max(0));
        self.accumulated_ms + segment
    }

    fn worked_sec(&self, now: DateTime<Utc>) -> u32 {
        u32::try_from(self.worked_ms(now) / 1000).unwrap_or(u32::MAX)
    }

    fn is_countdown(&self) -> bool {
        self.planned_sec > 0
    }

    fn planned_ms(&self) -> i64 {
        i64::from(self.planned_sec) * 1000
    }

    /// Ein laufender Countdown ist zum Zeitpunkt `now` abgelaufen.
    fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.running_since.is_some()
            && self.is_countdown()
            && self.worked_ms(now) >= self.planned_ms()
    }

    /// Rechnerischer Endzeitpunkt eines laufenden Countdowns.
    fn due_at(&self) -> Option<DateTime<Utc>> {
        let since = self.running_since?;
        self.is_countdown()
            .then(|| since + Duration::milliseconds(self.planned_ms() - self.accumulated_ms))
    }
}

/// Wie eine Session endete: natürlich (`timer:complete`) oder vom Nutzer vorzeitig (`timer:stopped`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum FinishKind {
    Completed,
    Stopped,
}

/// Zu speichernde Session (Felder wie in der Tabelle `sessions`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedSession {
    pub mode: TimerMode,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    /// Zieldauer in Minuten, nur bei Countdowns.
    pub planned_min: Option<u32>,
    pub actual_min: u32,
    pub todo_id: Option<i32>,
}

/// Was nach einer beendeten Session automatisch folgen soll (Einstellungen `auto_start_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoStart {
    /// Die vorgeschlagene Pause starten ([`Timer::start_break`]).
    Break,
    /// Die nächste Arbeitssession im selben Modus und mit demselben Todo starten ([`Timer::start`]).
    Work {
        mode: TimerMode,
        todo_id: Option<i32>,
    },
}

/// Ergebnis einer beendeten Session mit allen Folgeaktionen für den Aufrufer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finish {
    pub kind: FinishKind,
    pub session: FinishedSession,
    /// Empfohlene Pause (nur nach abgeschlossener Arbeit).
    pub suggested_break: Option<BreakSuggestion>,
    /// Todo, dessen Pomodoro-Zähler um eins steigt.
    pub increment_todo: Option<i32>,
    pub auto_start: Option<AutoStart>,
}

/// Momentaufnahme für die Anzeige (entspricht `TimerTickPayload` der Go-App).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshot {
    pub state: TimerState,
    pub mode: TimerMode,
    pub session_type: SessionType,
    /// Countdown: verbleibende Sekunden; Stoppuhr: verstrichene Sekunden.
    pub remaining: u32,
    /// Gesamtdauer in Sekunden, `0` bei der Stoppuhr.
    pub total: u32,
    pub todo_id: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct LastWork {
    mode: TimerMode,
    todo_id: Option<i32>,
}

/// Der Zustandsautomat. `Default` ist der Ruhezustand.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timer {
    active: Option<ActiveSession>,
    /// Abgeschlossene Pomodoros seit der letzten langen Pause.
    pomodoros_in_cycle: u32,
    suggestion: Option<BreakSuggestion>,
    last_work: Option<LastWork>,
}

impl Timer {
    pub fn active(&self) -> Option<&ActiveSession> {
        self.active.as_ref()
    }

    pub fn state(&self) -> TimerState {
        match &self.active {
            None => TimerState::Idle,
            Some(a) if a.running_since.is_some() => TimerState::Running,
            Some(_) => TimerState::Paused,
        }
    }

    pub fn suggested_break(&self) -> Option<BreakSuggestion> {
        self.suggestion
    }

    /// Rechnerischer Endzeitpunkt, solange ein Countdown läuft (für die geplante Benachrichtigung).
    /// `None` im Ruhezustand, pausiert und bei der Stoppuhr.
    pub fn due_at(&self) -> Option<DateTime<Utc>> {
        self.active.as_ref()?.due_at()
    }

    /// Startet eine Arbeitssession (Pomodoro: Countdown, Flowtime und Stopwatch: Stoppuhr).
    pub fn start(
        &mut self,
        now: DateTime<Utc>,
        mode: TimerMode,
        todo_id: Option<i32>,
        settings: &Settings,
    ) -> Result<(), TimerError> {
        let planned_sec = match mode {
            TimerMode::Pomodoro => settings.pomodoro_min.saturating_mul(60),
            TimerMode::Flowtime | TimerMode::Stopwatch => 0,
        };
        self.start_planned(now, mode, todo_id, planned_sec)
    }

    /// Wie [`Timer::start`], aber mit eigener Countdown-Dauer in Sekunden (`0` = Stoppuhr). Nur für Tests der
    /// Oberfläche gedacht, die keine 25 Minuten warten können (`NUNC_TEST_FAST_TIMER`).
    pub fn start_planned(
        &mut self,
        now: DateTime<Utc>,
        mode: TimerMode,
        todo_id: Option<i32>,
        planned_sec: u32,
    ) -> Result<(), TimerError> {
        self.ensure_idle()?;
        self.suggestion = None;
        self.begin(now, mode, SessionType::Work, planned_sec, todo_id);
        Ok(())
    }

    /// Startet die vorgeschlagene Pause.
    pub fn start_break(&mut self, now: DateTime<Utc>) -> Result<(), TimerError> {
        self.ensure_idle()?;
        let suggestion = self.suggestion.ok_or(TimerError::NoBreakSuggested)?;
        let mode = self.last_work.map_or(TimerMode::Pomodoro, |w| w.mode);
        self.begin(
            now,
            mode,
            suggestion.session_type,
            suggestion.minutes.saturating_mul(60),
            None,
        );
        Ok(())
    }

    pub fn pause(&mut self, now: DateTime<Utc>) -> Result<(), TimerError> {
        let active = self.active.as_mut().ok_or(TimerError::NotActive)?;
        let since = active.running_since.ok_or(TimerError::NotRunning)?;
        active.accumulated_ms += (now - since).num_milliseconds().max(0);
        active.running_since = None;
        Ok(())
    }

    pub fn resume(&mut self, now: DateTime<Utc>) -> Result<(), TimerError> {
        let active = self.active.as_mut().ok_or(TimerError::NotActive)?;
        if active.running_since.is_some() {
            return Err(TimerError::NotPaused);
        }
        active.running_since = Some(now);
        Ok(())
    }

    /// Beendet die Session vorzeitig.
    ///
    /// - Flowtime-Arbeit: gilt als abgeschlossen (`Completed`) und liefert die Pausenempfehlung.
    /// - Stopwatch-Arbeit: `Completed` ohne Pause und Todo-Zähler (ADR 0018).
    /// - Pomodoro-Arbeit: unter 50 % der geplanten Zeit `aborted`, sonst `completed`; beides ohne Pause,
    ///   Zyklus-Zählung und Todo-Zähler (nur ein ablaufender Pomodoro zählt voll).
    /// - Pause: immer `skipped`.
    pub fn stop(&mut self, now: DateTime<Utc>, settings: &Settings) -> Result<Finish, TimerError> {
        let active = self.active.take().ok_or(TimerError::NotActive)?;
        let worked_sec = active.worked_sec(now);
        let actual_min = worked_sec / 60;

        if active.mode == TimerMode::Flowtime && active.session_type == SessionType::Work {
            self.suggestion = None;
            let minutes = calculate_break_duration(
                actual_min,
                settings.flowtime_ratio,
                settings.flowtime_min_flow_state_min,
            );
            let suggestion = (minutes > 0).then_some(BreakSuggestion {
                session_type: SessionType::ShortBreak,
                minutes,
            });
            self.suggestion = suggestion;
            self.last_work = Some(LastWork {
                mode: active.mode,
                todo_id: active.todo_id,
            });
            return Ok(Finish {
                kind: FinishKind::Completed,
                session: finished(&active, SessionStatus::Completed, now, actual_min),
                suggested_break: suggestion,
                increment_todo: active.todo_id,
                auto_start: (suggestion.is_some() && settings.auto_start_breaks)
                    .then_some(AutoStart::Break),
            });
        }

        let status = if active.session_type != SessionType::Work {
            SessionStatus::Skipped
        } else if active.is_countdown() && worked_sec * 2 < active.planned_sec {
            SessionStatus::Aborted
        } else {
            SessionStatus::Completed
        };
        if active.session_type != SessionType::Work {
            self.suggestion = None;
        }
        Ok(Finish {
            kind: FinishKind::Stopped,
            session: finished(&active, status, now, actual_min),
            suggested_break: None,
            increment_todo: None,
            auto_start: None,
        })
    }

    /// Prüft, ob ein laufender Countdown abgelaufen ist, und beendet ihn dann zum rechnerischen Endzeitpunkt.
    /// Regelmäßig aufrufen (Anzeige-Tick) und nach Hintergrund oder Neustart.
    pub fn tick(&mut self, now: DateTime<Utc>, settings: &Settings) -> Option<Finish> {
        if !self.active.as_ref().is_some_and(|a| a.is_due(now)) {
            return None;
        }
        let active = self.active.take()?;
        let ended_at = active.due_at().unwrap_or(now);
        let actual_min = active.planned_sec / 60;
        Some(self.complete(active, ended_at, actual_min, settings))
    }

    /// Momentaufnahme für die Anzeige.
    pub fn snapshot(&self, now: DateTime<Utc>) -> TimerSnapshot {
        let Some(active) = &self.active else {
            return TimerSnapshot {
                state: TimerState::Idle,
                mode: TimerMode::Pomodoro,
                session_type: SessionType::Work,
                remaining: 0,
                total: 0,
                todo_id: None,
            };
        };
        let worked = active.worked_sec(now);
        let (remaining, total) = if active.is_countdown() {
            (
                active.planned_sec.saturating_sub(worked),
                active.planned_sec,
            )
        } else {
            (worked, 0)
        };
        TimerSnapshot {
            state: self.state(),
            mode: active.mode,
            session_type: active.session_type,
            remaining,
            total,
            todo_id: active.todo_id,
        }
    }

    fn ensure_idle(&self) -> Result<(), TimerError> {
        if self.active.is_some() {
            return Err(TimerError::AlreadyActive);
        }
        Ok(())
    }

    fn begin(
        &mut self,
        now: DateTime<Utc>,
        mode: TimerMode,
        session_type: SessionType,
        planned_sec: u32,
        todo_id: Option<i32>,
    ) {
        self.active = Some(ActiveSession {
            mode,
            session_type,
            todo_id,
            planned_sec,
            started_at: now,
            accumulated_ms: 0,
            running_since: Some(now),
        });
    }

    /// Natürliches Ende eines Countdowns.
    fn complete(
        &mut self,
        active: ActiveSession,
        ended_at: DateTime<Utc>,
        actual_min: u32,
        settings: &Settings,
    ) -> Finish {
        let session = finished(&active, SessionStatus::Completed, ended_at, actual_min);

        if active.session_type == SessionType::Work {
            self.last_work = Some(LastWork {
                mode: active.mode,
                todo_id: active.todo_id,
            });
            let suggestion = (active.mode == TimerMode::Pomodoro)
                .then(|| self.next_break(settings))
                .flatten();
            self.suggestion = suggestion;
            return Finish {
                kind: FinishKind::Completed,
                session,
                suggested_break: suggestion,
                increment_todo: active.todo_id,
                auto_start: (suggestion.is_some() && settings.auto_start_breaks)
                    .then_some(AutoStart::Break),
            };
        }

        self.suggestion = None;
        let auto_start = settings.auto_start_next_pomodoro.then(|| {
            let last = self.last_work;
            AutoStart::Work {
                mode: last.map_or(active.mode, |w| w.mode),
                todo_id: last.and_then(|w| w.todo_id),
            }
        });
        Finish {
            kind: FinishKind::Completed,
            session,
            suggested_break: None,
            increment_todo: None,
            auto_start,
        }
    }

    /// Zählt einen abgeschlossenen Pomodoro und liefert die passende Pause (lang nach N Pomodoros).
    fn next_break(&mut self, settings: &Settings) -> Option<BreakSuggestion> {
        self.pomodoros_in_cycle += 1;
        // `long_break_after == 0` heißt: nie eine lange Pause (`is_multiple_of(0)` ist für Zahlen > 0 `false`).
        let long = self
            .pomodoros_in_cycle
            .is_multiple_of(settings.long_break_after);
        let (session_type, minutes) = if long {
            self.pomodoros_in_cycle = 0;
            (SessionType::LongBreak, settings.long_break_min)
        } else {
            (SessionType::ShortBreak, settings.short_break_min)
        };
        (minutes > 0).then_some(BreakSuggestion {
            session_type,
            minutes,
        })
    }
}

fn finished(
    active: &ActiveSession,
    status: SessionStatus,
    ended_at: DateTime<Utc>,
    actual_min: u32,
) -> FinishedSession {
    FinishedSession {
        mode: active.mode,
        session_type: active.session_type,
        status,
        started_at: active.started_at,
        ended_at,
        planned_min: active.is_countdown().then_some(active.planned_sec / 60),
        actual_min,
        todo_id: active.todo_id,
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn due_at_is_the_end_of_a_running_countdown() {
        let now = Utc.with_ymd_and_hms(2026, 10, 3, 9, 0, 0).unwrap();
        let settings = Settings::default();
        let mut timer = Timer::default();
        assert_eq!(timer.due_at(), None);

        timer
            .start(now, TimerMode::Pomodoro, None, &settings)
            .unwrap();
        assert_eq!(timer.due_at(), Some(now + Duration::minutes(25)));

        // nach 10 Minuten pausiert gibt es keinen Endzeitpunkt, nach dem Fortsetzen bleiben 15 Minuten
        timer.pause(now + Duration::minutes(10)).unwrap();
        assert_eq!(timer.due_at(), None);
        let resumed = now + Duration::minutes(30);
        timer.resume(resumed).unwrap();
        assert_eq!(timer.due_at(), Some(resumed + Duration::minutes(15)));
    }

    #[test]
    fn a_custom_duration_replaces_the_setting() {
        let now = Utc.with_ymd_and_hms(2026, 10, 3, 9, 0, 0).unwrap();
        let mut timer = Timer::default();
        timer
            .start_planned(now, TimerMode::Pomodoro, Some(2), 3)
            .unwrap();
        assert_eq!(timer.due_at(), Some(now + Duration::seconds(3)));
        assert_eq!(timer.snapshot(now).total, 3);
        assert_eq!(
            timer.start_planned(now, TimerMode::Pomodoro, None, 3),
            Err(TimerError::AlreadyActive)
        );
        let finish = timer
            .tick(now + Duration::seconds(3), &Settings::default())
            .expect("abgelaufen");
        assert_eq!(finish.kind, FinishKind::Completed);
        assert_eq!(finish.increment_todo, Some(2));
    }

    #[test]
    fn a_stopwatch_has_no_end() {
        let now = Utc.with_ymd_and_hms(2026, 10, 3, 9, 0, 0).unwrap();
        let mut timer = Timer::default();
        timer
            .start(now, TimerMode::Flowtime, None, &Settings::default())
            .unwrap();
        assert_eq!(timer.due_at(), None);
    }

    #[test]
    fn a_stopped_stopwatch_is_completed_without_break_or_todo_count() {
        let now = Utc.with_ymd_and_hms(2026, 10, 3, 9, 0, 0).unwrap();
        let mut timer = Timer::default();
        let settings = Settings::default();
        timer
            .start(now, TimerMode::Stopwatch, Some(4), &settings)
            .unwrap();
        let finish = timer
            .stop(now + chrono::Duration::minutes(90), &settings)
            .unwrap();
        assert_eq!(finish.kind, FinishKind::Stopped);
        assert_eq!(finish.session.status, SessionStatus::Completed);
        assert_eq!(finish.session.mode, TimerMode::Stopwatch);
        assert_eq!(finish.session.actual_min, 90);
        assert_eq!(finish.suggested_break, None);
        assert_eq!(finish.increment_todo, None);
        assert_eq!(finish.auto_start, None);
        assert_eq!(timer.suggested_break(), None);
    }

    #[test]
    fn exposes_the_active_session() {
        let now = Utc.with_ymd_and_hms(2026, 10, 3, 9, 0, 0).unwrap();
        let mut timer = Timer::default();
        assert!(timer.active().is_none());
        timer
            .start(now, TimerMode::Pomodoro, Some(4), &Settings::default())
            .unwrap();
        let active = timer.active().expect("aktive Session");
        assert_eq!(active.todo_id, Some(4));
        assert_eq!(active.planned_sec, 25 * 60);
        assert_eq!(active.started_at, now);
        assert_eq!(timer.state(), TimerState::Running);
    }
}
