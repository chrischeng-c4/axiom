// extras_spec.rs — parse the `[extra1,extra2]` extras qualifier.
//
// A requirement string in pip / uv / mamba can specify which
// optional-dependency groups ("extras") to pull in:
//
//   requests[socks,security] >= 2.31
//
// The qualifier lives in PEP 508 grammar under the `extras` rule:
//
//   extras       = "[" wsp* (identifier-list)? wsp* "]"
//   identifier   = letterOrDigit (letterOrDigit | "-" | "_" | ".")*
//   identifier-list = identifier (wsp* "," wsp* identifier)*
//
// Two extra rules layered on top:
//
//   * PEP 685: extras are normalized the same way distribution names
//     are — lowercase + collapse runs of `._-` into a single `-`.
//     This means `Foo.Bar` and `foo-bar` and `foo__bar` are the same
//     extra. The normalization is required so that the resolver can
//     dedupe across multiple requirements that name the same extra
//     in different casing.
//
//   * Duplicates within one extras list are silently merged. PEP 508
//     doesn't forbid `pkg[a,a]`; we just keep one.
//
// What this module gives callers:
//
//   * `ExtrasSpec::parse_qualifier(s)` — parse the *qualifier
//     including the surrounding brackets*: `"[a,b]"` → set { "a", "b" }.
//   * `ExtrasSpec::parse_body(s)` — parse just the comma-separated
//     body (without brackets): `"a, b"` → set { "a", "b" }. Useful
//     for higher-level grammars that have already lifted the brackets.
//   * `ExtrasSpec::extras` — sorted, deduped, normalized.
//   * `ExtrasSpec::contains(name)` — case/punct-insensitive membership.
//   * `normalize_extra(name)` — exposed for callers that need to
//     normalize a single name without going through the parser.

use std::collections::BTreeSet;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Parsed extras list. Iteration order is the sorted (lex) order
/// of normalized names so the rendered form is stable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtrasSpec {
    pub extras: BTreeSet<String>,
}

impl ExtrasSpec {
    /// Parse the qualifier *with* surrounding brackets, e.g.
    /// `"[dev,test]"`. The empty bracket pair `"[]"` is allowed and
    /// yields an empty set (matches pip's behaviour).
    pub fn parse_qualifier(s: &str) -> Result<Self, IndexError> {
        let trimmed = s.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!(
                    "extras qualifier must be wrapped in [...], got: {trimmed:?}"
                ),
            });
        }
        let body = &trimmed[1..trimmed.len() - 1];
        Self::parse_body(body)
    }

    /// Parse the comma-separated body *without* brackets, e.g.
    /// `"dev, test"`. Empty / all-whitespace input yields an empty
    /// set.
    pub fn parse_body(s: &str) -> Result<Self, IndexError> {
        let mut extras = BTreeSet::new();
        if s.trim().is_empty() {
            return Ok(Self { extras });
        }
        for raw in s.split(',') {
            let name = raw.trim();
            if name.is_empty() {
                return Err(IndexError::ParseError {
                    url: String::new(),
                    detail: format!("extras list contains an empty entry in {s:?}"),
                });
            }
            validate_extra_name(name)?;
            extras.insert(normalize_extra(name));
        }
        Ok(Self { extras })
    }

    /// True iff the named extra is in the set (after normalization).
    pub fn contains(&self, name: &str) -> bool {
        self.extras.contains(&normalize_extra(name))
    }

    /// True iff the set is empty.
    pub fn is_empty(&self) -> bool {
        self.extras.is_empty()
    }

    /// Render back to the canonical `[a,b,c]` form (without spaces
    /// between members; pip's output style).
    pub fn render(&self) -> String {
        if self.extras.is_empty() {
            return "[]".into();
        }
        let mut out = String::from("[");
        let mut first = true;
        for e in &self.extras {
            if !first {
                out.push(',');
            }
            out.push_str(e);
            first = false;
        }
        out.push(']');
        out
    }
}

