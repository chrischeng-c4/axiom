// HANDWRITE-BEGIN gap="missing-generator:hand-written:d2aeab54" tracker="standardize-gap-projects-mamba-src-pkgmgr-resolver-requirement-rs" reason="PEP 508 parser. Subset for P1: name, specifier set, extras, python_version + sys_platform markers. Optionally backed by pep508_rs crate."
//! PEP 508 dependency declaration parser (Phase-1 subset).
//!
//! Schema source: `.aw/tech-design/apps/mamba/pkgmgr/resolver.md#schema`.
//! Recognised grammar:
//!     name [ "[" extras "]" ] [ specifier_set ] [ ";" marker ]
//! `name` is PEP 503-normalised (lowercase, runs of `[-_.]` collapsed to `-`).
//! `marker` is kept as raw text — evaluation lives in a later phase.

use serde::{Deserialize, Serialize};

use super::specifier::VersionSpecifier;

/// @spec .aw/tech-design/apps/mamba/pkgmgr/resolver.md#schema (Requirement)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    pub name: String,
    pub specifiers: Vec<VersionSpecifier>,
    pub extras: Vec<String>,
    /// Raw PEP 508 marker expression; `None` = always evaluates true.
    pub marker: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    InvalidName(String),
    UnclosedExtras,
    BadSpecifier(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty requirement string"),
            ParseError::InvalidName(s) => write!(f, "invalid distribution name: {s:?}"),
            ParseError::UnclosedExtras => write!(f, "unclosed `[…]` extras list"),
            ParseError::BadSpecifier(s) => write!(f, "bad version specifier: {s:?}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse one PEP 508 requirement line.
///
/// @spec .aw/tech-design/apps/mamba/pkgmgr/resolver.md#logic (parse_roots)
pub fn parse(input: &str) -> Result<Requirement, ParseError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ParseError::Empty);
    }

    // Split off marker after `;`
    let (head, marker) = match s.split_once(';') {
        Some((h, m)) => {
            let m = m.trim();
            (
                h.trim(),
                if m.is_empty() {
                    None
                } else {
                    Some(m.to_string())
                },
            )
        }
        None => (s, None),
    };

    // Split off extras `[…]`
    let (name_part, rest_after_extras, extras) = match head.find('[') {
        Some(lb) => {
            let close = head[lb..].find(']').ok_or(ParseError::UnclosedExtras)?;
            let extras_str = &head[lb + 1..lb + close];
            let extras: Vec<String> = extras_str
                .split(',')
                .map(|e| e.trim().to_string())
                .filter(|e| !e.is_empty())
                .collect();
            (head[..lb].trim(), head[lb + close + 1..].trim(), extras)
        }
        None => (head, "", Vec::new()),
    };

    // Distribution name — first token before any specifier op or whitespace.
    let name_end = name_part
        .find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '=' | '!' | '~'))
        .unwrap_or(name_part.len());
    let raw_name = &name_part[..name_end];
    if raw_name.is_empty() || !raw_name.chars().next().unwrap().is_ascii_alphanumeric() {
        return Err(ParseError::InvalidName(raw_name.to_string()));
    }
    let name = normalize_name(raw_name);

    // Specifier set: anything after the name, plus rest_after_extras.
    let mut spec_input = name_part[name_end..].trim().to_string();
    if !rest_after_extras.is_empty() {
        if !spec_input.is_empty() {
            spec_input.push(',');
        }
        spec_input.push_str(rest_after_extras);
    }

    let specifiers = if spec_input.is_empty() {
        Vec::new()
    } else {
        let unwrapped = if spec_input.starts_with('(') {
            strip_one_balanced_outer_parens(&spec_input)
                .ok_or_else(|| ParseError::BadSpecifier(spec_input.clone()))?
        } else {
            spec_input.clone()
        };
        super::specifier::parse_set(&unwrapped)
            .map_err(|e| ParseError::BadSpecifier(e.to_string()))?
    };

    Ok(Requirement {
        name,
        specifiers,
        extras,
        marker,
    })
}

