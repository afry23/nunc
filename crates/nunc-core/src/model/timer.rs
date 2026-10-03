use serde::{Deserialize, Serialize};
use specta::Type;

use super::text_enum;

/// Zeitmess-Technik.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    Pomodoro,
    Flowtime,
}

text_enum!(TimerMode, "TimerMode", {
    Pomodoro => "pomodoro",
    Flowtime => "flowtime",
});

/// Betriebszustand des Timers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum TimerState {
    Idle,
    Running,
    Paused,
}

text_enum!(TimerState, "TimerState", {
    Idle => "idle",
    Running => "running",
    Paused => "paused",
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_values_match_the_database() {
        assert_eq!(TimerMode::Pomodoro.as_str(), "pomodoro");
        assert_eq!(TimerMode::Flowtime.as_str(), "flowtime");
        assert_eq!(TimerState::Idle.as_str(), "idle");
        assert_eq!(TimerState::Running.as_str(), "running");
        assert_eq!(TimerState::Paused.as_str(), "paused");
    }

    #[test]
    fn parses_every_text_value() {
        for mode in [TimerMode::Pomodoro, TimerMode::Flowtime] {
            assert_eq!(mode.as_str().parse::<TimerMode>(), Ok(mode));
        }
        for state in [TimerState::Idle, TimerState::Running, TimerState::Paused] {
            assert_eq!(state.as_str().parse::<TimerState>(), Ok(state));
        }
    }

    #[test]
    fn rejects_unknown_values() {
        let err = "Pomodoro".parse::<TimerMode>().unwrap_err();
        assert_eq!(err.kind, "TimerMode");
        assert_eq!(err.value, "Pomodoro");
        assert!("".parse::<TimerState>().is_err());
    }

    #[test]
    fn serializes_like_the_text_values() {
        assert_eq!(
            serde_json::to_string(&TimerMode::Flowtime).unwrap(),
            "\"flowtime\""
        );
        assert_eq!(
            serde_json::from_str::<TimerState>("\"paused\"").unwrap(),
            TimerState::Paused
        );
    }
}
