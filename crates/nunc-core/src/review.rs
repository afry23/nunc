//! Rückblick-Bericht für einen Zeitraum (ADR 0025), pur aus den Roh-Sessions und Todos.
//!
//! Fokuszeit zählt nur **abgeschlossene** Arbeitssessions; abgebrochene gehen in die Abbruchquote ein.
//! Ein Tag ist der lokale Kalendertag des Beginns (`local_date` kommt vom Aufrufer).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::{Session, Todo};

/// Art eines Kalenderzeitraums.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PeriodKind {
    Week,
    Month,
    Quarter,
}

/// Zeitraum in lokalen Tagen, beide Grenzen inklusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

/// Summe einer Gruppe (`+projekt`, `@kontext`); `None` für Zeit ohne Tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GroupMinutes {
    pub group: Option<String>,
    pub minutes: u32,
}

/// Im Zeitraum erledigte Aufgabe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CompletedTodo {
    pub completed_at: NaiveDate,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub period: Period,
    pub previous_period: Period,
    pub focus_min: u32,
    pub previous_focus_min: u32,
    pub completed_sessions: u32,
    pub aborted_sessions: u32,
    pub abort_rate_percent: u32,
    pub projects: Vec<GroupMinutes>,
    pub contexts: Vec<GroupMinutes>,
    pub completed_todos: Vec<CompletedTodo>,
}

/// Kalenderzeitraum (Woche ab Montag, Monat, Quartal), der `day` enthält.
pub fn period_containing(_kind: PeriodKind, _day: NaiveDate) -> Period {
    todo!("ADR 0025: noch nicht implementiert")
}

/// Berechnet den Bericht; die Vorperiode ist der Zeitraum gleicher Länge direkt davor.
pub fn build_review(
    _period: Period,
    _sessions: &[Session],
    _todos: &[Todo],
    _local_date: &dyn Fn(DateTime<Utc>) -> NaiveDate,
) -> Review {
    todo!("ADR 0025: noch nicht implementiert")
}

/// Markdown für die Gesprächsvorbereitung.
pub fn review_markdown(_review: &Review) -> String {
    todo!("ADR 0025: noch nicht implementiert")
}
