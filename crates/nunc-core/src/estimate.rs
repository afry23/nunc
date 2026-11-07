//! Aufwandsschätzung einer Aufgabe und ihr Vergleich mit der tatsächlichen Arbeit (ADR 0035).
//!
//! Die Schätzung steht als `est:N` (Minuten) oder `estp:N` (Pomodoros) in der Beschreibung (siehe
//! [`crate::day_plan`]). Verglichen wird in der Einheit der Schätzung: Minuten gegen die Fokuszeit, Pomodoros gegen
//! die abgeschlossenen Pomodoro-Sessions. Alles ist pur.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::day_plan::plan_fields;
use crate::model::{Session, SessionStatus, SessionType, TimerMode, Todo};

/// Abweichung bis zu diesem Anteil (in Prozent) der Schätzung gilt als „im Rahmen“.
pub const TOLERANCE_PERCENT: u32 = 20;

/// Die Schätzung einer Aufgabe in ihrer Einheit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "unit", content = "value", rename_all = "snake_case")]
pub enum Estimate {
    Minutes(u32),
    Pomodoros(u32),
}

impl Estimate {
    fn value(self) -> u32 {
        match self {
            Self::Minutes(value) | Self::Pomodoros(value) => value,
        }
    }
}

/// Die Schätzung aus der Beschreibung; bei beiden Angaben gewinnen die Minuten.
pub fn estimate_of(description: &str) -> Option<Estimate> {
    let fields = plan_fields(description);
    fields
        .estimate_min
        .map(Estimate::Minutes)
        .or(fields.estimate_pomodoros.map(Estimate::Pomodoros))
}

/// Wie die tatsächliche Arbeit zur Schätzung steht.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Weniger als geschätzt, außerhalb der Toleranz.
    Under,
    /// Innerhalb der Toleranz.
    OnTarget,
    /// Mehr als geschätzt, außerhalb der Toleranz.
    Over,
}

/// Schätzung gegen tatsächlichen Aufwand, beides in der Einheit der Schätzung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EstimateCheck {
    pub estimate: Estimate,
    pub actual: u32,
    /// `actual − estimate`; positiv heißt länger als geschätzt.
    pub delta: i32,
    pub verdict: Verdict,
}

/// Vergleicht `estimate` mit dem tatsächlichen Aufwand (`actual_min` Fokuszeit, `actual_pomodoros` Pomodoros).
pub fn check(estimate: Estimate, actual_min: u32, actual_pomodoros: u32) -> EstimateCheck {
    let actual = match estimate {
        Estimate::Minutes(_) => actual_min,
        Estimate::Pomodoros(_) => actual_pomodoros,
    };
    let planned = estimate.value();
    let delta = i64::from(actual) - i64::from(planned);
    let within = delta.unsigned_abs() * 100 <= u64::from(planned) * u64::from(TOLERANCE_PERCENT);
    let verdict = match (within, delta.is_negative()) {
        (true, _) => Verdict::OnTarget,
        (false, true) => Verdict::Under,
        (false, false) => Verdict::Over,
    };
    EstimateCheck {
        estimate,
        actual,
        // Werte jenseits von i32 sättigen
        delta: i32::try_from(delta.clamp(i64::from(i32::MIN), i64::from(i32::MAX)))
            .unwrap_or_default(),
        verdict,
    }
}

/// Fokuszeit und Pomodoros eines Todos über seine ganze Lebenszeit: abgeschlossene Arbeitssessions.
pub fn lifetime_effort(sessions: &[Session], todo_id: i32) -> (u32, u32) {
    sessions
        .iter()
        .filter(|session| {
            session.todo_id == Some(todo_id)
                && session.session_type == SessionType::Work
                && session.status == SessionStatus::Completed
        })
        .fold((0, 0), |(minutes, pomodoros), session| {
            (
                minutes + session.actual_min.unwrap_or(0),
                pomodoros + u32::from(session.mode == TimerMode::Pomodoro),
            )
        })
}

/// Der Vergleich für ein Todo; `None` ohne Schätzung.
pub fn todo_check(todo: &Todo, sessions: &[Session]) -> Option<EstimateCheck> {
    let estimate = estimate_of(&todo.description)?;
    let (minutes, pomodoros) = lifetime_effort(sessions, todo.id);
    Some(check(estimate, minutes, pomodoros))
}

/// Trefferquote der Schätzungen im Zeitraum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EstimateSummary {
    pub count: u32,
    pub on_target: u32,
    pub under: u32,
    pub over: u32,
}

/// Zählt die Urteile.
pub fn summarize(checks: impl IntoIterator<Item = EstimateCheck>) -> EstimateSummary {
    checks
        .into_iter()
        .fold(EstimateSummary::default(), |mut sum, check| {
            sum.count += 1;
            match check.verdict {
                Verdict::OnTarget => sum.on_target += 1,
                Verdict::Under => sum.under += 1,
                Verdict::Over => sum.over += 1,
            }
            sum
        })
}
