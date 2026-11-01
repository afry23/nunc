//! Tauri-Commands: dünne Schicht – Argumente prüfen, an Store/Core delegieren, Ergebnis abbilden.
//! Neue Commands in `specta_builder()` (`lib.rs`) eintragen.

pub mod api;
pub mod app;
pub mod backup;
pub mod notifications;
pub mod review;
pub mod sessions;
pub mod settings;
pub mod stats;
pub mod sync;
pub mod timer;
pub mod todos;
pub mod todotxt;

#[cfg(test)]
mod tests;
