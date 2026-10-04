//! Tauri-Adapter für die Traits aus `nunc-platform`.

use chrono::{DateTime, Utc};
use nunc_platform::{EventSink, Notifier, NotifyError};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_notification::NotificationExt;

/// Sendet Ereignisse an alle Fenster.
pub struct TauriEventSink<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> EventSink for TauriEventSink<R> {
    fn emit(&self, event: &str, payload: Value) {
        // Ohne offenes Fenster hört niemand zu; das ist kein Fehler des Senders.
        if let Err(error) = self.0.emit(event, payload) {
            log::debug!("Ereignis {event} nicht zugestellt: {error}");
        }
    }
}

/// Benachrichtigungen über `tauri-plugin-notification`.
///
/// Mobil wird das Timer-Ende als geplante Benachrichtigung hinterlegt (ADR 0004), auf dem Desktop gibt es keine
/// Planung im Plugin: dort zeigt der laufende Prozess die Benachrichtigung beim Ablauf selbst.
pub struct TauriNotifier<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> Notifier for TauriNotifier<R> {
    fn can_schedule(&self) -> bool {
        cfg!(mobile)
    }

    #[cfg(mobile)]
    fn schedule(
        &self,
        id: i32,
        at: DateTime<Utc>,
        title: &str,
        body: &str,
    ) -> Result<(), NotifyError> {
        use tauri_plugin_notification::Schedule;

        let date = time::OffsetDateTime::from_unix_timestamp(plugin_timestamp(at))
            .map_err(|error| NotifyError(error.to_string()))?;
        self.0
            .notification()
            .builder()
            .id(id)
            .title(title)
            .body(body)
            .schedule(Schedule::At {
                date,
                repeating: false,
                allow_while_idle: true,
            })
            .show()
            .map_err(|error| NotifyError(error.to_string()))
    }

    #[cfg(not(mobile))]
    fn schedule(&self, _: i32, _: DateTime<Utc>, _: &str, _: &str) -> Result<(), NotifyError> {
        Ok(())
    }

    #[cfg(mobile)]
    fn cancel(&self, id: i32) -> Result<(), NotifyError> {
        self.0
            .notification()
            .cancel(vec![id])
            .map_err(|error| NotifyError(error.to_string()))
    }

    #[cfg(not(mobile))]
    fn cancel(&self, _: i32) -> Result<(), NotifyError> {
        Ok(())
    }

    fn notify_now(&self, title: &str, body: &str) -> Result<(), NotifyError> {
        self.0
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
            .map_err(|error| NotifyError(error.to_string()))
    }
}

/// Zeitstempel für `Schedule::At`. Das iOS-Plugin (2.5.1) liest das UTC-Datum als Ortszeit und lehnt es in
/// Zeitzonen östlich von UTC still ab (ADR 0004); dort wird der lokale UTC-Offset aufgeschlagen. Bei einem
/// Zeitumstellungswechsel zwischen jetzt und dem Termin ist das um eine Stunde ungenau (offen: Plugin patchen).
#[cfg(mobile)]
fn plugin_timestamp(at: DateTime<Utc>) -> i64 {
    #[cfg(target_os = "ios")]
    {
        use chrono::{Local, Offset, TimeZone};
        let offset = Local.offset_from_utc_datetime(&at.naive_utc()).fix();
        at.timestamp() + i64::from(offset.local_minus_utc())
    }
    #[cfg(not(target_os = "ios"))]
    {
        at.timestamp()
    }
}
