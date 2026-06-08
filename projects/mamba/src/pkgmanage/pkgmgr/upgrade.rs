// Upgrade strategy + candidate selection (Tick 24).
//
// uv exposes three resolution strategies and two upgrade scopes:
//
// Strategies (uv `--resolution=`):
//   highest      — pick the newest version that satisfies the requirement.
//                  Default. What you want for greenfield lockfiles.
//   lowest       — pick the *oldest* version that satisfies — used for
//                  detecting "I haven't pinned a lower bound and a
//                  decade-old release sneaks in".
//   lowest-direct— `lowest` for *directly-declared* dependencies and
//                  `highest` for everything else. Tests the lower bounds
//                  of your declared deps without dragging transitive
//                  dependencies back to ancient versions.
//
// Upgrade scope (uv `lock --upgrade[-package]`):
//   None                  — reuse existing lockfile pins where possible
//                           (default; `uv sync` semantics).
//   All                   — re-resolve everything from scratch
//                           (`uv lock --upgrade`).
//   Selective([names])    — re-resolve only these packages; keep the rest
//                           pinned (`uv lock --upgrade-package foo`).
//
// This module ships:
//   - typed `ResolutionStrategy` and `UpgradeScope`
//   - `pick_candidate(...)` — given a list of available PEP 440 versions
//     and the current lockfile pin (if any), pick which version the
//     resolver should propose.
//   - `detect_pin_conflicts(...)` — flag the (rare) case where two
//     selective upgrades transitively disagree on a shared sibling.
//
// All version comparisons go through `pkgmanage::pkgmgr::pep440`. We don't
// re-implement the comparator here.

use std::collections::BTreeSet;

use crate::pkgmanage::pkgmgr::pep440;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Resolution strategy for a single package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    Highest,
    Lowest,
    LowestDirect,
}

impl ResolutionStrategy {
    /// Effective strategy for a *specific* package under the global strategy,
    /// taking the direct/transitive distinction into account.
    pub fn effective(self, is_direct: bool) -> EffectiveStrategy {
        match (self, is_direct) {
            (ResolutionStrategy::Highest, _) => EffectiveStrategy::Highest,
            (ResolutionStrategy::Lowest, _) => EffectiveStrategy::Lowest,
            (ResolutionStrategy::LowestDirect, true) => EffectiveStrategy::Lowest,
            (ResolutionStrategy::LowestDirect, false) => EffectiveStrategy::Highest,
        }
    }
}

/// Resolved-to-a-single-choice strategy after the direct/transitive split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveStrategy {
    Highest,
    Lowest,
}

/// Per-package upgrade decision. `Selective(names)` is normalized
/// case-insensitively via PEP 503 normalize at call sites.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UpgradeScope {
    /// Default: reuse lockfile pins where possible.
    #[default]
    None,
    /// Re-resolve everything (`uv lock --upgrade`).
    All,
    /// Re-resolve only these names (`uv lock --upgrade-package foo bar`).
    Selective(BTreeSet<String>),
}

impl UpgradeScope {
    /// True iff `pkg` should be re-resolved instead of pinned to the
    /// lockfile entry. `pkg` should already be PEP 503-normalized.
    pub fn should_upgrade(&self, pkg: &str) -> bool {
        match self {
            UpgradeScope::None => false,
            UpgradeScope::All => true,
            UpgradeScope::Selective(set) => set.contains(pkg),
        }
    }

    pub fn selective<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        UpgradeScope::Selective(names.into_iter().map(Into::into).collect())
    }
}

/// Inputs to a single-package candidate decision.
#[derive(Debug, Clone)]
pub struct CandidateContext<'a> {
    /// PEP 503-normalized package name.
    pub package: &'a str,
    /// All versions the index advertises (any order).
    pub available: &'a [String],
    /// Versions that pass the resolver's *requirement* filter already.
    /// Empty means "no compatible version exists" — surfaced as an error.
    pub compatible: &'a [String],
    /// Current lockfile pin, if any (post-PEP-503-normalize). `None` for
    /// first-time resolution.
    pub locked: Option<&'a str>,
    /// `true` if this package is a top-level (directly declared) dep, vs a
    /// transitive pulled in by something else.
    pub is_direct: bool,
}

