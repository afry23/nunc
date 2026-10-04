use serde::{Deserialize, Serialize};
use specta::Type;

use super::{TimerMode, text_enum};

/// Aktiver Sync-Provider (exklusiv).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SyncProvider {
    None,
    GoogleDrive,
    Webdav,
}

text_enum!(SyncProvider, "SyncProvider", {
    None => "none",
    GoogleDrive => "google_drive",
    Webdav => "webdav",
});

impl SyncProvider {
    /// Ob der Anbieter auf der Plattform wählbar ist. Google Drive braucht auf Mobilgeräten Deep-Links und
    /// Plattform-Client-IDs (Backlog nach 1.0); dort gibt es nur „kein Sync“ und WebDAV.
    pub fn is_available(self, mobile: bool) -> bool {
        !(mobile && self == Self::GoogleDrive)
    }
}

/// Alle Benutzereinstellungen. Standardwerte siehe [`Settings::default`] (identisch zu
/// `models.DefaultSettings()` der Go-App).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub pomodoro_min: u32,
    pub short_break_min: u32,
    pub long_break_min: u32,
    pub long_break_after: u32,
    pub auto_start_breaks: bool,
    pub auto_start_next_pomodoro: bool,
    pub notifications_enabled: bool,
    pub sound_enabled: bool,
    /// 0–100.
    pub sound_volume: u32,
    pub sound_end_pomodoro: String,
    pub sound_end_break: String,
    pub active_mode: TimerMode,
    pub system_tray_enabled: bool,
    /// `YYYY-MM-DD`, leer wenn noch nie gesichert.
    pub last_backup_date: String,
    /// Eigener Backup-Ordner, leer = Standard des Betriebssystems.
    pub backup_dir: String,
    /// `false` = Sync pausiert (Zugangsdaten bleiben, kein Zugriff).
    pub sync_enabled: bool,
    pub sync_provider: SyncProvider,
    /// Work-to-Rest-Verhältnis, 0.1–0.5.
    pub flowtime_ratio: f64,
    /// Mindest-Arbeitszeit in Minuten (1–60), ab der eine Pause empfohlen wird.
    pub flowtime_min_flow_state_min: u32,
}

/// Eine verletzte Einstellungsgrenze.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsIssue {
    /// JSON-Name des Feldes (`flowtimeRatio`).
    pub field: String,
    /// Erlaubter Bereich als Text (`0.1–0.5`).
    pub expected: String,
}

