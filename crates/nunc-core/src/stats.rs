//! Statistik aus rohen Sessions (pur). Nichts wird vorab aggregiert gespeichert (CLAUDE.md „Do Not“).
//!
//! Zählregeln (Go-App `services/stats.go`, mit den unten genannten Abweichungen):
//! - **Pomodoro** = Session mit Modus `pomodoro`, Typ `work` und Status `completed`. Flowtime-Sessions zählen
//!   nicht als Pomodoro (auch nicht für Serien).
//! - **Arbeitszeit** = `actual_min` aller Arbeitssessions (abgeschlossen **und** abgebrochen), **Pausenzeit** =
//!   `actual_min` aller Pausen. Abweichung: Go summierte Gesamt- und Projektzeit nur über abgeschlossene Sessions,
//!   die Tageswerte aber über alle; hier gilt überall dasselbe.
//! - Ein Tag ist der **lokale Kalendertag**: der Aufrufer liefert `local_date` (Go gruppierte nach UTC-Datum).

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};

use crate::model::{
    ContextStats, DailyStats, MonthlyStats, ProjectStats, Session, SessionStatus, SessionType,
    StatsRequest, StatsResponse, TimerMode, Todo, TodoStats, WeekdayStats,
};

/// Anzahl der Todos in der Top-Liste.
pub const TOP_TODOS: usize = 10;

/// Datum einer Zeit in UTC (Standard für Tests und Rechner ohne Zeitzone).
pub fn utc_date(time: DateTime<Utc>) -> NaiveDate {
    time.date_naive()
}

fn is_pomodoro(session: &Session) -> bool {
    session.mode == TimerMode::Pomodoro
        && session.session_type == SessionType::Work
        && session.status == SessionStatus::Completed
}

fn minutes(session: &Session) -> u32 {
    session.actual_min.unwrap_or(0)
}

