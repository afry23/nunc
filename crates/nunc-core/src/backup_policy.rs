//! Backup-Regeln (pur): wann gesichert wird, wie die Dateien heißen, welche rotiert werden und wie der
//! Zustand des Backup-Ordners aussieht. Dateizugriff, `VACUUM INTO` und Restore liegen in `nunc-store` (M2).
//!
//! Verhalten der Go-App (`services/backup.go`), mit einer Verschärfung: Als Backup gilt nur der exakte Name
//! `nunc-backup-YYYY-MM-DD.db` mit gültigem Datum. Andere Dateien (`before-restore-…`, selbst benannte
//! Dateien, Datumsangaben wie `2025-13-40`) werden weder gelistet noch gezählt noch rotiert. Go erfasste per
//! Glob alles mit dem Präfix `nunc-backup-` und konnte so fremde Dateien löschen.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

/// Höchstzahl der aufbewahrten täglichen Backups.
pub const MAX_BACKUPS: usize = 7;

/// Ab diesem Alter des letzten Backups warnt die Oberfläche.
pub const OUTDATED_AFTER_HOURS: i64 = 48;

const PREFIX: &str = "nunc-backup-";
const EMERGENCY_PREFIX: &str = "before-restore-";
const SUFFIX: &str = ".db";

/// Eine Datei im Backup-Ordner, wie sie der Store meldet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupFile {
    pub name: String,
    /// Vollständiger Pfad (für die Anzeige; die Regeln lesen nur `name`).
    pub path: String,
    pub size_bytes: u64,
    pub modified_at: DateTime<Utc>,
}

/// Einordnung eines Backups nach Alter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum BackupCategory {
    Daily,
    Weekly,
    Monthly,
}

/// Ein Backup für die Liste in den Einstellungen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    pub filename: String,
    pub path: String,
    /// 12:00 Uhr UTC am Datum im Dateinamen.
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub category: BackupCategory,
}

/// Zustand des Backup-Ordners.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    /// Änderungszeit der neuesten Backup-Datei; `None` ohne Backups.
    pub last_backup_at: Option<DateTime<Utc>>,
    pub backup_count: u32,
    pub total_size_bytes: u64,
    /// Kein Backup vorhanden oder das letzte älter als 48 Stunden.
    pub is_outdated: bool,
}

/// Dateiname des täglichen Backups.
pub fn backup_file_name(date: NaiveDate) -> String {
    format!("{PREFIX}{}{SUFFIX}", date.format("%Y-%m-%d"))
}

/// Dateiname der Sicherung vor einem Restore (wird am selben Tag überschrieben).
pub fn emergency_backup_file_name(date: NaiveDate) -> String {
    format!("{EMERGENCY_PREFIX}{}{SUFFIX}", date.format("%Y-%m-%d"))
}

/// Datum aus einem Backup-Dateinamen; `None`, wenn der Name nicht dem Muster entspricht.
pub fn parse_backup_date(file_name: &str) -> Option<NaiveDate> {
    let date = file_name.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
    if date.len() != 10 {
        return None;
    }
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// Ob beim Start gesichert werden muss: außer wenn das gespeicherte Datum (`YYYY-MM-DD`, leer = nie) heute ist.
pub fn needs_backup(last_backup_date: &str, today: NaiveDate) -> bool {
    last_backup_date != today.format("%Y-%m-%d").to_string()
}

fn dated(files: &[BackupFile]) -> Vec<(NaiveDate, &BackupFile)> {
    let mut dated: Vec<_> = files
        .iter()
        .filter_map(|file| parse_backup_date(&file.name).map(|date| (date, file)))
        .collect();
    dated.sort_by_key(|(date, _)| *date);
    dated
}

/// Namen der Backups, die gelöscht werden müssen, damit höchstens `max` übrig bleiben (die ältesten zuerst).
pub fn files_to_delete(files: &[BackupFile], max: usize) -> Vec<String> {
    let dated = dated(files);
    let excess = dated.len().saturating_sub(max);
    dated[..excess]
        .iter()
        .map(|(_, file)| file.name.clone())
        .collect()
}

/// Zustand des Ordners zum Zeitpunkt `now`.
pub fn backup_status(files: &[BackupFile], now: DateTime<Utc>) -> BackupStatus {
    let backups = dated(files);
    let last_backup_at = backups.iter().map(|(_, file)| file.modified_at).max();
    BackupStatus {
        last_backup_at,
        backup_count: backups.len() as u32,
        total_size_bytes: backups.iter().map(|(_, file)| file.size_bytes).sum(),
        is_outdated: last_backup_at
            .is_none_or(|last| now - last > Duration::hours(OUTDATED_AFTER_HOURS)),
    }
}

/// Alter in Tagen → Kategorie: unter 7 täglich, unter 28 wöchentlich, sonst monatlich.
fn category(age_days: i64) -> BackupCategory {
    match age_days {
        ..7 => BackupCategory::Daily,
        7..28 => BackupCategory::Weekly,
        _ => BackupCategory::Monthly,
    }
}

/// Backups für die Liste, neueste zuerst.
pub fn list_backups(files: &[BackupFile], today: NaiveDate) -> Vec<BackupEntry> {
    let mut entries: Vec<BackupEntry> = dated(files)
        .into_iter()
        .map(|(date, file)| BackupEntry {
            filename: file.name.clone(),
            path: file.path.clone(),
            created_at: (date.and_time(NaiveTime::MIN) + Duration::hours(12)).and_utc(),
            size_bytes: file.size_bytes,
            category: category((today - date).num_days()),
        })
        .collect();
    entries.reverse();
    entries
}
