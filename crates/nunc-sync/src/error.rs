use nunc_platform::SecretError;
use nunc_store::StoreError;

/// Fehler der Synchronisierung. Die Texte sind für die Anzeige gedacht (deutsch, ohne Technikjargon, soweit
/// möglich) und stehen so in der Statusleiste.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// Netzwerkfehler, bereits in lesbaren Text übersetzt (`Verbindung abgelehnt – …`).
    #[error("{0}")]
    Transport(String),
    #[error("401 Unauthorized – Benutzername oder Passwort falsch")]
    Unauthorized,
    #[error("403 Forbidden – Zugriff verweigert")]
    Forbidden,
    #[error("Der Server verlangt {0}-Authentifizierung, eingestellt ist {1}")]
    WrongAuthMethod(&'static str, &'static str),
    #[error("Unerwartete Antwort des Servers ({status}) beim {action}")]
    Status { status: u16, action: &'static str },
    /// Die Datei auf dem Server hat sich seit dem Herunterladen geändert (ETag stimmt nicht mehr).
    #[error("Die Sync-Datei wurde auf dem Server gleichzeitig geändert")]
    Conflict,
    #[error("Sync-Datei nicht lesbar: {0}")]
    InvalidFile(String),
    #[error("Ungültige Server-Adresse: {0}")]
    InvalidUrl(String),
    #[error("WebDAV ist nicht vollständig eingerichtet")]
    NotConfigured,
    #[error("{0}")]
    Store(#[from] StoreError),
    #[error("{0}")]
    Secret(#[from] SecretError),
}
