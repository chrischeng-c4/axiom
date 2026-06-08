// uv `--no-binary` / `--only-binary` source-selection policy (Tick 140).
//
// pip and uv let the user constrain which *kind* of artifact the
// resolver may pick per package: wheels (binary) only, sdists
// (source) only, or both. Both flags accept three argument shapes:
//
//   * `:all:`           — apply to every package
//   * `:none:`          — clear any prior restriction (pip-compat)
//   * `<comma-list>`    — apply to the listed PEP 503-normalized names
//
// The two flags compose conjunctively. The matrix:
//
//   no-binary on `foo` + only-binary on `foo`   → contradiction; the
//                                                 caller has to reject
//                                                 this at CLI-parse
//                                                 time. We surface it
//                                                 via `is_contradictory`.
//   no-binary on `:all:` + only-binary empty    → sdist-only globally
//   only-binary on `:all:` + no-binary empty    → wheel-only globally
//   only-binary `foo` + no-binary `bar`         → mixed: `foo` is
//                                                 wheel-only, `bar`
//                                                 is sdist-only, the
//                                                 rest get both.
//
// This module is a pure policy ADT + parser. Resolver integration in
// `wheel_picker.rs` / `resolver.rs` reads `allows_wheel(pkg)` and
// `allows_sdist(pkg)` to gate candidates.

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::types::{IndexError, ReleaseFile};
use std::collections::BTreeSet;

const DETAIL: &str = "<--no-binary / --only-binary>";

/// One side of the policy (matches one CLI flag's argument).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SourceFilterSet {
    /// Default: this side of the policy hasn't been set.
    #[default]
    Empty,
    /// `:all:`: every package matches.
    All,
    /// PEP 503-normalized name set.
    Named(BTreeSet<String>),
}

impl SourceFilterSet {
    /// Parse one argument string. Empty input returns `Empty`.
    pub fn parse(raw: &str) -> Result<Self, IndexError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == ":none:" {
            return Ok(SourceFilterSet::Empty);
        }
        if trimmed == ":all:" {
            return Ok(SourceFilterSet::All);
        }
        let mut set = BTreeSet::new();
        for piece in trimmed.split(',') {
            let p = piece.trim();
            if p.is_empty() {
                continue;
            }
            if p == ":all:" || p == ":none:" {
                return Err(IndexError::ParseError {
                    url: DETAIL.into(),
                    detail: format!(
                        "`{p}` must appear alone, not mixed with other names"
                    ),
                });
            }
            set.insert(pep503_normalize(p));
        }
        if set.is_empty() {
            Ok(SourceFilterSet::Empty)
        } else {
            Ok(SourceFilterSet::Named(set))
        }
    }

    /// True iff this filter matches `pkg` (already PEP 503-normalized).
    pub fn matches(&self, pkg: &str) -> bool {
        match self {
            SourceFilterSet::Empty => false,
            SourceFilterSet::All => true,
            SourceFilterSet::Named(set) => set.contains(pkg),
        }
    }
}

/// Combined source-selection policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceStrategy {
    /// Names listed under `--no-binary` (sdist-only). When this set
    /// matches a package, wheels are excluded.
    pub no_binary: SourceFilterSet,
    /// Names listed under `--only-binary` (wheel-only). When this set
    /// matches a package, sdists are excluded.
    pub only_binary: SourceFilterSet,
}

impl SourceStrategy {
    /// True iff the resolver is allowed to consider wheel candidates
    /// for `pkg`. Wheels are excluded when `no_binary` matches.
    pub fn allows_wheel(&self, pkg: &str) -> bool {
        !self.no_binary.matches(pkg)
    }

    /// True iff the resolver is allowed to consider sdist candidates
    /// for `pkg`. Sdists are excluded when `only_binary` matches.
    pub fn allows_sdist(&self, pkg: &str) -> bool {
        !self.only_binary.matches(pkg)
    }

