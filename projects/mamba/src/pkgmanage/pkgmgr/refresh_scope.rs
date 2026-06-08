// uv `--refresh` / `--refresh-package` scope (Tick 138).
//
// uv ships three orthogonal "force-redo" flags that are easy to
// conflate:
//
//   --upgrade / --upgrade-package    — re-resolve the dependency
//                                      graph (`upgrade.rs`).
//   --reinstall / --reinstall-package — keep the resolution, but
//                                       force the installer to wipe
//                                       and re-extract the wheel on
//                                       disk (`reinstall_scope.rs`).
//   --refresh / --refresh-package    — keep both the resolution and
//                                      the install, but bypass the
//                                      HTTP-fetch cache for the
//                                      simple-API index responses and
//                                      release-file downloads.
//                                      (THIS module.)
//
// All three share the same Scope ADT shape (None / All / Selective)
// so call sites can decide independently per package whether the
// resolver, the installer, or the cache layer should fire.

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RefreshScope {
    /// Default: serve all cache hits.
    #[default]
    None,
    /// `--refresh`: bypass cache for every package.
    All,
    /// `--refresh-package foo bar`: bypass cache only for the
    /// listed PEP 503-normalized names.
    Selective(BTreeSet<String>),
}

impl RefreshScope {
    /// True iff the HTTP cache should be bypassed for `pkg` (already
    /// PEP 503-normalized).
    pub fn should_refresh(&self, pkg: &str) -> bool {
        match self {
            RefreshScope::None => false,
            RefreshScope::All => true,
            RefreshScope::Selective(set) => set.contains(pkg),
        }
    }

    /// Build a selective scope from an iterator of raw names. Names
    /// are PEP 503-normalized and trimmed at construction; empty
    /// inputs collapse to `None`.
    pub fn selective<I, S>(names: I) -> Self
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
            RefreshScope::None
        } else {
            RefreshScope::Selective(set)
        }
    }

    /// True when no refresh will ever fire.
    pub fn is_empty(&self) -> bool {
        matches!(self, RefreshScope::None)
    }

    /// True when the cache is bypassed for every package.
    pub fn is_blanket(&self) -> bool {
        matches!(self, RefreshScope::All)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scope_is_none() {
        let s = RefreshScope::default();
        assert!(s.is_empty());
        assert!(!s.should_refresh("anything"));
    }

    #[test]
    fn all_scope_refreshes_every_package() {
        let s = RefreshScope::All;
        assert!(s.is_blanket());
        assert!(s.should_refresh("requests"));
        assert!(s.should_refresh("some-other-pkg"));
    }

    #[test]
    fn selective_scope_only_matches_listed_names() {
        let s = RefreshScope::selective(["requests", "urllib3"]);
        assert!(s.should_refresh("requests"));
        assert!(s.should_refresh("urllib3"));
        assert!(!s.should_refresh("flask"));
    }

    #[test]
    fn selective_normalizes_names_to_pep503() {
        let s = RefreshScope::selective(["Foo.Bar"]);
        assert!(s.should_refresh("foo-bar"));
    }

    #[test]
    fn selective_with_no_names_collapses_to_none() {
        let s = RefreshScope::selective(Vec::<&str>::new());
        assert!(s.is_empty());
        let s = RefreshScope::selective(["", "   "]);
        assert!(s.is_empty());
    }

    #[test]
    fn dedup_via_btreeset() {
        let s = RefreshScope::selective(["foo", "Foo", "FOO", "foo"]);
        match s {
            RefreshScope::Selective(set) => assert_eq!(set.len(), 1),
            _ => panic!("expected selective"),
        }
    }

    #[test]
    fn refresh_independent_of_upgrade_and_reinstall_semantics() {
        // Verify the predicates are truly orthogonal: a RefreshScope
        // result on one package says nothing about whether upgrade
        // or reinstall would also fire — that's the caller's job to
        // combine.
        let s = RefreshScope::selective(["foo"]);
        assert!(s.should_refresh("foo"));
        assert!(!s.should_refresh("bar"));
    }
}
