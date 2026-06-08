// keyring_spec.rs — keyring provider declarations and lookup keys.
//
// `keyring` is the de-facto credential helper used alongside netrc for
// fetching index credentials. pip + uv shell out to `keyring --mode
// creds get <url> <username>` (or the older `keyring get <url> <user>`
// subcommand) and inject the result into outbound HTTP requests.
//
// This module covers the wire side of that integration:
//
//   * `KeyringProvider` — which provider class to invoke. `disabled`
//     short-circuits the lookup entirely. `subprocess` is the default
//     since uv 0.6 (matches pip's behaviour). `auto` defers to the
//     installed Python `keyring` package's configured backend.
//
//   * `KeyringMode` — `basic` (legacy single-string `get`) vs `creds`
//     (PEP 711 JSON response). `creds` is required for IDs that include
//     username + token+ optional metadata.
//
//   * `parse_creds_response(json)` — decodes a `keyring --mode creds`
//     JSON response and returns a typed `KeyringCredentials` struct.
//
//   * `service_key(url)` — canonicalize a PEP 503 index URL into the
//     "service" string used as the keyring lookup key. uv strips the
//     path component but preserves the scheme + host + port — matching
//     pip 23.3+ behaviour. URLs that don't look like an index URL are
//     returned verbatim.

use serde::{Deserialize, Serialize};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Which keyring provider implementation to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyringProvider {
    /// Do not consult any keyring. Equivalent to `keyring --mode never`
    /// or pip's `--no-input` flag from the user's perspective.
    Disabled,
    /// Defer to the installed Python `keyring` package's configured
    /// backend (the historical default before pip 23.3).
    Auto,
    /// Invoke the `keyring` CLI as a subprocess. Matches uv's default
    /// and pip 23.3+'s recommendation.
    #[default]
    Subprocess,
}

/// Which lookup mode to use against the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyringMode {
    /// Legacy `keyring get <url> <user>` — returns a single string.
    Basic,
    /// `keyring --mode creds get <url>` — returns JSON
    /// `{"username": "...", "password": "..."}`.
    #[default]
    Creds,
}

impl KeyringProvider {
    pub fn parse(s: &str) -> Result<Self, IndexError> {
        match s.to_ascii_lowercase().as_str() {
            "disabled" | "off" | "false" | "no" => Ok(Self::Disabled),
            "auto" => Ok(Self::Auto),
            "subprocess" | "" => Ok(Self::Subprocess),
            other => Err(IndexError::ParseError {
                url: String::new(),
                detail: format!("unknown keyring provider: {other:?}"),
            }),
        }
    }
}

impl KeyringMode {
    pub fn parse(s: &str) -> Result<Self, IndexError> {
        match s.to_ascii_lowercase().as_str() {
            "basic" | "legacy" => Ok(Self::Basic),
            "creds" | "" => Ok(Self::Creds),
            other => Err(IndexError::ParseError {
                url: String::new(),
                detail: format!("unknown keyring mode: {other:?}"),
            }),
        }
    }
}

/// Decoded payload of a `keyring --mode creds get <service>` call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyringCredentials {
    pub username: String,
    pub password: String,
}

/// Parse the JSON body returned by `keyring --mode creds get <service>`.
///
/// On miss the keyring CLI prints nothing and exits with a non-zero
/// status; callers should not pass an empty body here. We reject it
/// explicitly so a silent failure surfaces as a parse error rather
/// than empty credentials.
pub fn parse_creds_response(body: &str) -> Result<KeyringCredentials, IndexError> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "keyring creds response is empty".into(),
        });
    }
    serde_json::from_str::<KeyringCredentials>(trimmed).map_err(|e| IndexError::ParseError {
        url: String::new(),
        detail: format!("keyring creds JSON: {e}"),
    })
}

