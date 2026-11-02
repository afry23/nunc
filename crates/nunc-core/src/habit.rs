//! Habit-Tracker (ADR 0028): Gewohnheiten und Haken als CRDT, Rhythmus, Ziel und Auswertung der Serien.
//!
//! Alles ist pur: Zeit kommt als Parameter, der lokale Tag einer Session über `local_date`.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::crdt::{Register, TOMBSTONE_TTL_MS};
use crate::hlc::Hlc;
use crate::model::{Session, SessionStatus, SessionType, Todo, effective_tags};

/// Längster Name einer Gewohnheit in Zeichen.
pub const MAX_NAME_CHARS: usize = 100;
/// Größtes Minutenziel je Tag (ein Tag).
pub const MAX_GOAL_MINUTES: u32 = 24 * 60;

/// Ungültige Angabe zu einer Gewohnheit.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HabitError {
    #[error("Der Name darf nicht leer sein.")]
    EmptyName,
    #[error("Der Name darf höchstens {MAX_NAME_CHARS} Zeichen lang sein.")]
    NameTooLong,
    #[error("Der Rhythmus ist ungültig.")]
    InvalidSchedule,
    #[error("Das Ziel ist ungültig.")]
    InvalidGoal,
}

/// Prüft und bereinigt den Namen (`trim`).
pub fn validate_name(name: &str) -> Result<String, HabitError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(HabitError::EmptyName);
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(HabitError::NameTooLong);
    }
    Ok(name.to_owned())
}

/// Wann eine Gewohnheit fällig ist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum Schedule {
    /// Jeden Tag.
    Daily,
    /// Nur an diesen Wochentagen, 1 = Montag … 7 = Sonntag.
    Days { days: Vec<u8> },
    /// So viele Tage pro Woche (1–7), beliebige Tage; die Woche läuft Montag bis Sonntag.
    Weekly { per_week: u8 },
}

impl Schedule {
    /// Prüft die Angaben und ordnet die Wochentage (sortiert, ohne Wiederholung).
    pub fn checked(self) -> Result<Self, HabitError> {
        match self {
            Self::Daily => Ok(self),
            Self::Days { mut days } => {
                days.sort_unstable();
                days.dedup();
                if days.is_empty() || days.iter().any(|d| !(1..=7).contains(d)) {
                    return Err(HabitError::InvalidSchedule);
                }
                Ok(Self::Days { days })
            }
            Self::Weekly { per_week } => {
                if (1..=7).contains(&per_week) {
                    Ok(self)
                } else {
                    Err(HabitError::InvalidSchedule)
                }
            }
        }
    }

    /// Kodierung im Register: `daily`, `days:1,3,5`, `weekly:3`.
    pub fn encode(&self) -> String {
        match self {
            Self::Daily => "daily".to_owned(),
            Self::Days { days } => {
                let list: Vec<String> = days.iter().map(u8::to_string).collect();
                format!("days:{}", list.join(","))
            }
            Self::Weekly { per_week } => format!("weekly:{per_week}"),
        }
    }

    /// Liest eine kanonische Kodierung; alles andere (unbekannt, ungeordnet, außerhalb des Bereichs) ist `None`.
    pub fn parse(text: &str) -> Option<Self> {
        let schedule = if text == "daily" {
            Self::Daily
        } else if let Some(list) = text.strip_prefix("days:") {
            let days = list
                .split(',')
                .map(|d| d.parse::<u8>().ok())
                .collect::<Option<Vec<u8>>>()?;
            Self::Days { days }
        } else {
            let count = text.strip_prefix("weekly:")?;
            Self::Weekly {
                per_week: count.parse().ok()?,
            }
        };
        // nur die kanonische Form gilt: `days:3,1` oder `weekly:03` lesen sich nicht
        let checked = schedule.checked().ok()?;
        (checked.encode() == text).then_some(checked)
    }

    fn is_due_on(&self, day: NaiveDate) -> bool {
        match self {
            Self::Daily => true,
            Self::Days { days } => days.contains(&weekday_number(day)),
            // Wochenziel: jeder Tag ist möglich, die Serie zählt in Wochen
            Self::Weekly { .. } => true,
        }
    }
}

