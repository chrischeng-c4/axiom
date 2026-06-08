// externally_managed.rs — PEP 668 `EXTERNALLY-MANAGED` marker reader.
//
// PEP 668 lets a Python distributor (Debian, Homebrew, conda, etc.) tag
// an interpreter as "externally managed" by writing a file named
// `EXTERNALLY-MANAGED` into the interpreter's stdlib directory (the
// value of `sysconfig.get_path('stdlib')` — e.g.
// `/usr/lib/python3.12/EXTERNALLY-MANAGED` on Debian). When the marker
// is present, pip and uv refuse to install into that interpreter
// outside a venv, surfacing the distributor's recommended remediation
// to the user.
//
// File format (PEP 668 §"Marker file"):
//
//   [externally-managed]
//   Error=The Python interpreter here is managed by APT.  …
//   Error-de=Der Python-Interpreter wird von APT verwaltet.  …
//   Error-zh_CN=…
//
// It's INI / RFC 822-style. The `[externally-managed]` section is the
// only one PEP 668 defines; the `Error` key plus locale-suffixed
// variants are the only keys. The default `Error` (no suffix) is
// required when any locale variant is present, but in practice some
// distributors omit it — we treat the absence as "no message",
// matching pip.

use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Parsed content of an `EXTERNALLY-MANAGED` marker.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExternallyManaged {
    /// The default `Error=…` message, if present.
    pub error: Option<String>,
    /// Locale-suffixed messages keyed by the suffix as written
    /// (e.g. `de`, `zh_CN`). The map preserves the raw locale token —
    /// callers normalise to compare against system locale.
    pub localized: BTreeMap<String, String>,
}

impl ExternallyManaged {
    /// Pick the best error message for a given locale token (e.g.
    /// `"zh_CN"`, `"de_DE"`, `"en_US"`). Lookup falls back from the
    /// full token to the language prefix (`zh_CN` → `zh`), then to the
    /// default `Error=`. Returns `None` if none of those are present.
    pub fn message_for_locale(&self, locale: &str) -> Option<&str> {
        // Exact match first.
        if let Some(msg) = self.localized.get(locale) {
            return Some(msg.as_str());
        }
        // Then language-only fallback (strip `_XX` suffix).
        if let Some((lang, _region)) = locale.split_once('_') {
            if let Some(msg) = self.localized.get(lang) {
                return Some(msg.as_str());
            }
        }
        self.error.as_deref()
    }
}

/// Parse the body of an `EXTERNALLY-MANAGED` file. Unknown sections
/// and unknown keys are tolerated (logged-out by callers if they
/// want — PEP 668 reserves the section name but extensions are
/// possible).
pub fn parse_externally_managed(src: &str) -> Result<ExternallyManaged, IndexError> {
    let mut out = ExternallyManaged::default();
    let mut current_section: Option<String> = None;
    let mut pending: Option<(String, String)> = None;

    for (lineno, raw) in src.lines().enumerate() {
        let line_no = lineno + 1;
        // Continuation: a line that starts with whitespace and has a
        // pending key extends that key's value (RFC 822 folding,
        // permitted by Python's configparser in interpolated mode).
        if raw.starts_with(' ') || raw.starts_with('\t') {
            if let Some((_, ref mut val)) = pending {
                val.push('\n');
                val.push_str(raw.trim_start());
                continue;
            }
        }
        // Flush any pending (key, value) we'd been accumulating.
        if let Some((k, v)) = pending.take() {
            store(&mut out, &current_section, &k, v)?;
        }
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            current_section = Some(rest.trim().to_string());
            continue;
        }
        let (k, v) = match line.split_once('=') {
            Some(kv) => kv,
            None => {
                return Err(pe(&format!(
                    "line {line_no}: expected 'key=value' or '[section]', got {line:?}"
                )));
            }
        };
        pending = Some((k.trim().to_string(), v.trim().to_string()));
    }
    if let Some((k, v)) = pending {
        store(&mut out, &current_section, &k, v)?;
    }
    Ok(out)
}

fn store(
    out: &mut ExternallyManaged,
    section: &Option<String>,
    key: &str,
    value: String,
) -> Result<(), IndexError> {
    // Only the `[externally-managed]` section carries Error keys.
    // Other sections are silently dropped to stay forward-compatible.
    let Some(sec) = section else {
        return Err(pe(&format!(
            "key {key:?} appears outside any [section]"
        )));
    };
    if sec != "externally-managed" {
        return Ok(());
    }
    if key == "Error" {
        out.error = Some(value);
        return Ok(());
    }
    if let Some(locale) = key.strip_prefix("Error-") {
        out.localized.insert(locale.to_string(), value);
        return Ok(());
    }
    // Unknown keys in the known section are tolerated for forward
    // compatibility (future PEP revisions may add fields).
    Ok(())
}

