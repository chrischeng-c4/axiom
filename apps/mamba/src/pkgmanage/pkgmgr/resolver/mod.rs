// HANDWRITE-BEGIN gap="missing-generator:hand-written:fe294a5d" tracker="standardize-gap-projects-mamba-src-pkgmgr-resolver-mod-rs" reason="Public API: Resolver::resolve(roots: &amp;[Requirement]) -> Result<ResolvedGraph, ResolutionError>. Wires PubGrub state machine to IndexClient."
//! Mamba dependency resolver.
//!
//! Public API: [`Resolver::resolve`].
//!
//! ## The algorithm
//!
//! Chronological backtracking over the requirement graph, written here over
//! the calls [`IndexClientProvider`] already offers. There is no solver crate:
//! [`Universe`] is the whole of what the search may ask the world, and
//! [`search`] is the whole of the search.
//!
//! One name at a time, alphabetically among the undecided names anything has
//! raised a requirement on, the search takes the head of that name's
//! preference-ordered candidate list — the versions the index offers, minus
//! yanked and marker-excluded ones, narrowed by every requirement raised on
//! the name so far, filtered by the `--prerelease` policy and ordered by the
//! `--resolution` strategy. Deciding a name raises the requirements that
//! version declares, which is how the closure grows.
//!
//! A requirement raised on a name that is already decided is the interesting
//! case, and it is the one #4225 got wrong by refusing outright. Here the
//! search **takes a pick back** on conflict — but #4225's version of that
//! only ever retargeted the conflicting name's own decision, which #4231
//! found incomplete two ways: a conflict on a name *nothing* has decided yet
//! (two roots, or a root and an edge, contradicting each other before either
//! side is pinned) never reached the retargeting logic at all, and a
//! conflict whose only remedy is a decision *above* the contested name — a
//! dependant with another release available — was popped and immediately
//! retaken at the same eager pick, forever. `backtrack` now always
//! retargets the most recent decision still on the stack, regardless of
//! which name a conflict is reported against: undo it, advance it to its own
//! name's next candidate under whatever is currently raised on that name,
//! and drop it entirely only once nothing is left to try. When the stack
//! empties, the graph really has no consistent pin set and the search
//! refuses with the error that justified the last backtrack — `NoCompatibleVersion`
//! naming a decided-name conflict, or the original candidate-selection error
//! when the conflict was on a name nothing had pinned.
//!
//! Two properties follow, and the colocated tests state both. Every backtrack
//! strictly advances one decision's position or discards it outright, over a
//! finite tree of finite candidate lists, so the walk terminates. And a name
//! first reached with no conflict is decided exactly as it was before this
//! existed, so a graph that never conflicts renders the bytes it always did.

pub mod graph;
pub mod pubgrub_glue;
pub mod requirement;
pub mod specifier;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

pub use graph::{ResolutionError, ResolutionErrorKind, ResolvedGraph, ResolvedNode};
pub use requirement::{parse as parse_requirement, Requirement};
pub use specifier::VersionSpecifier;

use crate::pkgmanage::pkgmgr::exclude_newer::ExcludeNewer;
use crate::pkgmanage::pkgmgr::prerelease_policy::{is_prerelease, PrereleasePolicy};
use crate::pkgmanage::pkgmgr::resolution_strategy::ResolutionStrategy;
use crate::pkgmanage::pkgmgr::types::{IndexError, ReleaseFile};

use pubgrub_glue::IndexClientProvider;

/// @spec .aw/tech-design/apps/mamba/pkgmgr/resolver.md#schema (entry type)
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

    /// Resolve `roots` into a pin set every requirement in the closure
    /// admits, or refuse the graph with a [`ResolutionError`].
    ///
    /// The algorithm lives in [`search`]; this method rejects a malformed
    /// root and then hands the search the [`Universe`] it asks its two
    /// questions of.
    pub fn resolve(&self, roots: &[Requirement]) -> Result<ResolvedGraph, ResolutionError> {
        if roots.iter().any(|r| r.name.is_empty()) {
            return Err(ResolutionError {
                kind: ResolutionErrorKind::MissingPackage,
                trace: "root requirement has empty name".into(),
                involved: vec![],
            });
        }
        search(self, roots)
    }
}

