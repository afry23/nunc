//! Zeitformate der Go-App: Zeitpunkte als RFC 3339 in UTC (`2026-09-22T09:00:00Z`), Tage als `YYYY-MM-DD`.

use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};

pub(crate) fn to_text(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub(crate) fn parse_instant(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

/// Datum aus `YYYY-MM-DD`; längere Texte (`2026-09-18T00:00:00Z`, so liefert der Go-Treiber DATE-Spalten)
/// zählen über ihre ersten zehn Zeichen. Unlesbares ist `None`.
pub(crate) fn parse_day(text: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(text.get(..10)?, "%Y-%m-%d").ok()
}

pub(crate) fn day_text(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instants_roundtrip_in_go_format() {
        let t = parse_instant("2026-09-22T09:00:00Z").unwrap();
        assert_eq!(to_text(t), "2026-09-22T09:00:00Z");
        assert_eq!(
            to_text(parse_instant("2026-09-22T11:00:00+02:00").unwrap()),
            "2026-09-22T09:00:00Z"
        );
        assert!(parse_instant("gestern").is_none());
    }

    #[test]
    fn days_accept_plain_and_rfc3339_text() {
        let d = NaiveDate::from_ymd_opt(2026, 9, 18).unwrap();
        assert_eq!(parse_day("2026-09-18"), Some(d));
        assert_eq!(parse_day("2026-09-18T00:00:00Z"), Some(d));
        assert_eq!(parse_day("2026-9-1"), None);
        assert_eq!(parse_day(""), None);
        assert_eq!(day_text(d), "2026-09-18");
    }
}
