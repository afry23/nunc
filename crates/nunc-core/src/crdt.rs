//! CRDT-Zustand für den Sync (ADR 0016), pur und ohne I/O.
//!
//! - Todo-Felder sind LWW-Register `(Wert, Hlc)`; der höhere Stempel gewinnt, bei völlig gleichem Stempel der
//!   größere Wert (damit der Merge nie von der Reihenfolge abhängt).
//! - `pomodoro_count` ist ein G-Counter pro Gerät, der Wert die Summe.
//! - Löschen ist ein Register; ein Todo gilt nur als gelöscht, solange keine spätere Bearbeitung vorliegt.
//! - Sessions sind ein G-Set über `(device_id, started_at)`.
//!
//! [`SyncState::merge`] ist kommutativ, assoziativ und idempotent. Das Kappen fremder Uhren
//! ([`SyncState::sanitized`]) geschieht vor dem Merge, nicht darin.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::hlc::Hlc;
use crate::model::SyncSession;

/// Ein Wert mit dem Stempel seines letzten Schreibvorgangs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Register<T> {
    pub value: T,
    pub clock: Hlc,
}

impl<T: Ord + Clone> Register<T> {
    pub fn new(value: T, clock: Hlc) -> Self {
        Self { value, clock }
    }

    /// Lokaler Schreibvorgang. `clock` kommt aus [`Hlc::next`] und ist damit neuer als der bisherige Stempel.
    pub fn set(&mut self, value: T, clock: Hlc) {
        self.value = value;
        self.clock = clock;
    }

    pub fn merge(&self, other: &Self) -> Self {
        let other_wins = match other.clock.cmp(&self.clock) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => other.value > self.value,
        };
        if other_wins {
            other.clone()
        } else {
            self.clone()
        }
    }

    fn capped(&self, now_ms: i64) -> Self {
        Self::new(self.value.clone(), self.clock.capped(now_ms))
    }
}

/// Ein Todo als CRDT. `sync_id` ist die geräteübergreifende Identität.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoCrdt {
    pub sync_id: String,
    pub raw_text: Register<String>,
    pub priority: Register<Option<String>>,
    pub completed: Register<bool>,
    pub completed_at: Register<Option<String>>,
    pub created_at: Register<Option<String>>,
    pub description: Register<String>,
    pub project_tags: Register<Vec<String>>,
    pub context_tags: Register<Vec<String>>,
    /// Fractional-Index, siehe [`crate::position`].
    pub position: Register<String>,
    pub deleted: Register<bool>,
    /// Pomodoros je Gerät; der Wert des Todos ist die Summe.
    pub pomodoros: BTreeMap<String, u32>,
}

impl TodoCrdt {
    /// Leeres, nicht gelöschtes Todo; alle Felder tragen `clock`.
    pub fn new(sync_id: impl Into<String>, clock: &Hlc) -> Self {
        fn reg<T: Ord + Clone>(value: T, clock: &Hlc) -> Register<T> {
            Register::new(value, clock.clone())
        }
        Self {
            sync_id: sync_id.into(),
            raw_text: reg(String::new(), clock),
            priority: reg(None, clock),
            completed: reg(false, clock),
            completed_at: reg(None, clock),
            created_at: reg(None, clock),
            description: reg(String::new(), clock),
            project_tags: reg(Vec::new(), clock),
            context_tags: reg(Vec::new(), clock),
            position: reg(String::new(), clock),
            deleted: reg(false, clock),
            pomodoros: BTreeMap::new(),
        }
    }

    /// Summe der Pomodoros aller Geräte.
    pub fn pomodoro_count(&self) -> u32 {
        self.pomodoros
            .values()
            .fold(0u32, |sum, count| sum.saturating_add(*count))
    }

    /// Ein Pomodoro auf diesem Gerät mehr.
    pub fn add_pomodoro(&mut self, device_id: &str) {
        let count = self.pomodoros.entry(device_id.to_owned()).or_insert(0);
        *count = count.saturating_add(1);
    }

    /// Jüngster Stempel aller Felder außer dem Löschkennzeichen.
    fn latest_edit(&self) -> &Hlc {
        [
            &self.raw_text.clock,
            &self.priority.clock,
            &self.completed.clock,
            &self.completed_at.clock,
            &self.created_at.clock,
            &self.description.clock,
            &self.project_tags.clock,
            &self.context_tags.clock,
            &self.position.clock,
        ]
        .into_iter()
        .max()
        .unwrap_or(&self.deleted.clock)
    }

