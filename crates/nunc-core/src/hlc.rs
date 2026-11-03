//! Hybrid Logical Clock (ADR 0016): totale Ordnung für Schreibvorgänge verschiedener Geräte.
//!
//! Ein Stempel ist `(wall_ms, counter, device_id)`. Die Wanduhr kommt als Parameter (injizierte `Clock`),
//! nie aus der Systemzeit. Gleichstand entscheidet die `device_id`, damit jeder Merge dasselbe Ergebnis
//! liefert, egal in welcher Reihenfolge zusammengeführt wird.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// So weit darf ein fremder Stempel der eigenen Uhr vorauseilen (24 h), bevor er gekappt wird.
pub const MAX_DRIFT_MS: i64 = 24 * 60 * 60 * 1000;

/// Ein Zeitstempel. Die Feldreihenfolge ist die Ordnung (abgeleitetes `Ord`).
///
/// Im JSON steht er kompakt als `[wall_ms, counter, device_id]` (ab Sync-Format 9); gelesen werden auch die
/// Objekte `{"wall_ms":…,"counter":…,"device_id":…}` älterer Dateien.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// Die beiden Schreibweisen im JSON.
#[derive(Deserialize)]
#[serde(untagged)]
enum Wire {
    Compact(i64, u32, String),
    Object {
        wall_ms: i64,
        counter: u32,
        device_id: String,
    },
}

impl Serialize for Hlc {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (self.wall_ms, self.counter, &self.device_id).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Hlc {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Compact(wall_ms, counter, device_id)
            | Wire::Object {
                wall_ms,
                counter,
                device_id,
            } => Self::new(wall_ms, counter, device_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stamp_is_written_compactly_and_read_in_both_forms() {
        let stamp = Hlc::new(1_700, 3, "dev");
        assert_eq!(serde_json::to_string(&stamp).unwrap(), r#"[1700,3,"dev"]"#);
        let compact: Hlc = serde_json::from_str(r#"[1700,3,"dev"]"#).unwrap();
        let object: Hlc =
            serde_json::from_str(r#"{"wall_ms":1700,"counter":3,"device_id":"dev"}"#).unwrap();
        assert_eq!(compact, stamp);
        assert_eq!(object, stamp);
    }

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
