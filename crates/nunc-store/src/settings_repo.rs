//! Einstellungen als Key-Value-Tabelle `settings`, gleiche Schlüssel und Textformate wie die Go-App
//! (`nunc-go/internal/services/settings.go`).
//!
//! Lesen: fehlende oder nicht lesbare Werte fallen auf den Standardwert zurück (wie in Go). Schreiben:
//! alle Schlüssel in einer Transaktion; fremde Schlüssel (`device_id`, …) bleiben unberührt.

use std::collections::HashMap;

use nunc_core::model::{Settings, SyncProvider};
use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;

/// Lädt die Einstellungen; ohne Zeilen die Standardwerte.
pub fn load_settings(conn: &Connection) -> Result<Settings, StoreError> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let kv = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<HashMap<String, String>, _>>()?;
    Ok(settings_from_kv(&kv))
}

/// Schreibt alle Einstellungen atomar. Prüfung der Wertebereiche (`Settings::validate`) ist Sache des Aufrufers.
pub fn save_settings(conn: &mut Connection, settings: &Settings) -> Result<(), StoreError> {
    let tx = conn.transaction()?;
    for (key, value) in kv_from_settings(settings) {
        tx.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Schreibt einen einzelnen Schlüssel, ohne die übrigen Einstellungen anzufassen.
pub(crate) fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

/// Wählt den Sync-Anbieter und schreibt nur diesen Schlüssel (sofort, ohne die übrigen Einstellungen).
pub fn set_sync_provider(conn: &Connection, provider: SyncProvider) -> Result<(), StoreError> {
    set_setting(conn, "sync_provider", provider.as_str())
}

/// Kennung dieses Geräts (Schlüssel `device_id`, wie in der Go-App). Fehlt sie oder ist sie leer, legt
/// `generate` eine neue an; sie bleibt danach stabil und kennzeichnet Sessions beim Sync.
pub fn device_id(
    conn: &Connection,
    generate: impl FnOnce() -> String,
) -> Result<String, StoreError> {
    let stored: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'device_id'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(id) = stored.filter(|id| !id.is_empty()) {
        return Ok(id);
    }
    let id = generate();
    set_setting(conn, "device_id", &id)?;
    Ok(id)
}

/// Go `strconv.ParseBool`.
fn parse_bool(text: &str) -> Option<bool> {
    match text {
        "1" | "t" | "T" | "TRUE" | "true" | "True" => Some(true),
        "0" | "f" | "F" | "FALSE" | "false" | "False" => Some(false),
        _ => None,
    }
}

fn settings_from_kv(kv: &HashMap<String, String>) -> Settings {
    let mut s = Settings::default();
    let get = |key: &str| kv.get(key).map(String::as_str);
    let number = |key: &str, slot: &mut u32| {
        if let Some(n) = get(key).and_then(|v| v.parse().ok()) {
            *slot = n;
        }
    };
    let flag = |key: &str, slot: &mut bool| {
        if let Some(b) = get(key).and_then(parse_bool) {
            *slot = b;
        }
    };
    let text = |key: &str, slot: &mut String| {
        if let Some(v) = get(key) {
            v.clone_into(slot);
        }
    };

    number("pomodoro_min", &mut s.pomodoro_min);
    number("short_break_min", &mut s.short_break_min);
    number("long_break_min", &mut s.long_break_min);
    number("long_break_after", &mut s.long_break_after);
    flag("auto_start_breaks", &mut s.auto_start_breaks);
    flag("auto_start_next_pomodoro", &mut s.auto_start_next_pomodoro);
    flag("notifications_enabled", &mut s.notifications_enabled);
    flag("sound_enabled", &mut s.sound_enabled);
    number("sound_volume", &mut s.sound_volume);
    text("sound_end_pomodoro", &mut s.sound_end_pomodoro);
    text("sound_end_break", &mut s.sound_end_break);
    if let Some(mode) = get("active_mode").and_then(|v| v.parse().ok()) {
        s.active_mode = mode;
    }
    flag("system_tray_enabled", &mut s.system_tray_enabled);
    text("last_backup_date", &mut s.last_backup_date);
    text("backup_dir", &mut s.backup_dir);
    flag("sync_enabled", &mut s.sync_enabled);
    if let Some(provider) = get("sync_provider").and_then(|v| v.parse().ok()) {
        s.sync_provider = provider;
    }
    if let Some(ratio) = get("flowtime_ratio").and_then(|v| v.parse().ok()) {
        s.flowtime_ratio = ratio;
    }
    number(
        "flowtime_min_flow_state_min",
        &mut s.flowtime_min_flow_state_min,
    );
    if let Some(language) = get("language").and_then(|v| v.parse().ok()) {
        s.language = language;
    }
    s
}

fn kv_from_settings(s: &Settings) -> [(&'static str, String); 20] {
    [
        ("pomodoro_min", s.pomodoro_min.to_string()),
        ("short_break_min", s.short_break_min.to_string()),
        ("long_break_min", s.long_break_min.to_string()),
        ("long_break_after", s.long_break_after.to_string()),
        ("auto_start_breaks", s.auto_start_breaks.to_string()),
        (
            "auto_start_next_pomodoro",
            s.auto_start_next_pomodoro.to_string(),
        ),
        ("notifications_enabled", s.notifications_enabled.to_string()),
        ("sound_enabled", s.sound_enabled.to_string()),
        ("sound_volume", s.sound_volume.to_string()),
        ("sound_end_pomodoro", s.sound_end_pomodoro.clone()),
        ("sound_end_break", s.sound_end_break.clone()),
        ("active_mode", s.active_mode.as_str().to_owned()),
        ("system_tray_enabled", s.system_tray_enabled.to_string()),
        ("last_backup_date", s.last_backup_date.clone()),
        ("backup_dir", s.backup_dir.clone()),
        ("sync_enabled", s.sync_enabled.to_string()),
        ("sync_provider", s.sync_provider.as_str().to_owned()),
        ("flowtime_ratio", s.flowtime_ratio.to_string()),
        (
            "flowtime_min_flow_state_min",
            s.flowtime_min_flow_state_min.to_string(),
        ),
        ("language", s.language.as_str().to_owned()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;
    use nunc_core::model::{Language, SyncProvider, TimerMode};

    fn raw(conn: &Connection, key: &str) -> Option<String> {
        conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
            r.get(0)
        })
        .ok()
    }

    fn set_raw(conn: &Connection, key: &str, value: &str) {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [key, value],
        )
        .unwrap();
    }

    #[test]
    fn device_id_is_created_once_and_then_kept() {
        let conn = open_in_memory().unwrap();
        assert_eq!(device_id(&conn, || "dev-1".into()).unwrap(), "dev-1");
        assert_eq!(device_id(&conn, || "dev-2".into()).unwrap(), "dev-1");
        assert_eq!(raw(&conn, "device_id").as_deref(), Some("dev-1"));
    }

    #[test]
    fn an_empty_device_id_is_replaced() {
        let conn = open_in_memory().unwrap();
        set_raw(&conn, "device_id", "");
        assert_eq!(device_id(&conn, || "dev-3".into()).unwrap(), "dev-3");
    }

    #[test]
    fn choosing_the_provider_writes_only_its_key() {
        let conn = open_in_memory().unwrap();
        set_raw(&conn, "pomodoro_min", "40");
        set_sync_provider(&conn, SyncProvider::Webdav).unwrap();
        assert_eq!(raw(&conn, "sync_provider").as_deref(), Some("webdav"));
        let loaded = load_settings(&conn).unwrap();
        assert_eq!(loaded.sync_provider, SyncProvider::Webdav);
        assert_eq!(loaded.pomodoro_min, 40);
        set_sync_provider(&conn, SyncProvider::None).unwrap();
        assert_eq!(raw(&conn, "sync_provider").as_deref(), Some("none"));
    }

    #[test]
    fn fresh_database_yields_defaults() {
        let conn = open_in_memory().unwrap();
        assert_eq!(load_settings(&conn).unwrap(), Settings::default());
    }

    #[test]
    fn roundtrip_of_every_field() {
        let mut conn = open_in_memory().unwrap();
        let s = Settings {
            pomodoro_min: 50,
            short_break_min: 7,
            long_break_min: 30,
            long_break_after: 2,
            auto_start_breaks: true,
            auto_start_next_pomodoro: true,
            notifications_enabled: false,
            sound_enabled: false,
            sound_volume: 12,
            sound_end_pomodoro: "digital".into(),
            sound_end_break: "bell".into(),
            active_mode: TimerMode::Flowtime,
            system_tray_enabled: false,
            last_backup_date: "2026-10-01".into(),
            backup_dir: "/tmp/x".into(),
            sync_enabled: false,
            sync_provider: SyncProvider::Webdav,
            flowtime_ratio: 0.35,
            flowtime_min_flow_state_min: 40,
            language: Language::En,
        };
        save_settings(&mut conn, &s).unwrap();
        assert_eq!(load_settings(&conn).unwrap(), s);
    }

    #[test]
    fn written_texts_match_go_formats() {
        let mut conn = open_in_memory().unwrap();
        let s = Settings {
            auto_start_breaks: true,
            flowtime_ratio: 0.5,
            ..Settings::default()
        };
        save_settings(&mut conn, &s).unwrap();
        assert_eq!(raw(&conn, "auto_start_breaks").as_deref(), Some("true"));
        assert_eq!(raw(&conn, "sound_enabled").as_deref(), Some("true"));
        assert_eq!(raw(&conn, "flowtime_ratio").as_deref(), Some("0.5"));
        assert_eq!(raw(&conn, "active_mode").as_deref(), Some("pomodoro"));
        assert_eq!(raw(&conn, "sync_provider").as_deref(), Some("none"));
        assert_eq!(raw(&conn, "backup_dir").as_deref(), Some(""));
    }

    #[test]
    fn unreadable_values_fall_back_to_defaults() {
        let conn = open_in_memory().unwrap();
        for (k, v) in [
            ("pomodoro_min", "lots"),
            ("sound_volume", "-5"),
            ("auto_start_breaks", "maybe"),
            ("flowtime_ratio", "x"),
            ("active_mode", "zen"),
            ("sync_provider", "dropbox"),
        ] {
            set_raw(&conn, k, v);
        }
        assert_eq!(load_settings(&conn).unwrap(), Settings::default());
    }

    #[test]
    fn go_bool_spellings_are_accepted() {
        let conn = open_in_memory().unwrap();
        for (v, expected) in [("1", true), ("T", true), ("False", false), ("0", false)] {
            set_raw(&conn, "auto_start_breaks", v);
            assert_eq!(
                load_settings(&conn).unwrap().auto_start_breaks,
                expected,
                "{v}"
            );
        }
        // Standard ist `true`: unlesbar darf `false` nicht erzwingen
        set_raw(&conn, "sound_enabled", "nope");
        assert!(load_settings(&conn).unwrap().sound_enabled);
    }

    #[test]
    fn foreign_keys_survive_a_save() {
        let mut conn = open_in_memory().unwrap();
        set_raw(&conn, "device_id", "abc");
        save_settings(&mut conn, &Settings::default()).unwrap();
        assert_eq!(raw(&conn, "device_id").as_deref(), Some("abc"));
    }

    #[test]
    fn save_is_atomic() {
        let mut conn = open_in_memory().unwrap();
        // Zeile, die das Schreiben eines Schlüssels scheitern lässt
        conn.execute_batch(
            "CREATE TRIGGER no_volume BEFORE INSERT ON settings
             WHEN NEW.key = 'sound_volume' BEGIN SELECT RAISE(ABORT, 'nope'); END;",
        )
        .unwrap();
        let s = Settings {
            pomodoro_min: 99,
            ..Settings::default()
        };
        assert!(save_settings(&mut conn, &s).is_err());
        assert_eq!(raw(&conn, "pomodoro_min").as_deref(), Some("25"));
    }
}
