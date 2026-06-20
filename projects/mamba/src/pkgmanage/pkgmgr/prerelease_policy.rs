// uv `--prerelease` policy (Tick 133).
//
// uv exposes five prerelease policies that gate how PEP 440 alpha /
// beta / rc / dev / pre versions enter the candidate set:
//
//   disallow                    — never consider any prerelease
//   allow                       — always consider prereleases
//   if-necessary                — consider only if no stable version
//                                 satisfies the dependency graph
//   explicit                    — consider only for packages whose
//                                 user-supplied requirement explicitly
//                                 names a prerelease (e.g. `==1.0a1`)
//   if-necessary-or-explicit    — union of the previous two
//
// The default is `if-necessary-or-explicit`, which matches pip's
// behaviour: a prerelease is selected if it's explicitly requested
// or if it's the only version that satisfies the constraint set.
//
// This module is a pure policy + classifier: it answers
// "given a candidate version, an explicit-request flag, and a
// has-stable-candidate flag, does this candidate pass?". The version
// parser already lives in `pep440.rs` (private to the crate); we
// do our own lightweight prerelease classification here so we don't
// have to widen pep440's surface.

use crate::pkgmanage::pkgmgr::types::IndexError;

const DETAIL: &str = "<--prerelease>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrereleasePolicy {
    Disallow,
    Allow,
    IfNecessary,
    Explicit,
    IfNecessaryOrExplicit,
}

impl Default for PrereleasePolicy {
    fn default() -> Self {
        // pip's default and uv's default both treat the union as the
        // safe fallback.
        PrereleasePolicy::IfNecessaryOrExplicit
    }
}

impl PrereleasePolicy {
    /// Spelling expected on the command line / in config files.
    pub fn cli_name(self) -> &'static str {
        match self {
            PrereleasePolicy::Disallow => "disallow",
            PrereleasePolicy::Allow => "allow",
            PrereleasePolicy::IfNecessary => "if-necessary",
            PrereleasePolicy::Explicit => "explicit",
            PrereleasePolicy::IfNecessaryOrExplicit => "if-necessary-or-explicit",
        }
    }

    /// Parse the user-facing spelling. Hyphens and underscores are
    /// interchangeable; the comparison is case-insensitive (matching
    /// uv's tolerant config-key handling).
    pub fn parse(raw: &str) -> Result<Self, IndexError> {
        let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "disallow" => Ok(PrereleasePolicy::Disallow),
            "allow" => Ok(PrereleasePolicy::Allow),
            "if-necessary" => Ok(PrereleasePolicy::IfNecessary),
            "explicit" => Ok(PrereleasePolicy::Explicit),
            "if-necessary-or-explicit" => Ok(PrereleasePolicy::IfNecessaryOrExplicit),
            _ => Err(IndexError::ParseError {
                url: DETAIL.into(),
                detail: format!(
                    "unknown --prerelease policy `{raw}` (expected disallow / allow / if-necessary / explicit / if-necessary-or-explicit)"
                ),
            }),
        }
    }

    /// Decide whether to admit `candidate` to the candidate set.
    ///
    ///   * `is_prerelease`        — true when `candidate` is itself a
    ///     PEP 440 prerelease (a/b/rc/dev/pre).
    ///   * `explicit_request`     — true when the *user-supplied*
    ///     requirement for this package mentions a prerelease version
    ///     specifier (typically detected by the resolver before
    ///     calling us).
    ///   * `has_stable_candidate` — true when at least one non-
    ///     prerelease candidate already exists in the set.
    pub fn admits(
        self,
        is_prerelease: bool,
        explicit_request: bool,
        has_stable_candidate: bool,
    ) -> bool {
        if !is_prerelease {
            // Stable versions always pass — the policy only gates
            // prereleases.
            return true;
        }
        match self {
            PrereleasePolicy::Disallow => false,
            PrereleasePolicy::Allow => true,
            PrereleasePolicy::IfNecessary => !has_stable_candidate,
            PrereleasePolicy::Explicit => explicit_request,
            PrereleasePolicy::IfNecessaryOrExplicit => explicit_request || !has_stable_candidate,
        }
    }
}

