// Multi-index configuration + private auth scaffolding (Tick 27).
//
// uv supports declaring multiple package indexes in `pyproject.toml`:
//
//     [[tool.uv.index]]
//     name = "internal"
//     url = "https://artifactory.example.com/simple/"
//     default = false
//
//     [[tool.uv.index]]
//     name = "pypi"
//     url = "https://pypi.org/simple/"
//     default = true
//
//     [tool.uv]
//     index-strategy = "first-index"   # default
//     # or "unsafe-first-match" / "unsafe-best-match"
//
// Index priority strategies (uv-faithful semantics):
//   `first-index` — for a given package, query indexes in declared
//                   order and STOP at the first one that has the
//                   package. Safe: prevents an attacker who publishes
//                   `internal-only-pkg` to PyPI from hijacking your build.
//   `unsafe-first-match` — like `first-index` but stops after finding any
//                   single matching version (don't accumulate across
//                   indexes). Useful for caching layers.
//   `unsafe-best-match` — query EVERY index and pick the best version
//                   across the union. Convenient but dangerous (the
//                   dependency-confusion attack vector).
//
// Auth: uv reads basic auth from
//   1. embedded `user:pass@` in the URL
//   2. `UV_INDEX_<NAME>_USERNAME` / `_PASSWORD` env vars
//   3. system keyring (platform-specific; deferred to a follow-up tick)
//
// This tick ships the *data layer*: configuration parsing, strategy
// dispatch logic, and basic-auth extraction from URLs/env. Real keyring
// integration is OS-specific (Keychain on macOS, libsecret on Linux,
// CredManager on Windows) — that gets its own atomic tick once the rest
// of the pipeline is wired up enough to need it.

use std::collections::BTreeMap;
use std::env;

use crate::pkgmanage::pkgmgr::types::IndexError;

const INDEXES_URL: &str = "<pyproject.toml [[tool.uv.index]]>";

/// One configured index endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexConfig {
    /// User-facing name (used for env var lookup and error messages).
    /// Required by uv — anonymous indexes can't be re-keyed for auth env.
    pub name: String,
    /// Base URL (PEP 503 Simple API root).
    pub url: String,
    /// Marked as the "default" index. uv allows multiple non-default
    /// indexes plus one default (PyPI by convention). When all configured
    /// indexes are non-default, PyPI is implicitly appended as the default.
    pub default: bool,
    /// Optional explicit auth. When unset, `resolved_auth` consults env vars.
    pub auth: Option<IndexAuth>,
}

/// Index-priority strategy. Mirrors uv's `index-strategy` config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IndexStrategy {
    /// Safe default: stop at the FIRST index that has the package at all.
    /// Prevents dependency-confusion attacks.
    #[default]
    FirstIndex,
    /// First index that has ANY matching version of the requested spec.
    UnsafeFirstMatch,
    /// Query every index and pick the best version across the union.
    UnsafeBestMatch,
}

/// Resolved auth credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexAuth {
    /// HTTP Basic Auth with user + password.
    Basic { username: String, password: String },
    /// Bearer token in `Authorization: Bearer ...` header.
    Bearer { token: String },
    /// Defer to the OS keyring. Resolution happens at call site; this
    /// variant just records intent.
    Keyring,
}

/// Parse a `pyproject.toml` source and return the full `[[tool.uv.index]]`
/// + `index-strategy` configuration.
///
/// Returns `Ok(IndexesConfig::default())` when no `tool.uv` section is
/// present. Validates uniqueness of index names and that at most one is
/// marked `default = true`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexesConfig {
    pub indexes: Vec<IndexConfig>,
    pub strategy: IndexStrategy,
}

