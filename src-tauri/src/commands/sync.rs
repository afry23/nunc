//! Commands für Sync-Anbieter, WebDAV-Konfiguration und Synchronisierung.

use nunc_core::model::{
    ConnectionTestResult, SyncProvider, SyncResult, SyncStatus, WebDavConfig, is_http_url,
};
use nunc_sync::{Trigger, WebDavClient};
use serde::{Deserialize, Serialize};
use specta::Type;
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

/// Gespeicherte WebDAV-Einstellungen. Das Passwort verlässt den Tresor nicht: die Oberfläche erfährt nur, ob eins
/// gespeichert ist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WebDavSettings {
    pub config: WebDavConfig,
    pub has_password: bool,
}

#[tauri::command]
#[specta::specta]
pub fn get_webdav_config(state: State<'_, AppState>) -> Result<WebDavSettings, AppError> {
    let config = state.db.with(|conn| nunc_store::load_webdav_config(conn))?;
    let has_password = nunc_sync::has_webdav_password(&*state.secrets)?;
    Ok(WebDavSettings {
        config,
        has_password,
    })
}

/// Eine leere Adresse ist erlaubt (Einstellungen leeren), sonst muss es `http(s)://host` sein.
fn validate_url(config: &WebDavConfig) -> Result<(), AppError> {
    if config.url.trim().is_empty() || is_http_url(&config.url) {
        Ok(())
    } else {
        Err(AppError::validation(
            "Die Server-Adresse muss mit http:// oder https:// beginnen.",
        ))
    }
}

/// Speichert die Einstellungen in der Datenbank und das Passwort im Tresor. `password: None` lässt das
/// gespeicherte Passwort unverändert.
#[tauri::command]
#[specta::specta]
pub fn save_webdav_config(
    state: State<'_, AppState>,
    config: WebDavConfig,
    password: Option<String>,
) -> Result<(), AppError> {
    validate_url(&config)?;
    let config = WebDavConfig {
        url: config.url.trim().to_owned(),
        ..config
    };
    state
        .db
        .with(|conn| nunc_sync::save_webdav(conn, &*state.secrets, &config, password.as_deref()))
}

/// Prüft die Verbindung mit den übergebenen (noch nicht gespeicherten) Werten. Ohne Passwort gilt das
/// gespeicherte.
#[tauri::command]
#[specta::specta]
pub async fn test_webdav_connection(
    state: State<'_, AppState>,
    config: WebDavConfig,
    password: Option<String>,
) -> Result<ConnectionTestResult, AppError> {
    let password = match password {
        Some(password) => password,
        None => state
            .secrets
            .get(nunc_sync::SECRET_SERVICE, nunc_sync::SECRET_ACCOUNT)?
            .unwrap_or_default(),
    };
    Ok(match WebDavClient::new(config, password) {
        Ok(client) => client.test_connection().await,
        Err(error) => ConnectionTestResult {
            success: false,
            message: error.to_string(),
        },
    })
}

/// Zustand der Synchronisierung (für die Statusleiste beim Öffnen; danach kommt `sync:status`).
#[tauri::command]
#[specta::specta]
pub fn get_sync_status(state: State<'_, AppState>) -> SyncStatus {
    state.sync.status()
}

/// Synchronisiert jetzt und wartet auf das Ergebnis. `None`, wenn nichts zu synchronisieren ist (kein Anbieter
/// gewählt oder Sync pausiert). Ein Fehler steht zusätzlich im Status.
#[tauri::command]
#[specta::specta]
pub async fn sync_now(state: State<'_, AppState>) -> Result<Option<SyncResult>, AppError> {
    let sync = state.sync.clone();
    Ok(sync.sync_now().await?)
}

/// Die App geht in den Hintergrund (Mobil): ein letzter Sync, bevor das Betriebssystem sie einfriert.
#[tauri::command]
#[specta::specta]
pub fn sync_in_background(state: State<'_, AppState>) {
    state.sync.trigger(Trigger::Background);
}
