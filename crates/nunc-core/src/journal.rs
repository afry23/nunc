//! Journal (ADR 0026): Tages-/Wochennotizen und Erfolge als zustandsbasierter CRDT.
//!
//! Wie [`TodoCrdt`](crate::crdt::TodoCrdt) und [`SessionCrdt`](crate::crdt::SessionCrdt): Art und Identität gelten ab
//! der Anlage, `date`, `text` und `deleted` sind LWW-Register. Notizen haben eine **deterministische** `sync_id`
//! (Art und Tag), zwei Geräte bilden also für dieselbe Notiz dieselbe ID. Die Wochennotiz liegt unter dem
//! **Sonntag** ihrer Woche (Ende des Rückblick-Zeitraums). Ein leerer Text löscht die Notiz.

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::crdt::{Register, TOMBSTONE_TTL_MS};
use crate::hlc::Hlc;

/// Längster Text eines Eintrags in Zeichen.
pub const MAX_TEXT_CHARS: usize = 4000;

/// Art eines Journal-Eintrags.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Type,
)]
#[serde(rename_all = "snake_case")]
pub enum JournalKind {
    DayNote,
    WeekNote,
    Achievement,
}

impl JournalKind {
    /// Textwert wie in der Datenbank und im JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DayNote => "day_note",
            Self::WeekNote => "week_note",
            Self::Achievement => "achievement",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum JournalError {
    #[error("Der Text darf höchstens {MAX_TEXT_CHARS} Zeichen lang sein.")]
    TextTooLong,
}

/// Der Sonntag der Woche (Montag bis Sonntag), die `day` enthält. Unter ihm liegt die Wochennotiz.
pub fn week_end(day: NaiveDate) -> NaiveDate {
    day + Duration::days(i64::from(6 - day.weekday().num_days_from_monday()))
}

/// Text ohne Leerraum an den Rändern; zu lange Texte sind ein Fehler.
pub fn validate_text(text: &str) -> Result<String, JournalError> {
    let text = text.trim();
    if text.chars().count() > MAX_TEXT_CHARS {
        return Err(JournalError::TextTooLong);
    }
    Ok(text.to_owned())
}

/// `note:day:2026-03-02` bzw. `note:week:2026-03-08` (Sonntag).
fn note_id(kind: &str, date: NaiveDate) -> String {
    format!("note:{kind}:{date}")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub sync_id: String,
    pub kind: JournalKind,
    pub date: Register<NaiveDate>,
    pub text: Register<String>,
    pub deleted: Register<bool>,
}

impl JournalEntry {
    /// Tagesnotiz; alle Felder tragen `clock`. Ein leerer Text ergibt einen gelöschten Eintrag.
    pub fn day_note(date: NaiveDate, text: &str, clock: &Hlc) -> Self {
        Self::new(
            note_id("day", date),
            JournalKind::DayNote,
            date,
            text,
            clock,
        )
    }

    /// Wochennotiz der Woche, die `day` enthält (abgelegt unter dem Sonntag).
    pub fn week_note(day: NaiveDate, text: &str, clock: &Hlc) -> Self {
        let end = week_end(day);
        Self::new(
            note_id("week", end),
            JournalKind::WeekNote,
            end,
            text,
            clock,
        )
    }

    /// Erfolg mit der vom Gerät vergebenen `sync_id` (UUID).
    pub fn achievement(
        sync_id: impl Into<String>,
        date: NaiveDate,
        text: &str,
        clock: &Hlc,
    ) -> Self {
        Self::new(sync_id.into(), JournalKind::Achievement, date, text, clock)
    }

    fn new(sync_id: String, kind: JournalKind, date: NaiveDate, text: &str, clock: &Hlc) -> Self {
        let text = text.trim().to_owned();
        Self {
            sync_id,
            kind,
            date: Register::new(date, clock.clone()),
            deleted: Register::new(text.is_empty(), clock.clone()),
            text: Register::new(text, clock.clone()),
        }
    }

    /// Lokaler Schreibvorgang: Ein leerer Text löscht den Eintrag, ein Text danach stellt ihn wieder her.
    pub fn set_text(&mut self, text: &str, clock: Hlc) {
        let text = text.trim().to_owned();
        self.deleted.set(text.is_empty(), clock.clone());
        if !text.is_empty() {
            self.text.set(text, clock);
        }
    }

    /// Verschiebt einen Erfolg auf einen anderen Tag. Notizen gehören zu ihrem Tag (er steckt in der ID); für sie
    /// ändert sich nichts und es kommt `false` zurück.
    pub fn set_date(&mut self, date: NaiveDate, clock: Hlc) -> bool {
        if self.kind != JournalKind::Achievement {
            return false;
        }
        self.date.set(date, clock);
        true
    }

    /// Löscht den Eintrag (Tombstone).
    pub fn delete(&mut self, clock: Hlc) {
        self.deleted.set(true, clock);
    }

    /// Jüngster Stempel der Inhaltsfelder.
    fn latest_edit(&self) -> &Hlc {
        (&self.date.clock).max(&self.text.clock)
    }

    /// Wie bei Todos und Sessions: Wer einen Eintrag bearbeitet, den ein anderes Gerät gleichzeitig gelöscht hat,
    /// behält ihn; bei genau demselben Stempel gilt er als gelöscht.
    pub fn is_deleted(&self) -> bool {
        self.deleted.value && self.deleted.clock >= *self.latest_edit()
    }

    /// Ist der Eintrag seit mehr als [`TOMBSTONE_TTL_MS`] gelöscht?
    pub fn is_collectable(&self, now_ms: i64) -> bool {
        self.is_deleted() && self.deleted.clock.wall_ms < now_ms.saturating_sub(TOMBSTONE_TTL_MS)
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            sync_id: self.sync_id.clone(),
            kind: self.kind.max(other.kind),
            date: self.date.merge(&other.date),
            text: self.text.merge(&other.text),
            deleted: self.deleted.merge(&other.deleted),
        }
    }

    /// Kappt zu weit vorauseilende Stempel eines fremden Geräts und zu lange Texte.
    pub fn sanitized(&self, now_ms: i64) -> Self {
        let mut text = self.text.capped(now_ms);
        text.value = text.value.chars().take(MAX_TEXT_CHARS).collect();
        Self {
            date: self.date.capped(now_ms),
            text,
            deleted: self.deleted.capped(now_ms),
            ..self.clone()
        }
    }
}
