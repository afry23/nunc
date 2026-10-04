//! todo.txt-Datei importieren und exportieren. Die Dialoge öffnet der Command (auch Ziele des Datei-Pickers auf
//! Mobilgeräten, die keinen Pfad haben, laufen über das Fs-Plugin); Lesen und Schreiben des Inhalts steckt in
//! [`import_text`] und [`export_text`], damit es ohne Dialog testbar ist.

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FsExt;

use super::stats::write_file;
use crate::error::AppError;
use crate::state::AppState;

/// Größer als das ist keine Aufgabenliste mehr (die Oberfläche bräuchte ewig, der Sync auch).
const MAX_IMPORT_BYTES: usize = 5 * 1024 * 1024;
const EXTENSIONS: &[&str] = &["txt"];

/// Ergebnis eines Imports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TodoImportResult {
    /// Neu angelegte Todos.
    pub added: u32,
    /// Zeilen, die schon vorhanden waren oder mehrfach in der Datei standen.
    pub duplicates: u32,
}

/// Legt die neuen Zeilen einer todo.txt-Datei als Todos an.
pub(crate) fn import_text(state: &AppState, text: &str) -> Result<TodoImportResult, AppError> {
    let now = state.clock.now();
    let result = state.db.with(|conn| {
        nunc_store::import_todotxt(conn, text, now, &mut || uuid::Uuid::new_v4().to_string())
    })?;
    if !result.added.is_empty() {
        state.todos_changed();
    }
    Ok(TodoImportResult {
        added: u32::try_from(result.added.len()).unwrap_or(u32::MAX),
        duplicates: u32::try_from(result.duplicates).unwrap_or(u32::MAX),
    })
}

/// Alle nicht gelöschten Todos als todo.txt-Text.
pub(crate) fn export_text(state: &AppState) -> Result<String, AppError> {
    state.db.with(|conn| nunc_store::export_todotxt(conn, true))
}

/// Text einer gewählten Datei; zu große oder keine UTF-8-Dateien sind ein Eingabefehler.
pub(crate) fn decode_import(bytes: Vec<u8>) -> Result<String, AppError> {
    if bytes.len() > MAX_IMPORT_BYTES {
        return Err(AppError::validation(
            "Die Datei ist zu groß für eine Aufgabenliste (höchstens 5 MB).",
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| AppError::validation("Die Datei ist keine Textdatei (UTF-8)."))
}

/// Fragt nach einer todo.txt-Datei und importiert sie. `None`, wenn der Dialog abgebrochen wurde.
#[tauri::command]
#[specta::specta]
pub async fn import_todotxt_file<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<Option<TodoImportResult>, AppError> {
    let Some(file) = app
        .dialog()
        .file()
        .set_title("todo.txt importieren")
        .add_filter("todo.txt", EXTENSIONS)
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let text = decode_import(app.fs().read(file)?)?;
    import_text(&state, &text).map(Some)
}

/// Schreibt alle Todos in eine vom Nutzer gewählte Datei. Liefert den Pfad, `None` bei Abbruch.
#[tauri::command]
#[specta::specta]
pub async fn save_todotxt_file<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<Option<String>, AppError> {
    let content = export_text(&state)?;
    let Some(target) = app
        .dialog()
        .file()
        .set_title("Aufgaben exportieren")
        .add_filter("todo.txt", EXTENSIONS)
        .set_file_name("todo.txt")
        .blocking_save_file()
    else {
        return Ok(None);
    };
    write_file(&app, target.clone(), content.as_bytes())?;
    Ok(Some(target.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_utf8_is_an_input_error() {
        let error = decode_import(vec![0xff, 0xfe, 0x41]).unwrap_err();
        assert_eq!(error.code, "invalid_input");
    }

    #[test]
    fn oversized_files_are_rejected() {
        let error = decode_import(vec![b'a'; MAX_IMPORT_BYTES + 1]).unwrap_err();
        assert_eq!(error.code, "invalid_input");
        assert!(decode_import(vec![b'a'; MAX_IMPORT_BYTES]).is_ok());
    }

    #[test]
    fn utf8_text_passes_unchanged() {
        assert_eq!(
            decode_import("Käse @shop".as_bytes().to_vec()).unwrap(),
            "Käse @shop"
        );
    }
}
