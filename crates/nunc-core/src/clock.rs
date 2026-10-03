//! Zeitquelle. Die Fachlogik ruft nie selbst die Systemzeit ab: Zustandsübergänge bekommen `now` als
//! Parameter. Dieses Trait nutzen die Dienste außen (Tauri-Commands, Hintergrund-Tasks); die echte
//! Implementierung (`SystemClock`) liegt in `nunc-platform`, Tests nutzen eine feste oder manuell vorgestellte Uhr.

use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}
