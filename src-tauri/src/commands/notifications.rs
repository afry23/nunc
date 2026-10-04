//! Benachrichtigungs-Berechtigung (Android 13+, iOS). Auf dem Desktop ist sie immer erteilt.

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::{NotificationExt, PermissionState};

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum NotificationPermission {
    Granted,
    Denied,
    /// Noch nicht entschieden: die Abfrage zeigt den Systemdialog.
    Prompt,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStatus {
    pub permission: NotificationPermission,
}

impl From<PermissionState> for NotificationStatus {
    fn from(state: PermissionState) -> Self {
        let permission = match state {
            PermissionState::Granted => NotificationPermission::Granted,
            PermissionState::Denied => NotificationPermission::Denied,
            // Android fragt nach einer Ablehnung erneut mit Begründung; für die Oberfläche ist das „noch offen“
            _ => NotificationPermission::Prompt,
        };
        Self { permission }
    }
}

fn plugin_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "notifications",
        format!("Benachrichtigungen nicht verfügbar: {error}"),
    )
}

/// Aktueller Stand, ohne den Nutzer zu fragen.
#[tauri::command]
#[specta::specta]
pub async fn get_notification_status<R: Runtime>(
    app: AppHandle<R>,
) -> Result<NotificationStatus, AppError> {
    let state = app
        .notification()
        .permission_state()
        .map_err(plugin_error)?;
    Ok(state.into())
}

/// Fragt die Berechtigung an (Systemdialog, falls noch offen) und liefert den neuen Stand.
///
/// Der Aufruf wartet auf die Antwort des Nutzers und läuft deshalb nicht auf dem Hauptthread, den der
/// Systemdialog selbst braucht.
#[tauri::command]
#[specta::specta]
pub async fn request_notification_permission<R: Runtime>(
    app: AppHandle<R>,
) -> Result<NotificationStatus, AppError> {
    let state =
        tauri::async_runtime::spawn_blocking(move || app.notification().request_permission())
            .await
            .map_err(plugin_error)?
            .map_err(plugin_error)?;
    Ok(state.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_plugin_state() {
        let permission = |state| NotificationStatus::from(state).permission;
        assert_eq!(
            permission(PermissionState::Granted),
            NotificationPermission::Granted
        );
        assert_eq!(
            permission(PermissionState::Denied),
            NotificationPermission::Denied
        );
        assert_eq!(
            permission(PermissionState::Prompt),
            NotificationPermission::Prompt
        );
        assert_eq!(
            permission(PermissionState::PromptWithRationale),
            NotificationPermission::Prompt
        );
    }
}