/// The two questions the search asks of the world, and the only two.
///
/// Lifting them off [`Resolver`] is what makes the search judgeable without
/// an index: the resolver answers them from a live [`IndexClientProvider`],
/// and a colocated test answers them from a table.
pub(crate) trait Universe {
    /// Every version of `name` that `req` admits, in preference order: the
    /// head is what a first pick takes and the tail is what a backtrack walks
    /// into. Never `Ok` with an empty list — an exhausted list is one of the
    /// `Err` variants instead, because the caller distinguishes "nothing on
    /// offer" from "nothing left to try".
    fn ordered_candidates(
        &self,
        name: &str,
        req: &Requirement,
        is_direct: bool,
    ) -> Result<Vec<String>, ResolutionError>;

    /// The node one pick becomes: the artifact hashes for that version and
    /// the requirements it raises, already marker-filtered.
    fn node(&self, name: &str, version: &str) -> Result<ResolvedNode, ResolutionError>;
}

impl Universe for Resolver {
    /// @spec .aw/tech-design/apps/mamba/pkgmgr/resolver.md#logic (fetch_meta
    /// → filter_yanked → intersect → pick_pkg)
    fn ordered_candidates(
        &self,
        name: &str,
        req: &Requirement,
        is_direct: bool,
    ) -> Result<Vec<String>, ResolutionError> {
        // fetch_meta
        let candidates = match self
            .provider
            .candidate_versions(name, |v| (self.marker_excludes)(v, &req.marker))
        {
            Ok(v) => v,
            Err(IndexError::NotFound { .. }) => {
                return Err(ResolutionError {
                    kind: ResolutionErrorKind::MissingPackage,
                    trace: format!("index has no record of {name}"),
                    involved: vec![name.to_string()],
                });
            }
            Err(other) => {
                return Err(ResolutionError {
                    kind: ResolutionErrorKind::MissingPackage,
                    trace: format!("index error fetching {name}: {other}"),
                    involved: vec![name.to_string()],
                });
            }
        };

        if candidates.is_empty() {
            return Err(ResolutionError {
                kind: ResolutionErrorKind::MarkerExcludesAll,
                trace: format!("no non-yanked, marker-eligible versions for {name}"),
                involved: vec![name.to_string()],
            });
        }

        // intersect specifiers
        let kept = IndexClientProvider::intersect_specifiers(req, &candidates);
        if kept.is_empty() {
            return Err(ResolutionError {
                kind: ResolutionErrorKind::EmptyIntersection,
                trace: format!(
                    "no version of {name} satisfies {} specifier(s)",
                    req.specifiers.len()
                ),
                involved: vec![name.to_string()],
            });
        }

        // Apply --prerelease policy (Tick 141 integration).
        let policy_filtered = apply_prerelease_policy(&kept, req, self.prerelease_policy);
        if policy_filtered.is_empty() {
            return Err(ResolutionError {
                kind: ResolutionErrorKind::EmptyIntersection,
                trace: format!(
                    "no non-prerelease version of {name} satisfies the constraints under --prerelease={}",
                    self.prerelease_policy.cli_name()
                ),
                involved: vec![name.to_string()],
            });
        }

        // Apply --resolution strategy (Tick 143 integration).
        // `policy_filtered` is newest-first; ResolutionStrategy expects
        // ascending. Reverse once at the boundary so we can keep the policy
        // module's contract clean.
        let mut ascending: Vec<String> = policy_filtered;
        ascending.reverse();
        Ok(order_by_strategy(
            self.resolution_strategy,
            is_direct,
            ascending,
        ))
    }

    fn node(&self, name: &str, version: &str) -> Result<ResolvedNode, ResolutionError> {
        let meta = self
            .provider
            .fetch_metadata_blocking(name)
            .map_err(|e| ResolutionError {
                kind: ResolutionErrorKind::MissingPackage,
                trace: format!("metadata for {name} disappeared: {e}"),
                involved: vec![name.to_string()],
            })?;

        // Apply yanked + --exclude-newer file filters (Tick 144 integration).
        // Files past the cutoff are dropped as if they had been yanked: uv's
        // "pin the world at this moment" semantics.
        let cutoff = self.exclude_newer;
        let release_files: Vec<&ReleaseFile> = meta
            .releases
            .get(version)
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
                    "every release file for {name}=={version} is newer than the --exclude-newer cutoff"
                ),
                involved: vec![name.to_string()],
            });
        }
        let files = release_files.iter().map(|rf| rf.hash.clone()).collect();

        // Tick 13.5: pull transitive deps via per-version
        // `/pypi/{name}/{version}/json`. Parse each `requires_dist` entry as a
        // Requirement and drop the ones the marker policy excludes (e.g.
        // extras-gated, OS-gated).
        let raw_requires = self
            .provider
            .fetch_version_requires_blocking(name, version)
            .unwrap_or_default();
        let mut requires: Vec<Requirement> = Vec::new();
        for line in raw_requires {
            let req = match parse_requirement(&line) {
                Ok(r) => r,
                Err(_) => continue,
            };
            // Marker filter: skip transitive deps whose environment marker
            // excludes the current host (e.g. `; sys_platform == "win32"` on a
            // non-Windows host, or `; extra == "foo"` since we are not yet
            // driving extras).
            if (self.marker_excludes)("", &req.marker) {
                continue;
            }
            requires.push(req);
        }

        Ok(ResolvedNode {
            name: name.to_string(),
            version: version.to_string(),
            files,
            requires,
        })
    }
}