fn weekday_number(day: NaiveDate) -> u8 {
    // 1 = Montag … 7 = Sonntag
    u8::try_from(day.weekday().number_from_monday()).unwrap_or(1)
}

/// Woran eine Gewohnheit erkannt wird, dass sie erledigt ist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Goal {
    /// Von Hand abhaken.
    Check,
    /// Automatisch: mindestens `minutes` Arbeitszeit am Tag mit dem Tag `tag` (`+projekt` oder `@kontext`).
    Minutes { minutes: u32, tag: String },
}

fn is_tag(tag: &str) -> bool {
    tag.len() > 1
        && (tag.starts_with('+') || tag.starts_with('@'))
        && !tag.chars().any(char::is_whitespace)
}

impl Goal {
    /// Prüft die Angaben (Minuten 1–1440, Tag mit `+`/`@` ohne Leerraum) und bereinigt den Tag (`trim`).
    pub fn checked(self) -> Result<Self, HabitError> {
        match self {
            Self::Check => Ok(self),
            Self::Minutes { minutes, tag } => {
                let tag = tag.trim().to_owned();
                if (1..=MAX_GOAL_MINUTES).contains(&minutes) && is_tag(&tag) {
                    Ok(Self::Minutes { minutes, tag })
                } else {
                    Err(HabitError::InvalidGoal)
                }
            }
        }
    }

    /// Kodierung im Register: `check` oder `minutes:30:+work`.
    pub fn encode(&self) -> String {
        match self {
            Self::Check => "check".to_owned(),
            Self::Minutes { minutes, tag } => format!("minutes:{minutes}:{tag}"),
        }
    }

    /// Liest eine kanonische Kodierung; alles andere ist `None`.
    pub fn parse(text: &str) -> Option<Self> {
        if text == "check" {
            return Some(Self::Check);
        }
        let mut parts = text.splitn(3, ':');
        let (Some("minutes"), Some(minutes), Some(tag)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        let goal = Self::Minutes {
            minutes: minutes.parse().ok()?,
            tag: tag.to_owned(),
        }
        .checked()
        .ok()?;
        (goal.encode() == text).then_some(goal)
    }
}

/// Eine Gewohnheit. Alles außer `sync_id` und `created` sind LWW-Register.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Habit {
    pub sync_id: String,
    /// Tag der Anlage; ordnet die Liste und ändert sich nicht.
    pub created: NaiveDate,
    pub name: Register<String>,
    pub schedule: Register<String>,
    pub goal: Register<String>,
    pub archived: Register<bool>,
    pub deleted: Register<bool>,
}

impl Habit {
    /// Neue Gewohnheit; `name` muss vorher mit [`validate_name`] geprüft sein.
    pub fn new(
        sync_id: impl Into<String>,
        created: NaiveDate,
        name: &str,
        schedule: &Schedule,
        goal: &Goal,
        clock: &Hlc,
    ) -> Self {
        Self {
            sync_id: sync_id.into(),
            created,
            name: Register::new(name.trim().to_owned(), clock.clone()),
            schedule: Register::new(schedule.encode(), clock.clone()),
            goal: Register::new(goal.encode(), clock.clone()),
            archived: Register::new(false, clock.clone()),
            deleted: Register::new(false, clock.clone()),
        }
    }

    /// Rhythmus; `None`, wenn die Kodierung dieser App unbekannt ist (neuere Version).
    pub fn schedule(&self) -> Option<Schedule> {
        Schedule::parse(&self.schedule.value)
    }

    /// Ziel; `None`, wenn die Kodierung dieser App unbekannt ist (neuere Version).
    pub fn goal(&self) -> Option<Goal> {
        Goal::parse(&self.goal.value)
    }

    pub fn set_name(&mut self, name: &str, clock: Hlc) {
        self.name.set(name.trim().to_owned(), clock);
    }

