//! Domain-Kern: pure Fachlogik ohne I/O.
//!
//! Keine Systemzeit, kein Dateisystem, kein Netz: Zeit wird als Parameter oder über eine injizierte
//! `Clock` hereingereicht (Feature `clock` von `chrono` ist hier bewusst nicht aktiviert).

pub mod backup_policy;
pub mod clock;
pub mod crdt;
pub mod export;
pub mod flowtime;
pub mod hlc;
pub mod manual_entry;
pub mod model;
pub mod position;
pub mod stats;
pub mod sync_document;
pub mod timeline;
pub mod timer;
pub mod timesheet;
pub mod todo_query;
pub mod todotxt;
pub mod todotxt_file;
