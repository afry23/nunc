//! WebDAV-Zugangsdaten: Einstellungen in der Datenbank, das Passwort im Tresor (ADR 0005).

use nunc_core::model::WebDavConfig;
use nunc_platform::SecretStore;
use nunc_store::{Connection, load_webdav_config, save_webdav_config};

use crate::SyncError;

pub const SECRET_SERVICE: &str = "nunc";
pub const SECRET_ACCOUNT: &str = "webdav";

/// Vollständige Zugangsdaten für einen Sync.
#[derive(Clone, PartialEq, Eq)]
pub struct WebDavCredentials {
    pub config: WebDavConfig,
    pub password: String,
}

// Das Passwort gehört nicht in Logs und Testausgaben
impl std::fmt::Debug for WebDavCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebDavCredentials")
            .field("config", &self.config)
            .field("password", &"…")
            .finish()
    }
}

/// Speichert die Einstellungen in der Datenbank und – wenn angegeben – das Passwort im Tresor. `None`
/// lässt das gespeicherte Passwort unverändert. Zuerst der Tresor: scheitert er, bleibt alles beim Alten.
pub fn save_webdav(
    conn: &mut Connection,
    secrets: &dyn SecretStore,
    config: &WebDavConfig,
    password: Option<&str>,
) -> Result<(), SyncError> {
    if let Some(password) = password {
        secrets.set(SECRET_SERVICE, SECRET_ACCOUNT, password)?;
    }
    save_webdav_config(conn, config)?;
    Ok(())
}

/// Ob ein Passwort gespeichert ist (das Passwort selbst verlässt den Tresor nur für den Sync).
pub fn has_webdav_password(secrets: &dyn SecretStore) -> Result<bool, SyncError> {
    Ok(secrets.get(SECRET_SERVICE, SECRET_ACCOUNT)?.is_some())
}

/// Einstellungen und Passwort für einen Sync; `NotConfigured`, wenn URL, Benutzer oder Passwort fehlen.
pub fn load_credentials(
    conn: &Connection,
    secrets: &dyn SecretStore,
) -> Result<WebDavCredentials, SyncError> {
    let config = load_webdav_config(conn)?;
    let password = secrets.get(SECRET_SERVICE, SECRET_ACCOUNT)?;
    match password {
        Some(password) if config.is_complete() => Ok(WebDavCredentials { config, password }),
        _ => Err(SyncError::NotConfigured),
    }
}

#[cfg(test)]
mod tests {
    use nunc_platform::MemorySecretStore;

    use super::*;

    #[test]
    fn the_stored_password_is_reported_but_never_printed() {
        let secrets = MemorySecretStore::default();
        assert!(!has_webdav_password(&secrets).unwrap());
        secrets
            .set(SECRET_SERVICE, SECRET_ACCOUNT, "geheim")
            .unwrap();
        assert!(has_webdav_password(&secrets).unwrap());
        let credentials = WebDavCredentials {
            config: WebDavConfig::default(),
            password: "geheim".into(),
        };
        let text = format!("{credentials:?}");
        assert!(!text.contains("geheim") && text.contains("WebDavCredentials"));
    }
}
