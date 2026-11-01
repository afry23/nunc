//! CRDT-Zustand für den Sync (ADR 0016), pur und ohne I/O.
//!
//! - Todo-Felder sind LWW-Register `(Wert, Hlc)`; der höhere Stempel gewinnt, bei völlig gleichem Stempel der
//!   größere Wert (damit der Merge nie von der Reihenfolge abhängt).
//! - `pomodoro_count` ist ein G-Counter pro Gerät, der Wert die Summe.
//! - Zeile (`raw_text`) und Tags sind abgeleitet, keine Register.
//! - Löschen ist ein Register; ein Todo gilt nur als gelöscht, solange keine spätere Bearbeitung vorliegt.
//! - Sessions sind ab ADR 0017 wie Todos Register-Records mit stabiler `sync_id` ([`SessionCrdt`]).
//!
//! [`SyncState::merge`] ist kommutativ, assoziativ und idempotent. Das Kappen fremder Uhren
//! ([`SyncState::sanitized`]) geschieht vor dem Merge, nicht darin.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::hlc::Hlc;
use crate::model::SyncSession;
use crate::todotxt::{ParsedTodo, build_raw_text};

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

/// So lange bleibt ein gelöschtes Todo als Tombstone in der Sync-Datei (90 Tage). Danach darf es verschwinden: Ein
/// Gerät, das länger als das offline war und das Todo noch hat, würde es beim nächsten Sync wieder anlegen.
pub const TOMBSTONE_TTL_MS: i64 = 90 * 24 * 60 * 60 * 1000;

/// Ein Todo als CRDT. `sync_id` ist die geräteübergreifende Identität.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoCrdt {
    pub sync_id: String,
    pub priority: Register<Option<String>>,
    pub completed: Register<bool>,
    pub completed_at: Register<Option<String>>,
    pub created_at: Register<Option<String>>,
    pub description: Register<String>,
    /// Fractional-Index, siehe [`crate::position`].
    pub position: Register<String>,
    /// `sync_id` des Elternknotens (ADR 0022); fehlt in Dateien vor Schema 5.
    #[serde(default = "no_parent")]
    pub parent: Register<Option<String>>,
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
            priority: reg(None, clock),
            completed: reg(false, clock),
            completed_at: reg(None, clock),
            created_at: reg(None, clock),
            description: reg(String::new(), clock),
            position: reg(String::new(), clock),
            parent: reg(None, clock),
            deleted: reg(false, clock),
            pomodoros: BTreeMap::new(),
        }
    }

    /// Die todo.txt-Zeile, aus den Feldern aufgebaut. Sie ist kein eigenes Register: Zeile und Tags folgen immer
    /// aus den zusammengeführten Feldern, sonst könnten sie nach einem Merge von ihnen abweichen
    /// (Gerät A hakt ab, Gerät B ändert den Text).
    pub fn raw_text(&self) -> String {
        let day = |text: &Option<String>| {
            text.as_deref()
                .and_then(|text| NaiveDate::parse_from_str(text, "%Y-%m-%d").ok())
        };
        build_raw_text(&ParsedTodo {
            priority: self
                .priority
                .value
                .as_deref()
                .and_then(|text| text.chars().next()),
            completed: self.completed.value,
            completed_at: day(&self.completed_at.value),
            created_at: day(&self.created_at.value),
            description: self.description.value.clone(),
            ..ParsedTodo::default()
        })
    }

    /// `+projekt`-Tags der Beschreibung.
    pub fn project_tags(&self) -> Vec<String> {
        tags(&self.description.value, '+')
    }

    /// `@kontext`-Tags der Beschreibung.
    pub fn context_tags(&self) -> Vec<String> {
        tags(&self.description.value, '@')
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
            &self.priority.clock,
            &self.completed.clock,
            &self.completed_at.clock,
            &self.created_at.clock,
            &self.description.clock,
            &self.position.clock,
            &self.parent.clock,
        ]
        .into_iter()
        .max()
        .unwrap_or(&self.deleted.clock)
    }

    /// Gelöscht gilt nur, wenn das Löschen nicht älter ist als jede Bearbeitung: Wer ein Todo bearbeitet, das ein
    /// anderes Gerät gleichzeitig gelöscht hat, behält es. Bei genau demselben Stempel (Bestandsdaten und der Import
    /// der alten Datei stempeln alle Felder gleich) gilt das Todo als gelöscht.
    pub fn is_deleted(&self) -> bool {
        self.deleted.value && self.deleted.clock >= *self.latest_edit()
    }

    /// Ist das Todo seit mehr als [`TOMBSTONE_TTL_MS`] gelöscht und damit nur noch ein Tombstone ohne Nutzen?
    pub fn is_collectable(&self, now_ms: i64) -> bool {
        self.is_deleted() && self.deleted.clock.wall_ms < now_ms.saturating_sub(TOMBSTONE_TTL_MS)
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut pomodoros = self.pomodoros.clone();
        for (device, count) in &other.pomodoros {
            let entry = pomodoros.entry(device.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }
        Self {
            sync_id: self.sync_id.clone(),
            priority: self.priority.merge(&other.priority),
            completed: self.completed.merge(&other.completed),
            completed_at: self.completed_at.merge(&other.completed_at),
            created_at: self.created_at.merge(&other.created_at),
            description: self.description.merge(&other.description),
            position: self.position.merge(&other.position),
            parent: self.parent.merge(&other.parent),
            deleted: self.deleted.merge(&other.deleted),
            pomodoros,
        }
    }

    fn sanitized(&self, now_ms: i64) -> Self {
        Self {
            sync_id: self.sync_id.clone(),
            priority: self.priority.capped(now_ms),
            completed: self.completed.capped(now_ms),
            completed_at: self.completed_at.capped(now_ms),
            created_at: self.created_at.capped(now_ms),
            description: self.description.capped(now_ms),
            position: self.position.capped(now_ms),
            parent: self.parent.capped(now_ms),
            deleted: self.deleted.capped(now_ms),
            pomodoros: self.pomodoros.clone(),
        }
    }
}

