//! Colocated tests for the resolver's search and for the specifier grammar it
//! reads its bounds through.
//!
//! # Why these are here and not in `e2e/`
//!
//! The e2e case drives the binary and reads `mamba.lock`; it can only see the
//! pins that survived. These tests hold the two rules a lock cannot show:
//! that the search *undoes* a pick rather than refusing a solvable graph, and
//! that the node it leaves behind carries the requirements the version it
//! finally settled on raised — not the ones an abandoned pick raised. Both are
//! stated against an in-memory [`Universe`], so no index, no registry and no
//! network is involved and the graph under test is written out in full.
//!
//! # The invariant
//!
//! [`assert_consistent`] is the whole contract in one sentence: every pinned
//! version satisfies every requirement any root or any other pinned node
//! raises on it. Each search test asserts it alongside the concrete pins, so a
//! search that returns *a* graph rather than *the* graph is caught even where
//! the expected pin list happens to agree.

use std::collections::BTreeMap;

use super::specifier::{all_match, parse_one, parse_set, Op};
use super::{
    order_by_strategy, parse_requirement, search, ResolutionErrorKind, ResolvedGraph, ResolvedNode,
    Universe,
};
use crate::pkgmanage::pkgmgr::resolution_strategy::ResolutionStrategy;
use crate::pkgmanage::pkgmgr::resolver::graph::ResolutionError;
use crate::pkgmanage::pkgmgr::resolver::requirement::Requirement;

// --------------------------------------------------------------------------
// an index written out as a table
// --------------------------------------------------------------------------

/// One index, written out: the versions each name has and the `requires_dist`
/// lines each of those versions declares. Nothing else about a package matters
/// to the search.
struct Table {
    /// name → versions, in ascending PEP 440 order.
    releases: BTreeMap<String, Vec<String>>,
    /// (name, version) → the raw requirement lines that version raises.
    edges: BTreeMap<(String, String), Vec<String>>,
    strategy: ResolutionStrategy,
}

impl Table {
    fn new(releases: &[(&str, &[&str])], edges: &[(&str, &str, &[&str])]) -> Table {
        Table {
            releases: releases
                .iter()
                .map(|(n, vs)| {
                    (
                        (*n).to_string(),
                        vs.iter().map(|v| (*v).to_string()).collect(),
                    )
                })
                .collect(),
            edges: edges
                .iter()
                .map(|(n, v, reqs)| {
                    (
                        ((*n).to_string(), (*v).to_string()),
                        reqs.iter().map(|r| (*r).to_string()).collect(),
                    )
                })
                .collect(),
            strategy: ResolutionStrategy::Highest,
        }
    }
}

impl Universe for Table {
    fn ordered_candidates(
        &self,
        name: &str,
        req: &Requirement,
        is_direct: bool,
    ) -> Result<Vec<String>, ResolutionError> {
        let Some(all) = self.releases.get(name) else {
            return Err(ResolutionError {
                kind: ResolutionErrorKind::MissingPackage,
                trace: format!("index has no record of {name}"),
                involved: vec![name.to_string()],
            });
        };
        let kept: Vec<String> = all
            .iter()
            .filter(|v| all_match(&req.specifiers, v))
            .cloned()
            .collect();
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
        Ok(order_by_strategy(self.strategy, is_direct, kept))
    }

    fn node(&self, name: &str, version: &str) -> Result<ResolvedNode, ResolutionError> {
        let lines = self
            .edges
            .get(&(name.to_string(), version.to_string()))
            .cloned()
            .unwrap_or_default();
        let mut requires = Vec::new();
        for line in lines {
            match parse_requirement(&line) {
                Ok(r) => requires.push(r),
                Err(e) => {
                    return Err(ResolutionError {
                        kind: ResolutionErrorKind::UnparseableRequiresDist,
                        trace: format!(
                            "{name}=={version} declares a requires_dist line that does not parse: {line:?}: {e}"
                        ),
                        involved: vec![name.to_string()],
                    });
                }
            }
        }
        Ok(ResolvedNode {
            name: name.to_string(),
            version: version.to_string(),
            files: vec![],
            requires,
        })
    }
}

fn roots(specs: &[&str]) -> Vec<Requirement> {
    specs
        .iter()
        .map(|s| parse_requirement(s).expect("fixture: root requirement must parse"))
        .collect()
}

/// `name==version` for every node, in the graph's own order.
fn pins(graph: &ResolvedGraph) -> Vec<String> {
    graph
        .nodes
        .iter()
        .map(|n| format!("{}=={}", n.name, n.version))
        .collect()
}

