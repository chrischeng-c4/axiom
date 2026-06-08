// uv `--no-build-isolation` / `--no-build-isolation-package` (Tick 139).
//
// PEP 517 mandates that each sdist build runs in an *isolated*
// environment: only the `build-system.requires` declared in
// pyproject.toml are visible to the backend, never the host's
// site-packages. uv defaults to this (PEP 517-compliant) behaviour
// and exposes two opt-outs:
//
//   --no-build-isolation              — disable isolation for every
//                                       package being built. Falls
//                                       back to the legacy pre-PEP-517
//                                       behaviour where the backend
//                                       sees the host site-packages.
//   --no-build-isolation-package foo  — disable isolation only for
//                                       `foo`. Useful when a project
//                                       declares its build-system
//                                       requirements incorrectly and
//                                       the maintainer hasn't fixed
//                                       it yet.
//
// Disabling isolation is unsafe — it makes the build non-hermetic
// and depends on whatever happens to be installed in the host env.
// We surface the user's choice as a typed policy so the build-driver
// can refuse to cache a non-isolated build (the cache key would not
// reflect the host env state).
//
// Same Scope ADT shape as upgrade/reinstall/refresh: None / All /
// Selective.

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BuildIsolationScope {
    /// Default: every build runs PEP 517-isolated.
    #[default]
    AlwaysIsolate,
    /// `--no-build-isolation`: every build sees the host site-packages.
    NeverIsolate,
    /// `--no-build-isolation-package foo`: only the listed PEP 503-
    /// normalized names skip isolation.
    DisableFor(BTreeSet<String>),
}

impl BuildIsolationScope {
    /// True iff PEP 517 isolation should be enabled for `pkg`.
    /// `pkg` should already be PEP 503-normalized.
    pub fn should_isolate(&self, pkg: &str) -> bool {
        match self {
            BuildIsolationScope::AlwaysIsolate => true,
            BuildIsolationScope::NeverIsolate => false,
            BuildIsolationScope::DisableFor(set) => !set.contains(pkg),
        }
    }

    /// Build a selective scope from an iterator of raw names. Names
    /// are PEP 503-normalized and trimmed at construction; an
    /// all-empty input collapses to `AlwaysIsolate`.
    pub fn disable_for<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let set: BTreeSet<String> = names
            .into_iter()
            .map(|s| pep503_normalize(s.as_ref().trim()))
            .filter(|s| !s.is_empty())
            .collect();
        if set.is_empty() {
            BuildIsolationScope::AlwaysIsolate
        } else {
            BuildIsolationScope::DisableFor(set)
        }
    }

    /// True when every package is isolated (the safe default).
    pub fn is_safe_default(&self) -> bool {
        matches!(self, BuildIsolationScope::AlwaysIsolate)
    }

    /// True when at least one package builds without isolation. Used
    /// by the build-cache layer to decide whether the build result
    /// is reusable.
    pub fn has_non_isolated_builds(&self) -> bool {
        match self {
            BuildIsolationScope::AlwaysIsolate => false,
            BuildIsolationScope::NeverIsolate => true,
            BuildIsolationScope::DisableFor(set) => !set.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_always_isolate() {
        let s = BuildIsolationScope::default();
        assert!(s.is_safe_default());
        assert!(s.should_isolate("anything"));
    }

    #[test]
    fn never_isolate_disables_isolation_for_every_package() {
        let s = BuildIsolationScope::NeverIsolate;
        assert!(!s.is_safe_default());
        assert!(!s.should_isolate("foo"));
        assert!(!s.should_isolate("bar"));
        assert!(s.has_non_isolated_builds());
    }

    #[test]
    fn disable_for_only_affects_listed_names() {
        let s = BuildIsolationScope::disable_for(["foo", "bar"]);
        assert!(!s.should_isolate("foo"));
        assert!(!s.should_isolate("bar"));
        assert!(s.should_isolate("baz"));
        assert!(s.has_non_isolated_builds());
    }

    #[test]
    fn disable_for_normalizes_to_pep503() {
        let s = BuildIsolationScope::disable_for(["My.Pkg"]);
        assert!(!s.should_isolate("my-pkg"));
    }

    #[test]
    fn disable_for_with_empty_input_collapses_to_default() {
        let s = BuildIsolationScope::disable_for(Vec::<&str>::new());
        assert!(s.is_safe_default());
        let s = BuildIsolationScope::disable_for(["", "  "]);
        assert!(s.is_safe_default());
    }

    #[test]
    fn dedup_via_btreeset() {
        let s = BuildIsolationScope::disable_for(["foo", "Foo", "foo"]);
        match s {
            BuildIsolationScope::DisableFor(set) => assert_eq!(set.len(), 1),
            _ => panic!("expected disable_for"),
        }
    }

    #[test]
    fn has_non_isolated_builds_is_false_for_default() {
        assert!(!BuildIsolationScope::default().has_non_isolated_builds());
    }

    #[test]
    fn opposite_semantics_to_other_scopes() {
        // Important asymmetry: where UpgradeScope::None means "no
        // package is force-redone", BuildIsolationScope::AlwaysIsolate
        // means "every package gets the safe behaviour". They're both
        // defaults, but the predicate they answer is the opposite
        // direction. Verify the should_isolate predicate at least
        // doesn't accidentally invert.
        let safe = BuildIsolationScope::AlwaysIsolate;
        assert!(safe.should_isolate("any"));
        let unsafe_ = BuildIsolationScope::NeverIsolate;
        assert!(!unsafe_.should_isolate("any"));
    }
}
