//! Fractional-Index für die Reihenfolge der Todos (ADR 0016): Schlüssel aus den Ziffern `0-9a-z`, die sich
//! lexikografisch sortieren. Zwischen zwei Schlüsseln lässt sich immer ein neuer einfügen, ohne andere
//! Todos zu verschieben; so kollidiert Umordnen auf zwei Geräten nicht.
//!
//! Schlüssel enden nie auf die Ziffer `0` (sonst gäbe es keinen Schlüssel direkt davor).

const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
const BASE: usize = DIGITS.len();

fn digit(byte: u8) -> usize {
    DIGITS.iter().position(|&d| d == byte).unwrap_or(0)
}

/// Ein Schlüssel zwischen `before` und `after` (exklusiv). `None` heißt Anfang bzw. Ende der Liste.
/// Voraussetzung: `before < after` (lexikografisch), sonst ist das Ergebnis nicht definiert.
pub fn position_between(before: Option<&str>, after: Option<&str>) -> String {
    let lower = before.unwrap_or("").as_bytes();
    let upper = after.map(str::as_bytes);
    String::from_utf8(midpoint(lower, upper)).unwrap_or_default()
}

fn midpoint(lower: &[u8], upper: Option<&[u8]>) -> Vec<u8> {
    if let Some(upper) = upper {
        // Gemeinsamen Anfang übernehmen; `lower` gilt dahinter als mit Nullen aufgefüllt.
        let mut common = 0;
        while common < upper.len()
            && lower.get(common).map_or(0, |&byte| digit(byte)) == digit(upper[common])
        {
            common += 1;
        }
        if common > 0 {
            let mut key = upper[..common].to_vec();
            let rest = lower.get(common..).unwrap_or(&[]);
            key.extend(midpoint(rest, Some(&upper[common..])));
            return key;
        }
    }
    let low = lower.first().map_or(0, |&byte| digit(byte));
    let high = upper
        .and_then(|upper| upper.first())
        .map_or(BASE, |&byte| digit(byte));
    if high - low > 1 {
        return vec![DIGITS[(low + high) / 2]];
    }
    // Erste Ziffern liegen direkt nebeneinander.
    match upper {
        Some(upper) if upper.len() > 1 => vec![upper[0]],
        _ => {
            let mut key = vec![DIGITS[low]];
            key.extend(midpoint(lower.get(1..).unwrap_or(&[]), None));
            key
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_key_is_in_the_middle() {
        assert_eq!(position_between(None, None), "i");
    }

    #[test]
    fn keys_after_and_before_stay_ordered() {
        let after = position_between(Some("i"), None);
        assert!("i" < after.as_str());
        let before = position_between(None, Some("i"));
        assert!(before.as_str() < "i");
    }

    #[test]
    fn adjacent_digits_extend_the_key() {
        let key = position_between(Some("a"), Some("b"));
        assert!("a" < key.as_str() && key.as_str() < "b");
        assert_eq!(key, "ai");
    }

    #[test]
    fn a_longer_upper_bound_gives_its_first_digit() {
        assert_eq!(position_between(Some("a"), Some("b5")), "b");
    }

    #[test]
    fn repeated_insertion_at_the_same_spot_keeps_order() {
        let mut upper = "b".to_owned();
        for _ in 0..50 {
            let key = position_between(Some("a"), Some(&upper));
            assert!("a" < key.as_str() && key.as_str() < upper.as_str());
            upper = key;
        }
    }
}
