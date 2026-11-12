use nunc_platform::SecretError;
use nunc_store::StoreError;

/// Fehler des Jira-Imports. Die Texte enthalten weder Token noch Header.
#[derive(Debug, thiserror::Error)]
pub enum JiraError {
    #[error("Jira ist nicht vollständig eingerichtet")]
    NotConfigured,
    #[error("Jira lehnt die Zugangsdaten ab (401/403)")]
    Unauthorized,
    #[error("Unerwartete Antwort von Jira ({0})")]
    Http(u16),
    /// Netzwerkfehler, bereits in Text übersetzt.
    #[error("Jira nicht erreichbar: {0}")]
    Transport(String),
    #[error("{0}")]
    BadResponse(String),
    #[error("{0}")]
    Store(#[from] StoreError),
    #[error("{0}")]
    Secret(#[from] SecretError),
}
