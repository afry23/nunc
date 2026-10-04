//! Zugriffstoken der Schnittstelle: 32 Zufallsbytes, URL-sicher als Base64 ohne Auffüllung (43 Zeichen).

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// Anzahl Zufallsbytes (256 Bit).
pub const TOKEN_BYTES: usize = 32;

/// Erzeugt ein neues Token aus dem Zufallsgenerator des Betriebssystems.
pub fn generate_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_long_url_safe_and_different_every_time() {
        let a = generate_token().unwrap();
        let b = generate_token().unwrap();
        assert_eq!(a.len(), 43);
        assert!(
            a.bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        );
        assert_ne!(a, b);
    }
}
