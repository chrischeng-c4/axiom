// uv `--fork-strategy` strategy (Tick 135).
//
// uv's universal resolver represents a project's lockfile as a single
// resolution that holds across every supported `python_version` and
// platform. When a candidate version constraint conflicts on
// different markers (e.g. a transitive dep that requires
// `requires-python >= 3.12` but the project supports `>=3.9`),
// the resolver can either:
//
//   fewest          — minimize forks; prefer one universal version
//                     that satisfies every marker even if it's older.
//                     This is the default and matches what most
//                     CI matrices want.
//   requires-python — fork once per supported Python version, so each
//                     `python_version` slice gets the newest version
//                     of every dep that still satisfies that slice.
//                     Useful when newer interpreters can use newer
//                     library versions and the project doesn't mind
//                     a larger lockfile.
//
// This module is a pure policy enum + parser. Resolver integration
// in `resolver.rs` consumes `prefers_fork_per_python(self)`.

use crate::pkgmanage::pkgmgr::types::IndexError;

const DETAIL: &str = "<--fork-strategy>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkStrategy {
    Fewest,
    RequiresPython,
}

impl Default for ForkStrategy {
    fn default() -> Self {
        ForkStrategy::Fewest
    }
}

impl ForkStrategy {
    /// Canonical lowercase-hyphenated spelling.
    pub fn cli_name(self) -> &'static str {
        match self {
            ForkStrategy::Fewest => "fewest",
            ForkStrategy::RequiresPython => "requires-python",
        }
    }

    /// Parse the user-facing spelling. Case-insensitive, with `_` and
    /// `-` interchangeable (uv convention).
    pub fn parse(raw: &str) -> Result<Self, IndexError> {
        let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "fewest" => Ok(ForkStrategy::Fewest),
            "requires-python" => Ok(ForkStrategy::RequiresPython),
            _ => Err(IndexError::ParseError {
                url: DETAIL.into(),
                detail: format!(
                    "unknown --fork-strategy `{raw}` (expected fewest / requires-python)"
                ),
            }),
        }
    }

    /// Returns true when the resolver should fork once per supported
    /// `python_version` (i.e. when `self == RequiresPython`).
    pub fn prefers_fork_per_python(self) -> bool {
        matches!(self, ForkStrategy::RequiresPython)
    }

    /// Returns true when the resolver should minimize the number of
    /// forks across markers (i.e. when `self == Fewest`).
    pub fn prefers_universal(self) -> bool {
        matches!(self, ForkStrategy::Fewest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_strategy_is_fewest() {
        assert_eq!(ForkStrategy::default(), ForkStrategy::Fewest);
    }

    #[test]
    fn parses_canonical_spellings() {
        for s in ["fewest", "requires-python"] {
            let f = ForkStrategy::parse(s).unwrap();
            assert_eq!(f.cli_name(), s);
        }
    }

    #[test]
    fn parse_is_case_insensitive_and_underscore_tolerant() {
        assert_eq!(
            ForkStrategy::parse("Requires_Python").unwrap(),
            ForkStrategy::RequiresPython
        );
        assert_eq!(
            ForkStrategy::parse("FEWEST").unwrap(),
            ForkStrategy::Fewest
        );
        assert_eq!(
            ForkStrategy::parse(" requires-python ").unwrap(),
            ForkStrategy::RequiresPython
        );
    }

    #[test]
    fn rejects_unknown_strategy() {
        assert!(ForkStrategy::parse("most").is_err());
        assert!(ForkStrategy::parse("").is_err());
    }

    #[test]
    fn fork_predicates_are_complementary() {
        let f = ForkStrategy::Fewest;
        assert!(f.prefers_universal());
        assert!(!f.prefers_fork_per_python());

        let r = ForkStrategy::RequiresPython;
        assert!(!r.prefers_universal());
        assert!(r.prefers_fork_per_python());
    }

    #[test]
    fn round_trip_through_cli_name_and_parse() {
        for f in [ForkStrategy::Fewest, ForkStrategy::RequiresPython] {
            assert_eq!(ForkStrategy::parse(f.cli_name()).unwrap(), f);
        }
    }
}