/// Decide which version the resolver should propose for this package.
///
/// Decision tree:
///   1. If the package is *not* in `upgrade_scope` AND a `locked` version
///      is still in `compatible`, reuse the pin.
///   2. Otherwise pick by `strategy.effective(is_direct)`:
///      - `Highest` → max(compatible)
///      - `Lowest`  → min(compatible)
///
/// Returns the picked version string (a clone from `compatible`).
pub fn pick_candidate(
    ctx: &CandidateContext<'_>,
    strategy: ResolutionStrategy,
    upgrade: &UpgradeScope,
) -> Result<String, IndexError> {
    if ctx.compatible.is_empty() {
        return Err(IndexError::ParseError {
            url: format!("<resolver: {}>", ctx.package),
            detail: format!(
                "no compatible version for {} (available: {})",
                ctx.package,
                ctx.available.len()
            ),
        });
    }

    if !upgrade.should_upgrade(ctx.package) {
        if let Some(locked) = ctx.locked {
            if ctx.compatible.iter().any(|v| v == locked) {
                return Ok(locked.to_string());
            }
            // Pinned version no longer satisfies requirements (e.g. a
            // tightened constraint) — fall through and pick fresh.
        }
    }

    let effective = strategy.effective(ctx.is_direct);
    let picked = match effective {
        EffectiveStrategy::Highest => max_version(ctx.compatible),
        EffectiveStrategy::Lowest => min_version(ctx.compatible),
    };

    Ok(picked.to_string())
}

fn max_version(versions: &[String]) -> &str {
    debug_assert!(!versions.is_empty());
    let mut best = &versions[0];
    for v in &versions[1..] {
        if pep440_gt(v, best) {
            best = v;
        }
    }
    best.as_str()
}

fn min_version(versions: &[String]) -> &str {
    debug_assert!(!versions.is_empty());
    let mut best = &versions[0];
    for v in &versions[1..] {
        if pep440_gt(best, v) {
            best = v;
        }
    }
    best.as_str()
}

fn pep440_gt(a: &str, b: &str) -> bool {
    let pa = pep440::parse(a);
    let pb = pep440::parse(b);
    match (pa, pb) {
        (Some(pa), Some(pb)) => pa > pb,
        // Fall back to lexicographic for un-parseable inputs — preserves
        // determinism, even if it's not semantically ideal. The resolver
        // upstream filters most of these out already.
        _ => a > b,
    }
}

/// A conflict surfaced by `detect_pin_conflicts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinConflict {
    pub package: String,
    /// Pin we previously committed to (from earlier in this resolution).
    pub previous: String,
    /// Pin a later requester wants instead.
    pub requested: String,
    /// Names of the two requesters, for the error message.
    pub requester_a: String,
    pub requester_b: String,
}

