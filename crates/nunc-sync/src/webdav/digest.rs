//! HTTP-Digest-Authentifizierung (RFC 2617 / RFC 7616) mit `qop=auth`, MD5 und SHA-256.

use std::fmt::Write;

use md5::Md5;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Md5,
    Sha256,
}

impl Algorithm {
    fn name(self) -> &'static str {
        match self {
            Self::Md5 => "MD5",
            Self::Sha256 => "SHA-256",
        }
    }

    fn hash(self, text: &str) -> String {
        let bytes = match self {
            Self::Md5 => Md5::digest(text.as_bytes()).to_vec(),
            Self::Sha256 => Sha256::digest(text.as_bytes()).to_vec(),
        };
        bytes.iter().fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
    }
}

/// Die Herausforderung des Servers (`WWW-Authenticate: Digest …`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenge {
    pub realm: String,
    pub nonce: String,
    pub opaque: Option<String>,
    pub algorithm: Algorithm,
    /// Der Server bietet `qop=auth` an (sonst gilt das alte Verfahren nach RFC 2069).
    pub qop_auth: bool,
}

/// Zerlegt `key=value, key="quoted, value"`-Listen.
fn parameters(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = text.trim_start();
    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else { break };
        let key = rest[..eq]
            .trim()
            .trim_start_matches(',')
            .trim()
            .to_lowercase();
        rest = rest[eq + 1..].trim_start();
        let value;
        if let Some(quoted) = rest.strip_prefix('"') {
            let mut text = String::new();
            let mut chars = quoted.char_indices();
            let mut end = quoted.len();
            while let Some((index, c)) = chars.next() {
                match c {
                    '\\' => {
                        if let Some((_, escaped)) = chars.next() {
                            text.push(escaped);
                        }
                    }
                    '"' => {
                        end = index + 1;
                        break;
                    }
                    other => text.push(other),
                }
            }
            value = text;
            rest = quoted.get(end..).unwrap_or_default();
        } else {
            let end = rest.find(',').unwrap_or(rest.len());
            value = rest[..end].trim().to_owned();
            rest = &rest[end..];
        }
        rest = rest.trim_start().trim_start_matches(',').trim_start();
        out.push((key, value));
    }
    out
}

/// Liest einen `WWW-Authenticate`-Wert; alles außer Digest (oder ohne `nonce`) ist `None`.
pub fn parse_challenge(header: &str) -> Option<Challenge> {
    let header = header.trim_start();
    let params = header
        .get(..6)
        .filter(|scheme| scheme.eq_ignore_ascii_case("digest"))
        .and_then(|_| header.get(6..))?;
    let params = parameters(params);
    let get = |name: &str| {
        params
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    };
    let algorithm = match get("algorithm").map(|a| a.to_uppercase()).as_deref() {
        None | Some("MD5") => Algorithm::Md5,
        Some("SHA-256") => Algorithm::Sha256,
        Some(_) => return None,
    };
    Some(Challenge {
        realm: get("realm").unwrap_or_default(),
        nonce: get("nonce")?,
        opaque: get("opaque"),
        algorithm,
        qop_auth: get("qop").is_some_and(|qop| {
            qop.split(',')
                .any(|option| option.trim().eq_ignore_ascii_case("auth"))
        }),
    })
}