/// Canonicalize an index URL into its keyring "service" key.
///
/// Rules (matching pip 23.3+ + uv 0.6+):
///   * Preserve scheme, host, and explicit port; drop path / query / fragment.
///   * Drop any embedded userinfo so the lookup doesn't depend on a
///     pre-existing credential pair.
///   * Lowercase the host so `Example.com` and `example.com` hash to
///     the same key.
///   * Inputs that don't have a `scheme://…` shape pass through
///     verbatim (useful for tests + arbitrary keyring records).
pub fn service_key(url: &str) -> String {
    let scheme_end = match url.find("://") {
        Some(idx) => idx + 3,
        None => return url.to_string(),
    };
    let scheme = &url[..scheme_end - 3];
    let after_scheme = &url[scheme_end..];

    // Authority extends until the first /?# separator.
    let authority_end = after_scheme
        .find(|c: char| c == '/' || c == '?' || c == '#')
        .unwrap_or(after_scheme.len());
    let mut authority = &after_scheme[..authority_end];

    // Strip userinfo (last `@` wins per RFC 3986 §3.2.1).
    if let Some(at) = authority.rfind('@') {
        authority = &authority[at + 1..];
    }

    // Lowercase the host portion. Port (if present after `:`) stays as-is.
    let (host, port_suffix) = match authority.rfind(':') {
        Some(idx) if !authority[idx + 1..].contains(']') => {
            (&authority[..idx], &authority[idx..])
        }
        _ => (authority, ""),
    };
    let host_lower = host.to_ascii_lowercase();

    format!("{scheme}://{host_lower}{port_suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    // ---- KeyringProvider::parse ---------------------------------------

    #[test]
    fn provider_parse_known_variants() {
        assert_eq!(KeyringProvider::parse("disabled").unwrap(), KeyringProvider::Disabled);
        assert_eq!(KeyringProvider::parse("auto").unwrap(), KeyringProvider::Auto);
        assert_eq!(
            KeyringProvider::parse("subprocess").unwrap(),
            KeyringProvider::Subprocess
        );
    }

    #[test]
    fn provider_parse_aliases() {
        for alias in ["DISABLED", "off", "false", "no"] {
            assert_eq!(KeyringProvider::parse(alias).unwrap(), KeyringProvider::Disabled);
        }
    }

    #[test]
    fn provider_parse_empty_is_subprocess() {
        // uv treats missing provider as "subprocess" (its default).
        assert_eq!(KeyringProvider::parse("").unwrap(), KeyringProvider::Subprocess);
    }

    #[test]
    fn provider_parse_unknown_errors() {
        let err = KeyringProvider::parse("vault").unwrap_err();
        assert!(err_detail(err).contains("unknown keyring provider"));
    }

    #[test]
    fn provider_default_is_subprocess() {
        assert_eq!(KeyringProvider::default(), KeyringProvider::Subprocess);
    }

    // ---- KeyringMode::parse -------------------------------------------

    #[test]
    fn mode_parse_known_variants() {
        assert_eq!(KeyringMode::parse("basic").unwrap(), KeyringMode::Basic);
        assert_eq!(KeyringMode::parse("creds").unwrap(), KeyringMode::Creds);
    }

    #[test]
    fn mode_parse_aliases() {
        assert_eq!(KeyringMode::parse("legacy").unwrap(), KeyringMode::Basic);
        assert_eq!(KeyringMode::parse("CREDS").unwrap(), KeyringMode::Creds);
    }

    #[test]
    fn mode_parse_empty_is_creds() {
        assert_eq!(KeyringMode::parse("").unwrap(), KeyringMode::Creds);
    }

    #[test]
    fn mode_parse_unknown_errors() {
        let err = KeyringMode::parse("oauth").unwrap_err();
        assert!(err_detail(err).contains("unknown keyring mode"));
    }

    #[test]
    fn mode_default_is_creds() {
        assert_eq!(KeyringMode::default(), KeyringMode::Creds);
    }

    // ---- parse_creds_response -----------------------------------------

    #[test]
    fn parse_creds_minimal() {
        let body = r#"{"username":"alice","password":"s3cret"}"#;
        let creds = parse_creds_response(body).unwrap();
        assert_eq!(creds.username, "alice");
        assert_eq!(creds.password, "s3cret");
    }

    #[test]
    fn parse_creds_with_surrounding_whitespace() {
        let body = "\n  {\"username\":\"x\",\"password\":\"y\"}  \n";
        let creds = parse_creds_response(body).unwrap();
        assert_eq!(creds.username, "x");
    }

    #[test]
    fn parse_creds_rejects_empty_body() {
        let err = parse_creds_response("").unwrap_err();
        assert!(err_detail(err).contains("response is empty"));
    }

    #[test]
    fn parse_creds_rejects_whitespace_only() {
        let err = parse_creds_response("   \n  ").unwrap_err();
        assert!(err_detail(err).contains("response is empty"));
    }

    #[test]
    fn parse_creds_rejects_missing_field() {
        let body = r#"{"username":"x"}"#;
        let err = parse_creds_response(body).unwrap_err();
        assert!(err_detail(err).contains("creds JSON"));
    }

    #[test]
    fn parse_creds_rejects_invalid_json() {
        let err = parse_creds_response("not json").unwrap_err();
        assert!(err_detail(err).contains("creds JSON"));
    }

    // ---- service_key ---------------------------------------------------

    #[test]
    fn service_key_strips_path() {
        assert_eq!(
            service_key("https://pypi.org/simple/requests/"),
            "https://pypi.org"
        );
    }

    #[test]
    fn service_key_preserves_explicit_port() {
        assert_eq!(
            service_key("https://internal.example:8443/simple/"),
            "https://internal.example:8443"
        );
    }

    #[test]
    fn service_key_strips_userinfo() {
        assert_eq!(
            service_key("https://alice:secret@pypi.org/simple/"),
            "https://pypi.org"
        );
    }

    #[test]
    fn service_key_strips_query_and_fragment() {
        assert_eq!(
            service_key("https://pypi.org/simple/?foo=1#bar"),
            "https://pypi.org"
        );
    }

    #[test]
    fn service_key_lowercases_host() {
        assert_eq!(
            service_key("https://Example.COM/simple/"),
            "https://example.com"
        );
    }

    #[test]
    fn service_key_preserves_scheme_case() {
        // Scheme case is preserved (uv doesn't lowercase scheme); the
        // host is the only normalized component.
        assert_eq!(
            service_key("HTTPS://example.com/simple/"),
            "HTTPS://example.com"
        );
    }

    #[test]
    fn service_key_non_url_input_passes_through() {
        // Non-URL inputs (custom keyring "service" strings) round-trip.
        assert_eq!(service_key("my-custom-service"), "my-custom-service");
        assert_eq!(service_key(""), "");
    }

    #[test]
    fn service_key_handles_no_path() {
        assert_eq!(
            service_key("https://pypi.org"),
            "https://pypi.org"
        );
    }

    #[test]
    fn service_key_strips_multiple_at_signs() {
        // Last @ wins per RFC 3986 §3.2.1.
        assert_eq!(
            service_key("https://a@b@example.com/simple/"),
            "https://example.com"
        );
    }

    #[test]
    fn realistic_pypi_workflow() {
        // 1. Compute service key.
        let key = service_key("https://pypi.example.com/simple/mypkg/");
        assert_eq!(key, "https://pypi.example.com");

        // 2. Decode the creds JSON.
        let body = r#"{"username":"token","password":"pypi-AgEIcHl…"}"#;
        let creds = parse_creds_response(body).unwrap();
        assert_eq!(creds.username, "token");
        assert!(creds.password.starts_with("pypi-"));

        // 3. (caller) feed (key, creds) into auth_header::basic_auth.
    }
}
