// PEP 440 version-specifier set (Tick 114).
//
// Parses a comma-separated version-specifier clause from a PEP 508
// requirement string (e.g. `>=1.2.3, <2, !=1.5.*`) into a typed
// `SpecifierSet`, and tests candidate versions against it.
//
// Operators implemented (PEP 440 §"Version specifiers"):
//
//   ==      version match
//   !=      version exclusion
//   <       strictly less
//   <=      less or equal
//   >       strictly greater
//   >=      greater or equal
//   ~=      compatible-release (equivalent to `>=X.Y.Z, <X.(Y+1)`)
//   ==X.*   wildcard equality (prefix match on release segments)
//   !=X.*   wildcard exclusion (complement of `==X.*`)
//
// A `SpecifierSet` is satisfied by a candidate iff EVERY clause matches
// (logical AND), which matches both pip and uv resolver behavior.

use crate::pkgmanage::pkgmgr::pep440::{parse as parse_pep440, Pep440Version};
use crate::pkgmanage::pkgmgr::types::IndexError;

/// One comparator clause in a specifier set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Approx,
    EqWildcard,
    NotEqWildcard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Clause {
    op: Op,
    /// Original right-hand-side text (e.g. "1.2.*", "1.4.5").
    raw: String,
    /// PEP 440-parsed value of `raw` with any trailing `.*` removed.
    parsed: Pep440Version,
}

/// A parsed comma-separated version-specifier set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpecifierSet {
    clauses: Vec<Clause>,
}

impl SpecifierSet {
    /// Parse a comma-separated specifier set. Empty input yields an empty
    /// (universally-true) set.
    pub fn parse(src: &str) -> Result<Self, IndexError> {
        let trimmed = src.trim();
        if trimmed.is_empty() {
            return Ok(Self::default());
        }
        let mut clauses = Vec::new();
        for raw in trimmed.split(',') {
            let part = raw.trim();
            if part.is_empty() {
                return Err(parse_err("specifier set has an empty clause"));
            }
            clauses.push(parse_clause(part)?);
        }
        Ok(Self { clauses })
    }

    /// True when no clauses are present — every candidate satisfies an
    /// empty set.
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    /// Number of clauses.
    pub fn len(&self) -> usize {
        self.clauses.len()
    }

    /// Test a parsed `Pep440Version` against the set. Crate-visibility
    /// matches the `Pep440Version` type — external callers use
    /// [`SpecifierSet::matches`] which takes a `&str`.
    pub(crate) fn contains(&self, candidate: &Pep440Version) -> bool {
        self.clauses
            .iter()
            .all(|c| clause_matches(c, candidate))
    }

    /// Parse `candidate` and test it. Unparseable versions never match.
    pub fn matches(&self, candidate: &str) -> bool {
        match parse_pep440(candidate) {
            Some(v) => self.contains(&v),
            None => false,
        }
    }
}

fn parse_err(detail: impl Into<String>) -> IndexError {
    IndexError::ParseError {
        url: "<specifier>".to_string(),
        detail: detail.into(),
    }
}

fn parse_clause(src: &str) -> Result<Clause, IndexError> {
    // Longest-prefix operator first so `>=` doesn't tokenize as `>`.
    let candidates: &[(&str, Op)] = &[
        ("==", Op::Eq),
        ("!=", Op::NotEq),
        ("<=", Op::Le),
        (">=", Op::Ge),
        ("~=", Op::Approx),
        ("<", Op::Lt),
        (">", Op::Gt),
    ];
    for (sym, op) in candidates {
        if let Some(rhs) = src.strip_prefix(sym) {
            let rhs = rhs.trim();
            if rhs.is_empty() {
                return Err(parse_err(format!(
                    "specifier `{src}` has no version after the operator"
                )));
            }
            return finalize_clause(op.clone(), rhs);
        }
    }
    Err(parse_err(format!(
        "specifier `{src}` does not start with a known operator"
    )))
}

fn finalize_clause(op: Op, rhs: &str) -> Result<Clause, IndexError> {
    // Detect wildcard form `X.Y.*` — only legal on `==` and `!=` per PEP 440.
    if let Some(prefix) = rhs.strip_suffix(".*") {
        let op = match op {
            Op::Eq => Op::EqWildcard,
            Op::NotEq => Op::NotEqWildcard,
            _ => {
                return Err(parse_err(format!(
                    "wildcard `.*` only allowed with `==` / `!=`, got `{rhs}`"
                )))
            }
        };
        let parsed = parse_pep440(prefix).ok_or_else(|| {
            parse_err(format!("wildcard version `{rhs}` is not PEP 440 parseable"))
        })?;
        return Ok(Clause {
            op,
            raw: rhs.to_string(),
            parsed,
        });
    }

    // Approximate (`~=`) requires at least two release segments per PEP 440
    // §"Compatible release". `~=1` is invalid; `~=1.0` is the floor case.
    let parsed = parse_pep440(rhs).ok_or_else(|| {
        parse_err(format!("version `{rhs}` is not PEP 440 parseable"))
    })?;
    if matches!(op, Op::Approx) && parsed.release_segments().len() < 2 {
        return Err(parse_err(format!(
            "`~=` requires at least two release segments, got `{rhs}`"
        )));
    }

    Ok(Clause {
        op,
        raw: rhs.to_string(),
        parsed,
    })
}

