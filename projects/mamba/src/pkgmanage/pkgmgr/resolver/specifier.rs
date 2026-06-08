// HANDWRITE-BEGIN gap="missing-generator:hand-written:526b1361" tracker="standardize-gap-projects-mamba-src-pkgmgr-resolver-specifier-rs" reason="PEP 440 SpecifierSet + intersection. Reuses pkgmgr/pep440.rs for version ordering."
//! PEP 440 version specifier — parsing, evaluation, conjunctive intersection.
//!
//! Schema source: `.aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema`.
//! Logic source:  `.aw/tech-design/projects/mamba/pkgmgr/resolver.md#logic`
//! (intersect node — "Specifier intersection non-empty?").

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::pkgmanage::pkgmgr::pep440;

/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema (VersionSpecifier.op)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
    #[serde(rename = "==")]
    Eq,
    #[serde(rename = "!=")]
    NotEq,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = "<=")]
    Le,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = ">=")]
    Ge,
    #[serde(rename = "~=")]
    Compatible,
}

impl Op {
    fn parse(s: &str) -> Option<Op> {
        Some(match s {
            "==" => Op::Eq,
            "!=" => Op::NotEq,
            "<" => Op::Lt,
            "<=" => Op::Le,
            ">" => Op::Gt,
            ">=" => Op::Ge,
            "~=" => Op::Compatible,
            _ => return None,
        })
    }
}

/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema (VersionSpecifier)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionSpecifier {
    pub op: Op,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}

/// Parse a comma-separated specifier set, e.g. `">=2.0, <3.0"`.
pub fn parse_set(input: &str) -> Result<Vec<VersionSpecifier>, ParseError> {
    let mut out = Vec::new();
    for part in input.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        out.push(parse_one(p)?);
    }
    Ok(out)
}

/// Parse a single specifier like `">=2.0"`. Whitespace between op and version
/// is tolerated (PEP 440 allows it).
pub fn parse_one(input: &str) -> Result<VersionSpecifier, ParseError> {
    let s = input.trim();
    // Try 2-char ops first so "<=" doesn't get split as "<".
    for &cand in &["==", "!=", "<=", ">=", "~="] {
        if let Some(rest) = s.strip_prefix(cand) {
            return finish(cand, rest);
        }
    }
    for &cand in &["<", ">"] {
        if let Some(rest) = s.strip_prefix(cand) {
            return finish(cand, rest);
        }
    }
    Err(ParseError(format!("missing comparison operator: {s:?}")))
}

fn finish(op_str: &str, rest: &str) -> Result<VersionSpecifier, ParseError> {
    let op = Op::parse(op_str).ok_or_else(|| ParseError(format!("bad op {op_str:?}")))?;
    let version = rest.trim().trim_start_matches('=').trim().to_string();
    if version.is_empty() {
        return Err(ParseError(format!("missing version after {op_str:?}")));
    }
    if pep440::parse(&version).is_none() {
        return Err(ParseError(format!("not a PEP 440 version: {version:?}")));
    }
    Ok(VersionSpecifier { op, version })
}

impl VersionSpecifier {
    /// True if `candidate` (PEP 440) satisfies this specifier.
    ///
    /// Used by the `filter_yanked` / `intersect` nodes in the resolver flow.
    /// Returns `false` for unparseable candidates (defensive — index data is
    /// already filtered by `pep440::parse` upstream).
    pub fn matches(&self, candidate: &str) -> bool {
        let (Some(c), Some(v)) = (pep440::parse(candidate), pep440::parse(&self.version)) else {
            return false;
        };
        match self.op {
            Op::Eq => c.cmp(&v) == Ordering::Equal,
            Op::NotEq => c.cmp(&v) != Ordering::Equal,
            Op::Lt => c.cmp(&v) == Ordering::Less,
            Op::Le => matches!(c.cmp(&v), Ordering::Less | Ordering::Equal),
            Op::Gt => c.cmp(&v) == Ordering::Greater,
            Op::Ge => matches!(c.cmp(&v), Ordering::Greater | Ordering::Equal),
            Op::Compatible => {
                // ~= X.Y[.Z…]  ≡  >= X.Y[.Z…] , == X.Y.*  (drop final segment)
                if c.cmp(&v) == Ordering::Less {
                    return false;
                }
                compatible_upper_bound_matches(candidate, &self.version)
            }
        }
    }
}

/// PEP 440 §Compatible release: `~= V.N.M` requires the candidate to share
/// the leading `V.N` segments with `V.N.M`. Implemented via string-prefix on
/// the trimmed-of-trailing-component pin.
fn compatible_upper_bound_matches(candidate: &str, pin: &str) -> bool {
    let pin_segs: Vec<&str> = pin.split('.').collect();
    if pin_segs.len() < 2 {
        // ~= X is invalid per PEP 440; reject conservatively.
        return false;
    }
    let prefix_len = pin_segs.len() - 1;
    let pin_prefix = pin_segs[..prefix_len].join(".");
    let cand_segs: Vec<&str> = candidate.split('.').collect();
    if cand_segs.len() < prefix_len {
        return false;
    }
    cand_segs[..prefix_len].join(".") == pin_prefix
}

/// Conjunctive evaluation: every specifier must match.
///
/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#logic (intersect)
pub fn all_match(specs: &[VersionSpecifier], candidate: &str) -> bool {
    specs.iter().all(|s| s.matches(candidate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_set() {
        let specs = parse_set(">=2.0, <3.0").unwrap();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].op, Op::Ge);
        assert_eq!(specs[0].version, "2.0");
        assert_eq!(specs[1].op, Op::Lt);
    }

    #[test]
    fn matches_inclusive_and_exclusive() {
        let s: VersionSpecifier = parse_one(">=2.0").unwrap();
        assert!(s.matches("2.0"));
        assert!(s.matches("2.5.1"));
        assert!(!s.matches("1.9"));

        let s: VersionSpecifier = parse_one("<3.0").unwrap();
        assert!(s.matches("2.31.0"));
        assert!(!s.matches("3.0"));
    }

    #[test]
    fn compatible_release() {
        let s = parse_one("~=1.4").unwrap();
        assert!(s.matches("1.4"));
        assert!(s.matches("1.5.99"));
        assert!(!s.matches("2.0"));
        assert!(!s.matches("1.3"));
    }

    #[test]
    fn intersection_excludes_outliers() {
        let specs = parse_set(">=2.0, <3.0").unwrap();
        assert!(all_match(&specs, "2.31.0"));
        assert!(!all_match(&specs, "1.5"));
        assert!(!all_match(&specs, "3.0"));
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_one("nope").is_err());
        assert!(parse_one(">=").is_err());
        assert!(parse_one(">=garbage").is_err());
    }
}
// HANDWRITE-END