    /// True iff `pkg` is locked into the impossible "neither wheel
    /// nor sdist" state. The caller should treat that as a
    /// configuration error.
    pub fn is_contradictory(&self, pkg: &str) -> bool {
        !self.allows_wheel(pkg) && !self.allows_sdist(pkg)
    }

    /// True iff at least one global setting is active (`:all:` on
    /// either side). Used for diagnostic emission.
    pub fn has_global_restriction(&self) -> bool {
        matches!(self.no_binary, SourceFilterSet::All)
            || matches!(self.only_binary, SourceFilterSet::All)
    }

    /// Partition a release-file list into (allowed_wheels, allowed_sdists)
    /// for `pkg`. Files whose kind is excluded by the policy are dropped
    /// entirely. Files that are neither wheel nor sdist (e.g. legacy
    /// `.egg`) are also dropped — uv doesn't install them either.
    ///
    /// `pkg` should already be PEP 503-normalized to match the policy's
    /// internal key.
    ///
    /// Tick 142 integration point: the installer / resolver pass their
    /// raw `ReleaseFile` list through this method before handing
    /// candidate filenames to `wheel_picker::pick_best_wheel`.
    pub fn partition<'a>(
        &self,
        pkg: &str,
        files: &'a [ReleaseFile],
    ) -> (Vec<&'a ReleaseFile>, Vec<&'a ReleaseFile>) {
        let mut wheels: Vec<&ReleaseFile> = Vec::new();
        let mut sdists: Vec<&ReleaseFile> = Vec::new();
        let allow_wheels = self.allows_wheel(pkg);
        let allow_sdists = self.allows_sdist(pkg);
        for f in files {
            match classify_artifact(&f.filename) {
                ArtifactKind::Wheel if allow_wheels => wheels.push(f),
                ArtifactKind::Sdist if allow_sdists => sdists.push(f),
                _ => {} // excluded by policy or unrecognized kind
            }
        }
        (wheels, sdists)
    }
}

/// What kind of release artifact a filename represents. uv supports
/// wheels (`.whl`) and sdists (`.tar.gz` / `.tar.bz2` / `.zip`); legacy
/// `.egg` / `.exe` / `.msi` are unsupported and treated as Other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Wheel,
    Sdist,
    Other,
}

