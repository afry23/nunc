//! Kompatibilität zur Go-App (`zalando/go-keyring`) auf macOS.
//!
//! Alle Tests sind `#[ignore]`, weil sie die echte Login-Keychain berühren (und macOS beim ersten
//! Zugriff einer unsignierten Test-Binary nach dem Login-Passwort fragen kann).
//!
//! Ausführen: `cargo test -p nunc-platform --test keychain_compat -- --ignored --nocapture`
#![cfg(target_os = "macos")]

use std::io::Write;
use std::process::{Command, Stdio};

use nunc_platform::secret::{KeyringSecretStore, SecretStore};

const SECURITY: &str = "/usr/bin/security";

fn unique(name: &str) -> String {
    format!("nunc-compat-test-{name}-{}", std::process::id())
}

/// Bildet `keyring.Set` aus go-keyring v0.2.6 nach (base64-Präfix, `security -i`).
fn go_set(service: &str, account: &str, secret: &str) {
    use base64::Engine;
    let encoded = format!(
        "go-keyring-base64:{}",
        base64::engine::general_purpose::STANDARD.encode(secret)
    );
    let mut child = Command::new(SECURITY)
        .arg("-i")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(
        child.stdin.take().unwrap(),
        "add-generic-password -U -s '{service}' -a '{account}' -w '{encoded}'"
    )
    .unwrap();
    assert!(child.wait().unwrap().success());
}

/// Bildet `keyring.Get` aus go-keyring nach (ohne Dekodierung, roher Wert).
fn go_get_raw(service: &str, account: &str) -> String {
    let out = Command::new(SECURITY)
        .args(["find-generic-password", "-s", service, "-wa", account])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

fn go_delete(service: &str, account: &str) {
    let _ = Command::new(SECURITY)
        .args(["delete-generic-password", "-s", service, "-a", account])
        .output();
}

#[test]
#[ignore = "berührt die Login-Keychain"]
fn rust_reads_value_written_like_go() {
    let (svc, acc) = (unique("go-to-rust"), "access_token");
    let secret = "tok-äö\n{\"json\":true}";
    go_set(&svc, acc, secret);

    let store = KeyringSecretStore::init().unwrap();
    let read = store.get(&svc, acc).unwrap();
    go_delete(&svc, acc);

    assert_eq!(read.as_deref(), Some(secret));
}

#[test]
#[ignore = "berührt die Login-Keychain"]
fn go_reads_value_written_by_rust() {
    let (svc, acc) = (unique("rust-to-go"), "access_token");
    let store = KeyringSecretStore::init().unwrap();
    store.set(&svc, acc, "plain-token-123").unwrap();

    // go-keyring gibt Werte ohne Präfix unverändert zurück
    let raw = go_get_raw(&svc, acc);
    store.delete(&svc, acc).unwrap();

    assert_eq!(raw, "plain-token-123");
}

#[test]
#[ignore = "berührt die Login-Keychain"]
fn rust_overwrites_entry_created_by_go() {
    let (svc, acc) = (unique("overwrite"), "k");
    go_set(&svc, acc, "alt");
    let store = KeyringSecretStore::init().unwrap();
    store.set(&svc, acc, "neu").unwrap();
    let read = store.get(&svc, acc).unwrap();
    store.delete(&svc, acc).unwrap();
    assert_eq!(read.as_deref(), Some("neu"));
}

/// Prüft nur, OB die echten Go-Einträge vorhanden und dekodierbar sind – Werte werden nicht ausgegeben.
/// Von Hand starten (fragt ggf. nach dem Keychain-Zugriff):
/// `cargo test -p nunc-platform --test keychain_compat real_go_entries -- --ignored --nocapture`
#[test]
#[ignore = "liest echte Einträge der Go-App"]
fn real_go_entries() {
    let store = KeyringSecretStore::init().unwrap();
    for (svc, acc) in [("nunc-todoist", "access_token"), ("nunc", "google-drive")] {
        let state = match store.get(svc, acc) {
            Ok(Some(v)) => format!("vorhanden, dekodiert ({} Zeichen)", v.chars().count()),
            Ok(None) => "nicht vorhanden".to_owned(),
            Err(e) => format!("FEHLER: {e}"),
        };
        println!("{svc}/{acc}: {state}");
    }
    println!("nunc/<webdav-sha256>: Account-Name hängt von URL+User ab, hier nicht geprüft");
}