fn quote(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Wert für den `Authorization`-Header. `uri` ist das Anfrageziel (Pfad und Query), `nc` der Zähler der
/// Anfragen mit dieser `nonce`.
pub fn authorization(
    challenge: &Challenge,
    username: &str,
    password: &str,
    method: &str,
    uri: &str,
    nc: u32,
    cnonce: &str,
) -> String {
    let algorithm = challenge.algorithm;
    let ha1 = algorithm.hash(&format!("{username}:{}:{password}", challenge.realm));
    let ha2 = algorithm.hash(&format!("{method}:{uri}"));
    let nc_text = format!("{nc:08x}");
    let response = if challenge.qop_auth {
        algorithm.hash(&format!(
            "{ha1}:{}:{nc_text}:{cnonce}:auth:{ha2}",
            challenge.nonce
        ))
    } else {
        algorithm.hash(&format!("{ha1}:{}:{ha2}", challenge.nonce))
    };
    let mut header = format!(
        "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", uri=\"{}\", algorithm={}, response=\"{response}\"",
        quote(username),
        quote(&challenge.realm),
        quote(&challenge.nonce),
        quote(uri),
        algorithm.name(),
    );
    if challenge.qop_auth {
        let _ = write!(header, ", qop=auth, nc={nc_text}, cnonce=\"{cnonce}\"");
    }
    if let Some(opaque) = &challenge.opaque {
        let _ = write!(header, ", opaque=\"{}\"", quote(opaque));
    }
    header
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC_2617: &str = r#"Digest realm="testrealm@host.com",
        qop="auth,auth-int",
        nonce="dcd98b7102dd2f0e8b11d0f600bfb0c093",
        opaque="5ccc069c403ebaf9f0171e9517f40e41""#;

    #[test]
    fn the_challenge_of_rfc_2617_is_parsed() {
        let challenge = parse_challenge(RFC_2617).unwrap();
        assert_eq!(challenge.realm, "testrealm@host.com");
        assert_eq!(challenge.nonce, "dcd98b7102dd2f0e8b11d0f600bfb0c093");
        assert_eq!(
            challenge.opaque.as_deref(),
            Some("5ccc069c403ebaf9f0171e9517f40e41")
        );
        assert_eq!(challenge.algorithm, Algorithm::Md5);
        assert!(challenge.qop_auth);
    }

    #[test]
    fn the_response_matches_the_example_of_rfc_2617() {
        let challenge = parse_challenge(RFC_2617).unwrap();
        let header = authorization(
            &challenge,
            "Mufasa",
            "Circle Of Life",
            "GET",
            "/dir/index.html",
            1,
            "0a4f113b",
        );
        assert!(
            header.contains("response=\"6629fae49393a05397450978507c4ef1\""),
            "{header}"
        );
        assert!(header.contains("nc=00000001") && header.contains("cnonce=\"0a4f113b\""));
        assert!(header.contains("opaque=\"5ccc069c403ebaf9f0171e9517f40e41\""));
        assert!(header.contains("uri=\"/dir/index.html\""));
    }

    #[test]
    fn the_response_matches_the_sha256_example_of_rfc_7616() {
        let challenge = parse_challenge(
            r#"Digest realm="http-auth@example.org", qop="auth, auth-int", algorithm=SHA-256,
               nonce="7ypf/xlj9XXwfDPEoM4URrv/xwf94BcCAzFZH4GiTo0v", opaque="FQhe/qaU925kfnzjCev0ciny7QMkPqMAFRtzCUYo5tdS""#,
        )
        .unwrap();
        assert_eq!(challenge.algorithm, Algorithm::Sha256);
        let header = authorization(
            &challenge,
            "Mufasa",
            "Circle of Life",
            "GET",
            "/dir/index.html",
            1,
            "f2/wE4q74E6zIJEtWaHKaf5wv/H5QzzpXusqGemxURZJ",
        );
        assert!(
            header.contains(
                "response=\"753927fa0e85d155564e2e272a28d1802ca10daf4496794697cf8db5856cb6c1\""
            ),
            "{header}"
        );
    }

    #[test]
    fn the_algorithm_is_named_in_the_header() {
        let md5 = parse_challenge(r#"Digest realm="r", nonce="n""#).unwrap();
        assert!(authorization(&md5, "u", "p", "GET", "/", 1, "c").contains("algorithm=MD5,"));
        let sha = parse_challenge(r#"Digest realm="r", nonce="n", algorithm=SHA-256"#).unwrap();
        assert!(authorization(&sha, "u", "p", "GET", "/", 1, "c").contains("algorithm=SHA-256,"));
    }

    #[test]
    fn without_qop_the_old_scheme_is_used() {
        let challenge = parse_challenge(r#"Digest realm="r", nonce="n""#).unwrap();
        assert!(!challenge.qop_auth);
        let header = authorization(&challenge, "u", "p", "GET", "/", 1, "c");
        assert!(!header.contains("qop=") && !header.contains("cnonce"));
        // H(H(u:r:p):n:H(GET:/))
        let md5 = |t: &str| Algorithm::Md5.hash(t);
        let expected = md5(&format!("{}:n:{}", md5("u:r:p"), md5("GET:/")));
        assert!(header.contains(&format!("response=\"{expected}\"")));
    }

    #[test]
    fn other_schemes_and_unknown_algorithms_are_not_digest_challenges() {
        assert!(parse_challenge(r#"Basic realm="x""#).is_none());
        assert!(parse_challenge("Digest realm=\"x\"").is_none());
        assert!(parse_challenge(r#"Digest nonce="n", algorithm=SHA-512-256"#).is_none());
        assert!(parse_challenge("").is_none());
        assert!(parse_challenge("Dig").is_none());
    }

    #[test]
    fn quoted_values_may_contain_commas_quotes_and_escapes() {
        let challenge =
            parse_challenge(r#"digest realm="a, \"b\" c", nonce="n,1", algorithm=md5, qop="auth""#)
                .unwrap();
        assert_eq!(challenge.realm, r#"a, "b" c"#);
        assert_eq!(challenge.nonce, "n,1");
        let header = authorization(&challenge, r#"us"er"#, "p", "GET", "/", 2, "c");
        assert!(header.contains(r#"username="us\"er""#));
        assert!(header.contains(r#"realm="a, \"b\" c""#));
        assert!(header.contains("nc=00000002"));
    }

    #[test]
    fn unquoted_values_and_trailing_commas_are_accepted() {
        let challenge = parse_challenge("Digest realm=r,nonce=n,qop=auth,").unwrap();
        assert_eq!(
            (challenge.realm.as_str(), challenge.nonce.as_str()),
            ("r", "n")
        );
        assert!(challenge.qop_auth);
    }
}
