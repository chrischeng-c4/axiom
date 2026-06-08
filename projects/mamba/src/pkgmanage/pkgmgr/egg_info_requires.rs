// setuptools egg-info `requires.txt` reader (Tick 125).
//
// Setuptools (and the legacy distutils-derived sdist toolchain) writes
// `<dist>.egg-info/requires.txt` alongside `PKG-INFO`. The format
// predates PEP 621 and uses bracket-section syntax that is *not* RFC
// 822 / not PEP 508. uv reads it whenever consuming egg-info-style
// distributions (some sdists still ship them via setup.py).
//
// Grammar (informal — derived from setuptools.dist + uv's reader):
//
//   line = bare-line | section-header | blank | comment
//
//   bare-line  = PEP 508 requirement string (no semicolon allowed; the
//                file uses bracket sections instead).
//   blank      = whitespace-only line; ignored.
//   comment    = `#` to end-of-line; ignored.
//   section-header = `[` name? (`:` marker)? `]`
//
// Section examples:
//   []                                — empty header, equivalent to top-of-file
//   [argon2]                          — extras group `argon2`
//   [:python_version<"3.10"]          — top-level (no extras), conditional on marker
//   [ssl:sys_platform=="linux"]       — extras `ssl`, conditional on marker
//
// Output: every requirement is emitted in PEP 508 canonical form
// (`<bare>` or `<bare> ; <marker>`) so callers can re-route through
// `requirement_string::Requirement::parse` without a second format
// translation. This matches uv's egg-info → PEP 508 normalization.

use crate::pkgmanage::pkgmgr::types::IndexError;
use std::collections::BTreeMap;

const REQUIRES_TXT_DETAIL: &str = "<egg-info requires.txt>";

/// Parsed egg-info requires.txt.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EggInfoRequires {
    /// Top-of-file (no extras) requirements, with any section-level
    /// `:marker` appended in PEP 508 form.
    pub base: Vec<String>,
    /// Extras-keyed requirements. Each value is a vec of PEP 508
    /// requirement strings.
    pub extras: BTreeMap<String, Vec<String>>,
}

impl EggInfoRequires {
    /// True when both `base` and every extras vec are empty.
    pub fn is_empty(&self) -> bool {
        self.base.is_empty() && self.extras.values().all(Vec::is_empty)
    }

    /// Flatten into a single `Vec<String>` in PEP 508 form, with extras
    /// appended in alphabetical order. Useful for resolver hand-off
    /// when no specific extras-activation set is in play.
    pub fn flatten_all(&self) -> Vec<String> {
        let mut out = self.base.clone();
        for reqs in self.extras.values() {
            out.extend(reqs.iter().cloned());
        }
        out
    }
}

/// Parse the body of `<dist>.egg-info/requires.txt`.
pub fn parse_egg_info_requires(src: &str) -> Result<EggInfoRequires, IndexError> {
    let mut out = EggInfoRequires::default();
    // Current section state.
    let mut current_extras: Option<String> = None; // None = top-of-file
    let mut current_marker: Option<String> = None;

    for (lineno, raw_line) in src.lines().enumerate() {
        // Setuptools strips trailing whitespace and ignores `#`-prefixed
        // comments and blank lines. Inline comments are NOT supported by
        // the egg-info reader (matching setuptools).
        let line = raw_line.trim_end();
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(header_body) = strip_section_header(trimmed) {
            let (extras, marker) = parse_section_header(header_body, lineno + 1)?;
            current_extras = extras;
            current_marker = marker;
            continue;
        }

        // Bare requirement line. Append section marker (if any) in
        // PEP 508 form so the output is uniform.
        let req_text = match &current_marker {
            Some(m) => format!("{} ; {}", trimmed, m),
            None => trimmed.to_string(),
        };

        match &current_extras {
            None => out.base.push(req_text),
            Some(name) => out
                .extras
                .entry(name.clone())
                .or_default()
                .push(req_text),
        }
    }

    Ok(out)
}

/// If `s` is `[...]`, return the inside body (no brackets). Trims
/// inner whitespace.
fn strip_section_header(s: &str) -> Option<&str> {
    let s = s.trim();
    let stripped = s.strip_prefix('[')?.strip_suffix(']')?;
    Some(stripped.trim())
}