    pub fn set_schedule(&mut self, schedule: &Schedule, clock: Hlc) {
        self.schedule.set(schedule.encode(), clock);
    }

    pub fn set_goal(&mut self, goal: &Goal, clock: Hlc) {
        self.goal.set(goal.encode(), clock);
    }

    pub fn set_archived(&mut self, archived: bool, clock: Hlc) {
        self.archived.set(archived, clock);
    }

    pub fn delete(&mut self, clock: Hlc) {
        self.deleted.set(true, clock);
    }

    /// Der jüngste Stempel einer inhaltlichen Änderung.
    fn latest_edit(&self) -> &Hlc {
        [
            &self.name.clock,
            &self.schedule.clock,
            &self.goal.clock,
            &self.archived.clock,
        ]
        .into_iter()
        .max()
        .unwrap_or(&self.name.clock)
    }

    /// Gelöscht gilt nur, solange keine spätere Änderung vorliegt (wie bei Todos).
    pub fn is_deleted(&self) -> bool {
        self.deleted.value && self.deleted.clock >= *self.latest_edit()
    }

    pub fn is_collectable(&self, now_ms: i64) -> bool {
        self.is_deleted() && self.deleted.clock.wall_ms < now_ms.saturating_sub(TOMBSTONE_TTL_MS)
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            sync_id: self.sync_id.clone(),
            created: self.created.min(other.created),
            name: self.name.merge(&other.name),
            schedule: self.schedule.merge(&other.schedule),
            goal: self.goal.merge(&other.goal),
            archived: self.archived.merge(&other.archived),
            deleted: self.deleted.merge(&other.deleted),
        }
    }

    /// Kappt zu weit vorauseilende Stempel und überlange Namen eines fremden Geräts.
    pub fn sanitized(&self, now_ms: i64) -> Self {
        let mut name = self.name.capped(now_ms);
        name.value = name.value.chars().take(MAX_NAME_CHARS).collect();
        Self {
            name,
            schedule: self.schedule.capped(now_ms),
            goal: self.goal.capped(now_ms),
            archived: self.archived.capped(now_ms),
            deleted: self.deleted.capped(now_ms),
            ..self.clone()
        }
    }
}

/// Kennung des Hakens für (Gewohnheit, Tag): deterministisch, damit zwei Geräte keine Dubletten anlegen.
pub fn log_id(habit: &str, day: NaiveDate) -> String {
    format!("log:{habit}:{day}")
}

/// Ein Haken für (Gewohnheit, Tag). `done = false` ist der Tombstone („Haken entfernt“).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HabitLog {
    /// `sync_id` der Gewohnheit.
    pub habit: String,
    pub day: NaiveDate,
    pub done: Register<bool>,
}

impl HabitLog {
    pub fn new(habit: impl Into<String>, day: NaiveDate, done: bool, clock: &Hlc) -> Self {
        Self {
            habit: habit.into(),
            day,
            done: Register::new(done, clock.clone()),
        }
    }

    pub fn sync_id(&self) -> String {
        log_id(&self.habit, self.day)
    }

    pub fn set_done(&mut self, done: bool, clock: Hlc) {
        self.done.set(done, clock);
    }

    /// Ein entfernter Haken, der älter als [`TOMBSTONE_TTL_MS`] ist, darf verschwinden.
    pub fn is_collectable(&self, now_ms: i64) -> bool {
        !self.done.value && self.done.clock.wall_ms < now_ms.saturating_sub(TOMBSTONE_TTL_MS)
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            habit: self.habit.clone(),
            day: self.day,
            done: self.done.merge(&other.done),
        }
    }

    pub fn sanitized(&self, now_ms: i64) -> Self {
        Self {
            done: self.done.capped(now_ms),
            ..self.clone()
        }
    }
}

/// Einheit der Serie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum StreakUnit {
    Days,
    Weeks,
}

