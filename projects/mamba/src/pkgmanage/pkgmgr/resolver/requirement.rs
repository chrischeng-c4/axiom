// HANDWRITE-BEGIN gap="missing-generator:hand-written:d2aeab54" tracker="standardize-gap-projects-mamba-src-pkgmgr-resolver-requirement-rs" reason="PEP 508 parser. Subset for P1: name, specifier set, extras, python_version + sys_platform markers. Optionally backed by pep508_rs crate."
//! PEP 508 dependency declaration parser (Phase-1 subset).
//!
//! Schema source: `.aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema`.
//! Recognised grammar:
//!     name [ "[" extras "]" ] [ specifier_set ] [ ";" marker ]
//! `name` is PEP 503-normalised (lowercase, runs of `[-_.]` collapsed to `-`).
//! `marker` is kept as raw text — evaluation lives in a later phase.

use serde::{Deserialize, Serialize};

use super::specifier::VersionSpecifier;

/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema (Requirement)
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
/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#logic (parse_roots)
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
        super::specifier::parse_set(&spec_input)
            .map_err(|e| ParseError::BadSpecifier(e.to_string()))?
    };

    Ok(Requirement {
        name,
        specifiers,
        extras,
        marker,
    })
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
}
// HANDWRITE-END
