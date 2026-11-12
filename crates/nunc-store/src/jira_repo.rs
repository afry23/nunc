//! Jira-Zugang (ADR 0038): URL, Art und E-Mail in den Einstellungen. Das Token steht nie in der Datenbank.

use nunc_core::jira::{JiraConfig, JiraKind};
use rusqlite::{Connection, params_from_iter};

use crate::StoreError;
use crate::settings_repo::set_setting;
use crate::sync_repo::setting;

const URL_KEY: &str = "jira.base_url";
const KIND_KEY: &str = "jira.kind";
const EMAIL_KEY: &str = "jira.email";

/// Gespeicherter Zugang; fehlende Werte ergeben leere Felder, eine unbekannte Art wird zu Cloud.
pub fn load_jira_config(conn: &Connection) -> Result<JiraConfig, StoreError> {
    let text = |key: &str| setting(conn, key).map(Option::unwrap_or_default);
    Ok(JiraConfig {
        base_url: text(URL_KEY)?,
        kind: text(KIND_KEY)?.parse().unwrap_or(JiraKind::Cloud),
        email: text(EMAIL_KEY)?,
    })
}

/// Schreibt die drei Schlüssel in einer Transaktion.
pub fn save_jira_config(conn: &mut Connection, config: &JiraConfig) -> Result<(), StoreError> {
    let tx = conn.transaction()?;
    set_setting(&tx, URL_KEY, &config.base_url)?;
    set_setting(&tx, KIND_KEY, config.kind.as_str())?;
    set_setting(&tx, EMAIL_KEY, &config.email)?;
    tx.commit()?;
    Ok(())
}

/// Entfernt die drei Schlüssel.
pub fn clear_jira_config(conn: &Connection) -> Result<(), StoreError> {
    conn.execute(
        "DELETE FROM settings WHERE key IN (?1, ?2, ?3)",
        params_from_iter([URL_KEY, KIND_KEY, EMAIL_KEY]),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;

    fn config() -> JiraConfig {
        JiraConfig {
            base_url: "https://jira.firma.de".into(),
            kind: JiraKind::DataCenter,
            email: "me@firma.de".into(),
        }
    }

    #[test]
    fn nothing_saved_gives_empty_cloud_config() {
        let conn = open_in_memory().unwrap();
        let loaded = load_jira_config(&conn).unwrap();
        assert_eq!(loaded.base_url, "");
        assert_eq!(loaded.email, "");
        assert_eq!(loaded.kind, JiraKind::Cloud);
    }

    #[test]
    fn the_config_roundtrips() {
        let mut conn = open_in_memory().unwrap();
        save_jira_config(&mut conn, &config()).unwrap();
        assert_eq!(load_jira_config(&conn).unwrap(), config());
    }

    #[test]
    fn an_unknown_kind_falls_back_to_cloud() {
        let mut conn = open_in_memory().unwrap();
        save_jira_config(&mut conn, &config()).unwrap();
        conn.execute(
            "UPDATE settings SET value = 'server' WHERE key = 'jira.kind'",
            [],
        )
        .unwrap();
        assert_eq!(load_jira_config(&conn).unwrap().kind, JiraKind::Cloud);
    }

    #[test]
    fn clearing_removes_only_the_jira_keys() {
        let mut conn = open_in_memory().unwrap();
        save_jira_config(&mut conn, &config()).unwrap();
        set_setting(&conn, "other", "x").unwrap();
        clear_jira_config(&conn).unwrap();
        let left: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM settings WHERE key LIKE 'jira.%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(left, 0);
        assert_eq!(setting(&conn, "other").unwrap().as_deref(), Some("x"));
    }
}
