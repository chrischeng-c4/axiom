// Override file parser (Tick 34).
//
// uv ships `--override overrides.txt` to *force* the resolver's hand
// for a transitive dep. Where a `-c constraint` only narrows what the
// resolver may pick, an `--override` replaces the requirement string
// entirely — even if some transitive dep claims `requests<2`, an
// `--override requests==2.31.0` makes the resolver use 2.31.0 anyway
// (caveat emptor: skipping caps is on the user).
//
// The on-disk format is a strict subset of requirements.txt:
//
//   * one requirement per logical line (with `\`-continuation),
//   * comments (`#`) stripped,
//   * environment markers allowed (`; python_version >= "3.11"`),
//   * `-c FILE` / `-r FILE` includes accepted — same syntax as
//     constraints — so users can split overrides across files.
//
// What's specifically NOT allowed in an override file:
//   * options (`--index-url`, `--pre`, …). Overrides only carry
//     requirement strings + recursive includes. We surface an option
//     in an override file as a hard error so authors notice typos.
//
// This module reuses the constraints-file tokenizer (`constraints::
// parse_file`) and adds the "no options + dedup-by-marker" enforcement
// layer on top.

use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::constraints::{self, IncludeKind, ParsedLine};
use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// One override entry. We keep the raw requirement text so downstream
/// PEP 508 handling owns full parsing — this module's job is dedup +
/// validation, not full requirement understanding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideEntry {
    /// PEP 503-normalized package name.
    pub name: String,
    /// Full requirement string verbatim (e.g. `requests>=2.31; python_version >= "3.11"`).
    pub text: String,
    /// Verbatim marker text after the `;`, if any. Used as the dedup
    /// key together with `name` — two override lines for the same
    /// package with *different* markers are allowed and stack.
    pub marker: Option<String>,
}