/// Detect (package, version) conflicts across a sequence of pin proposals.
///
/// Input is a list of `(requester, package, version)` triples in resolution
/// order. Each `package` should already be PEP 503-normalized. The function
/// walks the list and reports the *first* point at which two different
/// requesters propose different versions for the same package.
///
/// This is the cheap front-line check that runs before invoking the PubGrub
/// resolver — catches the common case where two selective upgrades pull
/// transitively-conflicting requirements (e.g. `foo` wants `shared==1.0` and
/// `bar` wants `shared==2.0`) and produces a helpful error.
pub fn detect_pin_conflicts(proposals: &[(&str, &str, &str)]) -> Result<(), PinConflict> {
    let mut chosen: std::collections::BTreeMap<&str, (&str, &str)> =
        std::collections::BTreeMap::new();
    for (requester, package, version) in proposals {
        if let Some((prev_requester, prev_version)) = chosen.get(package) {
            if prev_version != version {
                return Err(PinConflict {
                    package: (*package).to_string(),
                    previous: (*prev_version).to_string(),
                    requested: (*version).to_string(),
                    requester_a: (*prev_requester).to_string(),
                    requester_b: (*requester).to_string(),
                });
            }
        } else {
            chosen.insert(*package, (*requester, *version));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(
        package: &'a str,
        available: &'a [String],
        compatible: &'a [String],
        locked: Option<&'a str>,
        is_direct: bool,
    ) -> CandidateContext<'a> {
        CandidateContext {
            package,
            available,
            compatible,
            locked,
            is_direct,
        }
    }

    #[test]
    fn upgrade_scope_default_keeps_pin() {
        let scope = UpgradeScope::None;
        assert!(!scope.should_upgrade("foo"));
    }

    #[test]
    fn upgrade_scope_all_upgrades_everything() {
        let scope = UpgradeScope::All;
        assert!(scope.should_upgrade("foo"));
        assert!(scope.should_upgrade("bar"));
    }

    #[test]
    fn upgrade_scope_selective_targets_specific_names() {
        let scope = UpgradeScope::selective(["foo", "baz"]);
        assert!(scope.should_upgrade("foo"));
        assert!(!scope.should_upgrade("bar"));
        assert!(scope.should_upgrade("baz"));
    }

    #[test]
    fn pick_candidate_reuses_locked_when_compatible() {
        let avail = vec![
            "1.0.0".to_string(),
            "1.1.0".to_string(),
            "2.0.0".to_string(),
        ];
        let compat = avail.clone();
        let c = ctx("foo", &avail, &compat, Some("1.1.0"), true);
        let pick = pick_candidate(&c, ResolutionStrategy::Highest, &UpgradeScope::None).unwrap();
        assert_eq!(pick, "1.1.0");
    }

    #[test]
    fn pick_candidate_upgrades_when_scope_demands() {
        let avail = vec![
            "1.0.0".to_string(),
            "1.1.0".to_string(),
            "2.0.0".to_string(),
        ];
        let compat = avail.clone();
        let c = ctx("foo", &avail, &compat, Some("1.1.0"), true);
        let pick = pick_candidate(&c, ResolutionStrategy::Highest, &UpgradeScope::All).unwrap();
        assert_eq!(pick, "2.0.0");
    }

    #[test]
    fn pick_candidate_selective_only_upgrades_named() {
        let avail = vec!["1.0.0".to_string(), "2.0.0".to_string()];
        let compat = avail.clone();

        // Selective targets foo — gets the highest.
        let c = ctx("foo", &avail, &compat, Some("1.0.0"), true);
        let pick = pick_candidate(
            &c,
            ResolutionStrategy::Highest,
            &UpgradeScope::selective(["foo"]),
        )
        .unwrap();
        assert_eq!(pick, "2.0.0");

        // bar isn't targeted — keeps pin.
        let c = ctx("bar", &avail, &compat, Some("1.0.0"), true);
        let pick = pick_candidate(
            &c,
            ResolutionStrategy::Highest,
            &UpgradeScope::selective(["foo"]),
        )
        .unwrap();
        assert_eq!(pick, "1.0.0");
    }

    #[test]
    fn pick_candidate_drops_stale_pin_when_no_longer_compatible() {
        let avail = vec![
            "1.0.0".to_string(),
            "1.1.0".to_string(),
            "2.0.0".to_string(),
        ];
        // Requirement tightened — only >=2 compatible.
        let compat = vec!["2.0.0".to_string()];
        let c = ctx("foo", &avail, &compat, Some("1.1.0"), true);
        let pick = pick_candidate(&c, ResolutionStrategy::Highest, &UpgradeScope::None).unwrap();
        assert_eq!(pick, "2.0.0");
    }

    #[test]
    fn pick_candidate_lowest_strategy_picks_min() {
        let avail = vec![
            "1.0.0".to_string(),
            "1.1.0".to_string(),
            "2.0.0".to_string(),
        ];
        let compat = avail.clone();
        let c = ctx("foo", &avail, &compat, None, true);
        let pick = pick_candidate(&c, ResolutionStrategy::Lowest, &UpgradeScope::None).unwrap();
        assert_eq!(pick, "1.0.0");
    }

    #[test]
    fn pick_candidate_lowest_direct_picks_min_for_direct_max_for_transitive() {
        let avail = vec!["1.0.0".to_string(), "2.0.0".to_string()];
        let compat = avail.clone();

        let direct = ctx("foo", &avail, &compat, None, true);
        let pick = pick_candidate(
            &direct,
            ResolutionStrategy::LowestDirect,
            &UpgradeScope::None,
        )
        .unwrap();
        assert_eq!(pick, "1.0.0");

        let transitive = ctx("foo", &avail, &compat, None, false);
        let pick = pick_candidate(
            &transitive,
            ResolutionStrategy::LowestDirect,
            &UpgradeScope::None,
        )
        .unwrap();
        assert_eq!(pick, "2.0.0");
    }

    #[test]
    fn pick_candidate_errors_when_no_compatible_version() {
        let avail = vec!["1.0.0".to_string()];
        let compat: Vec<String> = vec![];
        let c = ctx("foo", &avail, &compat, None, true);
        let err = pick_candidate(&c, ResolutionStrategy::Highest, &UpgradeScope::None).unwrap_err();
        assert!(format!("{err}").contains("no compatible version"));
    }

    #[test]
    fn pick_candidate_orders_via_pep440_not_lex() {
        // Lexicographic would prefer "10.0.0" < "9.0.0"; PEP 440 prefers 10.
        let avail = vec!["9.0.0".to_string(), "10.0.0".to_string()];
        let compat = avail.clone();
        let c = ctx("foo", &avail, &compat, None, true);
        let pick = pick_candidate(&c, ResolutionStrategy::Highest, &UpgradeScope::None).unwrap();
        assert_eq!(pick, "10.0.0");
    }

    #[test]
    fn pick_candidate_handles_pre_releases() {
        // PEP 440: 2.0.0rc1 < 2.0.0
        let avail = vec!["2.0.0".to_string(), "2.0.0rc1".to_string()];
        let compat = avail.clone();
        let c = ctx("foo", &avail, &compat, None, true);
        let high = pick_candidate(&c, ResolutionStrategy::Highest, &UpgradeScope::None).unwrap();
        assert_eq!(high, "2.0.0");

        let low = pick_candidate(&c, ResolutionStrategy::Lowest, &UpgradeScope::None).unwrap();
        assert_eq!(low, "2.0.0rc1");
    }

    #[test]
    fn detect_pin_conflicts_passes_when_consistent() {
        let proposals = vec![
            ("root", "foo", "1.0.0"),
            ("foo", "shared", "1.0.0"),
            ("bar", "shared", "1.0.0"),
        ];
        detect_pin_conflicts(&proposals).unwrap();
    }

    #[test]
    fn detect_pin_conflicts_flags_first_conflict() {
        let proposals = vec![
            ("root", "foo", "1.0.0"),
            ("foo", "shared", "1.0.0"),
            ("bar", "shared", "2.0.0"),
        ];
        let conflict = detect_pin_conflicts(&proposals).unwrap_err();
        assert_eq!(conflict.package, "shared");
        assert_eq!(conflict.previous, "1.0.0");
        assert_eq!(conflict.requested, "2.0.0");
        assert_eq!(conflict.requester_a, "foo");
        assert_eq!(conflict.requester_b, "bar");
    }

    #[test]
    fn detect_pin_conflicts_first_match_wins() {
        // Three requesters all disagree — surface the FIRST conflict, not
        // the last, so resolver users get a deterministic point of failure.
        let proposals = vec![
            ("a", "x", "1.0.0"),
            ("b", "x", "2.0.0"),
            ("c", "x", "3.0.0"),
        ];
        let conflict = detect_pin_conflicts(&proposals).unwrap_err();
        assert_eq!(conflict.requested, "2.0.0");
        assert_eq!(conflict.requester_b, "b");
    }
}
