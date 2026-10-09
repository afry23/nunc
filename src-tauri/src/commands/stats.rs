use std::io::Write;

use nunc_core::export::{ExportDetails, export_rows, to_csv, to_json};
use nunc_core::model::{StatsRequest, StatsResponse};
use nunc_core::stats::compute_stats;
use nunc_platform::{to_local, to_local_date};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Csv,
    Json,
}

impl ExportFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
        }
    }
}

/// Kennzahlen aus den rohen Sessions (nichts davon ist gespeichert).
#[tauri::command]
#[specta::specta]
pub fn get_stats(
    state: State<'_, AppState>,
    request: StatsRequest,
) -> Result<StatsResponse, AppError> {
    let (sessions, todos) = load_sessions_and_todos(&state)?;
    Ok(compute_stats(
        &sessions,
        &todos,
        &request,
        state.today(),
        &to_local_date,
    ))
}

/// Export der Sessions im Zeitraum als CSV- oder JSON-Text.
#[tauri::command]
#[specta::specta]
pub fn export_stats(
    state: State<'_, AppState>,
    request: StatsRequest,
    format: ExportFormat,
) -> Result<String, AppError> {
    build_export(&state, &request, format)
}

/// Schreibt den Export in eine vom Nutzer gewählte Datei. `None`, wenn der Dialog abgebrochen wurde.
#[tauri::command]
#[specta::specta]
pub async fn save_export_to_file<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    request: StatsRequest,
    format: ExportFormat,
) -> Result<Option<String>, AppError> {
    let content = build_export(&state, &request, format)?;
    let extension = format.extension();
    let file_name = format!(
        "nunc-{}-{}.{extension}",
        request.start_date.format("%Y-%m-%d"),
        request.end_date.format("%Y-%m-%d")
    );
    let Some(target) = app
        .dialog()
        .file()
        .set_title("Statistik exportieren")
        .add_filter(extension.to_uppercase(), &[extension])
        .set_file_name(file_name)
        .blocking_save_file()
    else {
        return Ok(None);
    };
    write_file(&app, target.clone(), content.as_bytes())?;
    Ok(Some(target.to_string()))
}

fn load_sessions_and_todos(
    state: &AppState,
) -> Result<(Vec<nunc_core::model::Session>, Vec<nunc_core::model::Todo>), AppError> {
    state.db.with(|conn| {
        Ok::<_, nunc_store::StoreError>((
            nunc_store::list_sessions(conn)?,
            nunc_store::list_all_todos(conn)?,
        ))
    })
}

fn build_export(
    state: &AppState,
    request: &StatsRequest,
    format: ExportFormat,
) -> Result<String, AppError> {
    let (sessions, todos) = load_sessions_and_todos(state)?;
    let details = state
        .db
        .with(|conn| nunc_store::list_session_details(conn))?
        .into_iter()
        .map(|(id, d)| {
            (
                id,
                ExportDetails {
                    note: d.note,
                    origin: d.origin,
                },
            )
        })
        .collect();
    let rows = export_rows(&sessions, &todos, request, &details, &to_local);
    match format {
        ExportFormat::Csv => Ok(to_csv(&rows)),
        ExportFormat::Json => to_json(&rows)
            .map_err(|error| AppError::internal(format!("Export nicht serialisierbar: {error}"))),
    }
}

/// Schreibt über das Fs-Plugin, damit auch Ziele des Datei-Pickers auf Mobilgeräten funktionieren.
pub(crate) fn write_file<R: Runtime>(
    app: &AppHandle<R>,
    target: FilePath,
    content: &[u8],
) -> Result<(), AppError> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    let mut file = app.fs().open(target, options)?;
    file.write_all(content)?;
    Ok(())
}