/// The edges one node declares, spelled `name<specifiers>`.
fn edges_of(graph: &ResolvedGraph, name: &str) -> Vec<String> {
    let node = graph
        .nodes
        .iter()
        .find(|n| n.name == name)
        .unwrap_or_else(|| panic!("no node named {name} in {:?}", pins(graph)));
    node.requires
        .iter()
        .map(|r| format!("{}{}", r.name, super::spell_specifiers(&r.specifiers)))
        .collect()
}

/// The rule a returned graph may never break: every pin satisfies every
/// requirement raised on it, by a root or by another pin.
fn assert_consistent(context: &str, graph: &ResolvedGraph, root_reqs: &[Requirement]) {
    let pinned: BTreeMap<&str, &str> = graph
        .nodes
        .iter()
        .map(|n| (n.name.as_str(), n.version.as_str()))
        .collect();
    let raised = root_reqs.iter().map(|r| ("<roots>", r)).chain(
        graph
            .nodes
            .iter()
            .flat_map(|n| n.requires.iter().map(move |r| (n.name.as_str(), r))),
    );
    for (by, req) in raised {
        let Some(version) = pinned.get(req.name.as_str()) else {
            panic!(
                "{context}: `{}` raised `{}` but no node pins it — a requirement \
                 in the closure that reached no decision\npins: {:?}",
                by,
                req.name,
                pins(graph)
            );
        };
        assert!(
            all_match(&req.specifiers, version),
            "{context}: `{by}` requires `{}{}` but the graph pinned {}=={version} — \
             a returned graph must satisfy every requirement in its own closure\n\
             pins: {:?}",
            req.name,
            super::spell_specifiers(&req.specifiers),
            req.name,
            pins(graph)
        );
    }
}

// --------------------------------------------------------------------------
// the search backs off a pick a later requirement contradicts
// --------------------------------------------------------------------------

/// Two roots reach one name. The first admits both releases and takes the
/// newer; the second admits only the older. The graph has one solution, and
/// reaching it means giving up the first pick instead of refusing.
#[test]
fn search_backs_off_a_first_pick_a_later_requirement_forbids() {
    let table = Table::new(
        &[
            ("sharedapp", &["1.0"]),
            ("sharedother", &["1.0"]),
            ("sharedlib", &["1.0", "3.0"]),
        ],
        &[
            ("sharedapp", "1.0", &["sharedlib>=1"]),
            ("sharedother", "1.0", &["sharedlib<2"]),
        ],
    );
    let root_reqs = roots(&["sharedapp", "sharedother"]);
    let graph = search(&table, &root_reqs).expect(
        "the graph has exactly one consistent pin set, so the search must find it \
         rather than refuse",
    );
    assert_eq!(
        pins(&graph),
        vec!["sharedapp==1.0", "sharedlib==1.0", "sharedother==1.0"],
        "`sharedlib<2` admits only 1.0, so the eager 3.0 pick has to be undone"
    );
    assert_consistent("one shared name", &graph, &root_reqs);
}

/// The name the conflict surfaces on has no release satisfying both bounds:
/// the decision that must move is the one below it, and the node left behind
/// must carry the edges its *final* version raised.
#[test]
fn search_moves_a_decision_below_the_name_the_conflict_surfaced_on() {
    let table = Table::new(
        &[
            ("deepapp", &["1.0"]),
            ("deepother", &["1.0"]),
            ("deepmid", &["1.0", "2.0"]),
            ("deeplib", &["1.0", "3.0"]),
        ],
        &[
            ("deepapp", "1.0", &["deepmid>=1"]),
            ("deepmid", "2.0", &["deeplib>=3"]),
            ("deepmid", "1.0", &["deeplib>=1"]),
            ("deepother", "1.0", &["deeplib<2"]),
        ],
    );
    let root_reqs = roots(&["deepapp", "deepother"]);
    let graph = search(&table, &root_reqs).expect(
        "`deepmid==1.0` with `deeplib==1.0` satisfies every requirement, so the \
         search must reach it",
    );
    assert_eq!(
        pins(&graph),
        vec![
            "deepapp==1.0",
            "deeplib==1.0",
            "deepmid==1.0",
            "deepother==1.0"
        ],
        "`deeplib` cannot move on its own — the decision that has to give up its \
         first pick is `deepmid`"
    );
    assert_consistent("intermediate decision", &graph, &root_reqs);
    assert_eq!(
        edges_of(&graph, "deepmid"),
        vec!["deeplib>=1"],
        "`deepmid`'s node must carry the edge `deepmid==1.0` declares; keeping \
         `deeplib>=3` from the abandoned 2.0 pick renders a dependency the lock \
         contradicts"
    );
}

