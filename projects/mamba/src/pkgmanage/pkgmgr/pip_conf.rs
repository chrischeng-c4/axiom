// pip_conf.rs — pip's standard INI configuration file reader.
//
// pip resolves configuration in the following precedence (high to low):
//
//   1. `PIP_*` environment variables
//   2. command-line flags
//   3. site-wide config: `$prefix/pip.conf` (Unix) or `%APPDATA%\pip\pip.ini`
//   4. user config: `~/.pip/pip.conf` or `~/.config/pip/pip.conf`
//   5. virtualenv config: `$VIRTUAL_ENV/pip.conf`
//
// This module parses any single one of those INI files into a typed
// surface. Env-var precedence and file lookup are out of scope; see
// `uv_config.rs` for the uv-native equivalent.
//
// pip itself defers to Python's `configparser` module. The relevant
// quirks we replicate:
//
//   * Sections are introduced by `[name]` on its own line.
//   * Keys use `=` or `:` as the separator.
//   * Keys are case-sensitive in configparser by default, but pip
//     normalizes `key-name` and `key_name` to the same option, so we
//     fold to lowercase + `_`-to-`-` for lookup keys.
//   * Lines beginning with `#` or `;` are full-line comments.
//   * A line indented relative to the preceding key continues that
//     key's value (RFC 822-style folding, used by `extra-index-url`
//     and `find-links` for newline-separated lists).
//   * Outside continuation, blank lines are ignored.
//
// Env-var interpolation (`%(HOME)s`, `${HOME}`) is intentionally
// skipped — uv refused to inherit that complexity, and downstream
// callers can run their own substitution if needed.

use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// A parsed pip.conf file. Sections are indexed by their bracketed
/// name (case-preserved); keys within a section are normalized to
/// lowercase with `-` separators.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PipConfig {
    pub sections: BTreeMap<String, PipSection>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PipSection {
    pub keys: BTreeMap<String, String>,
}

impl PipConfig {
    /// Look up a single value. Returns the raw string (continuation
    /// lines joined by `\n`).
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)?
            .keys
            .get(&normalize_key(key))
            .map(String::as_str)
    }

    /// Look up a list-valued option (one entry per non-empty line of
    /// the raw value). Used for `extra-index-url`, `find-links`, etc.
    pub fn get_list(&self, section: &str, key: &str) -> Vec<String> {
        match self.get(section, key) {
            Some(raw) => raw
                .lines()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect(),
            None => Vec::new(),
        }
    }
}

fn normalize_key(key: &str) -> String {
    key.to_ascii_lowercase().replace('_', "-")
}

/// Parse a pip.conf source string. Returns `IndexError::ParseError`
/// for malformed lines.
pub fn parse_pip_conf(src: &str) -> Result<PipConfig, IndexError> {
    let mut config = PipConfig::default();
    let mut current_section: Option<String> = None;
    let mut current_key: Option<String> = None;
    let mut current_indent: usize = 0;

    for (idx, raw_line) in src.lines().enumerate() {
        let lineno = idx + 1;
        let line = raw_line.trim_end_matches('\r');
        let trimmed = line.trim_start();

        // Blank line: end any in-flight continuation.
        if trimmed.is_empty() {
            current_key = None;
            continue;
        }

        // Full-line comment: skip without ending continuation —
        // configparser treats a comment between continuation lines as
        // a single comment, but we keep the simple rule: comments only
        // appear at column 0 (no leading whitespace).
        if line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        let indent = line.len() - trimmed.len();

        // Continuation line: indented relative to the key's first line.
        if let Some(key) = current_key.as_ref() {
            if indent > current_indent {
                let section_name =
                    current_section
                        .as_ref()
                        .ok_or_else(|| IndexError::ParseError {
                            url: String::new(),
                            detail: format!("pip.conf line {lineno}: continuation without section"),
                        })?;
                let section = config.sections.entry(section_name.clone()).or_default();
                let entry = section
                    .keys
                    .get_mut(key)
                    .expect("current_key must reference an inserted entry");
                entry.push('\n');
                entry.push_str(trimmed);
                continue;
            }
        }

        // Section header.
        if let Some(rest) = trimmed.strip_prefix('[') {
            let name = rest
                .strip_suffix(']')
                .ok_or_else(|| IndexError::ParseError {
                    url: String::new(),
                    detail: format!("pip.conf line {lineno}: malformed section header"),
                })?
                .trim();
            if name.is_empty() {
                return Err(IndexError::ParseError {
                    url: String::new(),
                    detail: format!("pip.conf line {lineno}: empty section name"),
                });
            }
            current_section = Some(name.to_string());
            current_key = None;
            config.sections.entry(name.to_string()).or_default();
            continue;
        }

        // Key = value (or key : value).
        let section_name = current_section
            .as_ref()
            .ok_or_else(|| IndexError::ParseError {
                url: String::new(),
                detail: format!("pip.conf line {lineno}: key outside any section"),
            })?;
        let (key, value) = split_kv(trimmed).ok_or_else(|| IndexError::ParseError {
            url: String::new(),
            detail: format!("pip.conf line {lineno}: missing '=' or ':' separator"),
        })?;
        let key_norm = normalize_key(key.trim());
        if key_norm.is_empty() {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!("pip.conf line {lineno}: empty key"),
            });
        }
        let value_trimmed = value.trim().to_string();
        let section = config.sections.entry(section_name.clone()).or_default();
        section.keys.insert(key_norm.clone(), value_trimmed);
        current_key = Some(key_norm);
        current_indent = indent;
    }

    Ok(config)
}