impl Settings {
    /// Prüft die festgelegten Wertebereiche (REQUIREMENTS 5): Flowtime-Verhältnis 0,1–0,5, Mindest-Flow-Zeit
    /// 1–60 Minuten, Lautstärke 0–100, Pomodoro-Dauer mindestens eine Minute. Pausenlängen und
    /// `long_break_after` dürfen 0 sein (keine bzw. nie eine lange Pause). Liefert **alle** Verstöße.
    pub fn validate(&self) -> Result<(), Vec<SettingsIssue>> {
        let mut issues = Vec::new();
        let mut check = |ok: bool, field: &str, expected: &str| {
            if !ok {
                issues.push(SettingsIssue {
                    field: field.to_owned(),
                    expected: expected.to_owned(),
                });
            }
        };
        check(self.pomodoro_min >= 1, "pomodoroMin", "≥ 1");
        check(self.sound_volume <= 100, "soundVolume", "0–100");
        check(
            (0.1..=0.5).contains(&self.flowtime_ratio),
            "flowtimeRatio",
            "0.1–0.5",
        );
        check(
            (1..=60).contains(&self.flowtime_min_flow_state_min),
            "flowtimeMinFlowStateMin",
            "1–60",
        );
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            pomodoro_min: 25,
            short_break_min: 5,
            long_break_min: 25,
            long_break_after: 4,
            auto_start_breaks: false,
            auto_start_next_pomodoro: false,
            notifications_enabled: true,
            sound_enabled: true,
            sound_volume: 80,
            sound_end_pomodoro: "bell".to_owned(),
            sound_end_break: "ding".to_owned(),
            active_mode: TimerMode::Pomodoro,
            system_tray_enabled: true,
            last_backup_date: String::new(),
            backup_dir: String::new(),
            sync_enabled: true,
            sync_provider: SyncProvider::None,
            flowtime_ratio: 0.2,
            flowtime_min_flow_state_min: 15,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ratio_that_is_not_a_number_is_invalid() {
        let settings = Settings {
            flowtime_ratio: f64::NAN,
            ..Settings::default()
        };
        let issues = settings.validate().unwrap_err();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].field, "flowtimeRatio");
    }

    #[test]
    fn all_violations_are_reported_together() {
        let settings = Settings {
            pomodoro_min: 0,
            sound_volume: 101,
            flowtime_ratio: 0.9,
            flowtime_min_flow_state_min: 0,
            ..Settings::default()
        };
        let fields: Vec<_> = settings
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|i| i.field)
            .collect();
        assert_eq!(
            fields,
            [
                "pomodoroMin",
                "soundVolume",
                "flowtimeRatio",
                "flowtimeMinFlowStateMin"
            ]
        );
    }

    #[test]
    fn defaults_match_the_go_app() {
        let s = Settings::default();
        assert_eq!(
            (s.pomodoro_min, s.short_break_min, s.long_break_min),
            (25, 5, 25)
        );
        assert_eq!(s.long_break_after, 4);
        assert!(!s.auto_start_breaks && !s.auto_start_next_pomodoro);
        assert!(s.notifications_enabled && s.sound_enabled && s.system_tray_enabled);
        assert_eq!(s.sound_volume, 80);
        assert_eq!(
            (s.sound_end_pomodoro.as_str(), s.sound_end_break.as_str()),
            ("bell", "ding")
        );
        assert_eq!(s.active_mode, TimerMode::Pomodoro);
        assert!(s.last_backup_date.is_empty() && s.backup_dir.is_empty());
        assert!(s.sync_enabled);
        assert_eq!(s.sync_provider, SyncProvider::None);
        assert_eq!(s.flowtime_ratio, 0.2);
        assert_eq!(s.flowtime_min_flow_state_min, 15);
    }

    #[test]
    fn sync_provider_text_values() {
        assert_eq!(SyncProvider::None.as_str(), "none");
        assert_eq!(SyncProvider::GoogleDrive.as_str(), "google_drive");
        assert_eq!(SyncProvider::Webdav.as_str(), "webdav");
        for p in [
            SyncProvider::None,
            SyncProvider::GoogleDrive,
            SyncProvider::Webdav,
        ] {
            assert_eq!(p.as_str().parse::<SyncProvider>(), Ok(p));
        }
        assert!("dropbox".parse::<SyncProvider>().is_err());
    }

    #[test]
    fn google_drive_is_not_available_on_mobile() {
        for provider in [SyncProvider::None, SyncProvider::Webdav] {
            assert!(provider.is_available(true) && provider.is_available(false));
        }
        assert!(SyncProvider::GoogleDrive.is_available(false));
        assert!(!SyncProvider::GoogleDrive.is_available(true));
    }

    #[test]
    fn serializes_with_the_go_json_keys() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        for key in [
            "pomodoroMin",
            "shortBreakMin",
            "longBreakMin",
            "longBreakAfter",
            "autoStartBreaks",
            "autoStartNextPomodoro",
            "notificationsEnabled",
            "soundEnabled",
            "soundVolume",
            "soundEndPomodoro",
            "soundEndBreak",
            "activeMode",
            "systemTrayEnabled",
            "lastBackupDate",
            "backupDir",
            "syncEnabled",
            "syncProvider",
            "flowtimeRatio",
            "flowtimeMinFlowStateMin",
        ] {
            assert!(value.get(key).is_some(), "Schlüssel {key} fehlt");
        }
        assert_eq!(value.as_object().unwrap().len(), 19);
        assert_eq!(value["activeMode"], "pomodoro");
        assert_eq!(value["syncProvider"], "none");
    }

    #[test]
    fn roundtrips_through_json() {
        let s = Settings {
            sync_provider: SyncProvider::GoogleDrive,
            flowtime_ratio: 0.35,
            ..Settings::default()
        };
        let back: Settings = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back, s);
    }
}
