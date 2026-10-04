//! Öffnet Adressen im Standardbrowser (Anmeldung per OAuth).

use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Der Browser konnte nicht geöffnet werden: {0}")]
pub struct OpenError(pub String);

pub trait UrlOpener: Send + Sync {
    fn open(&self, url: &str) -> Result<(), OpenError>;
}

/// Merkt sich die geöffneten Adressen (für Tests).
#[derive(Debug, Default)]
pub struct RecordingUrlOpener {
    urls: Mutex<Vec<String>>,
}

impl RecordingUrlOpener {
    pub fn urls(&self) -> Vec<String> {
        self.urls
            .lock()
            .map(|urls| urls.clone())
            .unwrap_or_default()
    }
}

impl UrlOpener for RecordingUrlOpener {
    fn open(&self, url: &str) -> Result<(), OpenError> {
        if let Ok(mut urls) = self.urls.lock() {
            urls.push(url.to_owned());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_recorder_keeps_the_urls_in_order() {
        let opener = RecordingUrlOpener::default();
        opener.open("https://a.example").unwrap();
        opener.open("https://b.example").unwrap();
        assert_eq!(opener.urls(), ["https://a.example", "https://b.example"]);
    }
}