/// Undoing a pick is not the same as tolerating a conflict: a graph with no
/// consistent pin set is still refused, and the refusal still names the
/// package and both bounds.
#[test]
fn search_still_refuses_a_graph_with_no_consistent_pin_set() {
    let table = Table::new(
        &[
            ("app", &["1.0"]),
            ("other", &["1.0"]),
            ("lib", &["1.0", "3.0"]),
        ],
        &[("app", "1.0", &["lib>=2"]), ("other", "1.0", &["lib<2"])],
    );
    let root_reqs = roots(&["app", "other"]);
    let err =
        search(&table, &root_reqs).expect_err("no release of `lib` satisfies both `>=2` and `<2`");
    assert_eq!(err.kind, ResolutionErrorKind::NoCompatibleVersion);
    assert_eq!(err.involved, vec!["lib".to_string()]);
    for token in ["lib", ">=2", "<2"] {
        assert!(
            err.trace.contains(token),
            "the refusal must name `{token}`; got {:?}",
            err.trace
        );
    }
}

/// A graph that never conflicts is decided the way it always was: highest
/// admissible version, first pick, no undo.
#[test]
fn search_leaves_a_conflict_free_graph_on_its_first_picks() {
    let table = Table::new(
        &[("app", &["1.0"]), ("lib", &["1.0", "2.0", "3.0"])],
        &[("app", "1.0", &["lib>=1"])],
    );
    let root_reqs = roots(&["app"]);
    let graph = search(&table, &root_reqs).expect("nothing here conflicts");
    assert_eq!(pins(&graph), vec!["app==1.0", "lib==3.0"]);
    assert_consistent("conflict-free", &graph, &root_reqs);
}

/// The search stops. A chain of names each of which must give up its first
/// pick terminates with the answer rather than cycling between two picks.
#[test]
fn search_terminates_on_a_chain_that_forces_every_decision_down() {
    let table = Table::new(
        &[
            ("chainapp", &["1.0"]),
            ("chainother", &["1.0"]),
            ("chaina", &["1.0", "2.0"]),
            ("chainb", &["1.0", "2.0"]),
            ("chainc", &["1.0", "2.0"]),
        ],
        &[
            ("chainapp", "1.0", &["chaina>=1"]),
            ("chaina", "2.0", &["chainb>=2"]),
            ("chaina", "1.0", &["chainb>=1"]),
            ("chainb", "2.0", &["chainc>=2"]),
            ("chainb", "1.0", &["chainc>=1"]),
            ("chainother", "1.0", &["chainc<2"]),
        ],
    );
    let root_reqs = roots(&["chainapp", "chainother"]);
    let graph = search(&table, &root_reqs)
        .expect("the all-1.0 assignment satisfies every requirement in the chain");
    assert_eq!(
        pins(&graph),
        vec![
            "chaina==1.0",
            "chainapp==1.0",
            "chainb==1.0",
            "chainc==1.0",
            "chainother==1.0"
        ]
    );
    assert_consistent("forced chain", &graph, &root_reqs);
}

// --------------------------------------------------------------------------
// candidate ordering
// --------------------------------------------------------------------------