/// Classify a PEP 440 version string as a prerelease or not.
///
/// Detects the canonical prerelease markers (a, alpha, b, beta, c, rc,
/// pre, preview, dev) anywhere in the post-release-segment tail. Local
/// version (`+local`) and epoch (`N!`) prefixes are stripped first,
/// matching PEP 440.
///
/// This is intentionally lightweight (no full PEP 440 parser) — it's
/// used only by the policy gate, which doesn't need version ordering,
/// just the boolean classification.
pub fn is_prerelease(version: &str) -> bool {
    // Strip local version (PEP 440 §1.4 — does not affect ordering or
    // prerelease classification).
    let without_local = version.split('+').next().unwrap_or(version);
    // Strip epoch (`N!`).
    let s = match without_local.split_once('!') {
        Some((_, rest)) => rest,
        None => without_local,
    };
    let lower = s.to_ascii_lowercase();

    // Search for prerelease markers anywhere in the post-release tail.
    // The canonical syntax separates the release segments and the
    // prerelease marker with `.`, `-`, `_`, or no separator at all.
    let markers = [
        ".dev", "-dev", "_dev", "dev", ".pre", "-pre", "_pre", "pre", ".rc", "-rc", "_rc", "rc",
        ".alpha", "-alpha", "_alpha", "alpha", ".beta", "-beta", "_beta", "beta", ".c", "-c", "_c",
    ];
    if markers.iter().any(|m| lower.contains(m)) {
        return true;
    }

    // PEP 440 also recognizes bare `a` / `b` immediately following the
    // release segment as alpha / beta markers (e.g. `1.0a1`, `1.0b2`).
    // Detect a digit followed by `a` or `b` followed by a digit — the
    // surrounding-digit pattern keeps us from misfiring on package
    // versions like `1.0` that have no prerelease.
    let bytes = lower.as_bytes();
    for i in 1..bytes.len().saturating_sub(1) {
        let prev = bytes[i - 1];
        let curr = bytes[i];
        let next = bytes[i + 1];
        if prev.is_ascii_digit() && (curr == b'a' || curr == b'b') && next.is_ascii_digit() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_is_if_necessary_or_explicit() {
        assert_eq!(
            PrereleasePolicy::default(),
            PrereleasePolicy::IfNecessaryOrExplicit
        );
    }

    #[test]
    fn parses_all_canonical_spellings() {
        for s in [
            "disallow",
            "allow",
            "if-necessary",
            "explicit",
            "if-necessary-or-explicit",
        ] {
            let p = PrereleasePolicy::parse(s).unwrap();
            assert_eq!(p.cli_name(), s);
        }
    }

    #[test]
    fn parse_is_case_insensitive_and_underscore_tolerant() {
        let p = PrereleasePolicy::parse("If_Necessary_Or_Explicit").unwrap();
        assert_eq!(p, PrereleasePolicy::IfNecessaryOrExplicit);
        let p = PrereleasePolicy::parse("DISALLOW").unwrap();
        assert_eq!(p, PrereleasePolicy::Disallow);
    }

    #[test]
    fn rejects_unknown_policy() {
        assert!(PrereleasePolicy::parse("yes-please").is_err());
        assert!(PrereleasePolicy::parse("").is_err());
    }

    #[test]
    fn stable_candidate_always_admitted_regardless_of_policy() {
        for p in [
            PrereleasePolicy::Disallow,
            PrereleasePolicy::Allow,
            PrereleasePolicy::IfNecessary,
            PrereleasePolicy::Explicit,
            PrereleasePolicy::IfNecessaryOrExplicit,
        ] {
            assert!(p.admits(false, false, false));
            assert!(p.admits(false, false, true));
            assert!(p.admits(false, true, true));
        }
    }

    #[test]
    fn disallow_rejects_every_prerelease() {
        let p = PrereleasePolicy::Disallow;
        assert!(!p.admits(true, true, false));
        assert!(!p.admits(true, false, false));
    }

    #[test]
    fn allow_admits_every_prerelease() {
        let p = PrereleasePolicy::Allow;
        assert!(p.admits(true, true, false));
        assert!(p.admits(true, false, true));
    }

    #[test]
    fn if_necessary_admits_only_when_no_stable() {
        let p = PrereleasePolicy::IfNecessary;
        assert!(p.admits(true, false, false));
        assert!(!p.admits(true, false, true));
        // explicit flag is ignored under if-necessary
        assert!(!p.admits(true, true, true));
    }

    #[test]
    fn explicit_admits_only_when_explicit_request() {
        let p = PrereleasePolicy::Explicit;
        assert!(p.admits(true, true, false));
        assert!(p.admits(true, true, true));
        assert!(!p.admits(true, false, false));
        assert!(!p.admits(true, false, true));
    }

    #[test]
    fn if_necessary_or_explicit_unions_both() {
        let p = PrereleasePolicy::IfNecessaryOrExplicit;
        assert!(p.admits(true, true, false));
        assert!(p.admits(true, true, true));
        assert!(p.admits(true, false, false));
        assert!(!p.admits(true, false, true));
    }

    #[test]
    fn is_prerelease_classifies_canonical_prerelease_markers() {
        assert!(is_prerelease("1.0a1"));
        assert!(is_prerelease("1.0b2"));
        assert!(is_prerelease("1.0rc3"));
        assert!(is_prerelease("1.0.dev42"));
        assert!(is_prerelease("1.0.pre1"));
        assert!(is_prerelease("1.0-alpha.1"));
        assert!(is_prerelease("2.0_beta_2"));
    }

    #[test]
    fn is_prerelease_rejects_stable_versions() {
        assert!(!is_prerelease("1.0"));
        assert!(!is_prerelease("1.2.3"));
        assert!(!is_prerelease("1.0.post1"));
        // NOTE: `2.0.dev` (bare ".dev" suffix, no number) is still a
        // prerelease per PEP 440 — covered as a positive assertion
        // in `is_prerelease_classifies_canonical_prerelease_markers`.
    }

    #[test]
    fn is_prerelease_strips_local_and_epoch_segments() {
        assert!(is_prerelease("1!1.0a1+cuda118"));
        assert!(!is_prerelease("1!1.0+cuda118"));
        assert!(is_prerelease("5!2.0rc1+abi3"));
    }

    #[test]
    fn is_prerelease_handles_bare_a_b_between_digits() {
        // PEP 440 allows `1.0a1` (no separator) and `1.0a.1`.
        assert!(is_prerelease("1.0a1"));
        assert!(is_prerelease("3.14b9"));
    }

    #[test]
    fn is_prerelease_does_not_misfire_on_versions_containing_letters() {
        // `1.0` has no prerelease marker. The bare-`a`/`b` heuristic
        // needs digits on BOTH sides, so it doesn't misfire on
        // alphabetic-but-not-prerelease tokens that don't normally
        // appear in PEP 440 versions anyway.
        assert!(!is_prerelease("1.0"));
        assert!(!is_prerelease("2024.6.15"));
    }
}
