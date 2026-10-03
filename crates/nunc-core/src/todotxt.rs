//! todo.txt-Parser und -Serializer (Verhalten der Go-App, `services/todos.go`).
//!
//! Unterstützt: `(A)`-Priorität, `YYYY-MM-DD`-Erstellungsdatum, `x`-Erledigt-Präfix mit Erledigungs- und
//! Erstellungsdatum, `+projekt`- und `@kontext`-Tags. Alles ist pur: nur die Eingabe bestimmt das Ergebnis.

use chrono::NaiveDate;

/// Ergebnis des Parsens einer todo.txt-Zeile (ohne Datenbank-Felder wie ID oder Position).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedTodo {
    /// `A`–`Z`. Erledigte Todos tragen keine Priorität (todo.txt-Spezifikation).
    pub priority: Option<char>,
    pub completed: bool,
    pub completed_at: Option<NaiveDate>,
    pub created_at: Option<NaiveDate>,
    /// Rest der Zeile nach Präfix und Datumsangaben, **inklusive** der Tags, unverändert (nicht getrimmt).
    pub description: String,
    /// Wörter mit `+`-Präfix (länger als das Präfix allein), in Reihenfolge, Duplikate bleiben.
    pub project_tags: Vec<String>,
    /// Wörter mit `@`-Präfix (länger als das Präfix allein), in Reihenfolge, Duplikate bleiben.
    pub context_tags: Vec<String>,
}

/// Parst eine einzelne todo.txt-Zeile.
///
/// Erkennung nur am Zeilenanfang: `x ` (Kleinbuchstabe, Leerzeichen) markiert erledigt, danach optional
/// Erledigungs- und Erstellungsdatum. Sonst optional `(A) ` (genau ein Großbuchstabe A–Z, Leerzeichen) und
/// ein Erstellungsdatum. Ein Datum zählt nur, wenn es gültig ist und von Leerzeichen oder dem Zeilenende
/// gefolgt wird.
pub fn parse_line(raw: &str) -> ParsedTodo {
    let mut todo = ParsedTodo::default();
    let mut rest = raw;

    if let Some(after_marker) = rest.strip_prefix("x ") {
        todo.completed = true;
        rest = after_marker;
        if let Some((date, after)) = take_date(rest) {
            todo.completed_at = Some(date);
            rest = after;
            if let Some((date, after)) = take_date(rest) {
                todo.created_at = Some(date);
                rest = after;
            }
        }
    } else {
        if let Some((priority, after)) = take_priority(rest) {
            todo.priority = Some(priority);
            rest = after;
        }
        if let Some((date, after)) = take_date(rest) {
            todo.created_at = Some(date);
            rest = after;
        }
    }

    todo.description = rest.to_owned();
    for word in rest.split_whitespace() {
        if word.len() > 1 {
            if word.starts_with('+') {
                todo.project_tags.push(word.to_owned());
            } else if word.starts_with('@') {
                todo.context_tags.push(word.to_owned());
            }
        }
    }
    todo
}

/// Baut die Textzeile aus den strukturierten Feldern (Umkehrung von [`parse_line`] für kanonische Zeilen).
///
/// Erledigt: `x [Erledigungsdatum ][Erstellungsdatum ]Beschreibung`, sonst `[(P) ][Erstellungsdatum ]Beschreibung`.
/// Wie in der Go-App wird bei erledigten Todos ohne Erledigungsdatum ein vorhandenes Erstellungsdatum direkt
/// hinter das `x` geschrieben.
pub fn build_raw_text(todo: &ParsedTodo) -> String {
    let mut out = String::new();
    if todo.completed {
        out.push_str("x ");
        push_date(&mut out, todo.completed_at);
        push_date(&mut out, todo.created_at);
    } else {
        if let Some(priority) = todo.priority {
            out.push('(');
            out.push(priority);
            out.push_str(") ");
        }
        push_date(&mut out, todo.created_at);
    }
    out.push_str(&todo.description);
    out
}

fn push_date(out: &mut String, date: Option<NaiveDate>) {
    if let Some(date) = date {
        out.push_str(&date.format("%Y-%m-%d").to_string());
        out.push(' ');
    }
}

/// `(X) ` am Anfang mit genau einem Großbuchstaben A–Z.
fn take_priority(s: &str) -> Option<(char, &str)> {
    match s.as_bytes() {
        [b'(', letter @ b'A'..=b'Z', b')', b' ', ..] => Some((char::from(*letter), &s[4..])),
        _ => None,
    }
}

/// `YYYY-MM-DD` (nur ASCII-Ziffern, gültiges Kalenderdatum), gefolgt von Leerzeichen oder Zeilenende.
/// Liefert das Datum und den Rest hinter dem Datum samt einem Leerzeichen.
fn take_date(s: &str) -> Option<(NaiveDate, &str)> {
    let head = s.get(..10)?.as_bytes();
    let digits_only = |range: std::ops::Range<usize>| head[range].iter().all(u8::is_ascii_digit);
    if head[4] != b'-'
        || head[7] != b'-'
        || !digits_only(0..4)
        || !digits_only(5..7)
        || !digits_only(8..10)
    {
        return None;
    }
    let number = |range: std::ops::Range<usize>| -> u32 {
        head[range]
            .iter()
            .fold(0, |acc, d| acc * 10 + u32::from(d - b'0'))
    };
    let date = NaiveDate::from_ymd_opt(number(0..4) as i32, number(5..7), number(8..10))?;
    match s.as_bytes().get(10) {
        None => Some((date, "")),
        Some(b' ') => Some((date, &s[11..])),
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn dates_must_be_valid_calendar_dates() {
        assert_eq!(take_date("2025-02-29 x"), None);
        assert_eq!(take_date("2025-13-01 x"), None);
        assert_eq!(take_date("2024-02-29 x"), Some((d(2024, 2, 29), "x")));
    }

    #[test]
    fn dates_need_exactly_the_iso_shape() {
        assert_eq!(take_date("2025-1-05 xy"), None);
        assert_eq!(take_date("2025/01/05 x"), None);
        assert_eq!(take_date("20x5-01-05 x"), None);
        assert_eq!(take_date("2025-01-05x"), None);
        assert_eq!(take_date("2025-01-0"), None);
    }

    #[test]
    fn a_date_may_end_the_line_or_be_followed_by_a_space() {
        assert_eq!(take_date("2025-01-10"), Some((d(2025, 1, 10), "")));
        assert_eq!(take_date("2025-01-10 "), Some((d(2025, 1, 10), "")));
        assert_eq!(take_date("2025-01-10  two"), Some((d(2025, 1, 10), " two")));
    }

    #[test]
    fn non_ascii_text_does_not_panic() {
        assert_eq!(take_date("ääääää"), None);
        assert_eq!(take_date("2025-01-1ä"), None);
        assert_eq!(parse_line("äää").description, "äää");
        assert_eq!(parse_line("(é) task").priority, None);
    }

    #[test]
    fn priority_needs_one_uppercase_ascii_letter_and_a_space() {
        assert_eq!(take_priority("(A) x"), Some(('A', "x")));
        assert_eq!(take_priority("(Z) "), Some(('Z', "")));
        assert_eq!(take_priority("(a) x"), None);
        assert_eq!(take_priority("(A)x"), None);
        assert_eq!(take_priority("(AB) x"), None);
        assert_eq!(take_priority("(1) x"), None);
        assert_eq!(take_priority("[A] x"), None);
        assert_eq!(take_priority("(A"), None);
    }
}