/// Classify a release filename by suffix. Mirrors uv's
/// `BuiltDist::is_wheel` / `SourceDist::is_archive` rules.
pub fn classify_artifact(filename: &str) -> ArtifactKind {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".whl") {
        ArtifactKind::Wheel
    } else if lower.ends_with(".tar.gz")
        || lower.ends_with(".tar.bz2")
        || lower.ends_with(".tgz")
        || lower.ends_with(".zip")
    {
        ArtifactKind::Sdist
    } else {
        ArtifactKind::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_strategy_allows_everything() {
        let s = SourceStrategy::default();
        assert!(s.allows_wheel("any"));
        assert!(s.allows_sdist("any"));
        assert!(!s.is_contradictory("any"));
        assert!(!s.has_global_restriction());
    }

    #[test]
    fn no_binary_all_excludes_wheels_globally() {
        let s = SourceStrategy {
            no_binary: SourceFilterSet::parse(":all:").unwrap(),
            ..Default::default()
        };
        assert!(!s.allows_wheel("foo"));
        assert!(s.allows_sdist("foo"));
        assert!(s.has_global_restriction());
    }

    #[test]
    fn only_binary_all_excludes_sdists_globally() {
        let s = SourceStrategy {
            only_binary: SourceFilterSet::parse(":all:").unwrap(),
            ..Default::default()
        };
        assert!(s.allows_wheel("foo"));
        assert!(!s.allows_sdist("foo"));
        assert!(s.has_global_restriction());
    }

    #[test]
    fn named_lists_compose_per_package() {
        let s = SourceStrategy {
            no_binary: SourceFilterSet::parse("foo,bar").unwrap(),
            only_binary: SourceFilterSet::parse("baz").unwrap(),
        };
        // foo, bar: sdist-only
        assert!(!s.allows_wheel("foo"));
        assert!(s.allows_sdist("foo"));
        assert!(!s.allows_wheel("bar"));
        // baz: wheel-only
        assert!(s.allows_wheel("baz"));
        assert!(!s.allows_sdist("baz"));
        // unlisted: unrestricted
        assert!(s.allows_wheel("unrelated"));
        assert!(s.allows_sdist("unrelated"));
    }

    #[test]
    fn contradictory_when_same_name_on_both_lists() {
        let s = SourceStrategy {
            no_binary: SourceFilterSet::parse("foo").unwrap(),
            only_binary: SourceFilterSet::parse("foo").unwrap(),
        };
        assert!(s.is_contradictory("foo"));
        assert!(!s.is_contradictory("bar"));
    }

    #[test]
    fn pep503_normalization_applied_to_name_lists() {
        let s = SourceStrategy {
            no_binary: SourceFilterSet::parse("My.Pkg,Other_Name").unwrap(),
            ..Default::default()
        };
        assert!(!s.allows_wheel("my-pkg"));
        assert!(!s.allows_wheel("other-name"));
    }

    #[test]
    fn none_token_resets_filter_to_empty() {
        let s = SourceFilterSet::parse(":none:").unwrap();
        assert_eq!(s, SourceFilterSet::Empty);
    }

    #[test]
    fn all_or_none_mixed_with_names_rejected() {
        assert!(SourceFilterSet::parse("foo,:all:").is_err());
        assert!(SourceFilterSet::parse(":none:,bar").is_err());
    }

    #[test]
    fn whitespace_and_empty_components_dropped() {
        let s = SourceFilterSet::parse("foo, , bar,  ,baz").unwrap();
        match s {
            SourceFilterSet::Named(set) => assert_eq!(set.len(), 3),
            _ => panic!("expected Named"),
        }
    }

    #[test]
    fn empty_input_yields_empty_filter() {
        assert_eq!(SourceFilterSet::parse("").unwrap(), SourceFilterSet::Empty);
        assert_eq!(
            SourceFilterSet::parse("   ").unwrap(),
            SourceFilterSet::Empty
        );
    }

    #[test]
    fn realistic_uv_invocation_with_pytorch_no_binary() {
        // Common real-world pattern: torch's wheels are huge so users
        // sometimes want sdist for everything else but wheel-only for
        // torch (it has no usable sdist).
        let s = SourceStrategy {
            no_binary: SourceFilterSet::parse(":all:").unwrap(),
            only_binary: SourceFilterSet::parse("torch").unwrap(),
        };
        // torch: only wheels — and the no_binary :all: AND only_binary
        // both apply, which is contradictory. Verify we detect.
        assert!(s.is_contradictory("torch"));
        // Anything else: sdist-only (from :all:).
        assert!(!s.allows_wheel("numpy"));
        assert!(s.allows_sdist("numpy"));
    }

    // ----- Tick 142: partition / classify_artifact integration -----

    fn rf(name: &str) -> ReleaseFile {
        use crate::pkgmanage::pkgmgr::types::FileHash;
        ReleaseFile {
            filename: name.into(),
            url: format!("https://example.invalid/{name}"),
            hash: FileHash {
                algorithm: "sha256".into(),
                digest: "0".repeat(64),
            },
            requires_python: None,
            size: None,
            upload_time: None,
            yanked: false,
            yanked_reason: None,
            dist_info_metadata: serde_json::Value::Null,
            source: None,
        }
    }

    #[test]
    fn classify_wheel_sdist_and_other() {
        assert_eq!(
            classify_artifact("numpy-1.26.0-cp312-cp312-linux_x86_64.whl"),
            ArtifactKind::Wheel
        );
        assert_eq!(
            classify_artifact("numpy-1.26.0.tar.gz"),
            ArtifactKind::Sdist
        );
        assert_eq!(classify_artifact("numpy-1.26.0.zip"), ArtifactKind::Sdist);
        assert_eq!(
            classify_artifact("setuptools-0.6c11-py2.4.egg"),
            ArtifactKind::Other
        );
        assert_eq!(
            classify_artifact("pyobjc-installer-1.5.exe"),
            ArtifactKind::Other
        );
        // Case insensitive (uppercase suffixes happen on Windows mirrors).
        assert_eq!(
            classify_artifact("FOO-1.0.TAR.GZ"),
            ArtifactKind::Sdist
        );
    }

    #[test]
    fn partition_default_strategy_keeps_both_kinds() {
        let s = SourceStrategy::default();
        let files = vec![
            rf("foo-1.0-py3-none-any.whl"),
            rf("foo-1.0.tar.gz"),
            rf("foo-1.0.zip"),
            rf("foo-1.0.weird"),
        ];
        let (wheels, sdists) = s.partition("foo", &files);
        assert_eq!(wheels.len(), 1);
        assert_eq!(sdists.len(), 2);
    }

    #[test]
    fn partition_no_binary_all_keeps_only_sdists() {
        let s = SourceStrategy {
            no_binary: SourceFilterSet::parse(":all:").unwrap(),
            ..Default::default()
        };
        let files = vec![
            rf("numpy-1.26.0-cp312-cp312-linux_x86_64.whl"),
            rf("numpy-1.26.0.tar.gz"),
        ];
        let (wheels, sdists) = s.partition("numpy", &files);
        assert!(wheels.is_empty());
        assert_eq!(sdists.len(), 1);
    }

    #[test]
    fn partition_only_binary_named_keeps_only_wheels_for_that_name() {
        let s = SourceStrategy {
            only_binary: SourceFilterSet::parse("torch").unwrap(),
            ..Default::default()
        };
        let files_torch = vec![
            rf("torch-2.1.0-cp312-cp312-linux_x86_64.whl"),
            rf("torch-2.1.0.tar.gz"),
        ];
        let (w, s_torch) = s.partition("torch", &files_torch);
        assert_eq!(w.len(), 1, "torch wheel kept");
        assert!(s_torch.is_empty(), "torch sdist excluded");

        // numpy is unconstrained: both kinds survive.
        let files_numpy = vec![rf("numpy-1.26.0.whl"), rf("numpy-1.26.0.tar.gz")];
        let (w, s_numpy) = s.partition("numpy", &files_numpy);
        assert_eq!(w.len(), 1);
        assert_eq!(s_numpy.len(), 1);
    }

    #[test]
    fn partition_contradictory_strategy_yields_empty_for_target() {
        // :all: no-binary + only_binary=torch ⇒ torch is excluded both
        // ways; the partition is empty so the caller can detect the
        // contradiction at install time.
        let s = SourceStrategy {
            no_binary: SourceFilterSet::parse(":all:").unwrap(),
            only_binary: SourceFilterSet::parse("torch").unwrap(),
        };
        assert!(s.is_contradictory("torch"));
        let files = vec![
            rf("torch-2.1.0-cp312-cp312-linux_x86_64.whl"),
            rf("torch-2.1.0.tar.gz"),
        ];
        let (w, sd) = s.partition("torch", &files);
        assert!(w.is_empty() && sd.is_empty());
    }

    #[test]
    fn partition_uses_pep503_normalized_lookup() {
        // Strategy stores `my-pkg`; caller passes `my-pkg` (already
        // normalized). Verify that the dot-normalised input the
        // strategy was *built* from still matches.
        let s = SourceStrategy {
            no_binary: SourceFilterSet::parse("My.Pkg").unwrap(),
            ..Default::default()
        };
        let files = vec![rf("my_pkg-1.0-py3-none-any.whl"), rf("my_pkg-1.0.tar.gz")];
        let (w, sd) = s.partition("my-pkg", &files);
        assert!(w.is_empty(), "no_binary blocks the wheel");
        assert_eq!(sd.len(), 1, "sdist survives");
    }
}
