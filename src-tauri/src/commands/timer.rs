use nunc_core::model::{Labels, TimerMode};
use nunc_core::timer::{BreakSuggestion, TimerSnapshot};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Startet eine Arbeitssession. Die Dauer kommt aus den Einstellungen, Flowtime ist eine Stoppuhr.
#[tauri::command]
#[specta::specta]
pub fn start_timer(
    state: State<'_, AppState>,
    mode: TimerMode,
    todo_id: Option<i32>,
    labels: Option<Labels>,
) -> Result<TimerSnapshot, AppError> {
    state
        .timer
        .start_labelled(mode, todo_id, &labels.unwrap_or_default())
}

/// Startet die vorgeschlagene Pause.
#[tauri::command]
#[specta::specta]
pub fn start_break(state: State<'_, AppState>) -> Result<TimerSnapshot, AppError> {
    state.timer.start_break()
}

#[tauri::command]
#[specta::specta]
pub fn pause_timer(state: State<'_, AppState>) -> Result<TimerSnapshot, AppError> {
    state.timer.pause()
}

#[tauri::command]
#[specta::specta]
pub fn resume_timer(state: State<'_, AppState>) -> Result<TimerSnapshot, AppError> {
    state.timer.resume()
}

/// Beendet die Session vorzeitig (Flowtime: abgeschlossen, Pomodoro: 50-%-Regel, Pause: übersprungen).
#[tauri::command]
#[specta::specta]
pub fn stop_timer(state: State<'_, AppState>) -> Result<TimerSnapshot, AppError> {
    state.timer.stop()
}

/// Aktueller Stand; verbucht vorher einen inzwischen abgelaufenen Countdown (z. B. nach dem Hintergrund).
#[tauri::command]
#[specta::specta]
pub fn get_timer_tick(state: State<'_, AppState>) -> Result<TimerSnapshot, AppError> {
    state.timer.snapshot()
}

/// Die nach der letzten Arbeitssession vorgeschlagene Pause, falls es eine gibt.
#[tauri::command]
#[specta::specta]
pub fn get_suggested_break(
    state: State<'_, AppState>,
) -> Result<Option<BreakSuggestion>, AppError> {
    state.timer.suggested_break()
}
