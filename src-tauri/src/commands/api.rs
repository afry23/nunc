//! Lokale HTTP-Schnittstelle (ADR 0013): Zustand und Token für die Einstellungen. Ein-/Ausschalten und Port sind
//! Teil von `save_settings`. Auf Mobilgeräten gibt es die Schnittstelle nicht (`unsupported`).

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Runtime};

use crate::error::AppError;

/// Zustand der Schnittstelle für die Oberfläche.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ApiStatus {
    /// Der Server lauscht.
    pub running: bool,
    /// Tatsächlicher Port (kann vom gewünschten abweichen, wenn dieser belegt war).
    pub port: Option<u32>,
    /// Warum der Server nicht läuft, obwohl er laufen soll.
    pub error: Option<String>,
}

#[cfg(desktop)]
fn service<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<tauri::State<'_, crate::api_service::ApiService>, AppError> {
    use tauri::Manager;
    app.try_state::<crate::api_service::ApiService>()
        .ok_or_else(|| AppError::internal("Die Schnittstelle ist nicht bereit."))
}

/// Läuft der Server, auf welchem Port, und warum nicht.
#[tauri::command]
#[specta::specta]
pub async fn get_api_status<R: Runtime>(app: AppHandle<R>) -> Result<ApiStatus, AppError> {
    #[cfg(desktop)]
    return Ok(service(&app)?.status().await);
    #[cfg(mobile)]
    {
        let _ = app;
        Err(unsupported())
    }
}

/// Das Token im Klartext (nur auf Klick in den Einstellungen; es steht sonst nirgends in der Oberfläche).
#[tauri::command]
#[specta::specta]
pub async fn get_api_token<R: Runtime>(app: AppHandle<R>) -> Result<String, AppError> {
    #[cfg(desktop)]
    return service(&app)?.token();
    #[cfg(mobile)]
    {
        let _ = app;
        Err(unsupported())
    }
}

/// Erzeugt ein neues Token; das alte gilt sofort nicht mehr.
#[tauri::command]
#[specta::specta]
pub async fn regenerate_api_token<R: Runtime>(app: AppHandle<R>) -> Result<String, AppError> {
    #[cfg(desktop)]
    return service(&app)?.regenerate().await;
    #[cfg(mobile)]
    {
        let _ = app;
        Err(unsupported())
    }
}

#[cfg(mobile)]
fn unsupported() -> AppError {
    AppError::unsupported("Die lokale Schnittstelle gibt es nur auf dem Desktop.")
}
