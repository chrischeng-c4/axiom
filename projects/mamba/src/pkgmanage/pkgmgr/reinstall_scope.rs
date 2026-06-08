// uv `--reinstall` / `--reinstall-package` scope (Tick 137).
//
// uv distinguishes reinstall from upgrade. Upgrade re-resolves the
// dependency graph; reinstall keeps the resolved version but forces
// the installer to remove and re-extract the wheel on disk. The
// common use cases are:
//
//   * Recovering from a corrupted `.pyc` cache or a partially-extracted
//     wheel after a crashed previous install.
//   * Forcing an editable to re-link after a `pyproject.toml` change.
//   * Picking up a re-uploaded artifact (rare but happens on private
//     mirrors that rebuild wheels in place).
//
// The CLI surface mirrors `--upgrade` / `--upgrade-package`:
//
//   --reinstall                — reinstall every package
//   --reinstall-package foo    — reinstall only `foo`
//
// This module is a pure scope-set + predicate, mirroring the
// `UpgradeScope` design in `upgrade.rs` so call sites can dispatch
// the two policies identically.

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ReinstallScope {
    /// Default: never reinstall.
    #[default]
    None,
    /// `--reinstall`: reinstall every package.
    All,
    /// `--reinstall-package foo bar`: reinstall only the listed
    /// PEP 503-normalized names.
    Selective(BTreeSet<String>),
}

impl ReinstallScope {
    /// True iff `pkg` (already PEP 503-normalized) should be
    /// reinstalled rather than skipped when the on-disk version
    /// already matches the resolved version.
    pub fn should_reinstall(&self, pkg: &str) -> bool {
        match self {
            ReinstallScope::None => false,
            ReinstallScope::All => true,
            ReinstallScope::Selective(set) => set.contains(pkg),
        }
    }

    /// Build a selective scope from an iterator of raw names. Names
    /// are PEP 503-normalized so the membership test matches the
    /// canonical key the resolver uses elsewhere.
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
            ReinstallScope::None
        } else {
            ReinstallScope::Selective(set)
        }
    }

    /// True when no reinstall will ever fire.
    pub fn is_empty(&self) -> bool {
        matches!(self, ReinstallScope::None)
    }

    /// True when every package will be reinstalled.
    pub fn is_blanket(&self) -> bool {
        matches!(self, ReinstallScope::All)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scope_is_none() {
        let s = ReinstallScope::default();
        assert!(s.is_empty());
        assert!(!s.should_reinstall("requests"));
    }

    #[test]
    fn all_scope_reinstalls_every_package() {
        let s = ReinstallScope::All;
        assert!(s.is_blanket());
        assert!(s.should_reinstall("requests"));
        assert!(s.should_reinstall("any-name"));
    }

    #[test]
    fn selective_scope_only_matches_listed_names() {
        let s = ReinstallScope::selective(["requests", "urllib3"]);
        assert!(s.should_reinstall("requests"));
        assert!(s.should_reinstall("urllib3"));
        assert!(!s.should_reinstall("flask"));
    }

    #[test]
    fn selective_normalizes_names_to_pep503() {
        // `Foo.Bar`, `foo-bar`, and `FOO_BAR` all canonicalize to
        // `foo-bar`. Selective should look the package up by that
        // canonical key.
        let s = ReinstallScope::selective(["Foo.Bar"]);
        assert!(s.should_reinstall("foo-bar"));
        assert!(s.should_reinstall("foo_bar")
            || s.should_reinstall("foo-bar")); // tolerate normalization choice
    }

    #[test]
    fn selective_with_no_names_collapses_to_none() {
        let s = ReinstallScope::selective(Vec::<&str>::new());
        assert!(s.is_empty());
        // An all-empty-string input also collapses to None.
        let s = ReinstallScope::selective(["", "   "]);
        assert!(s.is_empty());
    }

    #[test]
    fn is_empty_and_is_blanket_are_mutually_exclusive() {
        let none = ReinstallScope::None;
        let all = ReinstallScope::All;
        let sel = ReinstallScope::selective(["foo"]);
        assert!(none.is_empty() && !none.is_blanket());
        assert!(all.is_blanket() && !all.is_empty());
        assert!(!sel.is_empty() && !sel.is_blanket());
    }

    #[test]
    fn dedup_via_btreeset() {
        let s = ReinstallScope::selective(["foo", "foo", "FOO", "Foo"]);
        match s {
            ReinstallScope::Selective(set) => assert_eq!(set.len(), 1),
            _ => panic!("expected selective"),
        }
    }
}
