// auth_header.rs — build and parse HTTP `Authorization` header values.
//
// The package manager needs two outbound authentication shapes:
//
//   * RFC 7617 `Basic` — used by the vast majority of private indexes
//     (devpi, Artifactory, AWS CodeArtifact, GitLab PyPI, Azure
//     Artifacts). `netrc.rs` discovers `(user, pass)` pairs; this
//     module converts them into the `Authorization: Basic …` header.
//
//   * RFC 6750 `Bearer` — used by GitHub Packages, OAuth-fronted
//     mirrors, and CI token flows. The token is passed through after
//     a lightweight syntactic check (the `b68token` ABNF).
//
// We also expose a parser for the inverse direction. `pyproject.toml`
// and `uv.toml` can embed pre-built `Authorization` headers for
// alternate indexes; we round-trip them so downstream code can log /
// redact via `url_redact.rs`.
//
// All builder outputs are `String`; the caller decides where they go
// (reqwest `RequestBuilder::header`, cached HTTP transcript, etc.).
// Builder errors map to `IndexError::ParseError` with `url=""` so they
// surface cleanly through the same error funnel as URL / metadata
// validators.

use base64::Engine;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Build a `Basic` Authorization header value per RFC 7617.
///
/// The input `user` MUST NOT contain `:`; the password may contain
/// arbitrary bytes (RFC 7617 §2 permits any UTF-8 sequence after the
/// first colon). The returned string is `"Basic " + base64(user:pass)`
/// using standard alphabet with padding.
pub fn basic_auth(user: &str, password: &str) -> Result<String, IndexError> {
    if user.contains(':') {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Basic auth username cannot contain ':'".into(),
        });
    }
    let raw = format!("{user}:{password}");
    let engine = base64::engine::general_purpose::STANDARD;
    Ok(format!("Basic {}", engine.encode(raw.as_bytes())))
}

/// Build a `Bearer` Authorization header value per RFC 6750 §2.1.
///
/// The `b68token` ABNF allows: `ALPHA / DIGIT / "-" / "." / "_" / "~" /
/// "+" / "/"`, optionally followed by `=` padding. Empty tokens and
/// tokens containing whitespace / control characters are rejected.
pub fn bearer_auth(token: &str) -> Result<String, IndexError> {
    if token.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Bearer token cannot be empty".into(),
        });
    }
    if !is_valid_bearer_token(token) {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!("Bearer token contains invalid character: {token:?}"),
        });
    }
    Ok(format!("Bearer {token}"))
}

/// A parsed Authorization header value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthScheme {
    Basic { user: String, password: String },
    Bearer { token: String },
    Other { scheme: String, value: String },
}

/// Parse an Authorization header value into its scheme variant.
///
/// Leading / trailing whitespace is tolerated; the scheme is matched
/// case-insensitively per RFC 9110 §11.1. Unknown schemes round-trip
/// through `AuthScheme::Other` for forward compatibility.
pub fn parse_authorization(header: &str) -> Result<AuthScheme, IndexError> {
    let trimmed = header.trim();
    if trimmed.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Authorization header is empty".into(),
        });
    }
    let (scheme, rest) = match trimmed.split_once(|c: char| c.is_ascii_whitespace()) {
        Some((s, r)) => (s, r.trim_start()),
        None => {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!("Authorization header missing credentials: {trimmed:?}"),
            });
        }
    };
    let lower = scheme.to_ascii_lowercase();
    match lower.as_str() {
        "basic" => parse_basic(rest),
        "bearer" => parse_bearer(rest),
        _ => Ok(AuthScheme::Other {
            scheme: scheme.to_string(),
            value: rest.to_string(),
        }),
    }
}