fn clause_matches(c: &Clause, v: &Pep440Version) -> bool {
    match c.op {
        Op::Eq => v == &c.parsed,
        Op::NotEq => v != &c.parsed,
        Op::Lt => v < &c.parsed,
        Op::Le => v <= &c.parsed,
        Op::Gt => v > &c.parsed,
        Op::Ge => v >= &c.parsed,
        Op::Approx => approx_matches(c, v),
        Op::EqWildcard => wildcard_prefix_matches(c, v),
        Op::NotEqWildcard => !wildcard_prefix_matches(c, v),
    }
}

/// PEP 440 §"Compatible release": `~=X.Y.Z` ≡ `>=X.Y.Z, <X.(Y+1)`.
/// The upper bound drops the trailing segment of the specified version
/// and increments the new trailing segment.
fn approx_matches(c: &Clause, v: &Pep440Version) -> bool {
    if v < &c.parsed {
        return false;
    }
    let segs = c.parsed.release_segments();
    let prefix_len = segs.len() - 1; // already validated >= 2 in finalize_clause
    let v_segs = v.release_segments();
    // Each prefix segment must equal the specifier's prefix segments.
    for i in 0..prefix_len {
        let vs = v_segs.get(i).copied().unwrap_or(0);
        if vs != segs[i] {
            // The candidate has diverged from the compatible prefix.
            // Bound check: if the candidate's segment is smaller at any
            // position we already failed `>=` above; if larger, we exceed
            // `<X.(Y+1)`.
            return false;
        }
    }
    true
}