/// Vorgabe für Dateien ohne `parent`: Wurzel, uralter Stempel.
fn no_parent() -> Register<Option<String>> {
    Register::new(None, Hlc::new(0, 0, "legacy"))
}

/// Vorgabe für Dateien ohne eigene Projekte/Kontexte einer Session: leer, uralter Stempel.
fn no_tags() -> Register<Vec<String>> {
    Register::new(Vec::new(), Hlc::new(0, 0, "legacy"))
}

fn tags(description: &str, prefix: char) -> Vec<String> {
    description
        .split_whitespace()
        .filter(|word| word.len() > 1 && word.starts_with(prefix))
        .map(str::to_owned)
        .collect()
}

/// Identität einer Session: Ursprungsgerät und Beginn. Der Beginn zählt als Zeitpunkt, nicht als Text
/// (`…:00.5Z` und `…:00Z`, andere Zeitzonen); Unlesbares bleibt unverändert.
pub fn session_key(session: &SyncSession) -> String {
    let started = DateTime::parse_from_rfc3339(&session.started_at).map_or_else(
        |_| session.started_at.clone(),
        |time| {
            time.with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Nanos, true)
        },
    );
    format!("{}|{started}", session.device_id)
}

/// Eine Session als CRDT (ADR 0017). `sync_id` ist die geräteübergreifende Identität; Gerät, Modus, Typ, Status und
/// geplante Dauer gelten ab der Anlage und ändern sich nicht (bei Gleichstand gewinnt der größere Wert, damit der
/// Merge nie von der Reihenfolge abhängt). Alles Übrige sind LWW-Register.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCrdt {
    pub sync_id: String,
    pub device_id: String,
    pub mode: String,
    pub session_type: String,
    pub status: String,
    pub planned_min: u32,
    pub break_min: u32,
    /// `timer` oder `manual` (nachgetragen); gilt ab der Anlage.
    #[serde(default = "timer_origin")]
    pub origin: String,
    /// `regular` oder `stopwatch` (ADR 0018); gilt ab der Anlage. Eine Stopwatch-Session steht als `flowtime` in `mode`.
    #[serde(default = "regular_kind")]
    pub kind: String,
    pub started_at: Register<String>,
    pub ended_at: Register<Option<String>>,
    pub actual_min: Register<u32>,
    /// `sync_id` des Todos, nicht die lokale Zeilen-ID.
    pub todo: Register<Option<String>>,
    pub note: Register<String>,
    /// Eigene Projekte (`+name`), nur wirksam ohne Todo (ADR 0023); fehlt in Dateien vor Schema 6.
    #[serde(default = "no_tags")]
    pub project_tags: Register<Vec<String>>,
    /// Eigene Kontexte (`@name`), nur wirksam ohne Todo (ADR 0023); fehlt in Dateien vor Schema 6.
    #[serde(default = "no_tags")]
    pub context_tags: Register<Vec<String>>,
    pub deleted: Register<bool>,
}

