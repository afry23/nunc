//! Gemeinsamer HTTP-Client für WebDAV.

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use crate::SyncError;

/// HTTP-Client für WebDAV.
///
/// Vertraut neben den eingebauten Wurzeln auch dem Zertifikatsspeicher des Systems (Firmen-Proxys wie
/// Zscaler signieren mit einer CA, die nur dort liegt). Der Client wird einmal gebaut und danach
/// geklont, weil das Laden des Speichers ~100 ms dauert.
pub fn http_client() -> Result<reqwest::Client, SyncError> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    CLIENT
        .get_or_init(build_client)
        .clone()
        .map_err(SyncError::Transport)
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .use_preconfigured_tls(Arc::unwrap_or_clone(tls_config()?))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| error.to_string())
}

fn tls_config() -> Result<Arc<rustls::ClientConfig>, String> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let native = rustls_native_certs::load_native_certs();
    for error in &native.errors {
        log::warn!("Systemzertifikate: {error}");
    }
    // Einzelne ungültige Systemzertifikate werden übersprungen (bei Firmen-Stores nicht selten).
    let (added, ignored) = roots.add_parsable_certificates(native.certs);
    log::debug!("Systemzertifikate: {added} geladen, {ignored} übersprungen");
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| error.to_string())?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Arc::new(config))
}
