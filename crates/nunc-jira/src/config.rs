//! Jira-Zugang: Einstellungen in der Datenbank, das Token im Tresor (Dienst `nunc`, Konto `jira`).

use nunc_core::jira::JiraConfig;
use nunc_platform::SecretStore;
use nunc_store::{Connection, clear_jira_config, load_jira_config, save_jira_config};

use crate::JiraError;

pub const SECRET_SERVICE: &str = "nunc";
pub const SECRET_ACCOUNT: &str = "jira";

/// Vollständige Zugangsdaten für eine Abfrage.
#[derive(Clone, PartialEq, Eq)]
pub struct JiraCredentials {
    pub config: JiraConfig,
    pub token: String,
}

// Das Token gehört nicht in Logs und Testausgaben
impl std::fmt::Debug for JiraCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JiraCredentials")
            .field("config", &self.config)
            .field("token", &"…")
            .finish()
    }
}

/// Speichert Einstellungen und – wenn angegeben – das Token. `None` lässt das gespeicherte Token unverändert.
/// Zuerst der Tresor: scheitert er, bleibt alles beim Alten.
pub fn save_jira(
    conn: &mut Connection,
    secrets: &dyn SecretStore,
    config: &JiraConfig,
    token: Option<&str>,
) -> Result<(), JiraError> {
    if let Some(token) = token {
        secrets.set(SECRET_SERVICE, SECRET_ACCOUNT, token)?;
    }
    save_jira_config(conn, config)?;
    Ok(())
}

/// Ob ein Token gespeichert ist.
pub fn has_jira_token(secrets: &dyn SecretStore) -> Result<bool, JiraError> {
    Ok(secrets.get(SECRET_SERVICE, SECRET_ACCOUNT)?.is_some())
}

/// Einstellungen und Token; `NotConfigured`, wenn etwas fehlt.
pub fn load_jira_credentials(
    conn: &Connection,
    secrets: &dyn SecretStore,
) -> Result<JiraCredentials, JiraError> {
    let config = load_jira_config(conn)?;
    match secrets.get(SECRET_SERVICE, SECRET_ACCOUNT)? {
        Some(token) if config.is_complete() && !token.is_empty() => {
            Ok(JiraCredentials { config, token })
        }
        _ => Err(JiraError::NotConfigured),
    }
}

/// Entfernt Einstellungen und Token.
pub fn clear_jira(conn: &Connection, secrets: &dyn SecretStore) -> Result<(), JiraError> {
    secrets.delete(SECRET_SERVICE, SECRET_ACCOUNT)?;
    clear_jira_config(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use nunc_core::jira::JiraKind;
    use nunc_platform::MemorySecretStore;

    use super::*;

    #[test]
    fn debug_output_hides_the_token() {
        let credentials = JiraCredentials {
            config: JiraConfig {
                base_url: "https://x".into(),
                kind: JiraKind::Cloud,
                email: "a@b".into(),
            },
            token: "geheim".into(),
        };
        let text = format!("{credentials:?}");
        assert!(!text.contains("geheim") && text.contains("JiraCredentials"));
    }

    #[test]
    fn the_token_is_reported_but_not_loaded_without_config() {
        let conn = nunc_store::open_in_memory().unwrap();
        let secrets = MemorySecretStore::default();
        assert!(!has_jira_token(&secrets).unwrap());
        secrets.set(SECRET_SERVICE, SECRET_ACCOUNT, "t").unwrap();
        assert!(has_jira_token(&secrets).unwrap());
        assert!(matches!(
            load_jira_credentials(&conn, &secrets),
            Err(JiraError::NotConfigured)
        ));
    }
}
