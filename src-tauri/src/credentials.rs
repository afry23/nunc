//! Zugangsdaten der App bei Google und Todoist (OAuth-Client). Reihenfolge der Quellen:
//!
//! 1. Umgebungsvariable (`NUNC_GOOGLE_CLIENT_ID`, …) beim Start,
//! 2. nur in Debug-Builds eine `.env`-Datei (im aktuellen Ordner oder darüber, auch mit den Namen der Go-App:
//!    `GOOGLE_CLIENT_ID`, `TODOIST_CLIENT_SECRET`, …),
//! 3. der beim Bauen eingesetzte Wert (`option_env!`).
//!
//! Desktop-Clients gelten bei beiden Diensten nicht als geheim, gehören aber nicht ins Repository (`.env` ist
//! in `.gitignore`). Release-Builds lesen nie eine `.env`.

use std::collections::HashMap;
use std::path::PathBuf;

/// Client-ID und -Secret; leer, wenn keine Quelle etwas liefert.
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

/// Schlüssel und Werte einer `.env`-Datei: `NAME=wert`, `export NAME=wert`, Anführungszeichen um den Wert,
/// `#`-Kommentarzeilen. Leere Werte zählen als nicht gesetzt.
fn parse_dotenv(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('#') {
                return None;
            }
            let (name, value) = line
                .strip_prefix("export ")
                .unwrap_or(line)
                .split_once('=')?;
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);
            (!value.is_empty()).then(|| (name.trim().to_owned(), value.to_owned()))
        })
        .collect()
}

/// Erster gesetzter Name aus `names`.
fn first_of(values: &HashMap<String, String>, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| values.get(*name).cloned())
}

/// Die `.env` im aktuellen Ordner oder darüber (`tauri dev` läuft in `src-tauri/`). Nur Debug-Builds.
fn dotenv() -> HashMap<String, String> {
    if !cfg!(debug_assertions) {
        return HashMap::new();
    }
    let Ok(start) = std::env::current_dir() else {
        return HashMap::new();
    };
    start
        .ancestors()
        .take(4)
        .map(|dir| dir.join(".env"))
        .collect::<Vec<PathBuf>>()
        .iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .map(|text| parse_dotenv(&text))
        .unwrap_or_default()
}

fn pick(
    runtime: &str,
    dotenv_names: &[&str],
    dotenv: &HashMap<String, String>,
    built_in: Option<&'static str>,
) -> String {
    std::env::var(runtime)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .or_else(|| first_of(dotenv, dotenv_names))
        .or_else(|| built_in.map(|value| value.trim().to_owned()))
        .filter(|value| !value.is_empty())
        .unwrap_or_default()
}

pub fn google() -> Credentials {
    let env = dotenv();
    Credentials {
        client_id: pick(
            "NUNC_GOOGLE_CLIENT_ID",
            &["NUNC_GOOGLE_CLIENT_ID", "GOOGLE_CLIENT_ID"],
            &env,
            option_env!("NUNC_GOOGLE_CLIENT_ID"),
        ),
        client_secret: pick(
            "NUNC_GOOGLE_CLIENT_SECRET",
            &["NUNC_GOOGLE_CLIENT_SECRET", "GOOGLE_CLIENT_SECRET"],
            &env,
            option_env!("NUNC_GOOGLE_CLIENT_SECRET"),
        ),
    }
}

pub fn todoist() -> Credentials {
    let env = dotenv();
    Credentials {
        client_id: pick(
            "NUNC_TODOIST_CLIENT_ID",
            &["NUNC_TODOIST_CLIENT_ID", "TODOIST_CLIENT_ID"],
            &env,
            option_env!("NUNC_TODOIST_CLIENT_ID"),
        ),
        client_secret: pick(
            "NUNC_TODOIST_CLIENT_SECRET",
            &["NUNC_TODOIST_CLIENT_SECRET", "TODOIST_CLIENT_SECRET"],
            &env,
            option_env!("NUNC_TODOIST_CLIENT_SECRET"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dotenv_file_is_read_line_by_line() {
        let values = parse_dotenv(
            "# Kommentar\n\nGOOGLE_CLIENT_ID=abc\nexport TODOIST_CLIENT_ID = \"def\"\nQ='ghi'\nLEER=\nohne gleichheitszeichen\n  EINGERÜCKT = x  \n",
        );
        assert_eq!(values["GOOGLE_CLIENT_ID"], "abc");
        assert_eq!(values["TODOIST_CLIENT_ID"], "def");
        assert_eq!(values["Q"], "ghi");
        assert_eq!(values["EINGERÜCKT"], "x");
        assert!(!values.contains_key("LEER") && values.len() == 4);
    }

    #[test]
    fn the_first_known_name_wins() {
        let values = parse_dotenv("B=zwei\nC=drei\n");
        assert_eq!(first_of(&values, &["A", "B", "C"]).as_deref(), Some("zwei"));
        assert_eq!(first_of(&values, &["A"]), None);
    }

    #[test]
    fn the_environment_wins_over_the_dotenv_and_the_build_and_blank_values_do_not_count() {
        let env = parse_dotenv("X=aus-dotenv\n");
        // eigene Namen, damit parallele Tests nichts überschreiben
        assert_eq!(pick("NUNC_TEST_UNSET_1", &["Y"], &env, None), "");
        assert_eq!(
            pick("NUNC_TEST_UNSET_2", &["Y"], &env, Some(" eingebaut ")),
            "eingebaut"
        );
        assert_eq!(
            pick("NUNC_TEST_UNSET_3", &["X"], &env, Some("eingebaut")),
            "aus-dotenv"
        );
        // SAFETY: nur dieser Test liest und schreibt diese Variablen
        unsafe { std::env::set_var("NUNC_TEST_SET_4", " laufzeit ") };
        assert_eq!(
            pick("NUNC_TEST_SET_4", &["X"], &env, Some("eingebaut")),
            "laufzeit"
        );
        unsafe { std::env::set_var("NUNC_TEST_SET_5", "  ") };
        assert_eq!(
            pick("NUNC_TEST_SET_5", &["Y"], &env, Some("eingebaut")),
            "eingebaut"
        );
    }
}
