//! Tagesplaner (ADR 0027): Planungsfelder in der todo.txt-Beschreibung und die Ansicht „Heute“.
//!
//! Ein Todo trägt seine Planung als `key:value`-Wörter in der Beschreibung (todo.txt-Erweiterung): `due:YYYY-MM-DD`
//! (Frist), `plan:YYYY-MM-DD` (für diesen Tag eingeplant) und `est:N` (Schätzung in Minuten). Alles ist pur: nur die
//! Eingabe bestimmt das Ergebnis, den heutigen Tag bringt der Aufrufer mit.

use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::model::{Session, SessionStatus, SessionType, Todo};

/// Planungsfelder eines Todos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlanFields {
    pub due: Option<NaiveDate>,
    pub plan: Option<NaiveDate>,
    pub estimate_min: Option<u32>,
}

/// Höchste Schätzung in Minuten (eine Woche); größere Werte gelten nicht als Schätzung.
pub const MAX_ESTIMATE_MIN: u32 = 7 * 24 * 60;

/// Die Schlüssel der Planungsfelder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanKey {
    Due,
    Plan,
    Estimate,
}

impl PlanKey {
    fn name(self) -> &'static str {
        match self {
            Self::Due => "due",
            Self::Plan => "plan",
            Self::Estimate => "est",
        }
    }
}

fn parse_day(value: &str) -> Option<NaiveDate> {
    // `NaiveDate::parse_from_str` nimmt auch `2026-3-4`; verlangt wird die feste Form.
    (value.len() == 10)
        .then(|| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .flatten()
}

fn parse_estimate(value: &str) -> Option<u32> {
    // `parse` lehnt den leeren Text ab, lässt aber ein führendes `+` zu; verlangt sind nur Ziffern.
    if !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value
        .parse::<u32>()
        .ok()
        .filter(|minutes| (1..=MAX_ESTIMATE_MIN).contains(minutes))
}

/// Liest die Planungsfelder aus der Beschreibung. Kommt ein Schlüssel mehrfach vor, gilt der erste gültige Wert;
/// ungültige Werte (`due:morgen`, `est:0`) zählen nicht und bleiben gewöhnlicher Text.
pub fn plan_fields(description: &str) -> PlanFields {
    let mut fields = PlanFields::default();
    for word in description.split_whitespace() {
        let Some((key, value)) = word.split_once(':') else {
            continue;
        };
        match key {
            "due" if fields.due.is_none() => fields.due = parse_day(value),
            "plan" if fields.plan.is_none() => fields.plan = parse_day(value),
            "est" if fields.estimate_min.is_none() => fields.estimate_min = parse_estimate(value),
            _ => {}
        }
    }
    fields
}

fn is_field(word: &str, key: PlanKey) -> bool {
    let Some((name, value)) = word.split_once(':') else {
        return false;
    };
    name == key.name()
        && match key {
            PlanKey::Estimate => parse_estimate(value).is_some(),
            PlanKey::Due | PlanKey::Plan => parse_day(value).is_some(),
        }
}

/// Setzt, ersetzt oder entfernt ein Planungsfeld in der Beschreibung. Der übrige Text bleibt wortgleich; das Feld
/// steht am Ende, wenn es neu hinzukommt, und behält seinen Platz, wenn es ersetzt wird. `None` entfernt alle
/// gültigen Vorkommen des Schlüssels. Der Wert ist ein Tag bzw. eine Minutenzahl; ein ungültiger Wert ändert nichts.
pub fn with_field(description: &str, key: PlanKey, value: Option<&str>) -> String {
    let valid = value.is_none_or(|v| match key {
        PlanKey::Estimate => parse_estimate(v).is_some(),
        PlanKey::Due | PlanKey::Plan => parse_day(v).is_some(),
    });
    if !valid {
        return description.to_owned();
    }
    let mut out = String::with_capacity(description.len() + 16);
    let mut placed = false;
    for (lead, word) in words(description) {
        if !is_field(word, key) {
            out.push_str(lead);
            out.push_str(word);
        } else if let (Some(v), false) = (value, placed) {
            // erstes Vorkommen: an Ort und Stelle ersetzen; weitere Vorkommen entfallen
            out.push_str(lead);
            out.push_str(&format!("{}:{v}", key.name()));
            placed = true;
        }
    }
    if let (Some(v), false) = (value, placed) {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&format!("{}:{v}", key.name()));
    }
    out
}