/// Stand einer Gewohnheit am Tag `today`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HabitProgress {
    pub done_today: bool,
    /// Heute noch zu tun: fällig und nicht erledigt (beim Wochenziel: Wochenziel noch nicht erreicht).
    pub open_today: bool,
    pub current_streak: u32,
    pub best_streak: u32,
    pub unit: StreakUnit,
    /// Erledigte Tage in der laufenden Woche (Montag bis Sonntag).
    pub week_done: u32,
    /// Ziel der Woche: Anzahl fälliger Tage bzw. das Wochenziel.
    pub week_target: u32,
}

fn week_start(day: NaiveDate) -> NaiveDate {
    day - Duration::days(i64::from(day.weekday().num_days_from_monday()))
}

/// Wertet eine Gewohnheit aus. `done` sind die erledigten Tage (Haken oder berechnet), Tage nach `today` zählen nicht.
pub fn habit_progress(
    schedule: &Schedule,
    done: &BTreeSet<NaiveDate>,
    today: NaiveDate,
) -> HabitProgress {
    let done: BTreeSet<NaiveDate> = done.range(..=today).copied().collect();
    let done_today = done.contains(&today);
    let monday = week_start(today);
    let week_done = u32::try_from(done.range(monday..=today).count()).unwrap_or(u32::MAX);
    let (unit, current_streak, best_streak) = match schedule {
        Schedule::Weekly { per_week } => {
            let (current, best) = week_streaks(&done, today, u32::from(*per_week));
            (StreakUnit::Weeks, current, best)
        }
        _ => {
            let (current, best) = day_streaks(schedule, &done, today);
            (StreakUnit::Days, current, best)
        }
    };
    let week_target = match schedule {
        Schedule::Daily => 7,
        Schedule::Days { days } => u32::try_from(days.len()).unwrap_or(7),
        Schedule::Weekly { per_week } => u32::from(*per_week),
    };
    let open_today = !done_today
        && match schedule {
            Schedule::Weekly { per_week } => week_done < u32::from(*per_week),
            other => other.is_due_on(today),
        };
    HabitProgress {
        done_today,
        open_today,
        current_streak,
        best_streak,
        unit,
        week_done,
        week_target,
    }
}

/// Serie in Tagen über die fälligen Tage: (laufend, beste). Der offene heutige Tag bricht die Serie nicht.
fn day_streaks(schedule: &Schedule, done: &BTreeSet<NaiveDate>, today: NaiveDate) -> (u32, u32) {
    let Some(&first) = done.first() else {
        return (0, 0);
    };
    let mut current = 0;
    let mut day = today;
    if schedule.is_due_on(today) && !done.contains(&today) {
        day = today - Duration::days(1);
    }
    while day >= first {
        if schedule.is_due_on(day) {
            if !done.contains(&day) {
                break;
            }
            current += 1;
        }
        day -= Duration::days(1);
    }
    let (mut run, mut best) = (0, 0);
    for day in first.iter_days().take_while(|day| *day <= today) {
        if schedule.is_due_on(day) {
            if done.contains(&day) {
                run += 1;
                best = best.max(run);
            } else if day != today {
                run = 0;
            }
        }
    }
    (current, best)
}

/// Serie in Wochen, in denen mindestens `target` Tage erledigt sind: (laufend, beste). Die laufende Woche bricht
/// die Serie nicht, solange sie ihr Ziel noch erreichen kann.
fn week_streaks(done: &BTreeSet<NaiveDate>, today: NaiveDate, target: u32) -> (u32, u32) {
    let Some(&first) = done.first() else {
        return (0, 0);
    };
    let mut per_week: BTreeMap<NaiveDate, u32> = BTreeMap::new();
    for day in done {
        *per_week.entry(week_start(*day)).or_insert(0) += 1;
    }
    let met = |week: NaiveDate| per_week.get(&week).copied().unwrap_or(0) >= target;
    let this_week = week_start(today);
    let mut week = this_week;
    if !met(week) {
        week -= Duration::days(7);
    }
    let mut current = 0;
    while week >= week_start(first) && met(week) {
        current += 1;
        week -= Duration::days(7);
    }
    let (mut run, mut best) = (0, 0);
    let mut week = week_start(first);
    while week <= this_week {
        if met(week) {
            run += 1;
            best = best.max(run);
        } else if week != this_week {
            run = 0;
        }
        week += Duration::days(7);
    }
    (current, best)
}