    /// Gelöscht gilt nur, wenn das Löschen jünger ist als jede Bearbeitung: Wer ein Todo bearbeitet, das ein
    /// anderes Gerät gleichzeitig gelöscht hat, behält es.
    pub fn is_deleted(&self) -> bool {
        self.deleted.value && self.deleted.clock > *self.latest_edit()
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut pomodoros = self.pomodoros.clone();
        for (device, count) in &other.pomodoros {
            let entry = pomodoros.entry(device.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }
        Self {
            sync_id: self.sync_id.clone(),
            raw_text: self.raw_text.merge(&other.raw_text),
            priority: self.priority.merge(&other.priority),
            completed: self.completed.merge(&other.completed),
            completed_at: self.completed_at.merge(&other.completed_at),
            created_at: self.created_at.merge(&other.created_at),
            description: self.description.merge(&other.description),
            project_tags: self.project_tags.merge(&other.project_tags),
            context_tags: self.context_tags.merge(&other.context_tags),
            position: self.position.merge(&other.position),
            deleted: self.deleted.merge(&other.deleted),
            pomodoros,
        }
    }

    fn sanitized(&self, now_ms: i64) -> Self {
        Self {
            sync_id: self.sync_id.clone(),
            raw_text: self.raw_text.capped(now_ms),
            priority: self.priority.capped(now_ms),
            completed: self.completed.capped(now_ms),
            completed_at: self.completed_at.capped(now_ms),
            created_at: self.created_at.capped(now_ms),
            description: self.description.capped(now_ms),
            project_tags: self.project_tags.capped(now_ms),
            context_tags: self.context_tags.capped(now_ms),
            position: self.position.capped(now_ms),
            deleted: self.deleted.capped(now_ms),
            pomodoros: self.pomodoros.clone(),
        }
    }
}

/// Identität einer Session: Ursprungsgerät und Beginn.
pub fn session_key(session: &SyncSession) -> String {
    format!("{}|{}", session.device_id, session.started_at)
}

/// Der gesamte Sync-Zustand eines Geräts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncState {
    pub todos: BTreeMap<String, TodoCrdt>,
    pub sessions: BTreeMap<String, SyncSession>,
}

impl SyncState {
    pub fn merge(&self, other: &Self) -> Self {
        let mut todos = self.todos.clone();
        for (id, todo) in &other.todos {
            let merged = match todos.get(id) {
                Some(mine) => mine.merge(todo),
                None => todo.clone(),
            };
            todos.insert(id.clone(), merged);
        }
        let mut sessions = self.sessions.clone();
        for (key, session) in &other.sessions {
            let kept = match sessions.get(key) {
                Some(mine) if mine >= session => mine.clone(),
                _ => session.clone(),
            };
            sessions.insert(key.clone(), kept);
        }
        Self { todos, sessions }
    }

    /// Kappt alle Stempel, die der Uhr `now_ms` zu weit vorauseilen (siehe [`Hlc::capped`]). Auf den Zustand
    /// eines **anderen** Geräts anwenden, bevor er per [`merge`](Self::merge) einfließt.
    pub fn sanitized(&self, now_ms: i64) -> Self {
        Self {
            todos: self
                .todos
                .iter()
                .map(|(id, todo)| (id.clone(), todo.sanitized(now_ms)))
                .collect(),
            sessions: self.sessions.clone(),
        }
    }

    /// Spielt den Zustand eines anderen Geräts ein: erst Uhren kappen, dann zusammenführen.
    pub fn merge_remote(&self, remote: &Self, now_ms: i64) -> Self {
        self.merge(&remote.sanitized(now_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock(wall: i64) -> Hlc {
        Hlc::new(wall, 0, "d")
    }

    #[test]
    fn an_identical_stamp_keeps_the_larger_value_in_either_order() {
        let small = Register::new("a".to_owned(), clock(5));
        let large = Register::new("b".to_owned(), clock(5));
        assert_eq!(small.merge(&large).value, "b");
        assert_eq!(large.merge(&small).value, "b");
    }

    #[test]
    fn a_deletion_with_the_same_stamp_as_an_edit_does_not_delete() {
        let mut todo = TodoCrdt::new("a", &clock(1));
        todo.raw_text.set("edited".to_owned(), clock(9));
        todo.deleted.set(true, clock(9));
        assert!(!todo.is_deleted());
        todo.deleted.set(true, clock(10));
        assert!(todo.is_deleted());
    }

    #[test]
    fn a_session_known_under_one_key_keeps_the_larger_version() {
        let session = |minutes| SyncSession {
            device_id: "d".to_owned(),
            mode: "pomodoro".to_owned(),
            session_type: "work".to_owned(),
            status: "completed".to_owned(),
            started_at: "2026-03-01T10:00:00Z".to_owned(),
            ended_at: None,
            planned_min: 25,
            actual_min: minutes,
            break_min: 5,
        };
        let state = |minutes| {
            let session = session(minutes);
            SyncState {
                todos: BTreeMap::new(),
                sessions: BTreeMap::from([(session_key(&session), session)]),
            }
        };
        let merged = state(25).merge(&state(20));
        assert_eq!(merged.sessions.len(), 1);
        assert_eq!(merged.sessions.values().next().unwrap().actual_min, 25);
        assert_eq!(state(20).merge(&state(25)), merged);
    }
}
