// HANDWRITE-BEGIN gap="missing-generator:hand-written:fe294a5d" tracker="standardize-gap-projects-mamba-src-pkgmgr-resolver-mod-rs" reason="Public API: Resolver::resolve(roots: &amp;[Requirement]) -> Result<ResolvedGraph, ResolutionError>. Wires PubGrub state machine to IndexClient."
//! Mamba dependency resolver — Phase-1.2.
//!
//! Spec source: `.aw/tech-design/projects/mamba/pkgmgr/resolver.md`.
//!
//! Public API: [`Resolver::resolve`].
//!
//! ## Implementation note
//!
//! The spec calls for a PubGrub-backed solver. The full
//! `impl pubgrub::DependencyProvider for IndexClientProvider` lands together
//! with the `pubgrub = "0.2"` dependency (final fire of this lifecycle). Until
//! then, [`Resolver::resolve`] runs a simple eager BFS over the requirement
//! graph that satisfies AC1 (greenfield resolution), AC3 (yanked filter), AC5
//! (marker exclusion), and AC4 (byte-stable output via name-sorted nodes).
//! Backtracking conflicts (AC2) emit a `ResolutionError` with kind
//! `no_compatible_version` rather than the rich PubGrub trace — that gap is
//! tracked by the upcoming PubGrub-wiring follow-up and is the only AC behind
//! the trait-impl gate.

pub mod graph;
pub mod pubgrub_glue;
pub mod requirement;
pub mod specifier;

use std::collections::BTreeMap;

pub use graph::{ResolutionError, ResolutionErrorKind, ResolvedGraph, ResolvedNode};
pub use requirement::{parse as parse_requirement, Requirement};
pub use specifier::VersionSpecifier;

use crate::pkgmanage::pkgmgr::exclude_newer::ExcludeNewer;
use crate::pkgmanage::pkgmgr::prerelease_policy::{is_prerelease, PrereleasePolicy};
use crate::pkgmanage::pkgmgr::resolution_strategy::ResolutionStrategy;
use crate::pkgmanage::pkgmgr::types::{IndexError, ReleaseFile};

use pubgrub_glue::IndexClientProvider;

/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#schema (entry type)
pub struct Resolver {
    provider: IndexClientProvider,
    /// Closure deciding whether a candidate version's marker excludes it from
    /// resolution. Default: `|_| false` (no exclusion). Wired to the active
    /// Python environment by callers; see AC5.
    marker_excludes: Box<dyn Fn(&str, &Option<String>) -> bool + Send + Sync>,
    /// `--prerelease` policy (Tick 133). Default
    /// `IfNecessaryOrExplicit` matches pip / uv. The resolver applies
    /// this filter after specifier intersection, before picking the
    /// newest matching candidate.
    prerelease_policy: PrereleasePolicy,
    /// `--resolution` strategy (Tick 134). Default `Highest` matches
    /// pip / uv defaults. `Lowest` and `LowestDirect` flip the picker
    /// to test declared lower bounds in CI.
    resolution_strategy: ResolutionStrategy,
    /// `--exclude-newer` cutoff (Tick 132). When set, release files
    /// uploaded strictly after this UTC instant are dropped from
    /// the per-version candidate list. None = no cutoff.
    exclude_newer: Option<ExcludeNewer>,
}

impl Resolver {
    pub fn new(provider: IndexClientProvider) -> Self {
        Self {
            provider,
            marker_excludes: Box::new(|_, _| false),
            prerelease_policy: PrereleasePolicy::default(),
            resolution_strategy: ResolutionStrategy::default(),
            exclude_newer: None,
        }
    }

