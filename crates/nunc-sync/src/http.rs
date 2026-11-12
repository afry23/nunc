//! HTTP-Client für WebDAV: der gemeinsame Client aus `nunc-platform`, Fehler als `SyncError`.

use crate::SyncError;

/// Siehe [`nunc_platform::http_client`].
pub fn http_client() -> Result<reqwest::Client, SyncError> {
    nunc_platform::http_client().map_err(SyncError::Transport)
}
