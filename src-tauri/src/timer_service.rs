//! Timer-Laufzeit: verbindet den reinen Zustandsautomaten (`nunc-core::timer`) mit Datenbank, Ereignissen und
//! Benachrichtigungen. Keine Fachregeln – die stecken im `Timer`; hier wird nur ausgeführt, was er als Daten
//! zurückgibt (`Finish`).
//!
//! Jede Transition wird persistiert (ADR 0004). Das Ende einer Session (Session anfügen, Pomodoro-Zähler, neuer
//! Timer-Zustand) ist **eine** Transaktion. Fehlschläge rollen den Zustand im Speicher zurück.

use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use nunc_core::clock::Clock;
use nunc_core::model::{Labels, SessionType, Settings, TimerMode, TimerState};
use nunc_core::timer::{
    AutoStart, BreakSuggestion, Finish, FinishKind, Timer, TimerError, TimerSnapshot,
};
use nunc_platform::{EventSink, Notifier};
use nunc_store::{commit_finish, load_settings, load_timer, save_timer};

use crate::db::Db;
use crate::error::AppError;
use crate::events::{self, FinishPayload, TimerComplete, TimerStopped, TimerTick};

/// Kennung der geplanten Benachrichtigung zum Timer-Ende (es gibt höchstens eine).
pub const NOTIFICATION_ID: i32 = 1;

pub struct TimerService {
    db: Arc<Db>,
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventSink>,
    notifier: Arc<dyn Notifier>,
    device_id: String,
    timer: Mutex<Timer>,
    /// Wird aufgerufen, nachdem eine Session gespeichert wurde (löst den Sync aus).
    on_session_saved: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Feste Pomodoro-Dauer in Sekunden für Oberflächentests (`NUNC_TEST_FAST_TIMER`), nur in Debug-Builds.
    fast_pomodoro_sec: Option<u32>,
}

impl TimerService {
    /// Lädt den gespeicherten Timer. Aufräumen nach einem Neustart: [`TimerService::recover`].
    pub fn new(
        db: Arc<Db>,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventSink>,
        notifier: Arc<dyn Notifier>,
        device_id: String,
    ) -> Result<Self, AppError> {
        let timer = db.with(|conn| load_timer(conn))?;
        Ok(Self {
            db,
            clock,
            events,
            notifier,
            device_id,
            timer: Mutex::new(timer),
            on_session_saved: None,
            fast_pomodoro_sec: None,
        })
    }

    /// Meldet `hook` an, der nach jeder gespeicherten Session läuft (beendet oder abgebrochen). Er darf nicht
    /// blockieren: der Timer ist währenddessen gesperrt.
    pub fn with_session_hook(mut self, hook: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_session_saved = Some(hook);
        self
    }

    /// Setzt die Pomodoro-Dauer fest (Sekunden). Wirkt nur in Debug-Builds; Release-Builds ignorieren sie.
    pub fn with_fast_pomodoro(mut self, seconds: Option<u32>) -> Self {
        self.fast_pomodoro_sec = seconds.filter(|_| cfg!(debug_assertions));
        self
    }