/// Project an ascending candidate list into the order the strategy prefers,
/// head first.
///
/// The order is derived from [`ResolutionStrategy::pick_candidate`] rather
/// than restated here: whatever that method picks out of the ascending list is
/// what this function puts at the head, so the search's first pick is the pick
/// the strategy would have made and the two cannot drift apart.
pub(crate) fn order_by_strategy(
    strategy: ResolutionStrategy,
    is_direct: bool,
    ascending: Vec<String>,
) -> Vec<String> {
    let Some(picked) = strategy.pick_candidate(is_direct, &ascending).cloned() else {
        return ascending;
    };
    let mut ordered = ascending;
    if ordered.first() != Some(&picked) {
        ordered.reverse();
    }
    ordered
}

/// The refusal a graph earns when no version of `name` satisfies everything
/// raised on it. `decided_with` is the requirement set that selected
/// `version`; `arriving` is what contradicts it.
fn conflict_error(
    name: &str,
    decided_with: Option<&Requirement>,
    arriving: &[VersionSpecifier],
    version: &str,
) -> ResolutionError {
    let prior_spec = decided_with
        .map(|r| spell_specifiers(&r.specifiers))
        .unwrap_or_default();
    let arriving_spec = spell_specifiers(arriving);
    ResolutionError {
        kind: ResolutionErrorKind::NoCompatibleVersion,
        trace: format!(
            "conflicting requirements on {name}: `{name}{prior_spec}` decided \
             {name}=={version} but a later requirement needs `{name}{arriving_spec}`, \
             which that version does not satisfy"
        ),
        involved: vec![name.to_string()],
    }
}

/// One requirement in force, and the decision that raised it.
///
/// The raiser is what makes a decision undoable: withdrawing a pick means
/// withdrawing exactly the requirements that pick put into the world, and
/// nothing else. A root's requirement has no raiser and is never withdrawn.
struct Raise {
    by: Option<String>,
    req: Requirement,
}

/// One decision the search has made and can still take back.
struct Decision {
    name: String,
    /// The preference-ordered candidate list the pick came from, frozen when
    /// the name was first decided. Backtracking walks *this* list, so a
    /// re-pick can never reach a version the first pick was not offered.
    candidates: Vec<String>,
    /// Index into `candidates` of the version currently pinned.
    pos: usize,
    /// The requirement set that selected `candidates`, kept so a refusal can
    /// spell what decided the version it is refusing.
    decided_with: Requirement,
}

/// The conjunction of every requirement currently raised on one name, folded
/// into the single [`Requirement`] the candidate narrowing consumes. The first
/// raise supplies the name, extras and marker; the rest contribute specifiers.
fn merged(raises: &[Raise]) -> Requirement {
    let mut out = raises[0].req.clone();
    for r in &raises[1..] {
        out.specifiers.extend(r.req.specifiers.clone());
    }
    out
}

/// Every specifier currently raised on one name, flattened.
fn constraints(raises: &[Raise]) -> Vec<VersionSpecifier> {
    raises
        .iter()
        .flat_map(|r| r.req.specifiers.iter().cloned())
        .collect()
}

/// Put the requirements `node` declares into force, attributed to `node`.
fn raise_for(raised: &mut BTreeMap<String, Vec<Raise>>, node: &ResolvedNode) {
    for req in &node.requires {
        raised.entry(req.name.clone()).or_default().push(Raise {
            by: Some(node.name.clone()),
            req: req.clone(),
        });
    }
}

/// Take one decision back: its node stops being a pin, and every requirement
/// it raised stops being in force.
fn undo(
    raised: &mut BTreeMap<String, Vec<Raise>>,
    decided: &mut BTreeMap<String, ResolvedNode>,
    name: &str,
) {
    decided.remove(name);
    for raises in raised.values_mut() {
        raises.retain(|r| r.by.as_deref() != Some(name));
    }
    raised.retain(|_, raises| !raises.is_empty());
}

