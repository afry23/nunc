//! Einheitlicher Fehler für alle Commands: serialisiert als `{ code, message }` (camelCase wie alles andere).
//! Der `code` ist stabil und für die Oberfläche gedacht, die `message` ein deutscher Text zur Anzeige.

use std::sync::PoisonError;

use nunc_core::model::SettingsIssue;
use nunc_core::timer::TimerError;
use nunc_platform::{NotifyError, SecretError};
use nunc_store::StoreError;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[error("{message}")]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
        }
    }

    /// Ungültige Eingabe des Aufrufers.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("invalid_input", message)
    }

    /// Nicht auf dieser Plattform verfügbar.
    #[cfg_attr(
        desktop,
        expect(dead_code, reason = "nur die Mobil-Zweige melden „nicht verfügbar“")
    )]
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new("unsupported", message)
    }

    /// Unerwarteter interner Zustand (z. B. vergiftete Sperre).
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }
}

impl From<StoreError> for AppError {
    fn from(error: StoreError) -> Self {
        let code = match &error {
            StoreError::Io(_) => "io",
            StoreError::Sqlite(_) | StoreError::Migration { .. } => "database",
            StoreError::Json(_) => "state_unreadable",
            StoreError::BackupDir(_) => "backup_dir",
            StoreError::InvalidBackup(_) => "invalid_backup",
            StoreError::NotFound { .. } => "not_found",
        };
        Self::new(code, error.to_string())
    }
}

impl From<TimerError> for AppError {
    fn from(error: TimerError) -> Self {
        Self::new(error.code(), error.to_string())
    }
}

impl From<Vec<SettingsIssue>> for AppError {
    fn from(issues: Vec<SettingsIssue>) -> Self {
        let details: Vec<String> = issues
            .iter()
            .map(|issue| format!("{} (erlaubt: {})", issue.field, issue.expected))
            .collect();
        Self::new(
            "invalid_settings",
            format!("Ungültige Einstellungen: {}", details.join(", ")),
        )
    }
}

impl From<SecretError> for AppError {
    fn from(error: SecretError) -> Self {
        Self::new("secret_store", error.to_string())
    }
}

impl From<NotifyError> for AppError {
    fn from(error: NotifyError) -> Self {
        Self::new("notification", error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::new("io", format!("Dateizugriff fehlgeschlagen: {error}"))
    }
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(_: PoisonError<T>) -> Self {
        Self::internal("interner Zustand gesperrt")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_code_and_message() {
        let error = AppError::from(TimerError::AlreadyActive);
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["code"], "already_active");
        assert_eq!(json["message"], "es läuft bereits ein Timer");
    }

    #[test]
    fn store_errors_get_stable_codes() {
        let error = AppError::from(StoreError::NotFound {
            entity: "Todo",
            id: 7,
        });
        assert_eq!(error.code, "not_found");
        assert_eq!(error.message, "Todo 7 nicht gefunden");
        assert_eq!(
            AppError::from(StoreError::InvalidBackup("x".into())).code,
            "invalid_backup"
        );
        assert_eq!(
            AppError::from(StoreError::BackupDir("x".into())).code,
            "backup_dir"
        );
    }

    #[test]
    fn settings_issues_are_listed_together() {
        let error = AppError::from(vec![
            SettingsIssue {
                field: "pomodoroMin".into(),
                expected: "≥ 1".into(),
            },
            SettingsIssue {
                field: "soundVolume".into(),
                expected: "0–100".into(),
            },
        ]);
        assert_eq!(error.code, "invalid_settings");
        assert!(error.message.contains("pomodoroMin (erlaubt: ≥ 1)"));
        assert!(error.message.contains("soundVolume (erlaubt: 0–100)"));
    }
}
