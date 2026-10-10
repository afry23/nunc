//! System-Tray (nur Desktop): Symbol je Zustand (bereit/Fokus/Pause), Menü mit Anzeigen, Verbergen,
//! Start/Pause/Stop, Statuszeile und Beenden. Der Tray folgt dem Timer ausschließlich über `timer:tick`.

use nunc_core::model::{SessionType, TimerState};
use nunc_core::timer::TimerSnapshot;
use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Listener, Manager, Runtime};

use crate::error::AppError;
use crate::events::TimerTick;
use crate::state::AppState;
use crate::windows;
use tauri_specta::Event;

const TRAY_ID: &str = "main";
const ICON_SIZE: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayColor {
    /// Bereit oder pausiert.
    Idle,
    /// Fokus läuft.
    Work,
    /// Pause läuft.
    Break,
}

impl TrayColor {
    fn rgb(self) -> [u8; 3] {
        match self {
            Self::Idle => [0x9a, 0xa4, 0xb2],
            Self::Work => [0xe5, 0x53, 0x4b],
            Self::Break => [0x3f, 0xb9, 0x50],
        }
    }
}

/// Was der Tray für einen Timer-Stand anzeigt.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TrayView {
    color: TrayColor,
    status: String,
    /// Beschriftung des Pause-Eintrags: „Pause“ oder „Fortsetzen“.
    pause_label: &'static str,
    can_start: bool,
    can_pause: bool,
    can_stop: bool,
}

fn view(snapshot: &TimerSnapshot) -> TrayView {
    let kind = match (snapshot.session_type, snapshot.mode) {
        (SessionType::Work, nunc_core::model::TimerMode::Pomodoro) => "Fokus",
        (SessionType::Work, nunc_core::model::TimerMode::Flowtime) => "Flow",
        (SessionType::Work, nunc_core::model::TimerMode::Stopwatch) => "Stoppuhr",
        (SessionType::ShortBreak | SessionType::LongBreak, _) => "Pause",
    };
    let time = format_clock(snapshot.remaining);
    match snapshot.state {
        TimerState::Idle => TrayView {
            color: TrayColor::Idle,
            status: "Bereit".to_owned(),
            pause_label: "Pause",
            can_start: true,
            can_pause: false,
            can_stop: false,
        },
        TimerState::Running => TrayView {
            color: if snapshot.session_type == SessionType::Work {
                TrayColor::Work
            } else {
                TrayColor::Break
            },
            status: format!("{kind} · {time}"),
            pause_label: "Pause",
            can_start: false,
            can_pause: true,
            can_stop: true,
        },
        TimerState::Paused => TrayView {
            color: TrayColor::Idle,
            status: format!("{kind} pausiert · {time}"),
            pause_label: "Fortsetzen",
            can_start: false,
            can_pause: true,
            can_stop: true,
        },
    }
}

/// `MM:SS`, ab einer Stunde `H:MM:SS`.
fn format_clock(seconds: u32) -> String {
    let (hours, minutes, secs) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes:02}:{secs:02}")
    }
}

/// Gefüllter Kreis auf transparentem Grund (RGBA), Rand leicht geglättet.
fn icon_rgba(color: TrayColor) -> Vec<u8> {
    let [r, g, b] = color.rgb();
    let center = f64::from(ICON_SIZE) / 2.0;
    let radius = center - 3.0;
    let mut pixels = Vec::with_capacity((ICON_SIZE * ICON_SIZE * 4) as usize);
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let distance = (f64::from(x) + 0.5 - center).hypot(f64::from(y) + 0.5 - center);
            let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let alpha = (coverage * 255.0).round() as u8;
            pixels.extend_from_slice(&[r, g, b, alpha]);
        }
    }
    pixels
}

fn icon(color: TrayColor) -> Image<'static> {
    Image::new_owned(icon_rgba(color), ICON_SIZE, ICON_SIZE)
}

/// Menüeinträge, die dem Timer-Stand folgen.
struct TrayItems<R: Runtime> {
    status: MenuItem<R>,
    start: MenuItem<R>,
    pause: MenuItem<R>,
    stop: MenuItem<R>,
}

/// Legt den Tray an. Sichtbar ist er nur, wenn die Einstellung `systemTrayEnabled` es erlaubt.
pub fn setup<R: Runtime>(app: &AppHandle<R>, visible: bool) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "Bereit", false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Anzeigen", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Verbergen", true, None::<&str>)?;
    let start = MenuItem::with_id(app, "start", "Start", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause", false, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Stop", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
    let separator = || PredefinedMenuItem::separator(app);
    let menu = Menu::with_items(
        app,
        &[
            &status,
            &separator()?,
            &show,
            &hide,
            &separator()?,
            &start,
            &pause,
            &stop,
            &separator()?,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon(TrayColor::Idle))
        .tooltip("nunc")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu_event(app, &event))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                windows::toggle_main(tray.app_handle());
            }
        })
        .build(app)?;
    set_visible(app, visible);

    app.manage(TrayItems {
        status,
        start,
        pause,
        stop,
    });

    let handle = app.clone();
    app.listen(TimerTick::NAME, move |event| {
        match serde_json::from_str::<TimerSnapshot>(event.payload()) {
            Ok(snapshot) => update(&handle, &snapshot),
            Err(error) => log::warn!("timer:tick nicht lesbar: {error}"),
        }
    });
    if let Some(state) = app.try_state::<AppState>()
        && let Ok(snapshot) = state.timer.snapshot()
    {
        update(app, &snapshot);
    }
    Ok(())
}

