//! Tauri-Commands: dünne Schicht – Argumente prüfen, an Store/Core delegieren, Ergebnis abbilden.
//! Neue Commands in `specta_builder()` (`lib.rs`) eintragen.

pub mod app;
pub mod backup;
pub mod settings;
pub mod stats;
pub mod timer;
pub mod todos;

#[cfg(test)]
mod tests;
