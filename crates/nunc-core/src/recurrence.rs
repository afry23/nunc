//! Wiederkehrende Aufgaben (ADR 0036): das Planungswort `rec:[+]<n><d|b|w|m|y>` und die Berechnung der Folgeaufgabe.
//!
//! `rec:1w` setzt die nächste Frist eine Woche nach dem Abschlussdatum, `rec:+1w` eine Woche nach der bisherigen
//! Frist (todo.txt-Standard, „strikt“). Alles ist pur: das Abschlussdatum bringt der Aufrufer mit.

use std::fmt;

use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::day_plan::{PlanKey, plan_fields, with_field};

/// Größte Anzahl je Einheit.
pub const MAX_COUNT: u32 = 999;

/// Einheit des Intervalls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Day,
    BusinessDay,
    Week,
    Month,
    Year,
}

impl Unit {
    fn letter(self) -> char {
        match self {
            Self::Day => 'd',
            Self::BusinessDay => 'b',
            Self::Week => 'w',
            Self::Month => 'm',
            Self::Year => 'y',
        }
    }
}

/// Eine Wiederholungsregel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Recurrence {
    /// `true`: ab der bisherigen Frist (`rec:+…`), sonst ab dem Abschlussdatum.
    pub strict: bool,
    pub count: u32,
    pub unit: Unit,
}

impl fmt::Display for Recurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let plus = if self.strict { "+" } else { "" };
        write!(f, "{plus}{}{}", self.count, self.unit.letter())
    }
}

/// Liest den Wert eines `rec:`-Worts (`+1w`, `3b`). Ungültiges (`0d`, `1x`, `+`, `1000d`) ist `None`.
pub fn parse(value: &str) -> Option<Recurrence> {
    let (strict, rest) = match value.strip_prefix('+') {
        Some(rest) => (true, rest),
        None => (false, value),
    };
    let unit = match rest.chars().last()? {
        'd' => Unit::Day,
        'b' => Unit::BusinessDay,
        'w' => Unit::Week,
        'm' => Unit::Month,
        'y' => Unit::Year,
        _ => return None,
    };
    let digits = &rest[..rest.len() - 1];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let count = digits.parse::<u32>().ok()?;
    (1..=MAX_COUNT).contains(&count).then_some(Recurrence {
        strict,
        count,
        unit,
    })
}

/// Die Wiederholung aus der Beschreibung; bei mehreren gilt die erste gültige.
pub fn recurrence_of(description: &str) -> Option<Recurrence> {
    description
        .split_whitespace()
        .filter_map(|word| word.strip_prefix("rec:"))
        .find_map(parse)
}

/// Setzt, ersetzt oder entfernt `rec:` in der Beschreibung (übriger Text bleibt wortgleich).
pub fn with_recurrence(description: &str, rule: Option<Recurrence>) -> String {
    with_field(
        description,
        PlanKey::Recurrence,
        rule.map(|r| r.to_string()).as_deref(),
    )
}

fn is_weekend(day: NaiveDate) -> bool {
    matches!(day.weekday(), Weekday::Sat | Weekday::Sun)
}

fn add_business_days(mut day: NaiveDate, count: u32) -> NaiveDate {
    let mut left = count;
    while left > 0 {
        day = day.succ_opt().unwrap_or(day);
        if !is_weekend(day) {
            left -= 1;
        }
    }
    day
}

/// Die nächste Frist: Basis ist bei `strict` die bisherige Frist (fehlt sie, das Abschlussdatum), sonst das
/// Abschlussdatum. Monate und Jahre enden am letzten Tag des Zielmonats, wenn der Tag dort fehlt (31.1. + 1m = 28.2.).
/// Läuft das Datum über den darstellbaren Bereich, bleibt die Basis.
pub fn next_due(rule: Recurrence, due: Option<NaiveDate>, done_on: NaiveDate) -> NaiveDate {
    let base = if rule.strict {
        due.unwrap_or(done_on)
    } else {
        done_on
    };
    let next = match rule.unit {
        Unit::Day => base.checked_add_days(Days::new(u64::from(rule.count))),
        Unit::BusinessDay => Some(add_business_days(base, rule.count)),
        Unit::Week => base.checked_add_days(Days::new(u64::from(rule.count) * 7)),
        Unit::Month => base.checked_add_months(Months::new(rule.count)),
        Unit::Year => base.checked_add_months(Months::new(rule.count * 12)),
    };
    next.unwrap_or(base)
}

/// Beschreibung der Folgeaufgabe zu einer abgehakten Aufgabe: gleiche Zeile, neue Frist, `plan:` entfällt (der
/// eingeplante Tag gehörte zum alten Durchlauf). `None`, wenn die Aufgabe keine gültige Wiederholung trägt.
pub fn follow_up_description(description: &str, done_on: NaiveDate) -> Option<String> {
    let rule = recurrence_of(description)?;
    let due = next_due(rule, plan_fields(description).due, done_on);
    let next = with_field(
        description,
        PlanKey::Due,
        Some(&due.format("%Y-%m-%d").to_string()),
    );
    Some(with_field(&next, PlanKey::Plan, None))
}

/// Stabile Sync-Kennung der Folgeaufgabe (UUID-förmig) aus der Kennung der Vorlage und dem Zieldatum. Zwei Geräte,
/// die dieselbe Aufgabe unabhängig abhaken, erzeugen damit dieselbe Folgeaufgabe; der Sync-Merge führt sie zusammen.
pub fn follow_up_sync_id(template_sync_id: &str, next_due: &str) -> String {
    // FNV-1a mit 128 Bit: einfach, stabil über Plattformen und Rust-Versionen.
    const OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;
    let mut hash = OFFSET;
    for byte in template_sync_id
        .bytes()
        .chain(*b"|rec|")
        .chain(next_due.bytes())
    {
        hash ^= u128::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    let hex = format!("{hash:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}