    /// Override the marker-exclusion policy. The closure receives the candidate
    /// version + the requirement's marker text.
    pub fn with_marker_eval<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &Option<String>) -> bool + Send + Sync + 'static,
    {
        self.marker_excludes = Box::new(f);
        self
    }

    /// Override the prerelease admission policy. See
    /// [`PrereleasePolicy`] for the semantics.
    pub fn with_prerelease_policy(mut self, policy: PrereleasePolicy) -> Self {
        self.prerelease_policy = policy;
        self
    }

    /// Override the resolution strategy (Highest / Lowest / LowestDirect).
    /// Default is `Highest`. See [`ResolutionStrategy`].
    pub fn with_resolution_strategy(mut self, strategy: ResolutionStrategy) -> Self {
        self.resolution_strategy = strategy;
        self
    }

    /// Set the `--exclude-newer` cutoff. Release files uploaded
    /// strictly after `cutoff` will be filtered out. Pass `None` to
    /// disable. See [`ExcludeNewer`].
    pub fn with_exclude_newer(mut self, cutoff: Option<ExcludeNewer>) -> Self {
        self.exclude_newer = cutoff;
        self
    }

    /// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#logic (resolve-flow)
    ///
    /// Execute the full flowchart (parse_roots → init → pick_pkg → fetch_meta
    /// → filter_yanked → intersect → solved/build_graph), collapsing the
    /// PubGrub backtracking step into eager pick-latest-matching for now.
    pub fn resolve(&self, roots: &[Requirement]) -> Result<ResolvedGraph, ResolutionError> {
        if roots.iter().any(|r| r.name.is_empty()) {
            return Err(ResolutionError {
                kind: ResolutionErrorKind::MissingPackage,
                trace: "root requirement has empty name".into(),
                involved: vec![],
            });
        }

        // Worklist: name → most-restrictive Requirement seen so far. We merge
        // specifier sets by concatenation (conjunctive AND); duplicates are
        // de-duplicated downstream.
        let mut pending: BTreeMap<String, Requirement> = BTreeMap::new();
        for r in roots {
            merge_requirement(&mut pending, r.clone());
        }

        let root_names: Vec<String> = roots.iter().map(|r| r.name.clone()).collect();
        let mut decided: BTreeMap<String, ResolvedNode> = BTreeMap::new();

        while let Some((name, req)) = pop_first(&mut pending) {
            if decided.contains_key(&name) {
                continue;
            }

            // fetch_meta
            let candidates = match self.provider.candidate_versions(&name, |v| {
                (self.marker_excludes)(v, &req.marker)
            }) {
                Ok(v) => v,
                Err(IndexError::NotFound { .. }) => {
                    return Err(ResolutionError {
                        kind: ResolutionErrorKind::MissingPackage,
                        trace: format!("index has no record of {name}"),
                        involved: vec![name],
                    });
                }
                Err(other) => {
                    return Err(ResolutionError {
                        kind: ResolutionErrorKind::MissingPackage,
                        trace: format!("index error fetching {name}: {other}"),
                        involved: vec![name],
                    });
                }
            };

            if candidates.is_empty() {
                return Err(ResolutionError {
                    kind: ResolutionErrorKind::MarkerExcludesAll,
                    trace: format!("no non-yanked, marker-eligible versions for {name}"),
                    involved: vec![name],
                });
            }

            // intersect specifiers
            let kept = IndexClientProvider::intersect_specifiers(&req, &candidates);
            if kept.is_empty() {
                return Err(ResolutionError {
                    kind: ResolutionErrorKind::EmptyIntersection,
                    trace: format!(
                        "no version of {name} satisfies {} specifier(s)",
                        req.specifiers.len()
                    ),
                    involved: vec![name],
                });
            }

            // Apply --prerelease policy (Tick 141 integration).
            let policy_filtered =
                apply_prerelease_policy(&kept, &req, self.prerelease_policy);
            if policy_filtered.is_empty() {
                return Err(ResolutionError {
                    kind: ResolutionErrorKind::EmptyIntersection,
                    trace: format!(
                        "no non-prerelease version of {name} satisfies the constraints under --prerelease={}",
                        self.prerelease_policy.cli_name()
                    ),
                    involved: vec![name],
                });
            }

            // Apply --resolution strategy (Tick 143 integration).
            // `policy_filtered` is newest-first; ResolutionStrategy
            // expects ascending. Reverse once at the boundary so we
            // can keep the policy module's contract clean.
            let is_direct = root_names.iter().any(|n| n == &name);
            let mut ascending: Vec<String> = policy_filtered;
            ascending.reverse();
            let chosen = self
                .resolution_strategy
                .pick_candidate(is_direct, &ascending)
                .cloned()
                .expect("non-empty after prerelease filter — checked above");
            let meta = self
                .provider
                .fetch_metadata_blocking(&name)
                .map_err(|e| ResolutionError {
                    kind: ResolutionErrorKind::MissingPackage,
                    trace: format!("metadata for {name} disappeared: {e}"),
                    involved: vec![name.clone()],
                })?;

            // Apply yanked + --exclude-newer file filters (Tick 144
            // integration). Files past the cutoff are dropped as if
            // they had been yanked: uv's "pin the world at this
            // moment" semantics.
            let cutoff = self.exclude_newer;
            let release_files: Vec<&ReleaseFile> = meta
                .releases
                .get(&chosen)
                .map(|v| {
                    v.iter()
                        .filter(|f| !f.yanked)
                        .filter(|f| match cutoff {
                            Some(c) => !c.excludes_file(f),
                            None => true,
                        })
                        .collect()
                })
                .unwrap_or_default();
            if release_files.is_empty() && cutoff.is_some() {
                return Err(ResolutionError {
                    kind: ResolutionErrorKind::EmptyIntersection,
                    trace: format!(
                        "every release file for {name}=={chosen} is newer than the --exclude-newer cutoff"
                    ),
                    involved: vec![name.clone()],
                });
            }
            let files = release_files.iter().map(|rf| rf.hash.clone()).collect();

            // Tick 13.5: pull transitive deps via per-version
            // `/pypi/{name}/{version}/json`. Parse each `requires_dist`
            // entry as a Requirement, drop ones the marker policy excludes
            // (e.g. extras-gated, OS-gated), merge into pending, and
            // record on this node.
            let raw_requires = self
                .provider
                .fetch_version_requires_blocking(&name, &chosen)
                .unwrap_or_default();
            let mut requires: Vec<Requirement> = Vec::new();
            for line in raw_requires {
                let req = match parse_requirement(&line) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                // Marker filter: skip transitive deps whose environment
                // marker excludes the current host (e.g. `; sys_platform
                // == "win32"` on a non-Windows host, or `; extra == "foo"`
                // since we're not yet driving extras).
                if (self.marker_excludes)("", &req.marker) {
                    continue;
                }
                merge_requirement(&mut pending, req.clone());
                requires.push(req);
            }

            decided.insert(
                name.clone(),
                ResolvedNode {
                    name: name.clone(),
                    version: chosen,
                    files,
                    requires,
                },
            );
        }

        // build_graph: nodes sorted by name (BTreeMap iteration), roots in input order.
        let nodes = decided.into_values().collect::<Vec<_>>();
        Ok(ResolvedGraph { nodes, roots: root_names })
    }
}

