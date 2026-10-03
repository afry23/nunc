//! Domain-Kern: pure Fachlogik ohne I/O.
//!
//! Keine Systemzeit, kein Dateisystem, kein Netz: Zeit wird als Parameter oder über eine injizierte
//! `Clock` hereingereicht (Feature `clock` von `chrono` ist hier bewusst nicht aktiviert).

pub mod clock;
pub mod flowtime;
pub mod model;
pub mod timer;
pub mod todo_query;
pub mod todotxt;
