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

/// Spike M0-10: Schreiben/Lesen/Löschen im Tresor der Plattform.
#[tauri::command]
#[specta::specta]
pub fn secret_selftest() -> Result<String, String> {
    nunc_platform::secret::selftest().map_err(|e| e.to_string())
}
