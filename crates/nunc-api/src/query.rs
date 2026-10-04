//! Query-Strings (`a=b&c=d`, Prozent-Kodierung) ohne zusätzliche Abhängigkeit.

/// `%XX` und `+` (Leerzeichen) auflösen. Ungültige Folgen bleiben stehen; ungültiges UTF-8 wird ersetzt.
pub fn percent_decode(text: &str) -> String {
    let mut rest = text.bytes();
    let mut out = Vec::with_capacity(text.len());
    while let Some(byte) = rest.next() {
        match byte {
            b'+' => out.push(b' '),
            b'%' => {
                let mut ahead = rest.clone();
                match (ahead.next().and_then(hex), ahead.next().and_then(hex)) {
                    (Some(high), Some(low)) => {
                        out.push(high * 16 + low);
                        rest = ahead;
                    }
                    _ => out.push(b'%'),
                }
            }
            other => out.push(other),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Alle `name=wert`-Paare in Reihenfolge, dekodiert. Ein Name ohne `=` hat den Wert „“; leere Teile entfallen.
pub fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| match part.split_once('=') {
            Some((name, value)) => (percent_decode(name), percent_decode(value)),
            None => (percent_decode(part), String::new()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_percent_sequences_and_plus() {
        assert_eq!(percent_decode("%2Bwork"), "+work");
        assert_eq!(percent_decode("%40desk"), "@desk");
        assert_eq!(percent_decode("a+b"), "a b");
        assert_eq!(percent_decode("%c3%a4"), "ä");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz"), "%zz");
        assert_eq!(percent_decode("%4"), "%4");
        assert_eq!(percent_decode("%41"), "A");
        assert_eq!(percent_decode("%C3%A4"), "ä");
        assert_eq!(percent_decode("%2b%2B"), "++");
        assert_eq!(percent_decode("%0A%0a%FF%ff"), "\n\n\u{fffd}\u{fffd}");
    }

    #[test]
    fn parses_pairs_in_order() {
        assert_eq!(
            parse_query("completed=false&project=%2Bwork&flag&&q=a+b"),
            [
                ("completed".to_owned(), "false".to_owned()),
                ("project".to_owned(), "+work".to_owned()),
                ("flag".to_owned(), String::new()),
                ("q".to_owned(), "a b".to_owned()),
            ]
        );
        assert!(parse_query("").is_empty());
    }
}
