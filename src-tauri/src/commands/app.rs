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

/// Holt das Hauptfenster nach vorn und versteckt den Mini-Timer (Desktop; mobil ohne Wirkung).
#[tauri::command]
#[specta::specta]
pub fn show_main_window<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
    #[cfg(desktop)]
    crate::windows::show_main(&app);
    #[cfg(mobile)]
    let _ = app;
}
