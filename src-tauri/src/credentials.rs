//! Zugangsdaten der App bei Google und Todoist (OAuth-Client). Sie werden beim Bauen eingesetzt
//! (`NUNC_GOOGLE_CLIENT_ID`, …); beim Start überschreiben gleichnamige Umgebungsvariablen sie (Entwicklung).
//! Desktop-Clients gelten bei beiden Diensten nicht als geheim, gehören aber nicht ins Repository.

/// Client-ID und -Secret; leer, wenn der Build keine hat.
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

fn pick(runtime: &str, built_in: Option<&'static str>) -> String {
    std::env::var(runtime)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| built_in.map(str::to_owned))
        .map(|value| value.trim().to_owned())
        .unwrap_or_default()
}

pub fn google() -> Credentials {
    Credentials {
        client_id: pick(
            "NUNC_GOOGLE_CLIENT_ID",
            option_env!("NUNC_GOOGLE_CLIENT_ID"),
        ),
        client_secret: pick(
            "NUNC_GOOGLE_CLIENT_SECRET",
            option_env!("NUNC_GOOGLE_CLIENT_SECRET"),
        ),
    }
}

pub fn todoist() -> Credentials {
    Credentials {
        client_id: pick(
            "NUNC_TODOIST_CLIENT_ID",
            option_env!("NUNC_TODOIST_CLIENT_ID"),
        ),
        client_secret: pick(
            "NUNC_TODOIST_CLIENT_SECRET",
            option_env!("NUNC_TODOIST_CLIENT_SECRET"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_environment_wins_over_the_build_and_blank_values_do_not_count() {
        // eigene Namen, damit parallele Tests nichts überschreiben
        assert_eq!(pick("NUNC_TEST_UNSET_1", None), "");
        assert_eq!(pick("NUNC_TEST_UNSET_2", Some(" eingebaut ")), "eingebaut");
        // SAFETY: nur dieser Test liest und schreibt die Variable
        unsafe { std::env::set_var("NUNC_TEST_SET_3", " laufzeit ") };
        assert_eq!(pick("NUNC_TEST_SET_3", Some("eingebaut")), "laufzeit");
        unsafe { std::env::set_var("NUNC_TEST_SET_4", "  ") };
        assert_eq!(pick("NUNC_TEST_SET_4", Some("eingebaut")), "eingebaut");
    }
}