pub fn parse_indexes(toml_src: &str) -> Result<IndexesConfig, IndexError> {
    let doc: toml::Value = toml_src.parse().map_err(|err| IndexError::ParseError {
        url: INDEXES_URL.into(),
        detail: format!("malformed TOML: {err}"),
    })?;

    let Some(uv) = doc.get("tool").and_then(|t| t.get("uv")) else {
        return Ok(IndexesConfig::default());
    };

    let mut indexes: Vec<IndexConfig> = Vec::new();
    if let Some(arr) = uv.get("index") {
        let arr = arr.as_array().ok_or_else(|| IndexError::ParseError {
            url: INDEXES_URL.into(),
            detail: "[[tool.uv.index]] must be an array of tables".into(),
        })?;
        for entry in arr {
            let table = entry.as_table().ok_or_else(|| IndexError::ParseError {
                url: INDEXES_URL.into(),
                detail: format!("each tool.uv.index entry must be a table, got {entry:?}"),
            })?;
            indexes.push(parse_index_entry(table)?);
        }
    }

    // Uniqueness + at-most-one-default validation.
    let mut seen_names: BTreeMap<&str, ()> = BTreeMap::new();
    let mut default_count = 0;
    for idx in &indexes {
        if seen_names.insert(idx.name.as_str(), ()).is_some() {
            return Err(IndexError::ParseError {
                url: INDEXES_URL.into(),
                detail: format!("duplicate index name: {:?}", idx.name),
            });
        }
        if idx.default {
            default_count += 1;
        }
    }
    if default_count > 1 {
        return Err(IndexError::ParseError {
            url: INDEXES_URL.into(),
            detail: "at most one index may be marked `default = true`".into(),
        });
    }

    let strategy = match uv.get("index-strategy").and_then(|v| v.as_str()) {
        Some("first-index") | None => IndexStrategy::FirstIndex,
        Some("unsafe-first-match") => IndexStrategy::UnsafeFirstMatch,
        Some("unsafe-best-match") => IndexStrategy::UnsafeBestMatch,
        Some(other) => {
            return Err(IndexError::ParseError {
                url: INDEXES_URL.into(),
                detail: format!(
                    "unknown index-strategy {other:?}; expected first-index | unsafe-first-match | unsafe-best-match"
                ),
            });
        }
    };

    Ok(IndexesConfig { indexes, strategy })
}

fn parse_index_entry(table: &toml::value::Table) -> Result<IndexConfig, IndexError> {
    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| IndexError::ParseError {
            url: INDEXES_URL.into(),
            detail: "tool.uv.index entry requires a string `name`".into(),
        })?
        .to_string();
    let url = table
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| IndexError::ParseError {
            url: INDEXES_URL.into(),
            detail: format!("tool.uv.index.{name} entry requires a string `url`"),
        })?
        .to_string();
    let default = table
        .get("default")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let auth = match table.get("auth").and_then(|v| v.as_str()) {
        Some("keyring") => Some(IndexAuth::Keyring),
        Some(other) => {
            return Err(IndexError::ParseError {
                url: INDEXES_URL.into(),
                detail: format!(
                    "tool.uv.index.{name}.auth: only `\"keyring\"` is currently supported in-config, got {other:?}"
                ),
            });
        }
        None => None,
    };

    Ok(IndexConfig {
        name,
        url,
        default,
        auth,
    })
}

/// Decide which indexes (in priority order) to query for a given package.
///
/// `FirstIndex` / `UnsafeFirstMatch`: declared order, fall through on
/// 404 → first hit wins.
/// `UnsafeBestMatch`: every index, in declared order; caller is expected
/// to merge results.
///
/// In all cases, *if* none of the configured indexes are marked default,
/// a synthetic `pypi` entry pointing at `https://pypi.org/simple/` is
/// appended last so common installs still resolve transitive deps from
/// PyPI without explicit config.
pub fn query_order<'a>(cfg: &'a IndexesConfig, _package: &str) -> Vec<IndexConfig> {
    let mut out: Vec<IndexConfig> = cfg.indexes.clone();
    if !cfg.indexes.iter().any(|i| i.default) {
        out.push(IndexConfig {
            name: "pypi".to_string(),
            url: "https://pypi.org/simple/".to_string(),
            default: true,
            auth: None,
        });
    }
    let _ = cfg.strategy; // strategy is consumed by the HTTP-fetch layer
    out
}

/// Resolve auth for an index, consulting (in order):
///   1. Explicit `IndexAuth` on the IndexConfig (e.g. config-declared
///      keyring intent).
///   2. `user:pass@host` embedded in `IndexConfig.url`.
///   3. `UV_INDEX_<UPPER_NAME>_USERNAME` + `_PASSWORD` env vars (paired).
///   4. `UV_INDEX_<UPPER_NAME>_TOKEN` env var (bearer).
///
/// Returns `Ok(None)` for a public unauthenticated index.
pub fn resolved_auth(idx: &IndexConfig) -> Result<Option<IndexAuth>, IndexError> {
    if let Some(auth) = &idx.auth {
        return Ok(Some(auth.clone()));
    }
    if let Some(creds) = auth_from_url(&idx.url)? {
        return Ok(Some(creds));
    }
    let upper = name_to_env_segment(&idx.name);
    let user_var = format!("UV_INDEX_{upper}_USERNAME");
    let pass_var = format!("UV_INDEX_{upper}_PASSWORD");
    let token_var = format!("UV_INDEX_{upper}_TOKEN");
    if let (Ok(u), Ok(p)) = (env::var(&user_var), env::var(&pass_var)) {
        return Ok(Some(IndexAuth::Basic {
            username: u,
            password: p,
        }));
    }
    if let Ok(token) = env::var(&token_var) {
        return Ok(Some(IndexAuth::Bearer { token }));
    }
    Ok(None)
}

