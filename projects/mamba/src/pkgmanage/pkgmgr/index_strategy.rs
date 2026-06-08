// uv `--index-strategy` policy (Tick 136).
//
// When the user configures more than one package index (e.g. PyPI
// plus a private mirror), uv has to decide what to do when the same
// package name exists in multiple indexes. Three policies are
// exposed:
//
//   first-index         — only ever query the FIRST index that
//                         responds with the package (DEFAULT). This
//                         is the dependency-confusion-safe choice:
//                         a private name takes precedence over a
//                         later, less-trusted index even when the
//                         later index has a newer version, because
//                         "the wrong package" is worse than "an old
//                         package."
//   unsafe-first-match  — accept the first matching index but allow
//                         the resolver to fall through to subsequent
//                         indexes if the first has no satisfying
//                         version. Treats indexes as a fallback
//                         chain rather than a strict prefix.
//   unsafe-best-match   — query ALL indexes and pick the newest
//                         match across the union. Equivalent to
//                         pip's `--extra-index-url` semantics —
//                         convenient but unsafe (any index can be
//                         the source of truth for any name).
//
// The `unsafe-` prefixes are uv's chosen branding: the policies are
// not literally unsafe, they're just the ones that pip-style
// `--extra-index-url` use, which is what the dependency-confusion
// CVEs exploited. The first-index default is uv's correction.
//
// This module is a pure policy enum + parser. Index iteration in
// `simple_api.rs` / `resolver.rs` consumes the predicates.

use crate::pkgmanage::pkgmgr::types::IndexError;

const DETAIL: &str = "<--index-strategy>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexStrategy {
    FirstIndex,
    UnsafeFirstMatch,
    UnsafeBestMatch,
}

impl Default for IndexStrategy {
    fn default() -> Self {
        // uv's safe default. pip's `--extra-index-url` model is
        // closer to UnsafeBestMatch, but uv specifically chose the
        // dependency-confusion-safe path here.
        IndexStrategy::FirstIndex
    }
}

impl IndexStrategy {
    /// Canonical lowercase-hyphenated spelling.
    pub fn cli_name(self) -> &'static str {
        match self {
            IndexStrategy::FirstIndex => "first-index",
            IndexStrategy::UnsafeFirstMatch => "unsafe-first-match",
            IndexStrategy::UnsafeBestMatch => "unsafe-best-match",
        }
    }

    /// Parse the user-facing spelling. Case-insensitive with `_`/`-`
    /// interchangeable (uv convention).
    pub fn parse(raw: &str) -> Result<Self, IndexError> {
        let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "first-index" => Ok(IndexStrategy::FirstIndex),
            "unsafe-first-match" => Ok(IndexStrategy::UnsafeFirstMatch),
            "unsafe-best-match" => Ok(IndexStrategy::UnsafeBestMatch),
            _ => Err(IndexError::ParseError {
                url: DETAIL.into(),
                detail: format!(
                    "unknown --index-strategy `{raw}` (expected first-index / unsafe-first-match / unsafe-best-match)"
                ),
            }),
        }
    }

    /// True when the resolver is allowed to fall through to later
    /// indexes if the first index has no matching candidate. Both
    /// `unsafe-*` strategies allow this.
    pub fn allows_fallback_indexes(self) -> bool {
        matches!(
            self,
            IndexStrategy::UnsafeFirstMatch | IndexStrategy::UnsafeBestMatch
        )
    }

    /// True when the resolver should union candidates across every
    /// index (rather than stopping at the first matching one). Only
    /// `unsafe-best-match` does this.
    pub fn unions_across_indexes(self) -> bool {
        matches!(self, IndexStrategy::UnsafeBestMatch)
    }

    /// True when this strategy is the safe default. Used by config-
    /// emission diagnostics that want to flag user-configured
    /// `unsafe-*` choices.
    pub fn is_safe_default(self) -> bool {
        matches!(self, IndexStrategy::FirstIndex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_strategy_is_first_index() {
        assert_eq!(IndexStrategy::default(), IndexStrategy::FirstIndex);
    }

    #[test]
    fn parses_canonical_spellings() {
        for s in ["first-index", "unsafe-first-match", "unsafe-best-match"] {
            let p = IndexStrategy::parse(s).unwrap();
            assert_eq!(p.cli_name(), s);
        }
    }

    #[test]
    fn parse_is_case_insensitive_and_underscore_tolerant() {
        assert_eq!(
            IndexStrategy::parse("First_Index").unwrap(),
            IndexStrategy::FirstIndex
        );
        assert_eq!(
            IndexStrategy::parse("UNSAFE-BEST-MATCH").unwrap(),
            IndexStrategy::UnsafeBestMatch
        );
        assert_eq!(
            IndexStrategy::parse(" unsafe_first_match ").unwrap(),
            IndexStrategy::UnsafeFirstMatch
        );
    }

    #[test]
    fn rejects_unknown_strategy() {
        assert!(IndexStrategy::parse("union").is_err());
        assert!(IndexStrategy::parse("").is_err());
    }

    #[test]
    fn first_index_does_not_fall_back_or_union() {
        let s = IndexStrategy::FirstIndex;
        assert!(!s.allows_fallback_indexes());
        assert!(!s.unions_across_indexes());
        assert!(s.is_safe_default());
    }

    #[test]
    fn unsafe_first_match_falls_back_but_does_not_union() {
        let s = IndexStrategy::UnsafeFirstMatch;
        assert!(s.allows_fallback_indexes());
        assert!(!s.unions_across_indexes());
        assert!(!s.is_safe_default());
    }

    #[test]
    fn unsafe_best_match_falls_back_and_unions() {
        let s = IndexStrategy::UnsafeBestMatch;
        assert!(s.allows_fallback_indexes());
        assert!(s.unions_across_indexes());
        assert!(!s.is_safe_default());
    }

    #[test]
    fn round_trip_through_cli_name_and_parse() {
        for s in [
            IndexStrategy::FirstIndex,
            IndexStrategy::UnsafeFirstMatch,
            IndexStrategy::UnsafeBestMatch,
        ] {
            assert_eq!(IndexStrategy::parse(s.cli_name()).unwrap(), s);
        }
    }
}
