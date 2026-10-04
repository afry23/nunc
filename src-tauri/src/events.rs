//! Ereignisse vom Backend zur Oberfläche (typisiert über tauri-specta, Namen wie in der Go-App).
//!
//! Die Dienste senden über den [`EventSink`]; die Typen hier legen Namen und Nutzlast fest und landen in
//! `src/bindings.ts`.

use nunc_core::model::Session;
use nunc_core::timer::{BreakSuggestion, TimerSnapshot};
use nunc_platform::EventSink;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

/// Jede Sekunde, solange der Timer im Vordergrund läuft, und bei jeder Zustandsänderung.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "timer:tick")]
pub struct TimerTick(pub TimerSnapshot);

/// Eine beendete Session samt Folgeaktion.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FinishPayload {
    pub session: Session,
    /// Empfohlene Pause nach abgeschlossener Arbeit.
    pub suggested_break: Option<BreakSuggestion>,
}

/// Session natürlich abgelaufen (oder Flowtime-Arbeit beendet).
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "timer:complete")]
pub struct TimerComplete(pub FinishPayload);

/// Session vom Nutzer vorzeitig beendet.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "timer:stopped")]
pub struct TimerStopped(pub FinishPayload);

/// Datenbank wurde ausgetauscht (Restore): alle Stores neu laden.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "app:reload")]
pub struct AppReload;

/// Sendet ein typisiertes Ereignis über die Senke.
pub fn emit<E: Event + Serialize>(sink: &dyn EventSink, event: &E) {
    match serde_json::to_value(event) {
        Ok(payload) => sink.emit(E::NAME, payload),
        Err(error) => log::error!("Ereignis {} nicht serialisierbar: {error}", E::NAME),
    }
}

#[cfg(test)]
mod tests {
    use nunc_core::model::SessionType;
    use nunc_core::model::{TimerMode, TimerState};
    use nunc_platform::RecordingEventSink;

    use super::*;

    #[test]
    fn event_names_match_the_go_app() {
        assert_eq!(TimerTick::NAME, "timer:tick");
        assert_eq!(TimerComplete::NAME, "timer:complete");
        assert_eq!(TimerStopped::NAME, "timer:stopped");
        assert_eq!(AppReload::NAME, "app:reload");
    }

    #[test]
    fn emit_sends_name_and_payload() {
        let sink = RecordingEventSink::default();
        emit(
            &sink,
            &TimerTick(TimerSnapshot {
                state: TimerState::Running,
                mode: TimerMode::Pomodoro,
                session_type: SessionType::Work,
                remaining: 90,
                total: 1500,
                todo_id: Some(3),
            }),
        );
        let events = sink.events();
        assert_eq!(events[0].0, "timer:tick");
        assert_eq!(events[0].1["remaining"], 90);
        assert_eq!(events[0].1["sessionType"], "work");
        assert_eq!(events[0].1["todoId"], 3);
    }
}
