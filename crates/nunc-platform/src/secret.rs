//! SecretStore: Tokens und Passwörter im Betriebssystem-Tresor (ADR 0005).
//!
//! macOS: Login-Keychain, iOS: Keychain (protected data), Windows: Credential Manager, Linux: Secret Service,
//! Android: Keystore + SharedPreferences. Secrets der Go-App werden nicht gelesen.

use std::collections::HashMap;
use std::sync::Mutex;

use keyring_core::Entry;

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Tresor nicht verfügbar: {0}")]
    Unavailable(String),
    #[error("Tresor-Zugriff fehlgeschlagen: {0}")]
    Access(#[from] keyring_core::Error),
}

/// Ablage für Geheimnisse, getrennt nach Dienst (`nunc`) und Konto (`webdav`, …).
pub trait SecretStore: Send + Sync {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, SecretError>;
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), SecretError>;
    /// Entfernt den Eintrag; kein Fehler, wenn es keinen gibt.
    fn delete(&self, service: &str, account: &str) -> Result<(), SecretError>;
}

/// Tresor über `keyring-core` mit dem Store der Plattform.
#[derive(Debug)]
pub struct KeyringSecretStore;

impl KeyringSecretStore {
    /// Setzt den Standard-Store der Plattform. Einmal beim App-Start aufrufen.
    pub fn init() -> Result<Self, SecretError> {
        platform_store()?;
        Ok(Self)
    }
}

impl SecretStore for KeyringSecretStore {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, SecretError> {
        match Entry::new(service, account)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), SecretError> {
        Ok(Entry::new(service, account)?.set_password(secret)?)
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), SecretError> {
        match Entry::new(service, account)?.delete_credential() {
            Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

/// Ersatz, wenn der Tresor nicht startet (z. B. Linux ohne Secret-Service): jeder Zugriff meldet den Grund,
/// die App läuft weiter.
#[derive(Debug)]
pub struct UnavailableSecretStore {
    reason: String,
}

impl UnavailableSecretStore {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    fn error(&self) -> SecretError {
        SecretError::Unavailable(self.reason.clone())
    }
}

impl SecretStore for UnavailableSecretStore {
    fn get(&self, _: &str, _: &str) -> Result<Option<String>, SecretError> {
        Err(self.error())
    }

    fn set(&self, _: &str, _: &str, _: &str) -> Result<(), SecretError> {
        Err(self.error())
    }

    fn delete(&self, _: &str, _: &str) -> Result<(), SecretError> {
        Err(self.error())
    }
}

/// Tresor im Arbeitsspeicher (Tests).
#[derive(Debug, Default)]
pub struct MemorySecretStore {
    entries: Mutex<HashMap<(String, String), String>>,
}

impl SecretStore for MemorySecretStore {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, SecretError> {
        Ok(self
            .entries
            .lock()
            .map_err(|e| SecretError::Unavailable(e.to_string()))?
            .get(&(service.to_owned(), account.to_owned()))
            .cloned())
    }

    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), SecretError> {
        self.entries
            .lock()
            .map_err(|e| SecretError::Unavailable(e.to_string()))?
            .insert((service.to_owned(), account.to_owned()), secret.to_owned());
        Ok(())
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), SecretError> {
        self.entries
            .lock()
            .map_err(|e| SecretError::Unavailable(e.to_string()))?
            .remove(&(service.to_owned(), account.to_owned()));
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn platform_store() -> Result<(), SecretError> {
    let store = apple_native_keyring_store::keychain::Store::new()
        .map_err(|e| SecretError::Unavailable(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "ios")]
fn platform_store() -> Result<(), SecretError> {
    let store = apple_native_keyring_store::protected::Store::new()
        .map_err(|e| SecretError::Unavailable(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "windows")]
fn platform_store() -> Result<(), SecretError> {
    let store = windows_native_keyring_store::Store::new()
        .map_err(|e| SecretError::Unavailable(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "linux")]
fn platform_store() -> Result<(), SecretError> {
    let store = zbus_secret_service_keyring_store::Store::new()
        .map_err(|e| SecretError::Unavailable(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "android")]
fn platform_store() -> Result<(), SecretError> {
    let store = android_native_keyring_store::Store::new()
        .map_err(|e| SecretError::Unavailable(e.to_string()))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
)))]
fn platform_store() -> Result<(), SecretError> {
    Err(SecretError::Unavailable(
        "kein Tresor für diese Plattform".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_roundtrip() {
        let store = MemorySecretStore::default();
        assert_eq!(store.get("nunc", "password").unwrap(), None);
        store.set("nunc", "password", "tok-äö").unwrap();
        assert_eq!(
            store.get("nunc", "password").unwrap().as_deref(),
            Some("tok-äö")
        );
        assert_eq!(store.get("nunc", "webdav").unwrap(), None);
        store.delete("nunc", "password").unwrap();
        store.delete("nunc", "password").unwrap();
        assert_eq!(store.get("nunc", "password").unwrap(), None);
    }

    #[test]
    fn unavailable_store_reports_the_reason() {
        let store = UnavailableSecretStore::new("kein Daemon");
        let error = store.get("nunc", "x").unwrap_err().to_string();
        assert!(error.contains("kein Daemon"), "{error}");
        assert!(store.set("nunc", "x", "y").is_err());
        assert!(store.delete("nunc", "x").is_err());
    }

    /// Berührt die Login-Keychain (kann einen Dialog auslösen), nutzt nur einen Wegwerf-Eintrag.
    #[test]
    #[ignore = "berührt den echten Tresor des Betriebssystems"]
    fn keyring_roundtrip() {
        let store = KeyringSecretStore::init().unwrap();
        let (service, account) = ("nunc-test", "roundtrip");
        store.set(service, account, "geheim-äöü-123").unwrap();
        assert_eq!(
            store.get(service, account).unwrap().as_deref(),
            Some("geheim-äöü-123")
        );
        store.delete(service, account).unwrap();
        assert_eq!(store.get(service, account).unwrap(), None);
    }
}