/// Strip exactly one balanced outer pair of parentheses around a PEP 508
/// `versionspec` — `'(' version_many ')'` — returning the trimmed interior.
///
/// `s` must already start with `(` and end with `)`; returns `None` (refuse)
/// on an empty interior, an unmatched parenthesis, or any text following the
/// close of the outer pair (the outer `)` must be the string's last byte).
fn strip_one_balanced_outer_parens(s: &str) -> Option<String> {
    debug_assert!(s.starts_with('('));
    if !s.ends_with(')') {
        return None;
    }
    let bytes = s.as_bytes();
    let mut depth: i32 = 0;
    for (i, b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
                if depth == 0 && i != bytes.len() - 1 {
                    // the outer pair closed before the end of the string:
                    // either trailing text follows, or this was never one
                    // single outer pair.
                    return None;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return None;
    }
    Some(inner.to_string())
}

/// PEP 503 distribution-name normalisation: lowercase, runs of `[-_.]` → `-`.
pub fn normalize_name(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_sep = false;
    for c in raw.chars() {
        match c {
            '-' | '_' | '.' => {
                if !prev_sep && !out.is_empty() {
                    out.push('-');
                }
                prev_sep = true;
            }
            _ => {
                for lc in c.to_lowercase() {
                    out.push(lc);
                }
                prev_sep = false;
            }
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bare_name() {
        let r = parse("requests").unwrap();
        assert_eq!(r.name, "requests");
        assert!(r.specifiers.is_empty());
        assert!(r.extras.is_empty());
        assert!(r.marker.is_none());
    }

    #[test]
    fn parse_with_specifiers_and_extras_and_marker() {
        let r = parse("Requests[security,socks] >=2.0,<3.0 ; python_version>='3.8'").unwrap();
        assert_eq!(r.name, "requests");
        assert_eq!(r.extras, vec!["security".to_string(), "socks".to_string()]);
        assert_eq!(r.specifiers.len(), 2);
        assert!(r.marker.unwrap().contains("python_version"));
    }

    #[test]
    fn normalize_name_collapses_runs() {
        assert_eq!(normalize_name("Foo_._-Bar"), "foo-bar");
        assert_eq!(normalize_name("PyYAML"), "pyyaml");
    }

    #[test]
    fn empty_input_errors() {
        assert!(matches!(parse("   "), Err(ParseError::Empty)));
    }

    // ----------------------------------------------------------------
    // PEP 508 parenthesized specifier sets — `name (specifier_set)`
    // ----------------------------------------------------------------

    /// The six `requires_dist` lines pypi.org serves for `requests==2.31.0`
    /// verbatim: one balanced outer pair of parentheses wraps the specifier
    /// set and must be stripped before the set is parsed. The two with a
    /// marker keep it.
    #[test]
    fn parenthesized_specifier_set_matches_the_bare_form() {
        let pairs = [
            ("charset-normalizer (<4,>=2)", "charset-normalizer <4,>=2"),
            ("idna (<4,>=2.5)", "idna <4,>=2.5"),
            ("urllib3 (<3,>=1.21.1)", "urllib3 <3,>=1.21.1"),
            ("certifi (>=2017.4.17)", "certifi >=2017.4.17"),
            (
                "PySocks (!=1.5.7,>=1.5.6) ; extra == 'socks'",
                "pysocks !=1.5.7,>=1.5.6 ; extra == 'socks'",
            ),
            (
                "chardet (<6,>=3.0.2) ; extra == 'use_chardet_on_py3'",
                "chardet <6,>=3.0.2 ; extra == 'use_chardet_on_py3'",
            ),
        ];
        for (parenthesized, bare) in pairs {
            let p = parse(parenthesized)
                .unwrap_or_else(|e| panic!("{parenthesized:?} must parse: {e}"));
            let b = parse(bare).unwrap_or_else(|e| panic!("{bare:?} must parse: {e}"));
            assert_eq!(p.name, b.name, "name must match for {parenthesized:?}");
            assert_eq!(
                p.specifiers, b.specifiers,
                "the parenthesized form must yield the same specifier set as the bare form for {parenthesized:?}"
            );
            assert_eq!(
                p.marker, b.marker,
                "marker must match for {parenthesized:?}"
            );
        }
    }

    /// `name ()` — an empty interior — refuses.
    #[test]
    fn empty_parens_refuse() {
        assert!(matches!(parse("name ()"), Err(ParseError::BadSpecifier(_))));
    }

    /// `name (>=1` — an unmatched opening parenthesis — refuses.
    #[test]
    fn unmatched_opening_paren_refuses() {
        assert!(matches!(
            parse("name (>=1"),
            Err(ParseError::BadSpecifier(_))
        ));
    }

    /// `name (>=1) >=2` — trailing text after the closing parenthesis —
    /// refuses; only one balanced outer pair is unwrapped, never a prefix.
    #[test]
    fn trailing_text_after_closing_paren_refuses() {
        assert!(matches!(
            parse("name (>=1) >=2"),
            Err(ParseError::BadSpecifier(_))
        ));
    }

    /// The unparenthesized form is unaffected by the new unwrap step.
    #[test]
    fn bare_specifier_set_unchanged() {
        let r = parse("name >=1").unwrap();
        assert_eq!(r.name, "name");
        assert_eq!(r.specifiers.len(), 1);
    }
}
// HANDWRITE-END