/// Herkunft einer vom Timer aufgezeichneten Session (auch für Dateien ohne dieses Feld).
pub const ORIGIN_TIMER: &str = "timer";
/// Herkunft eines von Hand nachgetragenen Eintrags.
pub const ORIGIN_MANUAL: &str = "manual";

fn timer_origin() -> String {
    ORIGIN_TIMER.to_owned()
}

/// Art einer gewöhnlichen Session (auch für Dateien ohne das Feld `kind`).
pub const KIND_REGULAR: &str = "regular";
/// Art einer mit der Stopwatch aufgezeichneten Session (ADR 0018).
pub const KIND_STOPWATCH: &str = "stopwatch";

fn regular_kind() -> String {
    KIND_REGULAR.to_owned()
}

impl SessionCrdt {
    /// Eine Session aus ihren Rohdaten; alle Felder tragen `clock`.
    pub fn from_session(sync_id: impl Into<String>, session: &SyncSession, clock: &Hlc) -> Self {
        fn reg<T: Ord + Clone>(value: T, clock: &Hlc) -> Register<T> {
            Register::new(value, clock.clone())
        }
        Self {
            sync_id: sync_id.into(),
            device_id: session.device_id.clone(),
            mode: session.mode.clone(),
            session_type: session.session_type.clone(),
            status: session.status.clone(),
            planned_min: session.planned_min,
            break_min: session.break_min,
            origin: timer_origin(),
            kind: regular_kind(),
            started_at: reg(session.started_at.clone(), clock),
            ended_at: reg(session.ended_at.clone(), clock),
            actual_min: reg(session.actual_min, clock),
            todo: reg(None, clock),
            note: reg(String::new(), clock),
            project_tags: reg(Vec::new(), clock),
            context_tags: reg(Vec::new(), clock),
            deleted: reg(false, clock),
        }
    }

    /// Die Rohdaten mit den aufgelösten Registerwerten.
    pub fn to_session(&self) -> SyncSession {
        SyncSession {
            device_id: self.device_id.clone(),
            mode: self.mode.clone(),
            session_type: self.session_type.clone(),
            status: self.status.clone(),
            started_at: self.started_at.value.clone(),
            ended_at: self.ended_at.value.clone(),
            planned_min: self.planned_min,
            actual_min: self.actual_min.value,
            break_min: self.break_min,
        }
    }

    /// Jüngster Stempel aller Felder außer dem Löschkennzeichen.
    fn latest_edit(&self) -> &Hlc {
        [
            &self.started_at.clock,
            &self.ended_at.clock,
            &self.actual_min.clock,
            &self.todo.clock,
            &self.note.clock,
            &self.project_tags.clock,
            &self.context_tags.clock,
        ]
        .into_iter()
        .max()
        .unwrap_or(&self.deleted.clock)
    }

    /// Wie [`TodoCrdt::is_deleted`]: Wer eine Session bearbeitet, die ein anderes Gerät gleichzeitig gelöscht hat,
    /// behält sie; bei genau demselben Stempel gilt sie als gelöscht.
    pub fn is_deleted(&self) -> bool {
        self.deleted.value && self.deleted.clock >= *self.latest_edit()
    }

    /// Ist die Session seit mehr als [`TOMBSTONE_TTL_MS`] gelöscht?
    pub fn is_collectable(&self, now_ms: i64) -> bool {
        self.is_deleted() && self.deleted.clock.wall_ms < now_ms.saturating_sub(TOMBSTONE_TTL_MS)
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            sync_id: self.sync_id.clone(),
            device_id: self.device_id.clone().max(other.device_id.clone()),
            mode: self.mode.clone().max(other.mode.clone()),
            session_type: self.session_type.clone().max(other.session_type.clone()),
            status: self.status.clone().max(other.status.clone()),
            planned_min: self.planned_min.max(other.planned_min),
            break_min: self.break_min.max(other.break_min),
            origin: self.origin.clone().max(other.origin.clone()),
            kind: self.kind.clone().max(other.kind.clone()),
            started_at: self.started_at.merge(&other.started_at),
            ended_at: self.ended_at.merge(&other.ended_at),
            actual_min: self.actual_min.merge(&other.actual_min),
            todo: self.todo.merge(&other.todo),
            note: self.note.merge(&other.note),
            project_tags: self.project_tags.merge(&other.project_tags),
            context_tags: self.context_tags.merge(&other.context_tags),
            deleted: self.deleted.merge(&other.deleted),
        }
    }

    fn sanitized(&self, now_ms: i64) -> Self {
        Self {
            started_at: self.started_at.capped(now_ms),
            ended_at: self.ended_at.capped(now_ms),
            actual_min: self.actual_min.capped(now_ms),
            todo: self.todo.capped(now_ms),
            note: self.note.capped(now_ms),
            project_tags: self.project_tags.capped(now_ms),
            context_tags: self.context_tags.capped(now_ms),
            deleted: self.deleted.capped(now_ms),
            ..self.clone()
        }
    }
}