fn split_kv(line: &str) -> Option<(&str, &str)> {
    // Pick the earliest of `=` or `:` (configparser default).
    let eq = line.find('=');
    let colon = line.find(':');
    let idx = match (eq, colon) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => return None,
    };
    let (k, rest) = line.split_at(idx);
    Some((k, &rest[1..]))
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

    #[test]
    fn empty_input_yields_empty_config() {
        let cfg = parse_pip_conf("").unwrap();
        assert!(cfg.sections.is_empty());
    }

    #[test]
    fn parses_single_section_single_key() {
        let cfg = parse_pip_conf("[global]\nindex-url = https://pypi.org/simple/\n").unwrap();
        assert_eq!(
            cfg.get("global", "index-url"),
            Some("https://pypi.org/simple/")
        );
    }

    #[test]
    fn supports_colon_separator() {
        let cfg = parse_pip_conf("[global]\nindex-url : https://pypi.org/simple/\n").unwrap();
        assert_eq!(
            cfg.get("global", "index-url"),
            Some("https://pypi.org/simple/")
        );
    }

    #[test]
    fn underscore_and_dash_alias() {
        let cfg = parse_pip_conf("[global]\nindex_url = https://example.org/\n").unwrap();
        assert_eq!(cfg.get("global", "index-url"), Some("https://example.org/"));
        assert_eq!(cfg.get("global", "index_url"), Some("https://example.org/"));
    }

    #[test]
    fn lookup_key_is_case_insensitive() {
        let cfg = parse_pip_conf("[global]\nINDEX-URL = https://x/\n").unwrap();
        assert_eq!(cfg.get("global", "index-url"), Some("https://x/"));
    }

    #[test]
    fn section_name_preserves_case() {
        let cfg = parse_pip_conf("[Global]\nx = 1\n").unwrap();
        assert!(cfg.sections.contains_key("Global"));
        assert!(!cfg.sections.contains_key("global"));
    }

    #[test]
    fn skips_full_line_comments() {
        let src = "\
# leading comment
; semicolon comment
[global]
# inside-section comment
index-url = https://pypi.org/simple/
";
        let cfg = parse_pip_conf(src).unwrap();
        assert_eq!(
            cfg.get("global", "index-url"),
            Some("https://pypi.org/simple/")
        );
    }

    #[test]
    fn ignores_blank_lines() {
        let src = "\n\n[global]\n\nindex-url = https://pypi.org/simple/\n\n";
        let cfg = parse_pip_conf(src).unwrap();
        assert_eq!(
            cfg.get("global", "index-url"),
            Some("https://pypi.org/simple/")
        );
    }

    #[test]
    fn multi_value_via_continuation_lines() {
        let src = "\
[global]
extra-index-url =
    https://internal.example/simple/
    https://mirror.example/simple/
";
        let cfg = parse_pip_conf(src).unwrap();
        let list = cfg.get_list("global", "extra-index-url");
        assert_eq!(
            list,
            vec![
                "https://internal.example/simple/".to_string(),
                "https://mirror.example/simple/".to_string(),
            ]
        );
    }

    #[test]
    fn continuation_must_be_indented_more_than_key() {
        // Two equally indented keys are separate, NOT a continuation.
        let src = "[global]\nfind-links = a\nindex-url = b\n";
        let cfg = parse_pip_conf(src).unwrap();
        assert_eq!(cfg.get("global", "find-links"), Some("a"));
        assert_eq!(cfg.get("global", "index-url"), Some("b"));
    }

    #[test]
    fn blank_line_ends_continuation() {
        let src = "\
[global]
extra-index-url =
    https://a/

index-url = https://b/
";
        let cfg = parse_pip_conf(src).unwrap();
        assert_eq!(cfg.get("global", "extra-index-url"), Some("\nhttps://a/"));
        assert_eq!(cfg.get("global", "index-url"), Some("https://b/"));
    }

    #[test]
    fn multiple_sections_parsed() {
        let src = "\
[global]
index-url = https://pypi.org/simple/

[install]
no-deps = true
target = /opt/python

[wheel]
wheel-dir = /tmp/wheels
";
        let cfg = parse_pip_conf(src).unwrap();
        assert_eq!(
            cfg.get("global", "index-url"),
            Some("https://pypi.org/simple/")
        );
        assert_eq!(cfg.get("install", "no-deps"), Some("true"));
        assert_eq!(cfg.get("install", "target"), Some("/opt/python"));
        assert_eq!(cfg.get("wheel", "wheel-dir"), Some("/tmp/wheels"));
    }

    #[test]
    fn value_with_internal_equals_is_preserved() {
        let cfg = parse_pip_conf("[global]\nflag = name=value=other\n").unwrap();
        assert_eq!(cfg.get("global", "flag"), Some("name=value=other"));
    }

    #[test]
    fn earliest_separator_wins() {
        // "key:something=else" → key = "key", value = "something=else"
        let cfg = parse_pip_conf("[global]\nfoo:bar=baz\n").unwrap();
        assert_eq!(cfg.get("global", "foo"), Some("bar=baz"));
    }

    #[test]
    fn empty_value_is_empty_string() {
        let cfg = parse_pip_conf("[global]\nindex-url =\n").unwrap();
        assert_eq!(cfg.get("global", "index-url"), Some(""));
    }

    #[test]
    fn carriage_return_line_endings_handled() {
        let src = "[global]\r\nindex-url = https://x/\r\n";
        let cfg = parse_pip_conf(src).unwrap();
        assert_eq!(cfg.get("global", "index-url"), Some("https://x/"));
    }

    #[test]
    fn trusted_hosts_list() {
        let src = "\
[global]
trusted-host =
    pypi.example.com
    mirror.example.org
";
        let cfg = parse_pip_conf(src).unwrap();
        assert_eq!(
            cfg.get_list("global", "trusted-host"),
            vec![
                "pypi.example.com".to_string(),
                "mirror.example.org".to_string(),
            ]
        );
    }

    #[test]
    fn missing_section_in_get_returns_none() {
        let cfg = parse_pip_conf("[global]\nindex-url = x\n").unwrap();
        assert!(cfg.get("install", "no-deps").is_none());
    }

    #[test]
    fn missing_key_in_get_returns_none() {
        let cfg = parse_pip_conf("[global]\nindex-url = x\n").unwrap();
        assert!(cfg.get("global", "no-deps").is_none());
    }

    #[test]
    fn get_list_on_missing_key_is_empty() {
        let cfg = parse_pip_conf("[global]\nindex-url = x\n").unwrap();
        assert!(cfg.get_list("global", "find-links").is_empty());
    }

    #[test]
    fn rejects_key_outside_section() {
        let err = parse_pip_conf("foo = bar\n").unwrap_err();
        assert!(err_detail(err).contains("outside any section"));
    }

    #[test]
    fn rejects_unclosed_section_header() {
        let err = parse_pip_conf("[global\nfoo = bar\n").unwrap_err();
        assert!(err_detail(err).contains("malformed section header"));
    }

    #[test]
    fn rejects_empty_section_name() {
        let err = parse_pip_conf("[]\nfoo = bar\n").unwrap_err();
        assert!(err_detail(err).contains("empty section name"));
    }

    #[test]
    fn rejects_line_without_separator() {
        let err = parse_pip_conf("[global]\norphan_token\n").unwrap_err();
        assert!(err_detail(err).contains("missing '=' or ':'"));
    }

    #[test]
    fn rejects_empty_key() {
        let err = parse_pip_conf("[global]\n = bar\n").unwrap_err();
        assert!(err_detail(err).contains("empty key"));
    }

    #[test]
    fn duplicate_key_last_write_wins() {
        // configparser raises DuplicateOptionError by default; pip
        // uses strict=False so the later assignment wins. We follow pip.
        let src = "[global]\nindex-url = https://a/\nindex-url = https://b/\n";
        let cfg = parse_pip_conf(src).unwrap();
        assert_eq!(cfg.get("global", "index-url"), Some("https://b/"));
    }

    #[test]
    fn realistic_pip_conf() {
        let src = "\
# /etc/pip.conf
[global]
index-url = https://pypi.example.com/simple/
extra-index-url =
    https://internal.example/simple/
    https://mirror.example/simple/
trusted-host =
    pypi.example.com
    internal.example
timeout = 30

[install]
no-deps = false
no-build-isolation = true

[wheel]
wheel-dir = /tmp/wheels
";
        let cfg = parse_pip_conf(src).unwrap();
        assert_eq!(
            cfg.get("global", "index-url"),
            Some("https://pypi.example.com/simple/")
        );
        assert_eq!(
            cfg.get_list("global", "extra-index-url"),
            vec![
                "https://internal.example/simple/".to_string(),
                "https://mirror.example/simple/".to_string(),
            ]
        );
        assert_eq!(
            cfg.get_list("global", "trusted-host"),
            vec![
                "pypi.example.com".to_string(),
                "internal.example".to_string(),
            ]
        );
        assert_eq!(cfg.get("global", "timeout"), Some("30"));
        assert_eq!(cfg.get("install", "no-build-isolation"), Some("true"));
        assert_eq!(cfg.get("wheel", "wheel-dir"), Some("/tmp/wheels"));
    }
}
