use nunc_platform::{OpenError, SecretError};
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
    /// Es gibt keine gespeicherte Anmeldung (`Google Drive`, `Todoist`).
    #[error("Nicht mit {0} verbunden")]
    NotConnected(&'static str),
    /// Der Dienst lehnt das Token ab und es lässt sich nicht erneuern.
    #[error("{0}: Anmeldung abgelaufen – bitte neu verbinden")]
    AuthExpired(&'static str),
    /// Für Google Drive ist kein eigener OAuth-Client eingetragen (ADR 0019).
    #[error(
        "Kein Google-Zugang eingerichtet. Trage Client-ID und Client-Secret in den Einstellungen ein."
    )]
    GoogleClientMissing,
    /// Es gibt noch eine Verbindung, aber keinen eigenen Client (Anmeldung mit dem früher eingebauten Client).
    #[error(
        "Google Drive braucht jetzt einen eigenen Zugang – bitte in den Einstellungen einrichten"
    )]
    GoogleClientRequired,
    /// Client-ID oder -Secret unbrauchbar, bereits als Text für die Anzeige.
    #[error("{0}")]
    GoogleClientInvalid(&'static str),
    /// Todoist lehnt das eingegebene persönliche Token ab (oder es ist leer, ADR 0020).
    #[error("Todoist hat das Token abgelehnt. Bitte prüfe es in den Todoist-Einstellungen.")]
    TodoistTokenRejected,
    /// Das gespeicherte Todoist-Token gilt nicht mehr (widerrufen oder alte, abgelaufene OAuth-Anmeldung).
    #[error("Todoist: Token ungültig – bitte neu eintragen")]
    TodoistTokenInvalid,
    /// Fehler beim Anmelden (Browser, Rückruf, Token-Austausch), bereits als Text für die Anzeige.
    #[error("{0}")]
    OAuth(String),
    #[error("{0}")]
    Open(#[from] OpenError),
    #[error("{0}")]
    Store(#[from] StoreError),
    #[error("{0}")]
    Secret(#[from] SecretError),
}