/// Zeigt oder verbirgt das Tray-Symbol (Einstellung `systemTrayEnabled`).
pub fn set_visible<R: Runtime>(app: &AppHandle<R>, visible: bool) {
    if let Some(tray) = app.tray_by_id(TRAY_ID)
        && let Err(error) = tray.set_visible(visible)
    {
        log::warn!("Tray-Symbol nicht umschaltbar: {error}");
    }
}

fn update<R: Runtime>(app: &AppHandle<R>, snapshot: &TimerSnapshot) {
    let view = view(snapshot);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_icon(Some(icon(view.color)));
        let _ = tray.set_tooltip(Some(format!("nunc · {}", view.status)));
    }
    if let Some(items) = app.try_state::<TrayItems<R>>() {
        let _ = items.status.set_text(&view.status);
        let _ = items.pause.set_text(view.pause_label);
        let _ = items.start.set_enabled(view.can_start);
        let _ = items.pause.set_enabled(view.can_pause);
        let _ = items.stop.set_enabled(view.can_stop);
    }
}

fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: &MenuEvent) {
    match event.id().as_ref() {
        "show" => windows::show_main(app),
        "hide" => windows::hide_to_tray(app),
        "quit" => app.exit(0),
        id @ ("start" | "pause" | "stop") => {
            let Some(state) = app.try_state::<AppState>() else {
                return;
            };
            if let Err(error) = timer_action(&state, id) {
                log::warn!("Tray-Aktion {id} fehlgeschlagen: {error}");
            }
        }
        _ => {}
    }
}

/// Start, Pause/Fortsetzen oder Stop aus dem Tray; Start nutzt den eingestellten Modus ohne Todo.
fn timer_action(state: &AppState, action: &str) -> Result<(), AppError> {
    match action {
        "start" => {
            let mode = state
                .db
                .with(|conn| nunc_store::load_settings(conn))?
                .active_mode;
            state.timer.start(mode, None).map(drop)
        }
        "pause" => {
            if state.timer.snapshot()?.state == TimerState::Paused {
                state.timer.resume().map(drop)
            } else {
                state.timer.pause().map(drop)
            }
        }
        _ => state.timer.stop().map(drop),
    }
}

#[cfg(test)]
mod tests {
    use nunc_core::model::TimerMode;

    use super::*;

    fn snapshot(state: TimerState, session_type: SessionType, remaining: u32) -> TimerSnapshot {
        TimerSnapshot {
            state,
            mode: TimerMode::Pomodoro,
            session_type,
            remaining,
            total: 1500,
            todo_id: None,
        }
    }

    #[test]
    fn idle_offers_only_start() {
        let v = view(&snapshot(TimerState::Idle, SessionType::Work, 0));
        assert_eq!(v.color, TrayColor::Idle);
        assert_eq!(v.status, "Bereit");
        assert_eq!((v.can_start, v.can_pause, v.can_stop), (true, false, false));
    }

    #[test]
    fn a_running_pomodoro_is_red_and_shows_the_remaining_time() {
        let v = view(&snapshot(TimerState::Running, SessionType::Work, 754));
        assert_eq!(v.color, TrayColor::Work);
        assert_eq!(v.status, "Fokus · 12:34");
        assert_eq!(v.pause_label, "Pause");
        assert_eq!((v.can_start, v.can_pause, v.can_stop), (false, true, true));
    }

    #[test]
    fn a_running_break_is_green() {
        let v = view(&snapshot(TimerState::Running, SessionType::ShortBreak, 60));
        assert_eq!(v.color, TrayColor::Break);
        assert_eq!(v.status, "Pause · 01:00");
        let long = view(&snapshot(TimerState::Running, SessionType::LongBreak, 60));
        assert_eq!(long.color, TrayColor::Break);
    }

    #[test]
    fn a_paused_timer_offers_resume_and_is_grey() {
        let v = view(&snapshot(TimerState::Paused, SessionType::Work, 600));
        assert_eq!(v.color, TrayColor::Idle);
        assert_eq!(v.status, "Fokus pausiert · 10:00");
        assert_eq!(v.pause_label, "Fortsetzen");
        assert!(v.can_pause && v.can_stop && !v.can_start);
    }

    #[test]
    fn flowtime_shows_elapsed_time() {
        let mut s = snapshot(TimerState::Running, SessionType::Work, 3725);
        s.mode = TimerMode::Flowtime;
        assert_eq!(view(&s).status, "Flow · 1:02:05");
    }

    #[test]
    fn clock_formats_minutes_and_hours() {
        assert_eq!(format_clock(0), "00:00");
        assert_eq!(format_clock(59), "00:59");
        assert_eq!(format_clock(3599), "59:59");
        assert_eq!(format_clock(3600), "1:00:00");
    }

    #[test]
    fn icons_are_filled_circles_in_the_state_color() {
        for color in [TrayColor::Idle, TrayColor::Work, TrayColor::Break] {
            let pixels = icon_rgba(color);
            assert_eq!(pixels.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
            let at = |x: u32, y: u32| {
                let i = ((y * ICON_SIZE + x) * 4) as usize;
                [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
            };
            let [r, g, b] = color.rgb();
            assert_eq!(at(16, 16), [r, g, b, 255]);
            assert_eq!(at(0, 0)[3], 0);
            assert_eq!(at(31, 31)[3], 0);
        }
        assert_ne!(icon_rgba(TrayColor::Work), icon_rgba(TrayColor::Break));
    }
}