    fn lock(&self) -> Result<MutexGuard<'_, Timer>, AppError> {
        Ok(self.timer.lock()?)
    }

    fn settings(&self) -> Result<Settings, AppError> {
        self.db.with(|conn| load_settings(conn))
    }

    /// Nach dem Start oder der Rückkehr aus dem Hintergrund: ein inzwischen abgelaufener Countdown wird zum
    /// rechnerischen Ende verbucht (ohne Sofort-Benachrichtigung – der Nutzer hat die Zeit nicht live erlebt),
    /// ein laufender bekommt seine Benachrichtigung neu. Sendet immer einen `timer:tick`.
    pub fn recover(&self) -> Result<TimerSnapshot, AppError> {
        let mut timer = self.lock()?;
        let now = self.clock.now();
        let settings = self.settings()?;
        self.settle(&mut timer, now, &settings, false)?;
        self.sync_notification(&timer, &settings);
        let snapshot = timer.snapshot(now);
        events::emit(&*self.events, &TimerTick(snapshot.clone()));
        Ok(snapshot)
    }

    /// Aktueller Stand (verbucht vorher einen inzwischen abgelaufenen Countdown), z. B. beim Zurückkehren aus
    /// dem Hintergrund oder beim Öffnen der Oberfläche.
    pub fn snapshot(&self) -> Result<TimerSnapshot, AppError> {
        let mut timer = self.lock()?;
        let now = self.clock.now();
        let settings = self.settings()?;
        if self.settle(&mut timer, now, &settings, true)? {
            self.sync_notification(&timer, &settings);
            events::emit(&*self.events, &TimerTick(timer.snapshot(now)));
        }
        Ok(timer.snapshot(now))
    }

    /// Sekundentakt der Anzeige: verbucht ein Ende und sendet `timer:tick`, solange der Timer läuft.
    pub fn tick(&self) -> Result<(), AppError> {
        let mut timer = self.lock()?;
        let now = self.clock.now();
        let settings = self.settings()?;
        let concluded = self.settle(&mut timer, now, &settings, true)?;
        if concluded {
            self.sync_notification(&timer, &settings);
        }
        let snapshot = timer.snapshot(now);
        if concluded || snapshot.state == TimerState::Running {
            events::emit(&*self.events, &TimerTick(snapshot));
        }
        Ok(())
    }

    pub fn suggested_break(&self) -> Result<Option<BreakSuggestion>, AppError> {
        Ok(self.lock()?.suggested_break())
    }

    pub fn start(&self, mode: TimerMode, todo_id: Option<i32>) -> Result<TimerSnapshot, AppError> {
        self.start_labelled(mode, todo_id, &Labels::default())
    }

    /// Wie [`TimerService::start`], mit eigenen Projekten, Kontexten und Notiz für eine Session ohne Todo.
    pub fn start_labelled(
        &self,
        mode: TimerMode,
        todo_id: Option<i32>,
        labels: &Labels,
    ) -> Result<TimerSnapshot, AppError> {
        let fast = self
            .fast_pomodoro_sec
            .filter(|_| mode == TimerMode::Pomodoro);
        self.transition(|timer, now, settings| {
            match fast {
                Some(seconds) => timer.start_planned(now, mode, todo_id, seconds),
                None => timer.start(now, mode, todo_id, settings),
            }?;
            timer.set_labels(labels);
            Ok(())
        })
    }

    pub fn start_break(&self) -> Result<TimerSnapshot, AppError> {
        self.transition(|timer, now, _| timer.start_break(now))
    }

    pub fn pause(&self) -> Result<TimerSnapshot, AppError> {
        self.transition(|timer, now, _| timer.pause(now))
    }

    pub fn resume(&self) -> Result<TimerSnapshot, AppError> {
        self.transition(|timer, now, _| timer.resume(now))
    }

    /// Beendet die Session vorzeitig und verbucht sie.
    pub fn stop(&self) -> Result<TimerSnapshot, AppError> {
        let mut timer = self.lock()?;
        let now = self.clock.now();
        let settings = self.settings()?;
        self.settle(&mut timer, now, &settings, true)?;
        let before = timer.clone();
        let result = timer
            .stop(now, &settings)
            .map_err(AppError::from)
            .and_then(|finish| self.conclude(&mut timer, finish, now, &settings, false));
        if let Err(error) = result {
            *timer = before;
            return Err(error);
        }
        self.sync_notification(&timer, &settings);
        let snapshot = timer.snapshot(now);
        events::emit(&*self.events, &TimerTick(snapshot.clone()));
        Ok(snapshot)
    }

    /// Führt `action` aus, solange der Timer gesperrt ist (Datenbank-Austausch beim Restore), und lädt den
    /// Timer danach aus der neuen Datenbank.
    pub fn exclusive<T>(
        &self,
        action: impl FnOnce() -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let mut timer = self.lock()?;
        let result = action();
        // auch nach einem Fehler neu laden: `restore_backup` kann die Verbindung ersetzt haben
        *timer = self.db.with(|conn| load_timer(conn)).unwrap_or_default();
        let settings = self.settings().unwrap_or_default();
        self.sync_notification(&timer, &settings);
        result
    }

    /// Eine Transition des Automaten; zuvor wird ein abgelaufener Countdown verbucht.
    fn transition(
        &self,
        action: impl FnOnce(&mut Timer, DateTime<Utc>, &Settings) -> Result<(), TimerError>,
    ) -> Result<TimerSnapshot, AppError> {
        let mut timer = self.lock()?;
        let now = self.clock.now();
        let settings = self.settings()?;
        self.settle(&mut timer, now, &settings, true)?;
        let before = timer.clone();
        let persisted = action(&mut timer, now, &settings)
            .map_err(AppError::from)
            .and_then(|()| self.db.with(|conn| save_timer(conn, &timer, now)));
        if let Err(error) = persisted {
            *timer = before;
            return Err(error);
        }
        self.sync_notification(&timer, &settings);
        let snapshot = timer.snapshot(now);
        events::emit(&*self.events, &TimerTick(snapshot.clone()));
        Ok(snapshot)
    }

    /// Verbucht einen abgelaufenen Countdown. `true`, wenn eine Session endete.
    fn settle(
        &self,
        timer: &mut Timer,
        now: DateTime<Utc>,
        settings: &Settings,
        notify: bool,
    ) -> Result<bool, AppError> {
        let before = timer.clone();
        let Some(finish) = timer.tick(now, settings) else {
            return Ok(false);
        };
        if let Err(error) = self.conclude(timer, finish, now, settings, notify) {
            *timer = before;
            return Err(error);
        }
        Ok(true)
    }

    /// Setzt das Ende einer Session um: eine Transaktion (Session, Zähler, Timer-Zustand), Ereignis,
    /// Benachrichtigung, danach der automatische Start der Folge-Session. `timer` ist bereits im Zustand nach
    /// dem Ende.
    fn conclude(
        &self,
        timer: &mut Timer,
        finish: Finish,
        now: DateTime<Utc>,
        settings: &Settings,
        notify: bool,
    ) -> Result<(), AppError> {
        let session = self
            .db
            .with(|conn| commit_finish(conn, &finish, &self.device_id, timer, now))?;
        if let Some(hook) = &self.on_session_saved {
            hook();
        }
        let payload = FinishPayload {
            session,
            suggested_break: finish.suggested_break,
        };
        match finish.kind {
            FinishKind::Completed => events::emit(&*self.events, &TimerComplete(payload)),
            FinishKind::Stopped => events::emit(&*self.events, &TimerStopped(payload)),
        }

        // Ohne geplante Benachrichtigung (Desktop) meldet der Prozess das Ende selbst.
        if notify
            && finish.kind == FinishKind::Completed
            && settings.notifications_enabled
            && !self.notifier.can_schedule()
        {
            let (title, body) = end_message(finish.session.session_type);
            if let Err(error) = self.notifier.notify_now(title, body) {
                log::warn!("{error}");
            }
        }

        if let Some(auto) = finish.auto_start {
            let settled = timer.clone();
            let started = match auto {
                AutoStart::Break => timer.start_break(now),
                AutoStart::Work { mode, todo_id } => timer.start(now, mode, todo_id, settings),
            };
            let saved = started
                .map_err(AppError::from)
                .and_then(|()| self.db.with(|conn| save_timer(conn, timer, now)));
            if let Err(error) = saved {
                // Das Ende ist verbucht; nur der Folgestart entfällt.
                log::warn!("automatischer Start fehlgeschlagen: {error}");
                *timer = settled;
            }
        }
        Ok(())
    }

    /// Hält die geplante Benachrichtigung zum Endzeitpunkt des laufenden Countdowns aktuell.
    fn sync_notification(&self, timer: &Timer, settings: &Settings) {
        let result = match timer.due_at() {
            Some(at) if settings.notifications_enabled && self.notifier.can_schedule() => {
                let session_type = timer
                    .active()
                    .map_or(SessionType::Work, |active| active.session_type);
                let (title, body) = end_message(session_type);
                self.notifier.schedule(NOTIFICATION_ID, at, title, body)
            }
            _ => self.notifier.cancel(NOTIFICATION_ID),
        };
        // Ohne Erlaubnis bleibt der Timer korrekt, nur ohne Hinweis im Hintergrund (ADR 0004).
        if let Err(error) = result {
            log::warn!("{error}");
        }
    }
}

/// Titel und Text der Benachrichtigung zum Ende einer Session.
fn end_message(session_type: SessionType) -> (&'static str, &'static str) {
    match session_type {
        SessionType::Work => ("Pomodoro geschafft", "Zeit für eine Pause."),
        SessionType::ShortBreak | SessionType::LongBreak => {
            ("Pause vorbei", "Weiter geht's mit der nächsten Runde.")
        }
    }
}

#[cfg(test)]
mod tests;