/// Extract `user:pass@` from a URL, if present. Returns `Ok(None)` when
/// the URL has no userinfo. Errors only on malformed URLs that we can't
/// even split.
fn auth_from_url(url: &str) -> Result<Option<IndexAuth>, IndexError> {
    let scheme_end = url.find("://").ok_or_else(|| IndexError::ParseError {
        url: INDEXES_URL.into(),
        detail: format!("index URL has no scheme: {url:?}"),
    })?;
    let after_scheme = &url[scheme_end + 3..];
    let host_start = match after_scheme.find('@') {
        Some(i) => i,
        None => return Ok(None),
    };
    let userinfo = &after_scheme[..host_start];
    let (user, pass) = match userinfo.find(':') {
        Some(i) => (url_decode(&userinfo[..i]), url_decode(&userinfo[i + 1..])),
        None => (url_decode(userinfo), String::new()),
    };
    Ok(Some(IndexAuth::Basic {
        username: user,
        password: pass,
    }))
}

/// Map index name to env-var-safe segment: uppercase + non-alphanumeric → `_`.
fn name_to_env_segment(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Minimal percent-decode for userinfo components. Sufficient for the
/// common `%40` (`@`) / `%3A` (`:`) escapes that show up in basic-auth
/// URLs. Anything we don't recognize falls back to a literal.
fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_env<F: FnOnce()>(vars: &[(&str, &str)], f: F) {
        // Save and clear, run, then restore. Not thread-safe; tests that
        // touch env vars must not run in parallel via this helper. Cargo
        // runs lib tests in parallel by default, so we serialize via a
        // mutex below.
        let _guard = ENV_LOCK.lock().unwrap();
        let saved: Vec<(String, Option<String>)> = vars
            .iter()
            .map(|(k, _)| ((*k).to_string(), env::var(k).ok()))
            .collect();
        for (k, v) in vars {
            // SAFETY: protected by ENV_LOCK above for our test scope.
            unsafe {
                env::set_var(k, v);
            }
        }
        f();
        for (k, v) in saved {
            unsafe {
                match v {
                    Some(val) => env::set_var(&k, val),
                    None => env::remove_var(&k),
                }
            }
        }
    }

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn parse_returns_empty_when_no_tool_uv() {
        let src = r#"[project]
name = "x"
version = "0.1.0"
"#;
        let cfg = parse_indexes(src).unwrap();
        assert!(cfg.indexes.is_empty());
        assert_eq!(cfg.strategy, IndexStrategy::FirstIndex);
    }

    #[test]
    fn parse_picks_up_multiple_indexes_in_order() {
        let src = r#"[[tool.uv.index]]
name = "internal"
url = "https://internal.example/simple/"

[[tool.uv.index]]
name = "pypi"
url = "https://pypi.org/simple/"
default = true

[tool.uv]
index-strategy = "unsafe-best-match"
"#;
        let cfg = parse_indexes(src).unwrap();
        assert_eq!(cfg.indexes.len(), 2);
        assert_eq!(cfg.indexes[0].name, "internal");
        assert!(!cfg.indexes[0].default);
        assert_eq!(cfg.indexes[1].name, "pypi");
        assert!(cfg.indexes[1].default);
        assert_eq!(cfg.strategy, IndexStrategy::UnsafeBestMatch);
    }

    #[test]
    fn parse_rejects_duplicate_names() {
        let src = r#"[[tool.uv.index]]
name = "a"
url = "https://a/simple/"

[[tool.uv.index]]
name = "a"
url = "https://b/simple/"
"#;
        let err = parse_indexes(src).unwrap_err();
        assert!(format!("{err}").contains("duplicate index name"));
    }

    #[test]
    fn parse_rejects_multiple_defaults() {
        let src = r#"[[tool.uv.index]]
name = "a"
url = "https://a/simple/"
default = true

[[tool.uv.index]]
name = "b"
url = "https://b/simple/"
default = true
"#;
        let err = parse_indexes(src).unwrap_err();
        assert!(format!("{err}").contains("at most one"));
    }

    #[test]
    fn parse_rejects_unknown_strategy() {
        let src = r#"[tool.uv]
index-strategy = "yolo"
"#;
        let err = parse_indexes(src).unwrap_err();
        assert!(format!("{err}").contains("unknown index-strategy"));
    }

    #[test]
    fn parse_keyring_auth_in_config() {
        let src = r#"[[tool.uv.index]]
name = "internal"
url = "https://internal.example/simple/"
auth = "keyring"
"#;
        let cfg = parse_indexes(src).unwrap();
        assert_eq!(cfg.indexes[0].auth, Some(IndexAuth::Keyring));
    }

    #[test]
    fn query_order_appends_implicit_pypi_when_no_default() {
        let cfg = IndexesConfig {
            indexes: vec![IndexConfig {
                name: "internal".into(),
                url: "https://x/simple/".into(),
                default: false,
                auth: None,
            }],
            strategy: IndexStrategy::FirstIndex,
        };
        let order = query_order(&cfg, "anything");
        assert_eq!(order.len(), 2);
        assert_eq!(order[0].name, "internal");
        assert_eq!(order[1].name, "pypi");
        assert!(order[1].default);
    }

    #[test]
    fn query_order_does_not_double_default() {
        let cfg = IndexesConfig {
            indexes: vec![IndexConfig {
                name: "pypi".into(),
                url: "https://pypi.org/simple/".into(),
                default: true,
                auth: None,
            }],
            strategy: IndexStrategy::FirstIndex,
        };
        let order = query_order(&cfg, "anything");
        assert_eq!(order.len(), 1);
    }

    #[test]
    fn resolved_auth_picks_url_embedded_basic_credentials() {
        let idx = IndexConfig {
            name: "private".into(),
            url: "https://alice:s3cret@private.example/simple/".into(),
            default: false,
            auth: None,
        };
        match resolved_auth(&idx).unwrap() {
            Some(IndexAuth::Basic { username, password }) => {
                assert_eq!(username, "alice");
                assert_eq!(password, "s3cret");
            }
            other => panic!("expected Basic auth, got {other:?}"),
        }
    }

    #[test]
    fn resolved_auth_decodes_percent_escapes() {
        let idx = IndexConfig {
            name: "p".into(),
            url: "https://user%40corp:p%3Aw@x/simple/".into(),
            default: false,
            auth: None,
        };
        match resolved_auth(&idx).unwrap() {
            Some(IndexAuth::Basic { username, password }) => {
                assert_eq!(username, "user@corp");
                assert_eq!(password, "p:w");
            }
            other => panic!("expected Basic auth, got {other:?}"),
        }
    }

    #[test]
    fn resolved_auth_falls_back_to_env_basic() {
        with_env(
            &[
                ("UV_INDEX_INTERNAL_USERNAME", "bob"),
                ("UV_INDEX_INTERNAL_PASSWORD", "tok"),
            ],
            || {
                let idx = IndexConfig {
                    name: "internal".into(),
                    url: "https://internal.example/simple/".into(),
                    default: false,
                    auth: None,
                };
                match resolved_auth(&idx).unwrap() {
                    Some(IndexAuth::Basic { username, password }) => {
                        assert_eq!(username, "bob");
                        assert_eq!(password, "tok");
                    }
                    other => panic!("expected Basic, got {other:?}"),
                }
            },
        );
    }

    #[test]
    fn resolved_auth_falls_back_to_env_bearer() {
        with_env(&[("UV_INDEX_SECURE_TOKEN", "xyz123")], || {
            let idx = IndexConfig {
                name: "secure".into(),
                url: "https://secure.example/simple/".into(),
                default: false,
                auth: None,
            };
            match resolved_auth(&idx).unwrap() {
                Some(IndexAuth::Bearer { token }) => assert_eq!(token, "xyz123"),
                other => panic!("expected Bearer, got {other:?}"),
            }
        });
    }

    #[test]
    fn resolved_auth_none_for_public_index() {
        // Pre-clear in case a parallel test leaks; with_env will restore.
        with_env(
            &[
                ("UV_INDEX_PYPI_USERNAME", ""),
                ("UV_INDEX_PYPI_PASSWORD", ""),
                ("UV_INDEX_PYPI_TOKEN", ""),
            ],
            || {
                unsafe {
                    env::remove_var("UV_INDEX_PYPI_USERNAME");
                    env::remove_var("UV_INDEX_PYPI_PASSWORD");
                    env::remove_var("UV_INDEX_PYPI_TOKEN");
                }
                let idx = IndexConfig {
                    name: "pypi".into(),
                    url: "https://pypi.org/simple/".into(),
                    default: true,
                    auth: None,
                };
                assert!(resolved_auth(&idx).unwrap().is_none());
            },
        );
    }

    #[test]
    fn name_to_env_segment_normalizes_specials() {
        assert_eq!(name_to_env_segment("my-index"), "MY_INDEX");
        assert_eq!(name_to_env_segment("acme.io"), "ACME_IO");
        assert_eq!(name_to_env_segment("plain"), "PLAIN");
    }
}
