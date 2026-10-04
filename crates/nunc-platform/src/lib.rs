//! Plattform-Traits: Clock, EventSink, Notifier, SecretStore, UrlOpener
//!
//! Die Traits halten Dienste (Timer-Laufzeit, Sync) von Tauri unabhängig und damit mit Fakes testbar. Die
//! Tauri-Adapter (`TauriEventSink`, `TauriNotifier`) liegen in `src-tauri`, weil sie `AppHandle` brauchen.

mod clock;
mod events;
mod notify;
mod opener;
mod secret;

pub use clock::{ManualClock, SystemClock, to_local, to_local_date};
pub use events::{EventSink, RecordingEventSink};
pub use notify::{Notifier, NotifyError, RecordingNotifier, Reminder};
pub use opener::{OpenError, RecordingUrlOpener, UrlOpener};
pub use secret::{
    KeyringSecretStore, MemorySecretStore, SecretError, SecretStore, UnavailableSecretStore,
};
