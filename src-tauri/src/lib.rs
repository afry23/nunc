mod adapters;
mod commands;
mod db;
mod error;
mod events;
mod lifecycle;
mod state;
mod timer_service;
#[cfg(desktop)]
mod tray;
#[cfg(desktop)]
mod windows;

use tauri_specta::{Builder, collect_commands, collect_events};

/// Pfad der generierten TypeScript-Bindings (relativ zum Crate, nicht zum CWD).
#[cfg(any(desktop, test))]
const BINDINGS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/bindings.ts");

fn specta_builder() -> Builder<tauri::Wry> {
    use commands::{app, backup, notifications, settings, stats, sync, timer, todos};

    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            app::app_info,
            app::show_main_window::<tauri::Wry>,
            settings::get_settings,
            settings::save_settings::<tauri::Wry>,
            todos::get_todos,
            todos::add_todo,
            todos::update_todo,
            todos::delete_todo,
            todos::set_todo_completed,
            todos::reorder_todos,
            timer::start_timer,
            timer::start_break,
            timer::pause_timer,
            timer::resume_timer,
            timer::stop_timer,
            timer::get_timer_tick,
            timer::get_suggested_break,
            stats::get_stats,
            stats::export_stats,
            stats::save_export_to_file::<tauri::Wry>,
            backup::get_backup_status,
            backup::trigger_backup,
            backup::list_backups,
            backup::restore_from_backup,
            backup::select_backup_dir::<tauri::Wry>,
            backup::select_restore_file::<tauri::Wry>,
            backup::export_backup::<tauri::Wry>,
            sync::set_sync_provider,
            notifications::get_notification_status::<tauri::Wry>,
            notifications::request_notification_permission::<tauri::Wry>,
        ])
        .events(collect_events![
            events::TimerTick,
            events::TimerComplete,
            events::TimerStopped,
            events::AppReload,
        ])
        // Dateigrößen liegen weit unter 2^53; `u64` darf daher als `number` exportiert werden.
        .dangerously_cast_bigints_to_number()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    // Nur auf dem Entwicklungsrechner: auf Mobilgeräten existiert der Host-Pfad nicht.
    #[cfg(all(debug_assertions, desktop))]
    export_bindings(&builder);

    let builder_app = tauri::Builder::default();
    // Muss das erste Plugin sein: ein zweiter Start holt das vorhandene Fenster nach vorn und endet.
    #[cfg(desktop)]
    let builder_app = builder_app.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        windows::show_main(app);
    }));
    #[cfg(desktop)]
    let builder_app = builder_app.on_window_event(windows::on_window_event);

    let app = builder_app
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            lifecycle::setup(app)?;
            #[cfg(desktop)]
            tray::setup(app.handle(), lifecycle::tray_enabled(app.handle()))?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(lifecycle::on_run_event);
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
