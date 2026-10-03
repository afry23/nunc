use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::text_enum;

/// Zeitliche Auflösung einer Statistikabfrage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum StatsPeriod {
    Day,
    Week,
    Month,
    Year,
}

text_enum!(StatsPeriod, "StatsPeriod", {
    Day => "day",
    Week => "week",
    Month => "month",
    Year => "year",
});

/// Parameter einer Statistikabfrage (Datumsgrenzen inklusive).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatsRequest {
    pub period: StatsPeriod,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// Kennzahlen für einen Kalendertag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DailyStats {
    pub date: NaiveDate,
    /// Abgeschlossene Pomodoros.
    pub pomodoro_count: u32,
    pub work_minutes: u32,
    pub break_minutes: u32,
}

/// Arbeitszeit pro `+project`-Tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStats {
    pub project: String,
    pub work_minutes: u32,
}

/// Statistik-Antwort an das Frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub daily: Vec<DailyStats>,
    pub projects: Vec<ProjectStats>,
    pub total_pomodoros: u32,
    pub total_work_min: u32,
    /// Aufeinanderfolgende Tage mit mindestens einem Pomodoro.
    pub current_streak: u32,
    pub longest_streak: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_text_values() {
        for (p, text) in [
            (StatsPeriod::Day, "day"),
            (StatsPeriod::Week, "week"),
            (StatsPeriod::Month, "month"),
            (StatsPeriod::Year, "year"),
        ] {
            assert_eq!(p.as_str(), text);
            assert_eq!(text.parse::<StatsPeriod>(), Ok(p));
        }
        assert!("decade".parse::<StatsPeriod>().is_err());
    }

    #[test]
    fn request_uses_iso_dates_and_camel_case() {
        let req: StatsRequest = serde_json::from_str(
            r#"{"period":"week","startDate":"2026-09-21","endDate":"2026-09-27"}"#,
        )
        .unwrap();
        assert_eq!(req.period, StatsPeriod::Week);
        assert_eq!(
            req.start_date,
            NaiveDate::from_ymd_opt(2026, 9, 21).unwrap()
        );
        assert_eq!(req.end_date, NaiveDate::from_ymd_opt(2026, 9, 27).unwrap());
    }

    #[test]
    fn response_serializes_with_the_go_json_keys() {
        let resp = StatsResponse {
            daily: vec![DailyStats {
                date: NaiveDate::from_ymd_opt(2026, 9, 22).unwrap(),
                pomodoro_count: 2,
                work_minutes: 60,
                break_minutes: 6,
            }],
            projects: vec![ProjectStats {
                project: "+work".into(),
                work_minutes: 60,
            }],
            total_pomodoros: 2,
            total_work_min: 60,
            current_streak: 1,
            longest_streak: 3,
        };
        let v = serde_json::to_value(&resp).unwrap();
        assert_eq!(v["daily"][0]["pomodoroCount"], 2);
        assert_eq!(v["daily"][0]["breakMinutes"], 6);
        assert_eq!(v["projects"][0]["workMinutes"], 60);
        assert_eq!(v["totalPomodoros"], 2);
        assert_eq!(v["totalWorkMin"], 60);
        assert_eq!(v["currentStreak"], 1);
        assert_eq!(v["longestStreak"], 3);
    }
}