/// Wörter samt dem Leerraum davor; Leerraum am Ende fällt weg.
fn words(text: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    let mut rest = text;
    loop {
        let trimmed = rest.trim_start();
        if trimmed.is_empty() {
            return out;
        }
        let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        out.push((&rest[..rest.len() - trimmed.len()], &trimmed[..end]));
        rest = &trimmed[end..];
    }
}

/// Warum ein Todo heute auf dem Plan steht.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PlanReason {
    /// Die Frist liegt vor heute.
    Overdue,
    /// Die Frist ist heute.
    Due,
    /// Für heute oder einen früheren Tag eingeplant (liegen gebliebene Pläne rutschen auf heute).
    Planned,
}

/// Ein Todo auf dem Tagesplan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlanItem {
    pub todo_id: i32,
    pub reason: PlanReason,
    pub due: Option<NaiveDate>,
    pub estimate_min: Option<u32>,
    /// Bisher erfasste Arbeitszeit am Todo in Minuten (abgeschlossene und abgebrochene Arbeitsintervalle).
    pub spent_min: u32,
}

/// Der Plan für einen Tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DayPlan {
    pub day: NaiveDate,
    pub items: Vec<PlanItem>,
    /// Summe der Schätzungen aller Einträge in Minuten.
    pub estimate_min: u32,
    /// Einträge ohne Schätzung.
    pub unestimated: u32,
    /// Tageskapazität in Minuten (Einstellung).
    pub capacity_min: u32,
    /// Kapazität minus Schätzung; negativ, wenn der Plan zu voll ist.
    pub remaining_min: i64,
}

fn reason_of(fields: &PlanFields, today: NaiveDate) -> Option<PlanReason> {
    match fields.due {
        Some(due) if due < today => return Some(PlanReason::Overdue),
        Some(due) if due == today => return Some(PlanReason::Due),
        _ => {}
    }
    fields
        .plan
        .filter(|plan| *plan <= today)
        .map(|_| PlanReason::Planned)
}

/// Arbeitszeit je Todo aus den Sessions (Arbeitsintervalle, nicht übersprungen).
fn spent_by_todo(sessions: &[Session]) -> HashMap<i32, u32> {
    let mut spent = HashMap::new();
    for session in sessions {
        if session.session_type != SessionType::Work || session.status == SessionStatus::Skipped {
            continue;
        }
        if let Some(todo_id) = session.todo_id {
            *spent.entry(todo_id).or_insert(0) += session.actual_min.unwrap_or(0);
        }
    }
    spent
}

/// Baut den Tagesplan für `today`: alle offenen Todos, die überfällig, heute fällig oder für heute (oder früher)
/// eingeplant sind. Reihenfolge: überfällig, heute fällig, eingeplant; darin nach Priorität (ohne zuletzt), dann
/// Position. Erledigte Todos stehen nie auf dem Plan.
pub fn day_plan(
    todos: &[Todo],
    sessions: &[Session],
    today: NaiveDate,
    capacity_min: u32,
) -> DayPlan {
    let spent = spent_by_todo(sessions);
    let mut items: Vec<(PlanItem, Option<char>, u32)> = todos
        .iter()
        .filter(|todo| !todo.completed)
        .filter_map(|todo| {
            let fields = plan_fields(&todo.description);
            let reason = reason_of(&fields, today)?;
            let item = PlanItem {
                todo_id: todo.id,
                reason,
                due: fields.due,
                estimate_min: fields.estimate_min,
                spent_min: spent.get(&todo.id).copied().unwrap_or(0),
            };
            Some((item, todo.priority, todo.position))
        })
        .collect();
    let rank = |reason: PlanReason| match reason {
        PlanReason::Overdue => 0,
        PlanReason::Due => 1,
        PlanReason::Planned => 2,
    };
    items.sort_by_key(|(item, priority, position)| {
        (
            rank(item.reason),
            priority.map_or(u32::MAX, u32::from),
            *position,
            item.todo_id,
        )
    });
    let items: Vec<PlanItem> = items.into_iter().map(|(item, _, _)| item).collect();
    let estimate_min = items.iter().filter_map(|item| item.estimate_min).sum();
    let unestimated = items.iter().filter(|i| i.estimate_min.is_none()).count() as u32;
    DayPlan {
        day: today,
        items,
        estimate_min,
        unestimated,
        capacity_min,
        remaining_min: i64::from(capacity_min) - i64::from(estimate_min),
    }
}
