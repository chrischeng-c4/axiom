// uv `--resolution` strategy (Tick 134).
//
// uv lets the user pick how the resolver chooses among candidates
// when multiple versions satisfy a constraint:
//
//   highest        — pick the largest satisfying version (DEFAULT).
//                    Matches pip's default and is what every regular
//                    `mamba pip install` invocation should use.
//   lowest         — pick the smallest satisfying version. Useful for
//                    CI matrices that want to verify the declared
//                    lower-bound pins still actually work end-to-end.
//   lowest-direct  — pick the smallest for *direct* dependencies but
//                    the largest for transitive dependencies. This is
//                    uv's compromise mode: you exercise your declared
//                    lower bounds without artificially pinning every
//                    transitive lib at its EOL'd lower bound (which
//                    `lowest` would do).
//
// This module is a pure policy enum + parser. The resolver in
// `resolver.rs` consumes `pick_candidate(direct, candidates)` to
// project the sorted candidate list down to its winner.

use crate::pkgmanage::pkgmgr::types::IndexError;

const DETAIL: &str = "<--resolution>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    Highest,
    Lowest,
    LowestDirect,
}

impl Default for ResolutionStrategy {
    fn default() -> Self {
        ResolutionStrategy::Highest
    }
}

impl ResolutionStrategy {
    /// Canonical spelling (hyphenated lowercase) for diagnostics and
    /// config file emission.
    pub fn cli_name(self) -> &'static str {
        match self {
            ResolutionStrategy::Highest => "highest",
            ResolutionStrategy::Lowest => "lowest",
            ResolutionStrategy::LowestDirect => "lowest-direct",
        }
    }

    /// Parse the user-facing spelling. Case-insensitive, with `_` and
    /// `-` interchangeable (matches uv's tolerant TOML config keys).
    pub fn parse(raw: &str) -> Result<Self, IndexError> {
        let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "highest" => Ok(ResolutionStrategy::Highest),
            "lowest" => Ok(ResolutionStrategy::Lowest),
            "lowest-direct" => Ok(ResolutionStrategy::LowestDirect),
            _ => Err(IndexError::ParseError {
                url: DETAIL.into(),
                detail: format!(
                    "unknown --resolution strategy `{raw}` (expected highest / lowest / lowest-direct)"
                ),
            }),
        }
    }

    /// Pick the winning candidate from a list that's already sorted
    /// ascending by PEP 440 ordering. `is_direct` tells us whether
    /// this package was explicitly named in a user-supplied
    /// requirement (the user's top-level project dependency set), as
    /// opposed to a transitive dependency.
    pub fn pick_candidate<T>(self, is_direct: bool, candidates_ascending: &[T]) -> Option<&T> {
        if candidates_ascending.is_empty() {
            return None;
        }
        let want_lowest = match self {
            ResolutionStrategy::Highest => false,
            ResolutionStrategy::Lowest => true,
            ResolutionStrategy::LowestDirect => is_direct,
        };
        if want_lowest {
            candidates_ascending.first()
        } else {
            candidates_ascending.last()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_strategy_is_highest() {
        assert_eq!(ResolutionStrategy::default(), ResolutionStrategy::Highest);
    }

    #[test]
    fn parses_canonical_spellings() {
        for s in ["highest", "lowest", "lowest-direct"] {
            let r = ResolutionStrategy::parse(s).unwrap();
            assert_eq!(r.cli_name(), s);
        }
    }

    #[test]
    fn parse_is_case_insensitive_and_underscore_tolerant() {
        assert_eq!(
            ResolutionStrategy::parse("Lowest_Direct").unwrap(),
            ResolutionStrategy::LowestDirect
        );
        assert_eq!(
            ResolutionStrategy::parse("HIGHEST").unwrap(),
            ResolutionStrategy::Highest
        );
        assert_eq!(
            ResolutionStrategy::parse("  lowest  ").unwrap(),
            ResolutionStrategy::Lowest
        );
    }

    #[test]
    fn rejects_unknown_strategy() {
        assert!(ResolutionStrategy::parse("random").is_err());
        assert!(ResolutionStrategy::parse("").is_err());
    }

    #[test]
    fn highest_picks_last_of_ascending_list() {
        let cands = vec!["1.0", "1.1", "2.0"];
        let pick = ResolutionStrategy::Highest.pick_candidate(true, &cands);
        assert_eq!(pick, Some(&"2.0"));
    }

    #[test]
    fn lowest_picks_first_of_ascending_list() {
        let cands = vec!["1.0", "1.1", "2.0"];
        let pick = ResolutionStrategy::Lowest.pick_candidate(false, &cands);
        assert_eq!(pick, Some(&"1.0"));
    }

    #[test]
    fn lowest_direct_picks_first_for_direct_last_for_transitive() {
        let cands = vec!["1.0", "1.1", "2.0"];
        let strat = ResolutionStrategy::LowestDirect;
        assert_eq!(strat.pick_candidate(true, &cands), Some(&"1.0"));
        assert_eq!(strat.pick_candidate(false, &cands), Some(&"2.0"));
    }

    #[test]
    fn empty_candidate_list_yields_none() {
        let cands: Vec<&str> = vec![];
        for s in [
            ResolutionStrategy::Highest,
            ResolutionStrategy::Lowest,
            ResolutionStrategy::LowestDirect,
        ] {
            assert!(s.pick_candidate(true, &cands).is_none());
            assert!(s.pick_candidate(false, &cands).is_none());
        }
    }

    #[test]
    fn single_candidate_returned_for_every_strategy() {
        let cands = vec!["1.0"];
        for s in [
            ResolutionStrategy::Highest,
            ResolutionStrategy::Lowest,
            ResolutionStrategy::LowestDirect,
        ] {
            assert_eq!(s.pick_candidate(true, &cands), Some(&"1.0"));
            assert_eq!(s.pick_candidate(false, &cands), Some(&"1.0"));
        }
    }

    #[test]
    fn round_trip_through_cli_name_and_parse() {
        for s in [
            ResolutionStrategy::Highest,
            ResolutionStrategy::Lowest,
            ResolutionStrategy::LowestDirect,
        ] {
            assert_eq!(ResolutionStrategy::parse(s.cli_name()).unwrap(), s);
        }
    }
}