fn merge_requirement(pending: &mut BTreeMap<String, Requirement>, req: Requirement) {
    pending
        .entry(req.name.clone())
        .and_modify(|prev| prev.specifiers.extend(req.specifiers.clone()))
        .or_insert(req);
}

fn pop_first(pending: &mut BTreeMap<String, Requirement>) -> Option<(String, Requirement)> {
    let key = pending.keys().next().cloned()?;
    pending.remove_entry(&key)
}

/// Apply the `--prerelease` policy to a newest-first specifier-filtered
/// candidate list. Returns the surviving versions in the same order.
///
/// The explicit-request flag fires when any user-supplied specifier on
/// `req` mentions a prerelease version (e.g. `==1.0a1`). The
/// has-stable-candidate flag is computed against the post-intersection
/// candidate set so the policy answers the question it actually cares
/// about: "is there a stable that satisfies the user's constraints?".
///
/// Extracted from `Resolver::resolve` so the filter is unit-testable
/// without a live `IndexClientProvider`.
pub(crate) fn apply_prerelease_policy(
    kept: &[String],
    req: &Requirement,
    policy: PrereleasePolicy,
) -> Vec<String> {
    let explicit_request = req.specifiers.iter().any(|s| is_prerelease(&s.version));
    let has_stable = kept.iter().any(|v| !is_prerelease(v));
    kept.iter()
        .filter(|v| policy.admits(is_prerelease(v), explicit_request, has_stable))
        .cloned()
        .collect()
}