/// Parse the inside of a `[...]` section header.
///
/// Recognized:
///   ""                — no extras, no marker
///   "name"            — extras name only
///   ":marker"         — top-level marker
///   "name:marker"     — extras + marker
fn parse_section_header(
    body: &str,
    lineno: usize,
) -> Result<(Option<String>, Option<String>), IndexError> {
    if body.is_empty() {
        return Ok((None, None));
    }
    match body.find(':') {
        None => {
            // Extras name only.
            Ok((Some(body.trim().to_string()), None))
        }
        Some(colon) => {
            let extras = body[..colon].trim();
            let marker = body[colon + 1..].trim();
            if marker.is_empty() {
                return Err(IndexError::ParseError {
                    url: REQUIRES_TXT_DETAIL.into(),
                    detail: format!(
                        "line {lineno}: section header `[{body}]` has trailing `:` without marker"
                    ),
                });
            }
            let extras_opt = if extras.is_empty() {
                None
            } else {
                Some(extras.to_string())
            };
            Ok((extras_opt, Some(marker.to_string())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_empty_output() {
        let r = parse_egg_info_requires("").unwrap();
        assert!(r.is_empty());
        assert!(r.base.is_empty());
        assert!(r.extras.is_empty());
    }

    #[test]
    fn top_of_file_requirements_become_base() {
        let src = "requests>=2.31\nclick\nimportlib-metadata\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(
            r.base,
            vec!["requests>=2.31", "click", "importlib-metadata"]
        );
        assert!(r.extras.is_empty());
    }

    #[test]
    fn extras_section_collects_into_extras_map() {
        let src = "requests\n\n[argon2]\nargon2-cffi>=21\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(r.base, vec!["requests"]);
        assert_eq!(r.extras.get("argon2").unwrap(), &vec!["argon2-cffi>=21"]);
    }

    #[test]
    fn top_level_marker_only_section_appends_marker_to_base() {
        let src = "[:python_version<\"3.10\"]\nimportlib-metadata\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(
            r.base,
            vec!["importlib-metadata ; python_version<\"3.10\""]
        );
    }

    #[test]
    fn extras_plus_marker_section_combines_correctly() {
        let src = "[ssl:sys_platform==\"linux\"]\nopenssl-py\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(
            r.extras.get("ssl").unwrap(),
            &vec!["openssl-py ; sys_platform==\"linux\""]
        );
    }

    #[test]
    fn blank_lines_and_comments_are_ignored() {
        let src = "\n  \n# top comment\nrequests\n\n# section comment\n[argon2]\n# inner comment\nargon2-cffi\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(r.base, vec!["requests"]);
        assert_eq!(r.extras.get("argon2").unwrap(), &vec!["argon2-cffi"]);
    }

    #[test]
    fn empty_brackets_act_as_reset_to_top_of_file() {
        let src = "[argon2]\nargon2-cffi\n[]\nrequests\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(r.base, vec!["requests"]);
        assert_eq!(r.extras.get("argon2").unwrap(), &vec!["argon2-cffi"]);
    }

    #[test]
    fn multiple_sections_collect_in_order() {
        let src = "[a]\nx\n[b]\ny\n[a]\nz\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(r.extras.get("a").unwrap(), &vec!["x", "z"]);
        assert_eq!(r.extras.get("b").unwrap(), &vec!["y"]);
    }

    #[test]
    fn marker_with_trailing_whitespace_is_trimmed() {
        let src = "[:  python_version<\"3.10\"   ]\nfoo\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(r.base, vec!["foo ; python_version<\"3.10\""]);
    }

    #[test]
    fn header_with_trailing_colon_and_no_marker_errors() {
        let src = "[name:]\nfoo\n";
        let err = parse_egg_info_requires(src).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("trailing `:`"));
    }

    #[test]
    fn flatten_all_concatenates_base_then_extras_alphabetically() {
        let src = "base-dep\n[z-extra]\nz-dep\n[a-extra]\na-dep\n";
        let r = parse_egg_info_requires(src).unwrap();
        // BTreeMap order: a-extra before z-extra.
        let flat = r.flatten_all();
        assert_eq!(flat, vec!["base-dep", "a-dep", "z-dep"]);
    }

    #[test]
    fn flatten_all_on_empty_returns_empty() {
        let r = parse_egg_info_requires("# only comments\n  \n").unwrap();
        assert!(r.is_empty());
        assert!(r.flatten_all().is_empty());
    }

    #[test]
    fn realistic_setuptools_egg_info_corpus() {
        // Pattern taken from a real-world egg-info dump (cryptography sdist).
        let src = r#"
cffi>=1.12

[ssh]
bcrypt>=3.1.5

[pep8test]
black
mypy
ruff

[:python_version<"3.10"]
importlib-metadata>=4
"#;
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(
            r.base,
            vec![
                "cffi>=1.12",
                "importlib-metadata>=4 ; python_version<\"3.10\""
            ]
        );
        assert_eq!(r.extras.get("ssh").unwrap(), &vec!["bcrypt>=3.1.5"]);
        assert_eq!(
            r.extras.get("pep8test").unwrap(),
            &vec!["black", "mypy", "ruff"]
        );
    }

    #[test]
    fn windows_crlf_line_endings_handled() {
        let src = "requests\r\n[argon2]\r\nargon2-cffi\r\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(r.base, vec!["requests"]);
        assert_eq!(r.extras.get("argon2").unwrap(), &vec!["argon2-cffi"]);
    }

    #[test]
    fn requirement_with_existing_marker_is_preserved_when_section_has_no_marker() {
        // Note: setuptools egg-info bare lines technically don't allow
        // `;` markers (the section header is the marker carrier). But
        // we don't reject them — we pass them through verbatim, since
        // some real-world egg-infos contain hand-edited markers.
        let src = "foo ; python_version<'3.10'\n";
        let r = parse_egg_info_requires(src).unwrap();
        assert_eq!(r.base, vec!["foo ; python_version<'3.10'"]);
    }
}