/// Berechnet alle Kennzahlen.
///
/// `request` begrenzt (inklusive, lokale Tage) die Auswertungen nach Tag, Projekt, Kontext, Todo, Wochentag und
/// Monat. Summen, Durchschnitte und Serien nutzen alle Sessions; `today` ist der Bezugstag der Durchschnitte und
/// Serien.
pub fn compute_stats(
    sessions: &[Session],
    todos: &[Todo],
    request: &StatsRequest,
    today: NaiveDate,
    local_date: &dyn Fn(DateTime<Utc>) -> NaiveDate,
) -> StatsResponse {
    let dated: Vec<(NaiveDate, &Session)> = sessions
        .iter()
        .map(|session| (local_date(session.started_at), session))
        .collect();
    let in_range = |date: NaiveDate| date >= request.start_date && date <= request.end_date;
    let todo_by_id: HashMap<i32, &Todo> = todos.iter().map(|todo| (todo.id, todo)).collect();

    let mut daily: BTreeMap<NaiveDate, DailyStats> = BTreeMap::new();
    let mut monthly: BTreeMap<(i32, u32), MonthlyStats> = BTreeMap::new();
    let mut weekdays: Vec<WeekdayStats> = (0..7)
        .map(|weekday| WeekdayStats {
            weekday,
            pomodoro_count: 0,
            work_minutes: 0,
        })
        .collect();
    let mut projects: BTreeMap<&str, u32> = BTreeMap::new();
    let mut contexts: BTreeMap<&str, u32> = BTreeMap::new();
    let mut per_todo: BTreeMap<i32, (u32, u32)> = BTreeMap::new();

    for &(date, session) in dated.iter().filter(|(date, _)| in_range(*date)) {
        let is_work = session.session_type == SessionType::Work;
        let pomodoro = u32::from(is_pomodoro(session));
        let (work, rest) = if is_work {
            (minutes(session), 0)
        } else {
            (0, minutes(session))
        };

        let day = daily.entry(date).or_insert_with(|| DailyStats {
            date,
            pomodoro_count: 0,
            work_minutes: 0,
            break_minutes: 0,
            pomodoro_minutes: 0,
            flowtime_count: 0,
            flowtime_minutes: 0,
            stopwatch_count: 0,
            stopwatch_minutes: 0,
        });
        day.pomodoro_count += pomodoro;
        day.work_minutes += work;
        day.break_minutes += rest;
        let completed = u32::from(is_work && session.status == SessionStatus::Completed);
        match session.mode {
            _ if !is_work => {}
            TimerMode::Pomodoro => day.pomodoro_minutes += work,
            TimerMode::Flowtime => {
                day.flowtime_minutes += work;
                day.flowtime_count += completed;
            }
            TimerMode::Stopwatch => {
                day.stopwatch_minutes += work;
                day.stopwatch_count += completed;
            }
        }

        let month = monthly
            .entry((date.year(), date.month()))
            .or_insert_with(|| MonthlyStats {
                year: date.year(),
                month: date.month(),
                pomodoro_count: 0,
                work_minutes: 0,
            });
        month.pomodoro_count += pomodoro;
        month.work_minutes += work;

        let weekday = &mut weekdays[date.weekday().num_days_from_monday() as usize];
        weekday.pomodoro_count += pomodoro;
        weekday.work_minutes += work;

        if !is_work {
            continue;
        }
        let Some(todo) = session.todo_id.and_then(|id| todo_by_id.get(&id)) else {
            continue;
        };
        for tag in &todo.project_tags {
            *projects.entry(tag).or_default() += work;
        }
        for tag in &todo.context_tags {
            *contexts.entry(tag).or_default() += work;
        }
        let entry = per_todo.entry(todo.id).or_default();
        entry.0 += pomodoro;
        entry.1 += work;
    }

    let mut project_stats: Vec<ProjectStats> = projects
        .into_iter()
        .map(|(project, work_minutes)| ProjectStats {
            project: project.to_owned(),
            work_minutes,
        })
        .collect();
    project_stats.sort_by(|a, b| {
        b.work_minutes
            .cmp(&a.work_minutes)
            .then_with(|| a.project.cmp(&b.project))
    });

    let mut context_stats: Vec<ContextStats> = contexts
        .into_iter()
        .map(|(context, work_minutes)| ContextStats {
            context: context.to_owned(),
            work_minutes,
        })
        .collect();
    context_stats.sort_by(|a, b| {
        b.work_minutes
            .cmp(&a.work_minutes)
            .then_with(|| a.context.cmp(&b.context))
    });

    let mut top_todos: Vec<TodoStats> = per_todo
        .into_iter()
        .filter(|(_, (pomodoros, _))| *pomodoros > 0)
        .map(|(todo_id, (pomodoro_count, work_minutes))| TodoStats {
            todo_id,
            description: todo_by_id[&todo_id].description.clone(),
            pomodoro_count,
            work_minutes,
        })
        .collect();
    top_todos.sort_by(|a, b| {
        b.pomodoro_count
            .cmp(&a.pomodoro_count)
            .then_with(|| b.work_minutes.cmp(&a.work_minutes))
            .then_with(|| a.todo_id.cmp(&b.todo_id))
    });
    top_todos.truncate(TOP_TODOS);

    let pomodoro_dates: Vec<NaiveDate> = dated
        .iter()
        .filter(|(_, session)| is_pomodoro(session))
        .map(|(date, _)| *date)
        .collect();
    let (current_streak, longest_streak) = calculate_streaks(&pomodoro_dates, today);

    StatsResponse {
        daily: daily.into_values().collect(),
        projects: project_stats,
        contexts: context_stats,
        top_todos,
        weekdays,
        monthly: monthly.into_values().collect(),
        total_pomodoros: pomodoro_dates.len() as u32,
        total_work_min: dated
            .iter()
            .filter(|(_, session)| session.session_type == SessionType::Work)
            .map(|(_, session)| minutes(session))
            .sum(),
        average_pomodoros_7: average_per_day(&pomodoro_dates, today, 7),
        average_pomodoros_30: average_per_day(&pomodoro_dates, today, 30),
        current_streak,
        longest_streak,
    }
}