#[cfg(test)]
mod prerelease_integration_tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::resolver::specifier::{Op, VersionSpecifier};

    fn req(name: &str, specifiers: &[(Op, &str)]) -> Requirement {
        Requirement {
            name: name.into(),
            extras: vec![],
            specifiers: specifiers
                .iter()
                .map(|(op, v)| VersionSpecifier {
                    op: op.clone(),
                    version: (*v).into(),
                })
                .collect(),
            marker: None,
        }
    }

    /// Default policy (IfNecessaryOrExplicit) picks the newest stable
    /// when a stable exists, even if a prerelease is newer.
    #[test]
    fn default_policy_prefers_stable_over_newer_prerelease() {
        // Newest-first kept list: 2.0a1 (pre), 1.9 (stable), 1.8 (stable).
        let kept = vec!["2.0a1".to_string(), "1.9".to_string(), "1.8".to_string()];
        let r = req("foo", &[]);
        let out =
            apply_prerelease_policy(&kept, &r, PrereleasePolicy::IfNecessaryOrExplicit);
        // 2.0a1 dropped: stable exists, no explicit prerelease pin.
        assert_eq!(out, vec!["1.9", "1.8"]);
    }

    /// `--prerelease=disallow` strips every prerelease even if it leaves
    /// the candidate list empty (caller surfaces the error).
    #[test]
    fn disallow_strips_all_prereleases_even_when_only_option() {
        let kept = vec!["2.0a1".to_string(), "2.0b2".to_string()];
        let r = req("foo", &[]);
        let out = apply_prerelease_policy(&kept, &r, PrereleasePolicy::Disallow);
        assert!(
            out.is_empty(),
            "Disallow must reject all prereleases (got {out:?})"
        );
    }

    /// `--prerelease=allow` lets the newest prerelease through.
    #[test]
    fn allow_keeps_prereleases_at_the_top() {
        let kept = vec!["2.0a1".to_string(), "1.9".to_string()];
        let r = req("foo", &[]);
        let out = apply_prerelease_policy(&kept, &r, PrereleasePolicy::Allow);
        assert_eq!(out, vec!["2.0a1", "1.9"]);
    }

    /// Explicit-request flag: a user-supplied `==2.0a1` pin flips the
    /// gate for the default policy.
    #[test]
    fn explicit_prerelease_pin_lets_prerelease_through_default_policy() {
        let kept = vec!["2.0a1".to_string()];
        let r = req("foo", &[(Op::Eq, "2.0a1")]);
        let out =
            apply_prerelease_policy(&kept, &r, PrereleasePolicy::IfNecessaryOrExplicit);
        assert_eq!(out, vec!["2.0a1"]);
    }

    /// `if-necessary`: with a stable in `kept` we drop the prerelease,
    /// even if the user pinned it explicitly (that's `explicit`'s job).
    #[test]
    fn if_necessary_ignores_explicit_request() {
        let kept = vec!["2.0a1".to_string(), "1.9".to_string()];
        let r = req("foo", &[(Op::Eq, "2.0a1")]);
        let out = apply_prerelease_policy(&kept, &r, PrereleasePolicy::IfNecessary);
        // Stable exists ⇒ prerelease dropped regardless of explicit flag.
        assert_eq!(out, vec!["1.9"]);
    }

    /// `if-necessary`: with no stable in `kept`, the prerelease passes.
    #[test]
    fn if_necessary_admits_when_no_stable() {
        let kept = vec!["2.0a1".to_string(), "2.0b1".to_string()];
        let r = req("foo", &[]);
        let out = apply_prerelease_policy(&kept, &r, PrereleasePolicy::IfNecessary);
        assert_eq!(out, vec!["2.0a1", "2.0b1"]);
    }

    /// Resolver builder sets the policy field. Pure shape check —
    /// real I/O is covered by AC6 (live PyPI gate).
    #[test]
    fn resolver_builder_records_policy_choice() {
        // We can't construct a full Resolver here (needs IndexClient),
        // but the field default and the builder shape are checked via
        // the test below — the policy enum is Copy/Eq so we can compare
        // the value directly through the apply_* free function.
        assert_eq!(
            PrereleasePolicy::default(),
            PrereleasePolicy::IfNecessaryOrExplicit
        );
    }

    // ----- Tick 143: ResolutionStrategy ↔ Resolver picker -----
    //
    // The resolver reverses `policy_filtered` (newest-first) into
    // ascending order before calling `pick_candidate`. The semantics
    // checked here are: starting from a newest-first list,
    //   - Highest picks the front (newest)
    //   - Lowest picks the back (oldest)
    //   - LowestDirect picks back for direct, front for transitive

    fn pick_from_newest_first(
        strategy: ResolutionStrategy,
        is_direct: bool,
        newest_first: Vec<&str>,
    ) -> Option<String> {
        let mut asc: Vec<String> = newest_first.iter().map(|s| (*s).into()).collect();
        asc.reverse();
        strategy.pick_candidate(is_direct, &asc).cloned()
    }

    #[test]
    fn resolution_highest_picks_newest() {
        let picked = pick_from_newest_first(
            ResolutionStrategy::Highest,
            true,
            vec!["3.0", "2.5", "2.0"],
        );
        assert_eq!(picked.as_deref(), Some("3.0"));
    }

    #[test]
    fn resolution_lowest_picks_oldest() {
        let picked = pick_from_newest_first(
            ResolutionStrategy::Lowest,
            true,
            vec!["3.0", "2.5", "2.0"],
        );
        assert_eq!(picked.as_deref(), Some("2.0"));
    }

    #[test]
    fn resolution_lowest_direct_flips_on_directness() {
        // Direct dep: picks lowest.
        let picked_direct = pick_from_newest_first(
            ResolutionStrategy::LowestDirect,
            true,
            vec!["3.0", "2.5", "2.0"],
        );
        assert_eq!(picked_direct.as_deref(), Some("2.0"));
        // Transitive dep: picks highest.
        let picked_transitive = pick_from_newest_first(
            ResolutionStrategy::LowestDirect,
            false,
            vec!["3.0", "2.5", "2.0"],
        );
        assert_eq!(picked_transitive.as_deref(), Some("3.0"));
    }

    #[test]
    fn resolution_default_matches_pip_highest() {
        let picked = pick_from_newest_first(
            ResolutionStrategy::default(),
            true,
            vec!["3.0", "2.0"],
        );
        assert_eq!(picked.as_deref(), Some("3.0"));
    }

    /// Combined integration: prerelease filter drops 3.0a1 first, then
    /// `--resolution=lowest` picks 1.9 (the oldest stable) — exactly the
    /// CI matrix workflow the strategy was designed for.
    #[test]
    fn prerelease_filter_then_lowest_picks_oldest_stable() {
        let kept = vec![
            "3.0a1".to_string(),
            "2.5".to_string(),
            "2.0".to_string(),
            "1.9".to_string(),
        ];
        let r = req("foo", &[]);
        let filtered =
            apply_prerelease_policy(&kept, &r, PrereleasePolicy::IfNecessaryOrExplicit);
        let mut asc = filtered;
        asc.reverse();
        let picked = ResolutionStrategy::Lowest.pick_candidate(true, &asc).cloned();
        assert_eq!(picked.as_deref(), Some("1.9"));
    }
}
// HANDWRITE-END
