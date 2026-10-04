//! Echte Systemzeit und Umrechnung in die lokale Zeitzone.

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use nunc_core::clock::Clock;

/// Die Uhr des Betriebssystems.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Lokale Wanduhrzeit eines Zeitpunkts (Zeitzone des Geräts).
pub fn to_local(time: DateTime<Utc>) -> NaiveDateTime {
    time.with_timezone(&Local).naive_local()
}

/// Lokaler Kalendertag eines Zeitpunkts.
pub fn to_local_date(time: DateTime<Utc>) -> NaiveDate {
    to_local(time).date()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn system_clock_is_close_to_utc_now() {
        let before = Utc::now();
        let now = SystemClock.now();
        assert!(now >= before && now - before < chrono::Duration::seconds(5));
    }

    #[test]
    fn local_conversion_keeps_the_instant() {
        let utc = Utc.with_ymd_and_hms(2026, 6, 15, 12, 0, 0).unwrap();
        let local = to_local(utc);
        let offset = Local
            .from_utc_datetime(&utc.naive_utc())
            .offset()
            .local_minus_utc();
        assert_eq!(
            local,
            utc.naive_utc() + chrono::Duration::seconds(i64::from(offset))
        );
        assert_eq!(to_local_date(utc), local.date());
    }
}
