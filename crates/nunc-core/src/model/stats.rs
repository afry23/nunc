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
    /// Arbeitszeit der Pomodoro-Sessions (auch abgebrochener); `pomodoro_minutes + flowtime_minutes +
    /// stopwatch_minutes == work_minutes`.
    pub pomodoro_minutes: u32,
    /// Abgeschlossene Flowtime-Arbeitssessions.
    pub flowtime_count: u32,
    pub flowtime_minutes: u32,
    /// Abgeschlossene Stopwatch-Arbeitssessions (auch nachgetragene Einträge, ADR 0018).
    pub stopwatch_count: u32,
    pub stopwatch_minutes: u32,
}

/// Arbeitszeit pro `+project`-Tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStats {
    pub project: String,
    pub work_minutes: u32,
}

/// Arbeitszeit pro `@context`-Tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContextStats {
    pub context: String,
    pub work_minutes: u32,
}

/// Abgeschlossene Pomodoros und Arbeitszeit eines Todos im Zeitraum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TodoStats {
    pub todo_id: i32,
    pub description: String,
    pub pomodoro_count: u32,
    pub work_minutes: u32,
}

/// Kennzahlen je Wochentag (`0` = Montag … `6` = Sonntag) im Zeitraum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WeekdayStats {
    pub weekday: u8,
    pub pomodoro_count: u32,
    pub work_minutes: u32,
}

/// Kennzahlen je Kalendermonat im Zeitraum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyStats {
    pub year: i32,
    pub month: u32,
    pub pomodoro_count: u32,
    pub work_minutes: u32,
}

/// Statistik-Antwort an das Frontend.
///
/// `daily`, `projects`, `contexts`, `top_todos`, `weekdays` und `monthly` beziehen sich auf den angefragten
/// Zeitraum; die Summen, Durchschnitte und Serien auf alle Sessions bzw. auf „heute“.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    /// Nur Tage mit Sessions, aufsteigend.
    pub daily: Vec<DailyStats>,
    /// Absteigend nach Arbeitszeit, dann nach Name.
    pub projects: Vec<ProjectStats>,
    pub contexts: Vec<ContextStats>,
    /// Die zehn Todos mit den meisten abgeschlossenen Pomodoros.
    pub top_todos: Vec<TodoStats>,
    /// Immer sieben Einträge, Montag bis Sonntag.
    pub weekdays: Vec<WeekdayStats>,
    /// Nur Monate mit Sessions, aufsteigend.
    pub monthly: Vec<MonthlyStats>,
    /// Abgeschlossene Pomodoros insgesamt.
    pub total_pomodoros: u32,
    /// Arbeitszeit insgesamt in Minuten.
    pub total_work_min: u32,
    /// Pomodoros pro Tag in den letzten 7 Tagen (inklusive heute).
    pub average_pomodoros_7: f64,
    /// Pomodoros pro Tag in den letzten 30 Tagen (inklusive heute).
    pub average_pomodoros_30: f64,
    /// Aufeinanderfolgende Tage mit mindestens einem Pomodoro, endend heute oder gestern.
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
    fn response_serializes_with_camel_case_keys() {
        let resp = StatsResponse {
            daily: vec![DailyStats {
                date: NaiveDate::from_ymd_opt(2026, 9, 22).unwrap(),
                pomodoro_count: 2,
                work_minutes: 60,
                break_minutes: 6,
                pomodoro_minutes: 50,
                flowtime_count: 1,
                flowtime_minutes: 10,
                stopwatch_count: 0,
                stopwatch_minutes: 0,
            }],
            projects: vec![ProjectStats {
                project: "+work".into(),
                work_minutes: 60,
            }],
            contexts: vec![ContextStats {
                context: "@desk".into(),
                work_minutes: 60,
            }],
            top_todos: vec![TodoStats {
                todo_id: 1,
                description: "Report".into(),
                pomodoro_count: 2,
                work_minutes: 60,
            }],
            weekdays: vec![WeekdayStats {
                weekday: 0,
                pomodoro_count: 2,
                work_minutes: 60,
            }],
            monthly: vec![MonthlyStats {
                year: 2026,
                month: 9,
                pomodoro_count: 2,
                work_minutes: 60,
            }],
            total_pomodoros: 2,
            total_work_min: 60,
            average_pomodoros_7: 0.5,
            average_pomodoros_30: 0.25,
            current_streak: 1,
            longest_streak: 3,
        };
        let v = serde_json::to_value(&resp).unwrap();
        assert_eq!(v["daily"][0]["pomodoroCount"], 2);
        assert_eq!(v["daily"][0]["breakMinutes"], 6);
        assert_eq!(v["projects"][0]["workMinutes"], 60);
        assert_eq!(v["contexts"][0]["context"], "@desk");
        assert_eq!(v["topTodos"][0]["todoId"], 1);
        assert_eq!(v["weekdays"][0]["weekday"], 0);
        assert_eq!(v["monthly"][0]["month"], 9);
        assert_eq!(v["totalPomodoros"], 2);
        assert_eq!(v["totalWorkMin"], 60);
        assert_eq!(v["averagePomodoros7"], 0.5);
        assert_eq!(v["averagePomodoros30"], 0.25);
        assert_eq!(v["currentStreak"], 1);
        assert_eq!(v["longestStreak"], 3);
    }
}