/// Pomodoros pro Tag über die letzten `days` Tage bis einschließlich `today` (Tage ohne Pomodoro zählen mit).
fn average_per_day(pomodoro_dates: &[NaiveDate], today: NaiveDate, days: i64) -> f64 {
    let first = today - Duration::days(days - 1);
    let count = pomodoro_dates
        .iter()
        .filter(|date| **date >= first && **date <= today)
        .count();
    count as f64 / days as f64
}

/// Aktuelle und längste Serie aufeinanderfolgender Tage mit mindestens einem Pomodoro.
///
/// `dates` darf unsortiert sein und Duplikate enthalten. Die aktuelle Serie ist die Folge, die am letzten Tag
/// endet, aber nur, wenn dieser heute oder gestern ist; sonst `0`.
pub fn calculate_streaks(dates: &[NaiveDate], today: NaiveDate) -> (u32, u32) {
    let mut days = dates.to_vec();
    days.sort_unstable();
    days.dedup();
    let Some(&last) = days.last() else {
        return (0, 0);
    };

    let mut longest = 1;
    let mut run = 1;
    for pair in days.windows(2) {
        run = if pair[1] - pair[0] == Duration::days(1) {
            run + 1
        } else {
            1
        };
        longest = longest.max(run);
    }

    let current = if last == today || last == today - Duration::days(1) {
        run
    } else {
        0
    };
    (current, longest)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::model::StatsPeriod;

    fn todo(id: i32) -> Todo {
        Todo {
            id,
            raw_text: format!("Todo {id}"),
            priority: None,
            completed: false,
            completed_at: None,
            created_at: None,
            description: format!("Todo {id}"),
            project_tags: vec![],
            context_tags: vec![],
            pomodoro_count: 0,
            position: id as u32,
        }
    }

    fn pomodoro(day: u32, todo_id: i32) -> Session {
        Session {
            id: 0,
            mode: TimerMode::Pomodoro,
            session_type: SessionType::Work,
            status: SessionStatus::Completed,
            started_at: Utc.with_ymd_and_hms(2025, 3, day, 9, 0, 0).unwrap(),
            ended_at: None,
            planned_min: None,
            actual_min: Some(25),
            todo_id: Some(todo_id),
            break_min: None,
        }
    }

    #[test]
    fn the_top_list_is_limited_to_ten_todos_with_the_most_pomodoros() {
        let todos: Vec<Todo> = (1..=12).map(todo).collect();
        // Todo n bekommt n Pomodoros an verschiedenen Tagen
        let sessions: Vec<Session> = (1..=12)
            .flat_map(|id| (1..=id as u32).map(move |day| pomodoro(day, id)))
            .collect();
        let day = |d: u32| NaiveDate::from_ymd_opt(2025, 3, d).unwrap();
        let request = StatsRequest {
            period: StatsPeriod::Month,
            start_date: day(1),
            end_date: day(31),
        };
        let stats = compute_stats(&sessions, &todos, &request, day(31), &utc_date);
        let ids: Vec<i32> = stats.top_todos.iter().map(|t| t.todo_id).collect();
        assert_eq!(stats.top_todos.len(), TOP_TODOS);
        assert_eq!(ids, [12, 11, 10, 9, 8, 7, 6, 5, 4, 3]);
    }

    #[test]
    fn an_inverted_range_selects_nothing() {
        let day = |d: u32| NaiveDate::from_ymd_opt(2025, 3, d).unwrap();
        let request = StatsRequest {
            period: StatsPeriod::Day,
            start_date: day(10),
            end_date: day(9),
        };
        let stats = compute_stats(
            &[pomodoro(9, 1), pomodoro(10, 1)],
            &[todo(1)],
            &request,
            day(10),
            &utc_date,
        );
        assert!(stats.daily.is_empty());
        assert_eq!(stats.total_pomodoros, 2);
    }
}
