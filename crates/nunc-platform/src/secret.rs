//! SecretStore: Tokens und Passwörter im Betriebssystem-Tresor.
//!
//! macOS: Login-Keychain (gleiche Einträge wie `go-keyring` der Go-App), iOS: Keychain
//! (protected data), Windows: Credential Manager, Linux: Secret Service,
//! Android: Keystore + SharedPreferences.

use base64::Engine;
use keyring_core::Entry;

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Tresor nicht verfügbar: {0}")]
    Store(String),
    #[error("Tresor-Zugriff fehlgeschlagen: {0}")]
    Access(#[from] keyring_core::Error),
    #[error("Wert konnte nicht dekodiert werden: {0}")]
    Decode(String),
}

pub trait SecretStore: Send + Sync {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, SecretError>;
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), SecretError>;
    fn delete(&self, service: &str, account: &str) -> Result<(), SecretError>;
}

const GO_BASE64_PREFIX: &str = "go-keyring-base64:";
const GO_HEX_PREFIX: &str = "go-keyring-encoded:";

/// Dekodiert Werte, die `zalando/go-keyring` (macOS) geschrieben hat.
///
/// `go-keyring` v0.2.x speichert mit Präfix `go-keyring-base64:`, ältere Versionen mit
/// `go-keyring-encoded:` (Hex). Werte ohne Präfix werden unverändert zurückgegeben.
pub fn decode_go_keyring(raw: &str) -> Result<String, SecretError> {
    let bytes = if let Some(b64) = raw.strip_prefix(GO_BASE64_PREFIX) {
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| SecretError::Decode(e.to_string()))?
    } else if let Some(hex) = raw.strip_prefix(GO_HEX_PREFIX) {
        decode_hex(hex).ok_or_else(|| SecretError::Decode("ungültiges Hex".into()))?
    } else {
        return Ok(raw.to_owned());
    };
    String::from_utf8(bytes).map_err(|e| SecretError::Decode(e.to_string()))
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

/// Tresor über `keyring-core` mit plattformspezifischem Store.
pub struct KeyringSecretStore;

impl KeyringSecretStore {
    /// Setzt den Standard-Store der Plattform. Einmal beim App-Start aufrufen.
    pub fn init() -> Result<Self, SecretError> {
        platform_store()?;
        Ok(Self)
    }

    fn entry(service: &str, account: &str) -> Result<Entry, SecretError> {
        Ok(Entry::new(service, account)?)
    }
}

impl SecretStore for KeyringSecretStore {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, SecretError> {
        match Self::entry(service, account)?.get_password() {
            Ok(raw) => decode_go_keyring(&raw).map(Some),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), SecretError> {
        Ok(Self::entry(service, account)?.set_password(secret)?)
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), SecretError> {
        match Self::entry(service, account)?.delete_credential() {
            Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Schreiben, lesen, löschen mit einem Wegwerf-Eintrag; Ergebnis als Text (für Spikes/Diagnose).
pub fn selftest() -> Result<String, SecretError> {
    let store = KeyringSecretStore::init()?;
    let (svc, acc) = ("nunc-selftest", "roundtrip");
    store.set(svc, acc, "geheim-äöü-123")?;
    let read = store.get(svc, acc)?;
    store.delete(svc, acc)?;
    let gone = store.get(svc, acc)?;
    Ok(format!("read={read:?} after_delete={gone:?}"))
}

#[cfg(target_os = "macos")]
fn platform_store() -> Result<(), SecretError> {
    let store = apple_native_keyring_store::keychain::Store::new()
        .map_err(|e| SecretError::Store(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "ios")]
fn platform_store() -> Result<(), SecretError> {
    let store = apple_native_keyring_store::protected::Store::new()
        .map_err(|e| SecretError::Store(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "windows")]
fn platform_store() -> Result<(), SecretError> {
    let store = windows_native_keyring_store::Store::new()
        .map_err(|e| SecretError::Store(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "linux")]
fn platform_store() -> Result<(), SecretError> {
    let store = zbus_secret_service_keyring_store::Store::new()
        .map_err(|e| SecretError::Store(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "android")]
fn platform_store() -> Result<(), SecretError> {
    let store = android_native_keyring_store::Store::new()
        .map_err(|e| SecretError::Store(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_go_base64() {
        let raw = format!("{GO_BASE64_PREFIX}{}", "dG9rZW4tw6TDtg==");
        assert_eq!(decode_go_keyring(&raw).unwrap(), "token-äö");
    }

    #[test]
    fn decodes_go_hex() {
        assert_eq!(
            decode_go_keyring("go-keyring-encoded:616263").unwrap(),
            "abc"
        );
    }

    #[test]
    fn passes_plain_values_through() {
        assert_eq!(decode_go_keyring("plain").unwrap(), "plain");
    }

    #[test]
    fn rejects_broken_encodings() {
        assert!(decode_go_keyring("go-keyring-encoded:zz").is_err());
        assert!(decode_go_keyring("go-keyring-base64:!!!").is_err());
    }
}