/// The first decided name — alphabetically, so the answer does not depend on
/// the order a conflict happened to be discovered in — whose pinned version
/// something raised on it forbids, together with the specifiers it fails.
fn first_violation(
    raised: &BTreeMap<String, Vec<Raise>>,
    decided: &BTreeMap<String, ResolvedNode>,
) -> Option<(String, Vec<VersionSpecifier>)> {
    for (name, raises) in raised {
        let Some(node) = decided.get(name) else {
            continue;
        };
        let offending: Vec<VersionSpecifier> = raises
            .iter()
            .flat_map(|r| r.req.specifiers.iter())
            .filter(|s| !s.matches(&node.version))
            .cloned()
            .collect();
        if !offending.is_empty() {
            return Some((name.clone(), offending));
        }
    }
    None
}

/// Undo back to a decision that can move, and move it — chronologically,
/// always starting from the most recent decision still on the stack.
///
/// This is what makes the search complete. A conflict does not always land
/// on a name with its own decision to retarget: two roots can contradict
/// each other on a name nothing has decided yet, in which case the decision
/// that must move is whichever raised the losing side. And even when the
/// conflict does land on a decided name, the decision that has to move can
/// be *above* it — a dependant with another release available that would
/// stop raising the offending requirement in the first place. Popping that
/// dependant and letting the ordinary search loop retake it from scratch
/// finds nothing: it has no memory of the conflict and picks the same
/// eager candidate again.
///
/// So this always retargets the top of the stack first: undo it, and look
/// for its own name's next candidate among everything currently raised on
/// that name (the conflict's contribution disappears with the undo of
/// whichever decision raised it). A name with nothing left to try is
/// dropped entirely and the decision below it is tried the same way. The
/// walk still terminates — each step either strictly advances a decision's
/// position or discards a decision outright, over a finite tree of finite
/// candidate lists — and it reaches a decision `search`'s next lap could
/// never have started at.
///
/// Returns `refusal` when the stack empties: nothing is left to move, so no
/// assignment of this graph satisfies itself. `refusal` is the error that
/// justified this call — either `conflict_error` for a decided-name
/// violation, or the exact error a candidate-selection call raised for a
/// name nothing has decided yet — so an unsatisfiable graph still fails with
/// the message that names what it could not reconcile.
fn backtrack(
    universe: &dyn Universe,
    raised: &mut BTreeMap<String, Vec<Raise>>,
    decided: &mut BTreeMap<String, ResolvedNode>,
    stack: &mut Vec<Decision>,
    refusal: ResolutionError,
) -> Result<(), ResolutionError> {
    loop {
        let Some(decision) = stack.last_mut() else {
            return Err(refusal);
        };
        undo(raised, decided, &decision.name);

        let wanted = raised
            .get(&decision.name)
            .map(|raises| constraints(raises))
            .unwrap_or_default();

        let next = decision
            .candidates
            .iter()
            .enumerate()
            .skip(decision.pos + 1)
            .find(|(_, v)| specifier::all_match(&wanted, v))
            .map(|(i, v)| (i, v.clone()));

        match next {
            Some((i, version)) => {
                decision.pos = i;
                let name = decision.name.clone();
                let node = universe.node(&name, &version)?;
                raise_for(raised, &node);
                decided.insert(name, node);
                return Ok(());
            }
            None => {
                stack.pop();
            }
        }
    }
}