/// Collection of overrides + the file paths the parser would like the
/// caller to follow (`-c FILE` and `-r FILE` directives).
#[derive(Debug, Clone, Default)]
pub struct OverrideSet {
    /// Keyed by (name, marker) so two markers on the same package can
    /// coexist. BTreeMap for stable iteration.
    pub entries: BTreeMap<(String, Option<String>), OverrideEntry>,
    /// Recursive `-c FILE` / `-r FILE` include paths in order.
    /// Caller owns I/O + cycle detection.
    pub includes: Vec<IncludeRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeRequest {
    pub kind: IncludeKind,
    pub path: String,
}

impl OverrideSet {
    /// Number of distinct override entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Lookup the override(s) applied to a package. Returns every entry
    /// with that normalized name, regardless of marker.
    pub fn for_name(&self, name: &str) -> Vec<&OverrideEntry> {
        let normalized = pep503_normalize(name);
        self.entries
            .iter()
            .filter(|((n, _), _)| n == &normalized)
            .map(|(_, v)| v)
            .collect()
    }
}

/// Parse an override file body. Reuses the constraints tokenizer for
/// comments + continuations + includes; rejects options.
pub fn parse_overrides(src: &str) -> Result<OverrideSet, IndexError> {
    let lines = constraints::parse_file(src)?;
    let mut out = OverrideSet::default();
    for line in lines {
        match line {
            ParsedLine::Option { name, .. } => {
                return Err(IndexError::ParseError {
                    url: "<overrides file>".into(),
                    detail: format!(
                        "options not allowed in override files: --{name}; \
                         override files must only contain requirement strings \
                         and `-c`/`-r` includes"
                    ),
                });
            }
            ParsedLine::Include { kind, path } => {
                out.includes.push(IncludeRequest { kind, path });
            }
            ParsedLine::Requirement { text } => {
                let entry = parse_one(text.as_str())?;
                let key = (entry.name.clone(), entry.marker.clone());
                if let Some(prev) = out.entries.insert(key.clone(), entry.clone()) {
                    return Err(IndexError::ParseError {
                        url: "<overrides file>".into(),
                        detail: format!(
                            "duplicate override for {:?} (marker {:?}): \
                             first  {:?}, \
                             second {:?}",
                            key.0, key.1, prev.text, entry.text
                        ),
                    });
                }
            }
        }
    }
    Ok(out)
}

fn parse_one(raw: &str) -> Result<OverrideEntry, IndexError> {
    // Split off marker on top-level ';'. We don't peek inside quotes
    // because requirement names never contain ';' or '"' — anything
    // before the first ';' is the requirement spec, anything after is
    // the marker. uv uses the same heuristic for override files.
    let (spec, marker) = match raw.find(';') {
        Some(i) => {
            let m = raw[i + 1..].trim().to_string();
            let m = if m.is_empty() { None } else { Some(m) };
            (raw[..i].trim().to_string(), m)
        }
        None => (raw.trim().to_string(), None),
    };
    if spec.is_empty() {
        return Err(IndexError::ParseError {
            url: "<overrides file>".into(),
            detail: format!("empty requirement in override line {raw:?}"),
        });
    }
    let name = extract_name(&spec)?;
    Ok(OverrideEntry {
        name,
        text: raw.trim().to_string(),
        marker,
    })
}

/// PEP 508 requirement name extractor. Stops at the first character
/// that can't legally be part of a name: `=<>!~([@` plus whitespace.
fn extract_name(spec: &str) -> Result<String, IndexError> {
    let mut name = String::new();
    for c in spec.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
            name.push(c);
        } else {
            break;
        }
    }
    if name.is_empty() {
        return Err(IndexError::ParseError {
            url: "<overrides file>".into(),
            detail: format!("could not extract package name from {spec:?}"),
        });
    }
    Ok(pep503_normalize(&name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_yields_empty_set() {
        let set = parse_overrides("").unwrap();
        assert!(set.is_empty());
        assert!(set.includes.is_empty());
    }

    #[test]
    fn parse_single_pin() {
        let set = parse_overrides("requests==2.31.0\n").unwrap();
        assert_eq!(set.len(), 1);
        let entry = set.for_name("Requests");
        assert_eq!(entry.len(), 1);
        assert_eq!(entry[0].name, "requests");
        assert_eq!(entry[0].text, "requests==2.31.0");
        assert!(entry[0].marker.is_none());
    }

    #[test]
    fn parse_normalizes_name() {
        let set = parse_overrides("My_Package==1.0\n").unwrap();
        assert!(!set.for_name("my-package").is_empty());
        assert!(!set.for_name("my.package").is_empty());
    }

    #[test]
    fn parse_records_marker_text() {
        let set = parse_overrides(
            "flask>=2.0; python_version >= \"3.11\"\n",
        )
        .unwrap();
        let entries = set.for_name("flask");
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].marker.as_deref(),
            Some("python_version >= \"3.11\"")
        );
    }

    #[test]
    fn parse_dedups_strictly_on_name_and_marker_pair() {
        // Same name, same marker → duplicate error.
        let err =
            parse_overrides("flask==2.0\nflask==2.1\n").unwrap_err();
        assert!(format!("{err}").contains("duplicate override"));
    }

    #[test]
    fn parse_allows_same_name_different_markers() {
        let set = parse_overrides(
            "flask==2.0; python_version < \"3.11\"\n\
             flask==2.1; python_version >= \"3.11\"\n",
        )
        .unwrap();
        assert_eq!(set.len(), 2);
        let entries = set.for_name("flask");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn parse_records_includes() {
        let set = parse_overrides(
            "-c constraints/prod.txt\n\
             -r other-overrides.txt\n\
             requests==2.31\n",
        )
        .unwrap();
        assert_eq!(set.len(), 1);
        assert_eq!(set.includes.len(), 2);
        assert_eq!(set.includes[0].kind, IncludeKind::Constraint);
        assert_eq!(set.includes[0].path, "constraints/prod.txt");
        assert_eq!(set.includes[1].kind, IncludeKind::Requirement);
    }

    #[test]
    fn parse_rejects_options_in_override_file() {
        let err = parse_overrides("--index-url https://example.test\n").unwrap_err();
        assert!(
            format!("{err}").contains("options not allowed in override files"),
            "got: {err}"
        );
    }

    #[test]
    fn parse_skips_blank_and_comment_lines() {
        let set = parse_overrides(
            "# header\n\
             \n\
             requests==2.31\n\
             # trailing note\n",
        )
        .unwrap();
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn parse_handles_continuation() {
        let set = parse_overrides("flask==2.1; \\\n  python_version >= \"3.11\"\n").unwrap();
        assert_eq!(set.len(), 1);
        let e = &set.for_name("flask")[0];
        assert_eq!(
            e.marker.as_deref(),
            Some("python_version >= \"3.11\"")
        );
    }

    #[test]
    fn parse_rejects_empty_requirement_after_marker_split() {
        // A bare `;` line is not a valid requirement.
        let err = parse_overrides(";python_version >= \"3.11\"\n").unwrap_err();
        assert!(format!("{err}").contains("empty requirement"));
    }

    #[test]
    fn extract_name_works_for_various_specifiers() {
        assert_eq!(extract_name("flask").unwrap(), "flask");
        assert_eq!(extract_name("flask==2.0").unwrap(), "flask");
        assert_eq!(extract_name("flask>=2.0,<3").unwrap(), "flask");
        assert_eq!(extract_name("flask[async]>=2.0").unwrap(), "flask");
        assert_eq!(extract_name("My_Pkg.Name>=1").unwrap(), "my-pkg-name");
    }

    #[test]
    fn extract_name_errors_on_empty_spec() {
        let err = extract_name("==2.0").unwrap_err();
        assert!(format!("{err}").contains("could not extract"));
    }
}
