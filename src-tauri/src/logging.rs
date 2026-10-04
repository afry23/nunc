//! Protokoll der App (`tauri-plugin-log`): Datei im Log-Ordner des Systems mit Rotation, in Debug-Builds
//! zusätzlich Stdout (nicht auf iOS, siehe `Cargo.toml`). Tokens und Passwörter gehören nie in eine Meldung (siehe CLAUDE.md „Do Not“).

use std::sync::OnceLock;
use std::time::Instant;

#[cfg(not(target_os = "ios"))]
use log::LevelFilter;
#[cfg(not(target_os = "ios"))]
use tauri::Runtime;
#[cfg(not(target_os = "ios"))]
use tauri::plugin::TauriPlugin;
#[cfg(not(target_os = "ios"))]
use tauri_plugin_log::{Builder, RotationStrategy, Target, TargetKind};

#[cfg(not(target_os = "ios"))]
/// Dateiname ohne Endung: `nunc.log`, rotierte Dateien tragen das Datum.
const LOG_FILE: &str = "nunc";
#[cfg(not(target_os = "ios"))]
/// Eine Datei wird ab dieser Größe rotiert.
const MAX_FILE_BYTES: u128 = 1_000_000;
#[cfg(not(target_os = "ios"))]
/// So viele rotierte Dateien bleiben liegen, ältere werden gelöscht.
const KEPT_FILES: usize = 5;

#[cfg(not(target_os = "ios"))]
/// Mindeststufe: Debug-Builds protokollieren alles, Release-Builds ab `Info`.
pub fn default_level(debug: bool) -> LevelFilter {
    if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}

static STARTED: OnceLock<Instant> = OnceLock::new();

/// Merkt den Startzeitpunkt der App (erster Aufruf zählt).
pub fn mark_start() {
    STARTED.get_or_init(Instant::now);
}

/// Protokolliert einen Meilenstein des Starts mit der Zeit seit [`mark_start`]. Das Skript
/// `tests/nfr/measure-desktop.sh` liest diese Zeilen (NFR „Start < 2 s“, roadmap M8-01).
pub fn log_startup(milestone: &str) {
    if let Some(started) = STARTED.get() {
        log::info!(
            "Start: {milestone} nach {} ms",
            started.elapsed().as_millis()
        );
    }
}

#[cfg(not(target_os = "ios"))]
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    let debug = cfg!(debug_assertions);
    let mut targets = vec![Target::new(TargetKind::LogDir {
        file_name: Some(LOG_FILE.to_owned()),
    })];
    if debug {
        targets.push(Target::new(TargetKind::Stdout));
    }
    Builder::new()
        .clear_targets()
        .targets(targets)
        .level(default_level(debug))
        // HTTP-Bibliotheken protokollieren Verbindungsdetails, die niemand braucht
        .level_for("hyper", LevelFilter::Warn)
        .level_for("hyper_util", LevelFilter::Warn)
        .level_for("reqwest", LevelFilter::Warn)
        .level_for("tao", LevelFilter::Warn)
        .rotation_strategy(RotationStrategy::KeepSome(KEPT_FILES))
        .max_file_size(MAX_FILE_BYTES)
        .build()
}

#[cfg(all(test, not(target_os = "ios")))]
mod tests {
    use super::*;

    #[test]
    fn release_builds_skip_debug_output() {
        assert_eq!(default_level(false), LevelFilter::Info);
        assert_eq!(default_level(true), LevelFilter::Debug);
    }
}