/// PEP 685 normalization for a single extra name. Lowercase, then
/// collapse any run of `.`, `_`, `-` into a single `-`.
pub fn normalize_extra(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut in_sep = false;
    for ch in lower.chars() {
        if matches!(ch, '.' | '_' | '-') {
            if !in_sep {
                out.push('-');
                in_sep = true;
            }
        } else {
            out.push(ch);
            in_sep = false;
        }
    }
    out
}

fn validate_extra_name(name: &str) -> Result<(), IndexError> {
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "extras name cannot be empty".into(),
        });
    }
    if !is_ident_char(bytes[0]) {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "extras name {name:?} must start with a letter or digit (PEP 508)"
            ),
        });
    }
    for &b in &bytes[1..] {
        if !(is_ident_char(b) || matches!(b, b'-' | b'_' | b'.')) {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!(
                    "extras name {name:?} contains illegal character {:?} \
                     (PEP 508: letters, digits, `-`, `_`, `.`)",
                    b as char
                ),
            });
        }
    }
    Ok(())
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric()
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

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    // ---- normalize_extra ----------------------------------------------

    #[test]
    fn normalize_lowercases() {
        assert_eq!(normalize_extra("Dev"), "dev");
        assert_eq!(normalize_extra("FOO_BAR"), "foo-bar");
    }

    #[test]
    fn normalize_collapses_runs_of_punctuation() {
        assert_eq!(normalize_extra("a..b"), "a-b");
        assert_eq!(normalize_extra("a__b"), "a-b");
        assert_eq!(normalize_extra("a--b"), "a-b");
        assert_eq!(normalize_extra("a._-b"), "a-b");
    }

    #[test]
    fn normalize_preserves_alphanumeric() {
        assert_eq!(normalize_extra("py39"), "py39");
        assert_eq!(normalize_extra("test2"), "test2");
    }

    #[test]
    fn normalize_empty_returns_empty() {
        assert_eq!(normalize_extra(""), "");
    }

    // ---- parse_body ---------------------------------------------------

    #[test]
    fn parse_body_simple() {
        let s = ExtrasSpec::parse_body("dev,test").unwrap();
        assert_eq!(s.extras, set(&["dev", "test"]));
    }

    #[test]
    fn parse_body_trims_whitespace() {
        let s = ExtrasSpec::parse_body("  dev ,  test  ").unwrap();
        assert_eq!(s.extras, set(&["dev", "test"]));
    }

    #[test]
    fn parse_body_empty_input_is_empty_set() {
        assert!(ExtrasSpec::parse_body("").unwrap().is_empty());
        assert!(ExtrasSpec::parse_body("   ").unwrap().is_empty());
    }

    #[test]
    fn parse_body_dedupes_after_normalization() {
        let s = ExtrasSpec::parse_body("Foo_Bar, foo-bar, FOO.BAR").unwrap();
        assert_eq!(s.extras, set(&["foo-bar"]));
    }

    #[test]
    fn parse_body_rejects_empty_entry() {
        let err = ExtrasSpec::parse_body("dev,,test").unwrap_err();
        assert!(err_detail(err).contains("empty entry"));
    }

    #[test]
    fn parse_body_rejects_illegal_character() {
        let err = ExtrasSpec::parse_body("hello world").unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("illegal character"));
    }

    #[test]
    fn parse_body_rejects_leading_punctuation() {
        let err = ExtrasSpec::parse_body("-dev").unwrap_err();
        assert!(err_detail(err).contains("must start with"));
    }

    #[test]
    fn parse_body_rejects_leading_underscore() {
        let err = ExtrasSpec::parse_body("_dev").unwrap_err();
        assert!(err_detail(err).contains("must start with"));
    }

    #[test]
    fn parse_body_rejects_leading_dot() {
        let err = ExtrasSpec::parse_body(".dev").unwrap_err();
        assert!(err_detail(err).contains("must start with"));
    }

    #[test]
    fn parse_body_allows_leading_digit() {
        // PEP 508 grammar allows letterOrDigit as first char.
        let s = ExtrasSpec::parse_body("3d, 2d").unwrap();
        assert_eq!(s.extras, set(&["2d", "3d"]));
    }

    // ---- parse_qualifier ----------------------------------------------

    #[test]
    fn parse_qualifier_with_brackets() {
        let s = ExtrasSpec::parse_qualifier("[dev,test]").unwrap();
        assert_eq!(s.extras, set(&["dev", "test"]));
    }

    #[test]
    fn parse_qualifier_empty_brackets() {
        let s = ExtrasSpec::parse_qualifier("[]").unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn parse_qualifier_with_surrounding_whitespace() {
        let s = ExtrasSpec::parse_qualifier("  [ dev, test ]  ").unwrap();
        assert_eq!(s.extras, set(&["dev", "test"]));
    }

    #[test]
    fn parse_qualifier_rejects_missing_bracket() {
        let err = ExtrasSpec::parse_qualifier("dev,test").unwrap_err();
        assert!(err_detail(err).contains("wrapped in [...]"));
    }

    #[test]
    fn parse_qualifier_rejects_unbalanced_brackets() {
        let err = ExtrasSpec::parse_qualifier("[dev,test").unwrap_err();
        assert!(err_detail(err).contains("wrapped in [...]"));
        let err = ExtrasSpec::parse_qualifier("dev,test]").unwrap_err();
        assert!(err_detail(err).contains("wrapped in [...]"));
    }

    // ---- ExtrasSpec helpers -------------------------------------------

    #[test]
    fn contains_uses_normalized_lookup() {
        let s = ExtrasSpec::parse_body("dev, test-utils").unwrap();
        assert!(s.contains("dev"));
        assert!(s.contains("Test_Utils"));   // normalized match
        assert!(s.contains("test.utils"));   // normalized match
        assert!(!s.contains("docs"));
    }

    #[test]
    fn is_empty_default_and_parsed() {
        assert!(ExtrasSpec::default().is_empty());
        assert!(ExtrasSpec::parse_qualifier("[]").unwrap().is_empty());
        assert!(!ExtrasSpec::parse_qualifier("[a]").unwrap().is_empty());
    }

    #[test]
    fn render_is_canonical() {
        let s = ExtrasSpec::parse_body("Test_Utils, dev, Foo.Bar").unwrap();
        // Sorted: dev, foo-bar, test-utils.
        assert_eq!(s.render(), "[dev,foo-bar,test-utils]");
    }

    #[test]
    fn render_empty_set() {
        assert_eq!(ExtrasSpec::default().render(), "[]");
    }

    #[test]
    fn render_round_trips_through_parse() {
        let original = ExtrasSpec::parse_body("test_utils, Dev").unwrap();
        let rendered = original.render();
        let back = ExtrasSpec::parse_qualifier(&rendered).unwrap();
        assert_eq!(back, original);
    }

    // ---- realistic workflows ------------------------------------------

    #[test]
    fn realistic_requests_extras() {
        // `requests[socks,security] >= 2.31` — the bracketed body is
        // what callers hand us.
        let s = ExtrasSpec::parse_qualifier("[socks,security]").unwrap();
        assert!(s.contains("socks"));
        assert!(s.contains("security"));
        assert!(!s.contains("dev"));
    }

    #[test]
    fn realistic_django_extras_case_normalized() {
        // Real-world: django[Argon2] should resolve the same as
        // django[argon2].
        let lower = ExtrasSpec::parse_qualifier("[argon2]").unwrap();
        let upper = ExtrasSpec::parse_qualifier("[Argon2]").unwrap();
        assert_eq!(lower, upper);
    }

    #[test]
    fn realistic_pyproject_optional_dependencies_lookup() {
        // Resolver perspective: the project declares optional deps
        // `dev`, `test`, `docs`; the user asked for `[Dev, TESTS]`
        // (note plural typo). `contains` should still match `dev`,
        // but not the typo'd `tests`.
        let user_asked = ExtrasSpec::parse_qualifier("[Dev, TESTS]").unwrap();
        let declared = ["dev", "test", "docs"];
        let chosen: Vec<&str> = declared
            .into_iter()
            .filter(|d| user_asked.contains(d))
            .collect();
        assert_eq!(chosen, vec!["dev"]);
        // The `TESTS` typo is preserved in the user's set; the
        // caller can surface "extra `tests` not declared" by
        // computing user_asked.extras \ declared.
        assert!(user_asked.contains("tests"));
    }
}
