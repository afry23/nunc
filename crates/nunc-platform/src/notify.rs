//! Lokale Benachrichtigungen.

use std::sync::Mutex;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Benachrichtigung fehlgeschlagen: {0}")]
pub struct NotifyError(pub String);

/// Benachrichtigungen. Mobil wird das Ende eines Timers als geplante Benachrichtigung hinterlegt, weil die App im
/// Hintergrund angehalten wird (ADR 0004); auf dem Desktop läuft der Prozess weiter und zeigt sie beim Ablauf.
pub trait Notifier: Send + Sync {
    /// `true`, wenn das Betriebssystem Benachrichtigungen zu einem Zeitpunkt auslösen kann ([`Notifier::schedule`]).
    fn can_schedule(&self) -> bool;
    /// Hinterlegt eine Benachrichtigung zum Zeitpunkt `at`; gleiche `id` ersetzt die vorige.
    fn schedule(
        &self,
        id: i32,
        at: DateTime<Utc>,
        title: &str,
        body: &str,
    ) -> Result<(), NotifyError>;
    /// Entfernt eine geplante Benachrichtigung (kein Fehler, wenn es keine gibt).
    fn cancel(&self, id: i32) -> Result<(), NotifyError>;
    /// Zeigt sofort eine Benachrichtigung.
    fn notify_now(&self, title: &str, body: &str) -> Result<(), NotifyError>;
}

/// Geplante oder sofort gezeigte Benachrichtigung (für Tests).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reminder {
    Scheduled {
        id: i32,
        at: DateTime<Utc>,
        title: String,
        body: String,
    },
    Cancelled(i32),
    Shown {
        title: String,
        body: String,
    },
}

/// Merkt sich alle Aufrufe in Reihenfolge (für Tests).
#[derive(Debug)]
pub struct RecordingNotifier {
    can_schedule: bool,
    calls: Mutex<Vec<Reminder>>,
}

impl RecordingNotifier {
    pub fn new(can_schedule: bool) -> Self {
        Self {
            can_schedule,
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<Reminder> {
        self.calls.lock().map(|c| c.clone()).unwrap_or_default()
    }

    fn record(&self, call: Reminder) {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(call);
        }
    }
}

impl Notifier for RecordingNotifier {
    fn can_schedule(&self) -> bool {
        self.can_schedule
    }

    fn schedule(
        &self,
        id: i32,
        at: DateTime<Utc>,
        title: &str,
        body: &str,
    ) -> Result<(), NotifyError> {
        self.record(Reminder::Scheduled {
            id,
            at,
            title: title.to_owned(),
            body: body.to_owned(),
        });
        Ok(())
    }

    fn cancel(&self, id: i32) -> Result<(), NotifyError> {
        self.record(Reminder::Cancelled(id));
        Ok(())
    }

    fn notify_now(&self, title: &str, body: &str) -> Result<(), NotifyError> {
        self.record(Reminder::Shown {
            title: title.to_owned(),
            body: body.to_owned(),
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_calls_in_order() {
        let notifier = RecordingNotifier::new(true);
        assert!(notifier.can_schedule());
        notifier.cancel(1).unwrap();
        notifier.notify_now("t", "b").unwrap();
        assert_eq!(
            notifier.calls(),
            [
                Reminder::Cancelled(1),
                Reminder::Shown {
                    title: "t".into(),
                    body: "b".into()
                }
            ]
        );
    }
}
