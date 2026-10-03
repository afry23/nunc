//! Domänentypen. JSON-Feldnamen (camelCase) und Wertemengen entsprechen den Go-Modellen
//! (`nunc-go/internal/models`), damit Datenbank und Sync-Dateien kompatibel bleiben.

mod session;
mod settings;
mod stats;
mod timer;
mod todo;

pub use session::{Session, SessionStatus, SessionType};
pub use settings::{Settings, SettingsIssue, SyncProvider};
pub use stats::{
    ContextStats, DailyStats, MonthlyStats, ProjectStats, StatsPeriod, StatsRequest, StatsResponse,
    TodoStats, WeekdayStats,
};
pub use timer::{TimerMode, TimerState};
pub use todo::Todo;

/// Gemeinsame Hilfe für Enums mit festen Textwerten (so wie sie in der Datenbank stehen).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unbekannter Wert für {kind}: {value:?}")]
pub struct ParseEnumError {
    pub kind: &'static str,
    pub value: String,
}

/// Erzeugt `as_str` und `FromStr` für ein Enum, passend zu `#[serde(rename_all = …)]`.
macro_rules! text_enum {
    ($ty:ident, $kind:literal, { $($variant:ident => $text:literal),+ $(,)? }) => {
        impl $ty {
            /// Textwert wie in der Datenbank und im JSON.
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }
        }

        impl std::str::FromStr for $ty {
            type Err = $crate::model::ParseEnumError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($text => Ok(Self::$variant),)+
                    other => Err($crate::model::ParseEnumError {
                        kind: $kind,
                        value: other.to_owned(),
                    }),
                }
            }
        }
    };
}
pub(crate) use text_enum;
