//! Konfiguration und Zustand der Synchronisierung (Go: `models/sync.go`, JSON camelCase wie dort).

use serde::{Deserialize, Serialize};
use specta::Type;

use super::text_enum;

/// Anmeldeverfahren am WebDAV-Server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum WebDavAuth {
    Basic,
    Digest,
}

text_enum!(WebDavAuth, "WebDavAuth", {
    Basic => "basic",
    Digest => "digest",
});

/// Verbindung zu einem WebDAV-Server. Das Passwort gehört nicht dazu: es liegt im Tresor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WebDavConfig {
    pub url: String,
    pub auth_method: WebDavAuth,
    pub username: String,
    /// Ordner auf dem Server, in dem `nunc-sync.v2.json` liegt (Standard `/nunc/`).
    pub remote_path: String,
}

impl Default for WebDavConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            auth_method: WebDavAuth::Basic,
            username: String::new(),
            remote_path: "/nunc/".to_owned(),
        }
    }
}

impl WebDavConfig {
    /// Ob URL und Benutzername ausgefüllt sind und die URL ein `http(s)://`-Ziel mit Host nennt.
    pub fn is_complete(&self) -> bool {
        is_http_url(&self.url) && !self.username.trim().is_empty()
    }
}

/// `http://` oder `https://` gefolgt von einem Host.
pub fn is_http_url(text: &str) -> bool {
    let text = text.trim();
    let Some(rest) = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))
    else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    !host.is_empty() && !host.contains(char::is_whitespace) && !host.starts_with(':')
}

/// Ergebnis des Verbindungstests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub success: bool,
    /// Servername bei Erfolg, lesbarer Fehler sonst.
    pub message: String,
}

/// Zustand der Synchronisierung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SyncState {
    Idle,
    Syncing,
    Error,
}

/// Nutzlast von `sync:status` und Ergebnis von `get_sync_status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub state: SyncState,
    /// RFC 3339; `None`, solange noch nie synchronisiert wurde.
    pub last_sync_at: Option<String>,
    pub error_message: Option<String>,
}

impl SyncStatus {
    pub fn idle(last_sync_at: Option<String>) -> Self {
        Self {
            state: SyncState::Idle,
            last_sync_at,
            error_message: None,
        }
    }
}

/// Was ein Sync-Lauf vom Server übernommen hat (`sync:complete`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub imported_todos: u32,
    pub imported_sessions: u32,
    /// Lokale Todos, die durch eine neuere Server-Version ersetzt wurden.
    pub conflicted_todos: u32,
}

/// Zustand der Google-Drive-Verbindung (`get_drive_status`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DriveStatus {
    /// Ob dieser Build Zugangsdaten für Google hat (Client-ID und -Secret).
    pub configured: bool,
    pub connected: bool,
    pub email: Option<String>,
    /// Link auf den Ordner `nunc` im Drive (erst nach dem ersten Sync bekannt).
    pub folder_url: Option<String>,
}

/// Ergebnis der Prüfung nach dem Verbinden: liegt auf Drive Stand, den ein leeres Gerät übernehmen kann?
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncImportInfo {
    pub available: bool,
    pub todo_count: u32,
    pub session_count: u32,
}

/// Zustand der Todoist-Verbindung (`get_todoist_status`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TodoistStatus {
    /// Ob dieser Build Zugangsdaten für Todoist hat.
    pub configured: bool,
    pub connected: bool,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_need_a_scheme_and_a_host() {
        for ok in [
            "https://cloud.example.org/remote.php/dav/files/me",
            "http://192.168.1.2:8080",
            "  https://a.b  ",
        ] {
            assert!(is_http_url(ok), "{ok}");
        }
        for bad in [
            "",
            "cloud.example.org",
            "ftp://x.org",
            "https://",
            "https:///pfad",
            "https://a b/",
            "http://:80",
            "https:/x.org",
        ] {
            assert!(!is_http_url(bad), "{bad}");
        }
    }

    #[test]
    fn a_config_is_complete_with_url_and_user() {
        let mut config = WebDavConfig::default();
        assert!(!config.is_complete());
        config.url = "https://x.org".into();
        assert!(!config.is_complete());
        config.username = " ".into();
        assert!(!config.is_complete());
        config.username = "me".into();
        assert!(config.is_complete());
        config.url = "x.org".into();
        assert!(!config.is_complete());
    }

    #[test]
    fn serializes_with_the_go_keys() {
        let value = serde_json::to_value(WebDavConfig::default()).unwrap();
        assert_eq!(value["authMethod"], "basic");
        assert_eq!(value["remotePath"], "/nunc/");
        let status = serde_json::to_value(SyncStatus::idle(None)).unwrap();
        assert_eq!(status["state"], "idle");
        assert!(status["lastSyncAt"].is_null());
        assert_eq!(WebDavAuth::Digest.as_str(), "digest");
        assert_eq!("basic".parse(), Ok(WebDavAuth::Basic));
    }
}
