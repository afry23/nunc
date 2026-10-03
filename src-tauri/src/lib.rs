mod commands;

use tauri_specta::{Builder, collect_commands};

/// Pfad der generierten TypeScript-Bindings (relativ zum Crate, nicht zum CWD).
const BINDINGS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/bindings.ts");

fn specta_builder() -> Builder<tauri::Wry> {
    use nunc_core::model::{Session, Settings, StatsRequest, StatsResponse, TimerState, Todo};

    Builder::<tauri::Wry>::new()
        .commands(collect_commands![commands::app_info])
        // Domänentypen schon jetzt exportieren; Commands (M3) referenzieren sie später ohnehin.
        .typ::<Todo>()
        .typ::<Session>()
        .typ::<Settings>()
        .typ::<StatsRequest>()
        .typ::<StatsResponse>()
        .typ::<TimerState>()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    // Nur auf dem Entwicklungsrechner: auf Mobilgeräten existiert der Host-Pfad nicht.
    #[cfg(all(debug_assertions, desktop))]
    export_bindings(&builder);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(any(desktop, test))]
fn export_bindings(builder: &Builder<tauri::Wry>) {
    builder
        .export(specta_typescript::Typescript::default(), BINDINGS_PATH)
        .expect("Failed to export typescript bindings");
}

#[cfg(test)]
mod tests {
    /// Erzeugt `src/bindings.ts` ohne die App zu starten: `cargo test -p nunc export_bindings`.
    #[test]
    #[cfg(any(desktop, test))]
    fn export_bindings() {
        super::export_bindings(&super::specta_builder());
    }
}