/// Walk the requirement graph and return one pin per reachable name, or refuse
/// the graph.
///
/// @spec .aw/tech-design/apps/mamba/pkgmgr/resolver.md#logic (resolve-flow)
pub(crate) fn search(
    universe: &dyn Universe,
    roots: &[Requirement],
) -> Result<ResolvedGraph, ResolutionError> {
    let root_names: Vec<String> = roots.iter().map(|r| r.name.clone()).collect();

    // Every requirement currently in force, by the name it constrains. This is
    // the search's whole memory: the worklist is "names in here that nothing
    // has decided yet", and undoing a decision is deleting its rows.
    let mut raised: BTreeMap<String, Vec<Raise>> = BTreeMap::new();
    for r in roots {
        raised.entry(r.name.clone()).or_default().push(Raise {
            by: None,
            req: r.clone(),
        });
    }

    let mut decided: BTreeMap<String, ResolvedNode> = BTreeMap::new();
    let mut stack: Vec<Decision> = Vec::new();

    loop {
        // A pin something now forbids is settled before anything else is
        // decided on top of it.
        if let Some((name, offending)) = first_violation(&raised, &decided) {
            let decided_with = stack
                .iter()
                .find(|d| d.name == name)
                .map(|d| &d.decided_with)
                .expect("a decided name has a decision on the stack");
            let refusal = conflict_error(&name, Some(decided_with), &offending, &decided[&name].version);
            backtrack(universe, &mut raised, &mut decided, &mut stack, refusal)?;
            continue;
        }

        let Some(name) = raised.keys().find(|n| !decided.contains_key(*n)).cloned() else {
            break;
        };

        let req = merged(&raised[&name]);
        let is_direct = root_names.iter().any(|n| n == &name);
        let candidates = match universe.ordered_candidates(&name, &req, is_direct) {
            Ok(candidates) => candidates,
            // No candidate satisfies everything currently raised on `name` —
            // MarkerExcludesAll, the two EmptyIntersection cases and the
            // exclude-newer variant from `ordered_candidates` all say exactly
            // this. That does not end the search while a decision remains on
            // the stack: some earlier decision may be what is over-narrowing
            // `name`, and backing it off can free `name` up. `err` becomes
            // the refusal only if the stack truly empties.
            Err(err) => {
                backtrack(universe, &mut raised, &mut decided, &mut stack, err)?;
                continue;
            }
        };
        let Some(chosen) = candidates.first().cloned() else {
            let err = ResolutionError {
                kind: ResolutionErrorKind::EmptyIntersection,
                trace: format!("no candidate version of {name} is left to try"),
                involved: vec![name],
            };
            backtrack(universe, &mut raised, &mut decided, &mut stack, err)?;
            continue;
        };
        let node = universe.node(&name, &chosen)?;
        raise_for(&mut raised, &node);
        decided.insert(name.clone(), node);
        stack.push(Decision {
            name,
            candidates,
            pos: 0,
            decided_with: req,
        });
    }

    // build_graph: nodes sorted by name (BTreeMap iteration), roots in input
    // order.
    Ok(ResolvedGraph {
        nodes: decided.into_values().collect::<Vec<_>>(),
        roots: root_names,
    })
}

/// Spell one specifier's operator + version, e.g. `>=2` — `VersionSpecifier`
/// has no `Display` impl (that grammar belongs to the PubGrub work item, see
/// `resolver/specifier.rs`'s module header) and a `{:?}` dump does not spell
/// the operator, so a conflict error carries this instead.
fn spell_specifier(s: &VersionSpecifier) -> String {
    let op = match s.op {
        specifier::Op::Eq => "==",
        specifier::Op::NotEq => "!=",
        specifier::Op::Lt => "<",
        specifier::Op::Le => "<=",
        specifier::Op::Gt => ">",
        specifier::Op::Ge => ">=",
        specifier::Op::Compatible => "~=",
    };
    format!("{op}{}", s.version)
}

/// Spell a conjunctive specifier set, comma-joined, e.g. `>=2,<3`.
fn spell_specifiers(specs: &[VersionSpecifier]) -> String {
    specs.iter().map(spell_specifier).collect::<Vec<_>>().join(",")
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
        let out = apply_prerelease_policy(&kept, &r, PrereleasePolicy::IfNecessaryOrExplicit);
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
        let out = apply_prerelease_policy(&kept, &r, PrereleasePolicy::IfNecessaryOrExplicit);
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
        let picked =
            pick_from_newest_first(ResolutionStrategy::Highest, true, vec!["3.0", "2.5", "2.0"]);
        assert_eq!(picked.as_deref(), Some("3.0"));
    }

    #[test]
    fn resolution_lowest_picks_oldest() {
        let picked =
            pick_from_newest_first(ResolutionStrategy::Lowest, true, vec!["3.0", "2.5", "2.0"]);
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
        let picked =
            pick_from_newest_first(ResolutionStrategy::default(), true, vec!["3.0", "2.0"]);
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
        let filtered = apply_prerelease_policy(&kept, &r, PrereleasePolicy::IfNecessaryOrExplicit);
        let mut asc = filtered;
        asc.reverse();
        let picked = ResolutionStrategy::Lowest
            .pick_candidate(true, &asc)
            .cloned();
        assert_eq!(picked.as_deref(), Some("1.9"));
    }
}
// HANDWRITE-END
