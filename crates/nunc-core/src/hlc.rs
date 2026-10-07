//! Hybrid Logical Clock (ADR 0016): totale Ordnung für Schreibvorgänge verschiedener Geräte.
//!
//! Ein Stempel ist `(wall_ms, counter, device_id)`. Die Wanduhr kommt als Parameter (injizierte `Clock`),
//! nie aus der Systemzeit. Gleichstand entscheidet die `device_id`, damit jeder Merge dasselbe Ergebnis
//! liefert, egal in welcher Reihenfolge zusammengeführt wird.

use serde::{Deserialize, Serialize};

/// So weit darf ein fremder Stempel der eigenen Uhr vorauseilen (24 h), bevor er gekappt wird.
pub const MAX_DRIFT_MS: i64 = 24 * 60 * 60 * 1000;

/// Ein Zeitstempel. Die Feldreihenfolge ist die Ordnung (abgeleitetes `Ord`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Hlc {
    pub wall_ms: i64,
    pub counter: u32,
    pub device_id: String,
}

impl Hlc {
    pub fn new(wall_ms: i64, counter: u32, device_id: impl Into<String>) -> Self {
        Self {
            wall_ms,
            counter,
            device_id: device_id.into(),
        }
    }

    /// Stempel für ein lokales Ereignis: nie kleiner oder gleich dem letzten bekannten Stempel, auch wenn die
    /// Wanduhr stehen bleibt oder zurückspringt.
    pub fn next(last: Option<&Hlc>, now_ms: i64, device_id: &str) -> Hlc {
        match last {
            Some(last) if last.wall_ms >= now_ms => {
                Hlc::new(last.wall_ms, last.counter.saturating_add(1), device_id)
            }
            _ => Hlc::new(now_ms, 0, device_id),
        }
    }

    /// Kappt einen fremden Stempel, der mehr als [`MAX_DRIFT_MS`] in der Zukunft liegt (falsch gehende
    /// Fremduhr), auf die Grenze. Sonst wäre dieser Schreibvorgang für immer unschlagbar.
    pub fn capped(&self, now_ms: i64) -> Hlc {
        let limit = now_ms.saturating_add(MAX_DRIFT_MS);
        if self.wall_ms > limit {
            Hlc::new(limit, 0, self.device_id.clone())
        } else {
            self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_drift_limit_is_exactly_24_hours() {
        assert_eq!(MAX_DRIFT_MS, 86_400_000);
    }

    #[test]
    fn a_stamp_on_the_limit_is_kept_and_one_past_it_is_capped() {
        let now = 1_000;
        let limit = now + MAX_DRIFT_MS;
        let on_limit = Hlc::new(limit, 5, "x");
        assert_eq!(on_limit.capped(now), on_limit);
        let beyond = Hlc::new(limit + 1, 5, "x");
        assert_eq!(beyond.capped(now), Hlc::new(limit, 0, "x"));
    }
}