/// `==X.Y.*` matches a candidate whose first N release segments equal the
/// specifier's full release segments (after dropping `.*`).
fn wildcard_prefix_matches(c: &Clause, v: &Pep440Version) -> bool {
    let prefix = c.parsed.release_segments();
    let segs = v.release_segments();
    if segs.len() < prefix.len() {
        return false;
    }
    prefix
        .iter()
        .zip(segs.iter())
        .all(|(a, b)| a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(src: &str) -> SpecifierSet {
        SpecifierSet::parse(src).unwrap_or_else(|e| panic!("parse {src:?} failed: {e:?}"))
    }

    #[test]
    fn empty_set_matches_everything() {
        let s = SpecifierSet::parse("").unwrap();
        assert!(s.is_empty());
        assert!(s.matches("0.0.1"));
        assert!(s.matches("99.0.0"));
    }

    #[test]
    fn equality_clause() {
        let s = set("==1.2.3");
        assert!(s.matches("1.2.3"));
        assert!(!s.matches("1.2.4"));
        assert!(!s.matches("1.2.2"));
    }

    #[test]
    fn inequality_clause() {
        let s = set("!=1.2.3");
        assert!(!s.matches("1.2.3"));
        assert!(s.matches("1.2.4"));
    }

    #[test]
    fn lt_le_gt_ge_clauses() {
        let lt = set("<2");
        assert!(lt.matches("1.999"));
        assert!(!lt.matches("2"));
        assert!(!lt.matches("2.0.1"));

        let le = set("<=2");
        assert!(le.matches("2"));
        assert!(!le.matches("2.0.1"));

        let gt = set(">2");
        assert!(!gt.matches("2"));
        assert!(gt.matches("2.0.1"));

        let ge = set(">=2");
        assert!(ge.matches("2"));
        assert!(ge.matches("2.0.1"));
        assert!(!ge.matches("1.999"));
    }

    #[test]
    fn compound_range() {
        let s = set(">=1.2.3, <2");
        assert!(s.matches("1.2.3"));
        assert!(s.matches("1.9.9"));
        assert!(!s.matches("1.2.2"));
        assert!(!s.matches("2"));
        assert!(!s.matches("2.0.1"));
    }

    #[test]
    fn approx_three_segments() {
        // ~=1.4.5  ≡  >=1.4.5, <1.5
        let s = set("~=1.4.5");
        assert!(s.matches("1.4.5"));
        assert!(s.matches("1.4.99"));
        assert!(!s.matches("1.5"));
        assert!(!s.matches("1.4.4"));
        assert!(!s.matches("2.0"));
    }

    #[test]
    fn approx_two_segments() {
        // ~=1.4  ≡  >=1.4, <2
        let s = set("~=1.4");
        assert!(s.matches("1.4"));
        assert!(s.matches("1.99.99"));
        assert!(!s.matches("2.0"));
        assert!(!s.matches("1.3.9"));
    }

    #[test]
    fn approx_requires_two_segments() {
        let err = SpecifierSet::parse("~=1").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("two release segments"));
    }

    #[test]
    fn eq_wildcard() {
        let s = set("==1.2.*");
        assert!(s.matches("1.2"));
        assert!(s.matches("1.2.0"));
        assert!(s.matches("1.2.3"));
        assert!(s.matches("1.2.999"));
        assert!(!s.matches("1.3"));
        assert!(!s.matches("1.1.9"));
    }

    #[test]
    fn ne_wildcard() {
        let s = set("!=1.2.*");
        assert!(!s.matches("1.2"));
        assert!(!s.matches("1.2.3"));
        assert!(s.matches("1.3"));
        assert!(s.matches("2.0"));
    }

    #[test]
    fn wildcard_only_with_eq_or_ne() {
        assert!(SpecifierSet::parse(">=1.2.*").is_err());
        assert!(SpecifierSet::parse("<1.2.*").is_err());
        assert!(SpecifierSet::parse("~=1.2.*").is_err());
    }

    #[test]
    fn multiple_clauses_use_logical_and() {
        let s = set(">=1, <2, !=1.5.*");
        assert!(s.matches("1.0"));
        assert!(s.matches("1.9.9"));
        assert!(!s.matches("1.5.0"));
        assert!(!s.matches("1.5.999"));
        assert!(!s.matches("0.99"));
        assert!(!s.matches("2.0"));
    }

    #[test]
    fn pre_release_ordering_holds() {
        // 1.0a1 < 1.0b1 < 1.0rc1 < 1.0 < 1.0.post1
        let lt = set("<1.0");
        assert!(lt.matches("1.0a1"));
        assert!(lt.matches("1.0rc1"));
        assert!(!lt.matches("1.0"));

        let post = set(">1.0");
        assert!(post.matches("1.0.post1"));
        assert!(!post.matches("1.0"));
    }

    #[test]
    fn whitespace_around_operator_tolerated() {
        let s = set(" >=  1.2 ,   <2   ");
        assert!(s.matches("1.5"));
        assert!(!s.matches("2"));
    }

    #[test]
    fn unparseable_candidate_never_matches() {
        let s = set(">=1");
        assert!(!s.matches("garbage"));
    }

    #[test]
    fn rejects_unknown_operator() {
        let err = SpecifierSet::parse("@1.2.3").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("known operator"));
    }

    #[test]
    fn rejects_empty_rhs() {
        let err = SpecifierSet::parse(">=").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("no version"));
    }

    #[test]
    fn rejects_empty_clause() {
        let err = SpecifierSet::parse(">=1, ,<2").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("empty clause"));
    }

    #[test]
    fn rejects_unparseable_rhs() {
        assert!(SpecifierSet::parse(">=garbage").is_err());
        assert!(SpecifierSet::parse("==1.2.foo").is_err());
    }

    #[test]
    fn rejects_unparseable_wildcard_prefix() {
        assert!(SpecifierSet::parse("==foo.*").is_err());
    }

    #[test]
    fn len_reports_clause_count() {
        assert_eq!(set("").len(), 0);
        assert_eq!(set(">=1").len(), 1);
        assert_eq!(set(">=1, <2, !=1.5.*").len(), 3);
    }

    #[test]
    fn realistic_django_dependency_range() {
        // urllib3>=1.21.1,<3 — typical real-world bound from requests.
        let s = set(">=1.21.1, <3");
        assert!(s.matches("1.21.1"));
        assert!(s.matches("1.26.18"));
        assert!(s.matches("2.0"));
        assert!(s.matches("2.2.1"));
        assert!(!s.matches("1.20"));
        assert!(!s.matches("3.0"));
    }

    #[test]
    fn realistic_security_patch_exclusion() {
        // numpy>=1.21,<2,!=1.23.* — exclude a hypothetical broken minor.
        let s = set(">=1.21, <2, !=1.23.*");
        assert!(s.matches("1.21"));
        assert!(s.matches("1.22.5"));
        assert!(!s.matches("1.23.0"));
        assert!(!s.matches("1.23.4"));
        assert!(s.matches("1.24.0"));
        assert!(!s.matches("2.0"));
    }

    #[test]
    fn realistic_pin_to_minor_via_approx() {
        // pip-style: ~=1.4.5 pins to 1.4.x with floor 1.4.5.
        let s = set("~=1.4.5");
        assert!(s.matches("1.4.5"));
        assert!(s.matches("1.4.10"));
        assert!(!s.matches("1.5.0"));
        assert!(!s.matches("1.4.4"));
    }
}
