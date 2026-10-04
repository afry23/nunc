//! Echte Systemzeit und Umrechnung in die lokale Zeitzone.

use std::sync::Mutex;

use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, Utc};
use nunc_core::clock::Clock;

/// Die Uhr des Betriebssystems.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Von Hand gestellte Uhr (Tests).
#[derive(Debug)]
pub struct ManualClock {
    now: Mutex<DateTime<Utc>>,
}

impl ManualClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    pub fn set(&self, now: DateTime<Utc>) {
        if let Ok(mut current) = self.now.lock() {
            *current = now;
        }
    }

    pub fn advance(&self, by: Duration) {
        if let Ok(mut current) = self.now.lock() {
            *current += by;
        }
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        self.now.lock().map(|now| *now).unwrap_or_default()
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
    fn manual_clock_moves_only_when_told() {
        let start = Utc.with_ymd_and_hms(2026, 10, 4, 9, 0, 0).unwrap();
        let clock = ManualClock::new(start);
        assert_eq!(clock.now(), start);
        clock.advance(Duration::minutes(5));
        assert_eq!(clock.now(), start + Duration::minutes(5));
        clock.set(start);
        assert_eq!(clock.now(), start);
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