fn parse_basic(rest: &str) -> Result<AuthScheme, IndexError> {
    let engine = base64::engine::general_purpose::STANDARD;
    let decoded = engine
        .decode(rest.trim().as_bytes())
        .map_err(|e| IndexError::ParseError {
            url: String::new(),
            detail: format!("Basic auth base64 decode failed: {e}"),
        })?;
    let s = std::str::from_utf8(&decoded).map_err(|e| IndexError::ParseError {
        url: String::new(),
        detail: format!("Basic auth payload is not UTF-8: {e}"),
    })?;
    let (user, password) = s.split_once(':').ok_or_else(|| IndexError::ParseError {
        url: String::new(),
        detail: "Basic auth payload missing ':' separator".into(),
    })?;
    Ok(AuthScheme::Basic {
        user: user.to_string(),
        password: password.to_string(),
    })
}

fn parse_bearer(rest: &str) -> Result<AuthScheme, IndexError> {
    let token = rest.trim();
    if token.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "Bearer scheme missing token".into(),
        });
    }
    if !is_valid_bearer_token(token) {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!("Bearer token contains invalid character: {token:?}"),
        });
    }
    Ok(AuthScheme::Bearer {
        token: token.to_string(),
    })
}

fn is_valid_bearer_token(token: &str) -> bool {
    // RFC 6750 §2.1: b68token = 1*( ALPHA / DIGIT /
    //   "-" / "." / "_" / "~" / "+" / "/" ) *"="
    // We accept `=` anywhere (some IdPs emit non-canonical padding).
    token.bytes().all(|b| {
        b.is_ascii_alphanumeric()
            || matches!(b, b'-' | b'.' | b'_' | b'~' | b'+' | b'/' | b'=')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    // ---- basic_auth ---------------------------------------------------

    #[test]
    fn basic_auth_simple_credentials() {
        // "alice:secret" -> base64
        assert_eq!(
            basic_auth("alice", "secret").unwrap(),
            "Basic YWxpY2U6c2VjcmV0"
        );
    }

    #[test]
    fn basic_auth_empty_password() {
        // username with empty password — RFC 7617 permits.
        assert_eq!(basic_auth("token", "").unwrap(), "Basic dG9rZW46");
    }

    #[test]
    fn basic_auth_empty_user_and_pass() {
        assert_eq!(basic_auth("", "").unwrap(), "Basic Og==");
    }

    #[test]
    fn basic_auth_password_with_colon_allowed() {
        // RFC 7617 §2: the FIRST colon is the separator; password may
        // contain further colons.
        assert_eq!(
            basic_auth("u", "p:q:r").unwrap(),
            "Basic dTpwOnE6cg=="
        );
    }

    #[test]
    fn basic_auth_rejects_username_with_colon() {
        let err = basic_auth("bad:user", "pw").unwrap_err();
        assert!(parse_err_detail(err).contains("cannot contain ':'"));
    }

    #[test]
    fn basic_auth_utf8_password() {
        // Non-ASCII password — RFC 7617 permits raw UTF-8 bytes.
        let header = basic_auth("u", "pässwörd").unwrap();
        // Round-trip via parser.
        let parsed = parse_authorization(&header).unwrap();
        assert_eq!(
            parsed,
            AuthScheme::Basic {
                user: "u".into(),
                password: "pässwörd".into(),
            }
        );
    }

    // ---- bearer_auth --------------------------------------------------

    #[test]
    fn bearer_auth_simple_token() {
        assert_eq!(bearer_auth("abc123").unwrap(), "Bearer abc123");
    }

    #[test]
    fn bearer_auth_jwt_like_token() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.signature-here";
        assert_eq!(bearer_auth(jwt).unwrap(), format!("Bearer {jwt}"));
    }

    #[test]
    fn bearer_auth_token_with_padding() {
        assert_eq!(bearer_auth("token==").unwrap(), "Bearer token==");
    }

    #[test]
    fn bearer_auth_rejects_empty() {
        let err = bearer_auth("").unwrap_err();
        assert!(parse_err_detail(err).contains("cannot be empty"));
    }

    #[test]
    fn bearer_auth_rejects_whitespace() {
        let err = bearer_auth("two words").unwrap_err();
        assert!(parse_err_detail(err).contains("invalid character"));
    }

    #[test]
    fn bearer_auth_rejects_control_char() {
        let err = bearer_auth("good\x01bad").unwrap_err();
        assert!(parse_err_detail(err).contains("invalid character"));
    }

    #[test]
    fn bearer_auth_accepts_all_b68token_chars() {
        let token = "abcXYZ0123456789-._~+/=";
        assert_eq!(bearer_auth(token).unwrap(), format!("Bearer {token}"));
    }

    // ---- parse_authorization ------------------------------------------

    #[test]
    fn parse_basic_round_trip() {
        let parsed = parse_authorization("Basic YWxpY2U6c2VjcmV0").unwrap();
        assert_eq!(
            parsed,
            AuthScheme::Basic {
                user: "alice".into(),
                password: "secret".into(),
            }
        );
    }

    #[test]
    fn parse_bearer_round_trip() {
        let parsed = parse_authorization("Bearer ghp_abc123").unwrap();
        assert_eq!(
            parsed,
            AuthScheme::Bearer {
                token: "ghp_abc123".into(),
            }
        );
    }

    #[test]
    fn parse_scheme_case_insensitive() {
        // RFC 9110 §11.1: scheme is case-insensitive.
        for variant in ["BASIC", "basic", "Basic", "BaSiC"] {
            let header = format!("{variant} YWxpY2U6c2VjcmV0");
            let parsed = parse_authorization(&header).unwrap();
            assert!(matches!(parsed, AuthScheme::Basic { .. }));
        }
    }

    #[test]
    fn parse_unknown_scheme_round_trips() {
        let parsed = parse_authorization("Digest realm=\"example\"").unwrap();
        assert_eq!(
            parsed,
            AuthScheme::Other {
                scheme: "Digest".into(),
                value: "realm=\"example\"".into(),
            }
        );
    }

    #[test]
    fn parse_tolerates_leading_whitespace() {
        let parsed = parse_authorization("   Basic YWxpY2U6c2VjcmV0").unwrap();
        assert!(matches!(parsed, AuthScheme::Basic { .. }));
    }

    #[test]
    fn parse_tolerates_extra_whitespace_between_scheme_and_value() {
        let parsed = parse_authorization("Basic    YWxpY2U6c2VjcmV0").unwrap();
        assert_eq!(
            parsed,
            AuthScheme::Basic {
                user: "alice".into(),
                password: "secret".into(),
            }
        );
    }

    #[test]
    fn parse_empty_header_errors() {
        let err = parse_authorization("").unwrap_err();
        assert!(parse_err_detail(err).contains("empty"));
    }

    #[test]
    fn parse_scheme_without_credentials_errors() {
        let err = parse_authorization("Bearer").unwrap_err();
        assert!(parse_err_detail(err).contains("missing credentials"));
    }

    #[test]
    fn parse_basic_with_malformed_base64_errors() {
        let err = parse_authorization("Basic !!!not-b64!!!").unwrap_err();
        assert!(parse_err_detail(err).contains("base64 decode"));
    }

    #[test]
    fn parse_basic_without_colon_errors() {
        // base64("noColon") = "bm9Db2xvbg=="
        let err = parse_authorization("Basic bm9Db2xvbg==").unwrap_err();
        assert!(parse_err_detail(err).contains("':' separator"));
    }

    #[test]
    fn parse_bearer_with_invalid_char_errors() {
        let err = parse_authorization("Bearer bad\x01token").unwrap_err();
        assert!(parse_err_detail(err).contains("invalid character"));
    }

    #[test]
    fn parse_bearer_empty_after_scheme_errors() {
        let err = parse_authorization("Bearer    ").unwrap_err();
        // The split_once on whitespace leaves rest empty after trim,
        // so we get "missing token", not "missing credentials".
        let detail = parse_err_detail(err);
        assert!(
            detail.contains("missing token") || detail.contains("missing credentials"),
            "unexpected detail: {detail}"
        );
    }

    #[test]
    fn round_trip_basic_with_special_password() {
        let header = basic_auth("user", "p@ss w/ spaces & symbols!").unwrap();
        let parsed = parse_authorization(&header).unwrap();
        assert_eq!(
            parsed,
            AuthScheme::Basic {
                user: "user".into(),
                password: "p@ss w/ spaces & symbols!".into(),
            }
        );
    }
}
