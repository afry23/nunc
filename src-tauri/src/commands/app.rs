use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

/// Name und Version der App (Beispiel-Command für die Typ-Bridge).
#[tauri::command]
#[specta::specta]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME").to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

/// Holt das Hauptfenster nach vorn und versteckt den Mini-Timer (Desktop; mobil ohne Wirkung). Mit `route`
/// wechselt das Hauptfenster zusätzlich zu dieser Adresse.
#[tauri::command]
#[specta::specta]
pub fn show_main_window<R: tauri::Runtime>(app: tauri::AppHandle<R>, route: Option<String>) {
    #[cfg(desktop)]
    {
        crate::windows::show_main(&app);
        if let Some(route) = route.filter(|route| route.starts_with('/')) {
            use tauri_specta::Event;
            if let Err(error) = crate::events::AppNavigate(route).emit(&app) {
                log::warn!("Navigation nicht gesendet: {error}");
            }
        }
    }
    #[cfg(mobile)]
    let _ = (app, route);
}