/// Tage, an denen die Arbeitszeit mit dem Tag `tag` mindestens `minutes` Minuten erreicht (Ziel `minutes`).
/// Gezählt werden Arbeitsintervalle, die nicht übersprungen wurden; der Tag ist der lokale Tag des Beginns.
pub fn minutes_goal_days(
    sessions: &[Session],
    todos: &[Todo],
    tag: &str,
    minutes: u32,
    local_date: &dyn Fn(DateTime<Utc>) -> NaiveDate,
) -> BTreeSet<NaiveDate> {
    let by_id: BTreeMap<i32, &Todo> = todos.iter().map(|todo| (todo.id, todo)).collect();
    let mut per_day: BTreeMap<NaiveDate, u32> = BTreeMap::new();
    for session in sessions {
        if session.session_type != SessionType::Work || session.status == SessionStatus::Skipped {
            continue;
        }
        let todo = session.todo_id.and_then(|id| by_id.get(&id).copied());
        let (projects, contexts) = effective_tags(session, todo);
        let tags = if tag.starts_with('+') {
            projects
        } else {
            contexts
        };
        if tags.iter().any(|t| t == tag) {
            let day = local_date(session.started_at);
            *per_day.entry(day).or_insert(0) += session.actual_min.unwrap_or(0);
        }
    }
    per_day
        .into_iter()
        .filter(|(_, total)| *total >= minutes)
        .map(|(day, _)| day)
        .collect()
}

/// Wie viele Tage (einschließlich heute) die Übersicht als zuletzt erledigte Tage mitliefert.
pub const RECENT_DAYS: i64 = 35;

/// Eine Gewohnheit in der Übersicht: Angaben, Stand und die zuletzt erledigten Tage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HabitItem {
    pub id: String,
    pub name: String,
    pub created: NaiveDate,
    pub archived: bool,
    /// `None`, wenn die Kodierung unbekannt ist (neuere App).
    pub schedule: Option<Schedule>,
    /// `None`, wenn die Kodierung unbekannt ist (neuere App).
    pub goal: Option<Goal>,
    /// `None`, wenn Rhythmus oder Ziel nicht auswertbar sind.
    pub progress: Option<HabitProgress>,
    /// Erledigte Tage der letzten [`RECENT_DAYS`] Tage, aufsteigend.
    pub recent: Vec<NaiveDate>,
}

/// Übersicht über `habits` (Reihenfolge bleibt) am Tag `today`. `logs` sind die Haken aller Gewohnheiten; bei Ziel
/// `check` zählen die gesetzten, bei Ziel `minutes` die Tage aus `sessions` und `todos`.
pub fn habit_overview(
    habits: &[Habit],
    logs: &[HabitLog],
    sessions: &[Session],
    todos: &[Todo],
    today: NaiveDate,
    local_date: &dyn Fn(DateTime<Utc>) -> NaiveDate,
) -> Vec<HabitItem> {
    let recent_from = today - Duration::days(RECENT_DAYS - 1);
    habits
        .iter()
        .map(|habit| {
            let (schedule, goal) = (habit.schedule(), habit.goal());
            let done: BTreeSet<NaiveDate> = match &goal {
                Some(Goal::Minutes { minutes, tag }) => {
                    minutes_goal_days(sessions, todos, tag, *minutes, local_date)
                }
                _ => logs
                    .iter()
                    .filter(|log| log.habit == habit.sync_id && log.done.value)
                    .map(|log| log.day)
                    .collect(),
            };
            let progress = match (&schedule, &goal) {
                (Some(schedule), Some(_)) => Some(habit_progress(schedule, &done, today)),
                _ => None,
            };
            HabitItem {
                id: habit.sync_id.clone(),
                name: habit.name.value.clone(),
                created: habit.created,
                archived: habit.archived.value,
                schedule,
                goal,
                progress,
                recent: done.range(recent_from..=today).copied().collect(),
            }
        })
        .collect()
}