/// Der gesamte Sync-Zustand eines Geräts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncState {
    pub todos: BTreeMap<String, TodoCrdt>,
    /// Schlüssel ist die `sync_id`.
    pub sessions: BTreeMap<String, SessionCrdt>,
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
        for (id, session) in &other.sessions {
            let merged = match sessions.get(id) {
                Some(mine) => mine.merge(session),
                None => session.clone(),
            };
            sessions.insert(id.clone(), merged);
        }
        Self { todos, sessions }
    }

    /// Lässt Tombstones weg, die älter als [`TOMBSTONE_TTL_MS`] sind (siehe [`TodoCrdt::is_collectable`] und
    /// [`SessionCrdt::is_collectable`]). Erst
    /// **nach** dem Zusammenführen anwenden: Vorher würde ein altes Löschen verschwinden, bevor es ein lebendes
    /// Todo der Gegenseite überstimmt hat.
    pub fn collect_garbage(&self, now_ms: i64) -> Self {
        Self {
            todos: self
                .todos
                .iter()
                .filter(|(_, todo)| !todo.is_collectable(now_ms))
                .map(|(id, todo)| (id.clone(), todo.clone()))
                .collect(),
            sessions: self
                .sessions
                .iter()
                .filter(|(_, session)| !session.is_collectable(now_ms))
                .map(|(id, session)| (id.clone(), session.clone()))
                .collect(),
        }
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
            sessions: self
                .sessions
                .iter()
                .map(|(id, session)| (id.clone(), session.sanitized(now_ms)))
                .collect(),
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
    use crate::hlc::MAX_DRIFT_MS;

    fn clock(wall: i64) -> Hlc {
        Hlc::new(wall, 0, "d")
    }

    #[test]
    fn the_line_and_tags_follow_from_the_fields() {
        let mut todo = TodoCrdt::new("a", &clock(1));
        todo.priority.set(Some("B".to_owned()), clock(2));
        todo.created_at.set(Some("2026-09-20".to_owned()), clock(2));
        todo.description
            .set("Report +work @desk + @".to_owned(), clock(2));
        assert_eq!(todo.raw_text(), "(B) 2026-09-20 Report +work @desk + @");
        assert_eq!(todo.project_tags(), ["+work"]);
        assert_eq!(todo.context_tags(), ["@desk"]);
        todo.completed.set(true, clock(3));
        todo.completed_at
            .set(Some("2026-10-01".to_owned()), clock(3));
        assert_eq!(
            todo.raw_text(),
            "x 2026-10-01 2026-09-20 Report +work @desk + @"
        );
    }

    #[test]
    fn an_identical_stamp_keeps_the_larger_value_in_either_order() {
        let small = Register::new("a".to_owned(), clock(5));
        let large = Register::new("b".to_owned(), clock(5));
        assert_eq!(small.merge(&large).value, "b");
        assert_eq!(large.merge(&small).value, "b");
    }

    #[test]
    fn a_deletion_is_not_older_than_the_last_edit() {
        let mut todo = TodoCrdt::new("a", &clock(1));
        todo.description.set("edited".to_owned(), clock(9));
        todo.deleted.set(true, clock(8));
        assert!(!todo.is_deleted());
        todo.deleted.set(true, clock(9));
        assert!(todo.is_deleted());
        todo.deleted.set(true, clock(10));
        assert!(todo.is_deleted());
    }

    fn raw_session(minutes: u32) -> SyncSession {
        SyncSession {
            device_id: "d".to_owned(),
            mode: "pomodoro".to_owned(),
            session_type: "work".to_owned(),
            status: "completed".to_owned(),
            started_at: "2026-03-01T10:00:00Z".to_owned(),
            ended_at: None,
            planned_min: 25,
            actual_min: minutes,
            break_min: 5,
        }
    }

    #[test]
    fn the_origin_defaults_to_timer_also_for_files_without_it() {
        let session = SessionCrdt::from_session("s", &raw_session(25), &clock(1));
        assert_eq!(session.origin, "timer");
        let mut json = serde_json::to_value(&session).unwrap();
        json.as_object_mut().unwrap().remove("origin");
        let read: SessionCrdt = serde_json::from_value(json).unwrap();
        assert_eq!(read.origin, ORIGIN_TIMER);
        assert_eq!(read, session);
    }

    #[test]
    fn the_kind_defaults_to_regular_and_stopwatch_wins_a_merge() {
        let session = SessionCrdt::from_session("s", &raw_session(25), &clock(1));
        assert_eq!(session.kind, KIND_REGULAR);
        let mut json = serde_json::to_value(&session).unwrap();
        json.as_object_mut().unwrap().remove("kind");
        let read: SessionCrdt = serde_json::from_value(json).unwrap();
        assert_eq!(read, session);
        let mut watch = session.clone();
        watch.kind = KIND_STOPWATCH.to_owned();
        assert_eq!(session.merge(&watch).kind, KIND_STOPWATCH);
        assert_eq!(watch.merge(&session).kind, KIND_STOPWATCH);
    }

    #[test]
    fn registers_of_a_session_merge_field_by_field() {
        let mut a = SessionCrdt::from_session("s", &raw_session(25), &clock(1));
        let mut b = a.clone();
        a.actual_min.set(30, clock(5));
        b.note.set("review".to_owned(), clock(6));
        b.todo.set(Some("t".to_owned()), clock(6));
        let merged = a.merge(&b);
        assert_eq!(merged.actual_min.value, 30);
        assert_eq!(merged.note.value, "review");
        assert_eq!(merged.todo.value.as_deref(), Some("t"));
        assert_eq!(b.merge(&a), merged);
        assert_eq!(merged.to_session().actual_min, 30);
    }

    #[test]
    fn a_session_deletion_is_not_older_than_the_last_edit() {
        let mut session = SessionCrdt::from_session("s", &raw_session(25), &clock(1));
        session.note.set("edited".to_owned(), clock(9));
        session.deleted.set(true, clock(8));
        assert!(!session.is_deleted());
        session.deleted.set(true, clock(9));
        assert!(session.is_deleted());
        session.deleted.set(true, clock(10));
        assert!(session.is_deleted());
    }

    #[test]
    fn a_session_tombstone_is_collectable_after_90_days() {
        let mut session = SessionCrdt::from_session("s", &raw_session(25), &clock(1));
        session.deleted.set(true, clock(1000));
        assert!(!session.is_collectable(1000 + TOMBSTONE_TTL_MS));
        assert!(session.is_collectable(1001 + TOMBSTONE_TTL_MS));
        session.deleted.set(false, clock(2000));
        assert!(!session.is_collectable(i64::MAX));
    }

    #[test]
    fn the_state_collects_and_caps_sessions() {
        let mut gone = SessionCrdt::from_session("gone", &raw_session(25), &clock(1));
        gone.deleted.set(true, clock(2));
        let mut ahead = SessionCrdt::from_session("ahead", &raw_session(25), &clock(1));
        ahead.note.set("x".to_owned(), clock(MAX_DRIFT_MS + 5_000));
        let state = SyncState {
            todos: BTreeMap::new(),
            sessions: BTreeMap::from([("gone".to_owned(), gone), ("ahead".to_owned(), ahead)]),
        };
        let collected = state.collect_garbage(TOMBSTONE_TTL_MS + 10);
        assert_eq!(collected.sessions.keys().collect::<Vec<_>>(), ["ahead"]);
        let capped = state.sanitized(0);
        let ahead = &capped.sessions["ahead"];
        assert_eq!(ahead.note.clock.wall_ms, MAX_DRIFT_MS);
        // jedes Register wird gekappt, nicht nur die Notiz
        let mut all = SessionCrdt::from_session("all", &raw_session(25), &clock(MAX_DRIFT_MS + 5));
        all.note.set(String::new(), clock(MAX_DRIFT_MS + 5));
        let all = all.sanitized(0);
        let stamps = [
            all.started_at.clock.wall_ms,
            all.ended_at.clock.wall_ms,
            all.actual_min.clock.wall_ms,
            all.todo.clock.wall_ms,
            all.note.clock.wall_ms,
            all.deleted.clock.wall_ms,
        ];
        assert_eq!(stamps, [MAX_DRIFT_MS; 6]);
    }
}
