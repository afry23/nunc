//! Commands für Sync-Anbieter, WebDAV-Konfiguration und Synchronisierung.

use nunc_core::model::SyncProvider;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Wählt den Sync-Anbieter (exklusiv) und speichert die Wahl sofort. Zugangsdaten der übrigen Anbieter bleiben
/// erhalten, sie werden nur nicht mehr benutzt.
#[tauri::command]
#[specta::specta]
pub fn set_sync_provider(
    state: State<'_, AppState>,
    provider: SyncProvider,
) -> Result<(), AppError> {
    if !provider.is_available(cfg!(mobile)) {
        return Err(AppError::unsupported(
            "Dieser Sync-Anbieter ist auf Mobilgeräten nicht verfügbar.",
        ));
    }
    state
        .db
        .with(|conn| nunc_store::set_sync_provider(conn, provider))
}
