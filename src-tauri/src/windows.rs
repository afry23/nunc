//! Fenster (nur Desktop): Hauptfenster in den Tray schließen, Mini-Timer (always-on-top), Fokus bei zweitem Start.

use std::time::Duration;

use nunc_core::model::TimerState;
use tauri::{
    AppHandle, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window,
    WindowEvent,
};

use crate::state::AppState;

pub const MAIN: &str = "main";
pub const MINI: &str = "mini";

/// Route der Oberfläche für den Mini-Timer (M4-10).
const MINI_ROUTE: &str = "mini";

fn main_window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window(MAIN)
}

/// Hauptfenster zeigen, aus dem Minimieren holen und fokussieren; der Mini-Timer verschwindet dabei.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(mini) = app.get_webview_window(MINI) {
        let _ = mini.hide();
    }
    if let Some(window) = main_window(app) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn hide_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = main_window(app) {
        let _ = window.hide();
    }
}

/// Läuft oder pausiert ein Timer? Nur dann lohnt der Mini-Timer.
fn timer_active<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.try_state::<AppState>()
        .and_then(|state| state.timer.snapshot().ok())
        .is_some_and(|snapshot| wants_mini(snapshot.state))
}

fn wants_mini(state: TimerState) -> bool {
    state != TimerState::Idle
}

/// Hauptfenster in den Tray verbergen (Schließen, Tray-Menü). Läuft ein Timer, erscheint der Mini-Timer.
pub fn hide_to_tray<R: Runtime>(app: &AppHandle<R>) {
    hide_main(app);
    if timer_active(app) {
        open_mini(app);
    }
}

/// Tray-Klick: Hauptfenster umschalten.
pub fn toggle_main<R: Runtime>(app: &AppHandle<R>) {
    match main_window(app).map(|window| window.is_visible()) {
        Some(Ok(true)) => hide_to_tray(app),
        _ => show_main(app),
    }
}

/// Öffnet den Mini-Timer abseits des Ereignis-Threads: Fenster dürfen (v. a. unter Windows) nicht im
/// Ereignishandler selbst erzeugt werden, das blockiert die Ereignisschleife.
fn open_mini<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move { show_mini(&app) });
}

/// Zeigt den Mini-Timer (wird beim ersten Mal erzeugt) und versteckt das Hauptfenster.
fn show_mini<R: Runtime>(app: &AppHandle<R>) {
    let mini = match app.get_webview_window(MINI) {
        Some(window) => window,
        None => {
            let built = WebviewWindowBuilder::new(app, MINI, WebviewUrl::App(MINI_ROUTE.into()))
                .title("nunc mini")
                .inner_size(264.0, 280.0)
                .decorations(false)
                .resizable(false)
                .always_on_top(true)
                .visible(false)
                .build();
            match built {
                Ok(window) => window,
                Err(error) => {
                    log::warn!("Mini-Timer nicht erzeugbar: {error}");
                    return;
                }
            }
        }
    };
    hide_main(app);
    if let Err(error) = mini.show() {
        log::warn!("Mini-Timer nicht anzeigbar: {error}");
    }
}

/// Tauri meldet kein eigenes Minimieren-Ereignis, und je nach Plattform kommt dabei weder `Resized` noch ein
/// Fokusverlust zuverlässig vor dem Minimieren an. Deshalb wird nach jeder Fensteränderung kurz verzögert geprüft.
fn check_minimized_later<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(350)).await;
        let minimized = main_window(&app)
            .and_then(|window| window.is_minimized().ok())
            .unwrap_or(false);
        if minimized && timer_active(&app) {
            show_mini(&app);
        }
    });
}

/// Fensterereignisse: Schließen verbirgt in den Tray, Minimieren während eines Timers öffnet den Mini-Timer.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != MAIN {
        return;
    }
    let app = window.app_handle();
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            let tray_enabled = app
                .try_state::<AppState>()
                .and_then(|state| state.db.with(|conn| nunc_store::load_settings(conn)).ok())
                .is_some_and(|settings| settings.system_tray_enabled);
            if tray_enabled {
                api.prevent_close();
                hide_to_tray(app);
            }
        }
        // Ohne Tray endet die App mit dem Hauptfenster (der Mini-Timer hält sie nicht am Leben).
        WindowEvent::Destroyed => app.exit(0),
        WindowEvent::Resized(_) | WindowEvent::Focused(false) | WindowEvent::Moved(_) => {
            check_minimized_later(app);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mini_timer_is_for_running_and_paused_timers() {
        assert!(wants_mini(TimerState::Running));
        assert!(wants_mini(TimerState::Paused));
        assert!(!wants_mini(TimerState::Idle));
    }
}
