//! Ereignisse vom Backend zur Oberfläche.

use std::sync::Mutex;

use serde_json::Value;

/// Senke für Backend-Ereignisse (`timer:tick`, `app:reload`, …). Namen und Nutzlasten sind die Schnittstelle zum
/// Frontend; Fehler beim Senden (kein Fenster offen) sind kein Fehler des Senders.
pub trait EventSink: Send + Sync {
    fn emit(&self, event: &str, payload: Value);
}

/// Merkt sich alle Ereignisse in Reihenfolge (für Tests).
#[derive(Debug, Default)]
pub struct RecordingEventSink {
    events: Mutex<Vec<(String, Value)>>,
}

impl RecordingEventSink {
    /// Alle bisher gesendeten Ereignisse.
    pub fn events(&self) -> Vec<(String, Value)> {
        self.events.lock().map(|e| e.clone()).unwrap_or_default()
    }

    /// Nur die Namen, in Sendereihenfolge.
    pub fn names(&self) -> Vec<String> {
        self.events().into_iter().map(|(name, _)| name).collect()
    }
}

impl EventSink for RecordingEventSink {
    fn emit(&self, event: &str, payload: Value) {
        if let Ok(mut events) = self.events.lock() {
            events.push((event.to_owned(), payload));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_in_order() {
        let sink = RecordingEventSink::default();
        sink.emit("a", Value::Null);
        sink.emit("b", Value::from(1));
        assert_eq!(sink.names(), ["a", "b"]);
        assert_eq!(sink.events()[1].1, Value::from(1));
    }
}
