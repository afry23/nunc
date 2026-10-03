//! Pausenempfehlung im Flowtime-Modus.

/// Empfohlene Pausenlänge in Minuten: `floor(gearbeitet × ratio)`, aber `0` unter der Mindest-Flow-Zeit.
///
/// Rechnet wie die Go-App (`CalculateBreakDuration`) mit `f64`; die Rundung nach unten gilt für das
/// Gleitkomma-Produkt (z. B. `15 × 0.5 = 7.5 → 7`). Ungültige Verhältnisse (negativ, NaN) ergeben `0` (sättigende Umwandlung nach `u32`).
pub fn calculate_break_duration(worked_min: u32, ratio: f64, min_flow_state_min: u32) -> u32 {
    if worked_min < min_flow_state_min {
        return 0;
    }
    // `as u32` sättigt: NaN und negative Werte ergeben 0.
    (f64::from(worked_min) * ratio).floor() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_the_product() {
        assert_eq!(calculate_break_duration(50, 0.2, 15), 10);
        assert_eq!(calculate_break_duration(15, 0.5, 15), 7);
        assert_eq!(calculate_break_duration(59, 0.1, 15), 5);
        assert_eq!(calculate_break_duration(100, 0.25, 15), 25);
    }

    #[test]
    fn no_break_below_the_minimum_flow_state() {
        assert_eq!(calculate_break_duration(14, 0.2, 15), 0);
        assert_eq!(calculate_break_duration(0, 0.5, 1), 0);
    }

    #[test]
    fn the_minimum_flow_state_itself_counts() {
        assert_eq!(calculate_break_duration(15, 0.2, 15), 3);
        assert_eq!(calculate_break_duration(1, 0.5, 1), 0);
        assert_eq!(calculate_break_duration(2, 0.5, 1), 1);
    }

    #[test]
    fn invalid_ratios_give_no_break() {
        assert_eq!(calculate_break_duration(60, -0.5, 15), 0);
        assert_eq!(calculate_break_duration(60, f64::NAN, 15), 0);
    }
}