/// The order the search walks is the strategy's own preference: its head is
/// exactly what `pick_candidate` picks, for every strategy and both
/// directness values, so backtracking never walks a different list than the
/// one the first pick came from.
#[test]
fn order_by_strategy_puts_the_strategys_own_pick_at_the_head() {
    let ascending: Vec<String> = ["1.0", "2.0", "3.0"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for strategy in [
        ResolutionStrategy::Highest,
        ResolutionStrategy::Lowest,
        ResolutionStrategy::LowestDirect,
    ] {
        for is_direct in [true, false] {
            let ordered = order_by_strategy(strategy, is_direct, ascending.clone());
            let picked = strategy
                .pick_candidate(is_direct, &ascending)
                .cloned()
                .unwrap();
            assert_eq!(
                ordered.first(),
                Some(&picked),
                "{strategy:?} (direct={is_direct}) must lead with its own pick"
            );
            let mut sorted = ordered.clone();
            sorted.sort();
            assert_eq!(
                sorted, ascending,
                "{strategy:?} (direct={is_direct}) must keep every candidate"
            );
        }
    }
}

// --------------------------------------------------------------------------
// the `==X.*` / `!=X.*` grammar
// --------------------------------------------------------------------------

/// `==1.*` is a bound, not a parse error: it admits every release whose
/// release segments begin `1`, and excludes the rest.
#[test]
fn wildcard_equality_matches_on_release_segment_prefix() {
    let s = parse_one("==1.*").expect("`==1.*` is a PEP 440 specifier");
    assert_eq!(s.op, Op::Eq);
    assert!(s.matches("1.0"));
    assert!(s.matches("1.5"));
    assert!(s.matches("1.5.7"));
    assert!(!s.matches("2.0"));
    assert!(!s.matches("11.0"), "the prefix is a segment, not a string");
}

/// The wildcard compares release segments, so a shorter candidate is padded
/// with zeros rather than rejected: `==1.0.*` admits `1.0`.
#[test]
fn wildcard_pads_a_shorter_candidate_with_zeros() {
    let s = parse_one("==1.0.*").unwrap();
    assert!(s.matches("1.0"));
    assert!(s.matches("1.0.3"));
    assert!(!s.matches("1.1"));
}

/// The wildcard survives the suffixes a `>=X,<X+1` range would miss:
/// `1.0.post1` and `1.0.dev1` both begin `1`, and both are admitted by `==1.*`.
#[test]
fn wildcard_admits_post_and_dev_releases_of_the_prefix() {
    let s = parse_one("==1.*").unwrap();
    assert!(s.matches("1.0.post1"));
    assert!(s.matches("1.0.dev1"));
    assert!(s.matches("1.0a1"));
    assert!(s.matches("1.0+local"));
}

/// `!=1.*` is the negation of `==1.*`.
#[test]
fn wildcard_inequality_excludes_the_prefix() {
    let s = parse_one("!=1.*").unwrap();
    assert_eq!(s.op, Op::NotEq);
    assert!(!s.matches("1.0"));
    assert!(!s.matches("1.9.9"));
    assert!(s.matches("2.0"));
}

/// PEP 440 allows the wildcard under `==` and `!=` only. Every other operator
/// must still refuse it rather than silently reading it as a version.
#[test]
fn wildcard_is_refused_under_every_other_operator() {
    for spec in [">=1.*", "<=1.*", ">1.*", "<1.*", "~=1.*"] {
        assert!(
            parse_one(spec).is_err(),
            "`{spec}` is not a PEP 440 specifier and must not parse"
        );
    }
}

/// A wildcard inside a set parses with the rest of the set, and the set is
/// still conjunctive.
#[test]
fn wildcard_composes_inside_a_specifier_set() {
    let specs = parse_set("==1.*, !=1.5").unwrap();
    assert_eq!(specs.len(), 2);
    assert!(all_match(&specs, "1.0"));
    assert!(!all_match(&specs, "1.5"));
    assert!(!all_match(&specs, "2.0"));
}

/// A bare `*`, an empty prefix, or a wildcard anywhere but the tail is still
/// garbage.
#[test]
fn wildcard_grammar_rejects_the_shapes_pep440_does_not_define() {
    for spec in ["==*", "==.*", "==1.*.2", "==1.2.*.*"] {
        assert!(
            parse_one(spec).is_err(),
            "`{spec}` must not parse as a specifier"
        );
    }
}

// --------------------------------------------------------------------------
// `requires_dist` lines a version raises must not be silently dropped
// --------------------------------------------------------------------------

/// A node whose `requires_dist` line is the PEP 508 parenthesized form
/// resolves to the same edge the bare form would: the bounds are applied,
/// not just the name.
#[test]
fn parenthesized_requires_dist_line_resolves_to_the_bounded_edge() {
    let table = Table::new(
        &[
            ("legacyapp", &["1.0"]),
            ("legacylib", &["1.0", "2.5", "3.0"]),
        ],
        &[("legacyapp", "1.0", &["legacylib (<3,>=1.21.1)"])],
    );
    let root_reqs = roots(&["legacyapp"]);
    let graph = search(&table, &root_reqs).expect(
        "a parenthesized requires_dist line must parse and resolve, not be dropped",
    );
    assert_eq!(
        pins(&graph),
        vec!["legacyapp==1.0", "legacylib==2.5"],
        "the bound `(<3,>=1.21.1)` admits 1.0 and 2.5 but not 3.0; if the bound were \
         lost the highest release 3.0 would be picked instead"
    );
    assert_consistent("parenthesized requires_dist", &graph, &root_reqs);
}

/// A node whose `requires_dist` line does not parse at all must not vanish
/// from the graph unnoticed: `resolve` refuses, and the refusal names the
/// package, the version and the offending line.
#[test]
fn unparseable_requires_dist_line_refuses_naming_package_version_and_line() {
    let table = Table::new(
        &[("brokenapp", &["1.0"])],
        &[("brokenapp", "1.0", &["this is not a requirement"])],
    );
    let root_reqs = roots(&["brokenapp"]);
    let err = search(&table, &root_reqs).expect_err(
        "a requires_dist line that does not parse must refuse the resolution, not be dropped",
    );
    assert_eq!(err.kind, ResolutionErrorKind::UnparseableRequiresDist);
    for token in ["brokenapp", "1.0", "this is not a requirement"] {
        assert!(
            err.trace.contains(token),
            "the refusal must name `{token}`; got {:?}",
            err.trace
        );
    }
}
