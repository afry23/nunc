mod commands;

use tauri_specta::{Builder, collect_commands};

/// Pfad der generierten TypeScript-Bindings (relativ zum Crate, nicht zum CWD).
const BINDINGS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/bindings.ts");

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![commands::app_info])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    // Nur auf dem Entwicklungsrechner: auf Mobilgeräten existiert der Host-Pfad nicht.
    #[cfg(all(debug_assertions, desktop))]
    export_bindings(&builder);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
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