fn pe(msg: &str) -> IndexError {
    IndexError::ParseError {
        url: "EXTERNALLY-MANAGED".into(),
        detail: msg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn em(default: Option<&str>, locales: &[(&str, &str)]) -> ExternallyManaged {
        ExternallyManaged {
            error: default.map(str::to_string),
            localized: locales
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
        }
    }

    #[test]
    fn parses_minimal_marker() {
        let src = "[externally-managed]\nError=managed by APT\n";
        assert_eq!(
            parse_externally_managed(src).unwrap(),
            em(Some("managed by APT"), &[])
        );
    }

    #[test]
    fn parses_marker_with_locale_variants() {
        let src = "\
[externally-managed]
Error=managed by APT
Error-de=verwaltet von APT
Error-zh_CN=APT
";
        let want = em(
            Some("managed by APT"),
            &[("de", "verwaltet von APT"), ("zh_CN", "APT")],
        );
        assert_eq!(parse_externally_managed(src).unwrap(), want);
    }

    #[test]
    fn missing_default_error_is_allowed() {
        let src = "[externally-managed]\nError-de=verwaltet\n";
        let want = em(None, &[("de", "verwaltet")]);
        assert_eq!(parse_externally_managed(src).unwrap(), want);
    }

    #[test]
    fn empty_file_yields_default() {
        assert_eq!(parse_externally_managed("").unwrap(), ExternallyManaged::default());
    }

    #[test]
    fn comments_and_blank_lines_skipped() {
        let src = "\
# top-level comment
; semicolon also legal
[externally-managed]

# inline comment

Error=foo
";
        assert_eq!(parse_externally_managed(src).unwrap(), em(Some("foo"), &[]));
    }

    #[test]
    fn unknown_section_is_tolerated() {
        let src = "\
[future-extension]
Foo=bar
[externally-managed]
Error=ok
";
        assert_eq!(parse_externally_managed(src).unwrap(), em(Some("ok"), &[]));
    }

    #[test]
    fn unknown_key_in_known_section_is_tolerated() {
        let src = "\
[externally-managed]
Error=ok
NewKey=v2
";
        assert_eq!(parse_externally_managed(src).unwrap(), em(Some("ok"), &[]));
    }

    #[test]
    fn continuation_lines_concatenate_with_newline() {
        // PEP 668 doesn't formally require folding, but Python's
        // configparser supports it and several Debian-derived
        // distros emit multi-line messages this way.
        let src = "\
[externally-managed]
Error=line 1
 line 2
\tline 3
";
        let parsed = parse_externally_managed(src).unwrap();
        assert_eq!(parsed.error.as_deref(), Some("line 1\nline 2\nline 3"));
    }

    #[test]
    fn key_outside_section_rejected() {
        let err = parse_externally_managed("Error=oops\n").unwrap_err();
        assert!(err.to_string().contains("outside any [section]"));
    }

    #[test]
    fn malformed_line_rejected_with_lineno() {
        let src = "[externally-managed]\nnot-a-pair\n";
        let err = parse_externally_managed(src).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("line 2"), "missing line number in {msg:?}");
        assert!(msg.contains("key=value"), "missing hint in {msg:?}");
    }

    #[test]
    fn message_for_locale_exact_match() {
        let em = em(Some("d"), &[("de", "lokal")]);
        assert_eq!(em.message_for_locale("de"), Some("lokal"));
    }

    #[test]
    fn message_for_locale_full_token_match() {
        let em = em(Some("d"), &[("zh_CN", "中国")]);
        assert_eq!(em.message_for_locale("zh_CN"), Some("中国"));
    }

    #[test]
    fn message_for_locale_falls_back_to_language() {
        let em = em(Some("d"), &[("de", "deutsch")]);
        // `de_AT` not present → fall back to `de`.
        assert_eq!(em.message_for_locale("de_AT"), Some("deutsch"));
    }

    #[test]
    fn message_for_locale_falls_back_to_default() {
        let em = em(Some("default"), &[("de", "deutsch")]);
        // `fr` neither exact nor language-prefix match → default.
        assert_eq!(em.message_for_locale("fr_FR"), Some("default"));
    }

    #[test]
    fn message_for_locale_returns_none_when_nothing_set() {
        let em = ExternallyManaged::default();
        assert_eq!(em.message_for_locale("en_US"), None);
    }

    #[test]
    fn section_name_is_case_sensitive_per_pep() {
        // PEP 668 spells the section name lowercase. We treat any
        // other casing as an unknown section (tolerated, ignored).
        let src = "[Externally-Managed]\nError=ignored\n";
        let parsed = parse_externally_managed(src).unwrap();
        assert_eq!(parsed.error, None);
    }

    #[test]
    fn whitespace_around_keys_and_values_trimmed() {
        let src = "[externally-managed]\n  Error  =   trimmed   \n";
        assert_eq!(
            parse_externally_managed(src).unwrap(),
            em(Some("trimmed"), &[])
        );
    }

    #[test]
    fn value_can_contain_equals_signs() {
        let src = "[externally-managed]\nError=a=b=c\n";
        assert_eq!(parse_externally_managed(src).unwrap(), em(Some("a=b=c"), &[]));
    }

    #[test]
    fn empty_value_kept_as_empty_string() {
        let src = "[externally-managed]\nError=\n";
        assert_eq!(parse_externally_managed(src).unwrap(), em(Some(""), &[]));
    }
}
