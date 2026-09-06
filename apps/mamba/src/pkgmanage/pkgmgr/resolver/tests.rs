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
// #4231: the search must be complete, not just able to undo a decided name
// --------------------------------------------------------------------------

/// The conflict lands on a name nothing has decided yet: two roots reach it
/// directly, one narrowing it before the other has even been considered. The
/// old `?` at the candidate-selection call site ended the search here even
/// though the decision above it (the other root) has another candidate to
/// try. Both root orders must reach the same answer, since `search` always
/// walks undecided names alphabetically regardless of root order.
#[test]
fn search_resolves_a_conflict_on_a_name_nothing_has_decided_yet() {
    for order in [["lateapp>=2", "latelib<2"], ["latelib<2", "lateapp>=2"]] {
        let table = Table::new(
            &[("lateapp", &["2.0", "3.0"]), ("latelib", &["1.5", "2.5"])],
            &[
                ("lateapp", "3.0", &["latelib>=2"]),
                ("lateapp", "2.0", &["latelib>=1"]),
            ],
        );
        let root_reqs = roots(&order);
        let graph = search(&table, &root_reqs).expect(
            "lateapp==2.0 with latelib==1.5 satisfies every requirement, so the \
             search must reach it instead of refusing on the undecided name",
        );
        assert_eq!(
            pins(&graph),
            vec!["lateapp==2.0", "latelib==1.5"],
            "order {order:?}: the eager `lateapp==3.0` pick raises `latelib>=2`, \
             which nothing satisfies under the root's `latelib<2` — the decision \
             that must move is `lateapp`, since nothing has decided `latelib` yet"
        );
        assert_consistent("undecided-name conflict", &graph, &root_reqs);
    }
}

/// The only remedy is a decision *above* the one the conflict surfaces on:
/// the dependant that raised the offending requirement has another release
/// available, and only that release resolves the conflict. Popping it and
/// retaking it from scratch — the old behavior — reissues the same
/// requirement forever; the fix must advance it to its next candidate.
#[test]
fn search_moves_a_decision_above_the_conflict_when_that_is_the_only_remedy() {
    let table = Table::new(
        &[
            ("chainapp", &["2.0", "3.0"]),
            ("chainmid", &["1.0", "2.0"]),
            ("chainlib", &["1.5", "2.5"]),
        ],
        &[
            ("chainapp", "3.0", &["chainmid>=2"]),
            ("chainapp", "2.0", &["chainmid>=1"]),
            ("chainmid", "2.0", &["chainlib>=2"]),
            ("chainmid", "1.0", &["chainlib>=1"]),
        ],
    );
    let root_reqs = roots(&["chainapp>=1", "chainlib<2"]);
    let graph = search(&table, &root_reqs).expect(
        "chainapp==2.0 -> chainmid==1.0 -> chainlib==1.5 satisfies every \
         requirement in the graph",
    );
    assert_eq!(
        pins(&graph),
        vec!["chainapp==2.0", "chainlib==1.5", "chainmid==1.0"],
        "chainmid cannot pick any release that admits both chainlib bounds on \
         its own — the decision that has to move is chainapp, above it"
    );
    assert_consistent("decision above the conflict", &graph, &root_reqs);
}

/// Backing decisions off is not the same as always finding a solution: a
/// graph where the conflict lands on a name nothing has decided yet, and no
/// assignment of the graph is consistent, still empties the stack and
/// refuses — carrying the original candidate-selection error, not a
/// synthesized one, since no decision was ever pinned to name the conflict
/// against.
#[test]
fn search_still_refuses_when_the_undecided_name_conflict_has_no_solution() {
    let table = Table::new(
        &[("deadapp", &["1.0"]), ("deadlib", &["1.5", "2.5"])],
        &[("deadapp", "1.0", &["deadlib>=2"])],
    );
    let root_reqs = roots(&["deadapp>=1", "deadlib<2"]);
    let err = search(&table, &root_reqs)
        .expect_err("no release of `deadlib` satisfies both `>=2` and `<2`");
    assert_eq!(err.kind, ResolutionErrorKind::EmptyIntersection);
    assert_eq!(err.involved, vec!["deadlib".to_string()]);
    assert!(
        err.trace.contains("deadlib"),
        "the refusal must name `deadlib`; got {:?}",
        err.trace
    );
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

// --------------------------------------------------------------------------
// a `requires_dist` fetch that the index cannot answer at all must not be
// swallowed into a leaf node -- `Universe::node` for the real,
// `IndexClientProvider`-backed `Resolver`, driven against a `wiremock`
// registry (see `apps/mamba/src/pkgmanage/tests.rs` for the construction
// shape this copies: `IndexClientProvider::new` + `Resolver::new` over an
// `IndexClient` pointed at a loopback `MockServer`).
// --------------------------------------------------------------------------
mod requires_dist_unavailable {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::pkgmanage::pkgmgr::http::index_client_for_url;
    use crate::pkgmanage::pkgmgr::resolver::pubgrub_glue::IndexClientProvider;
    use crate::pkgmanage::pkgmgr::resolver::{parse_requirement, ResolutionErrorKind, Resolver};

    const NAME: &str = "flaky";
    const VERSION: &str = "1.0";

    /// Minimal PyPI JSON-API metadata body: one release, one file, no
    /// `requires_dist` at the top level (that lives on the per-version
    /// route, which each test below mounts separately).
    fn metadata_body() -> String {
        format!(
            "{{\"info\":{{\"name\":\"{NAME}\"}},\"releases\":{{\"{VERSION}\":[{{\
             \"filename\":\"{NAME}-{VERSION}-py3-none-any.whl\",\
             \"url\":\"https://example.invalid/{NAME}-{VERSION}-py3-none-any.whl\"\
             }}]}}}}"
        )
    }

    /// A throwaway, per-call cache directory so no two test runs -- and no
    /// real developer cache -- can share a memoised metadata entry.
    fn scratch_cache_dir() -> String {
        std::env::temp_dir()
            .join(format!(
                "mamba-resolver-unit-cache-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or_default()
            ))
            .to_string_lossy()
            .into_owned()
    }

    /// Build an `IndexClientProvider` + `Resolver` pointed at `server`, with
    /// `retry_max = 3` (the product default `resolve_via_pypi` uses) so the
    /// 503 case below actually exercises the client's retry loop. `handle`
    /// is the `Handle` of the runtime `server` and its mocks were mounted
    /// on; the resolver's blocking calls run through it via `block_on`.
    fn resolver_against(server: &MockServer, handle: tokio::runtime::Handle) -> Resolver {
        let client = index_client_for_url(&server.uri(), scratch_cache_dir(), 4, 5, 3, None);
        let provider = IndexClientProvider::new(client, handle);
        Resolver::new(provider)
    }

    /// `/pypi/{NAME}/{VERSION}/json` answers 503 on every request: the
    /// client exhausts its retries and `node` must refuse rather than treat
    /// the release as a leaf.
    #[test]
    fn requires_dist_503_on_every_attempt_refuses_naming_package_version() {
        let rt = tokio::runtime::Runtime::new()
            .expect("fixture: build a runtime for the mock registry");
        let server = rt.block_on(async {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path(format!("/pypi/{NAME}/json")))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(metadata_body().into_bytes(), "application/json"),
                )
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path(format!("/pypi/{NAME}/{VERSION}/json")))
                .respond_with(ResponseTemplate::new(503))
                .mount(&server)
                .await;
            server
        });

        let resolver = resolver_against(&server, rt.handle().clone());
        let roots = vec![parse_requirement(&format!("{NAME}=={VERSION}"))
            .expect("fixture: root requirement must parse")];

        let err = resolver.resolve(&roots).expect_err(
            "a requires_dist fetch that returns 503 on every attempt must refuse \
             the resolution, not resolve the release as a leaf",
        );

        assert_eq!(err.kind, ResolutionErrorKind::RequiresDistUnavailable);
        for token in [format!("{NAME}=={VERSION}"), "requires_dist could not be fetched".to_string()] {
            assert!(
                err.trace.contains(&token),
                "the refusal must name `{token}`; got {:?}",
                err.trace
            );
        }
        assert!(
            err.involved.iter().any(|n| n == NAME),
            "the refusal must involve `{NAME}`; got {:?}",
            err.involved
        );

        let received = rt.block_on(server.received_requests()).unwrap_or_default();
        let hits = received
            .iter()
            .filter(|r| r.url.path() == format!("/pypi/{NAME}/{VERSION}/json"))
            .count();
        assert!(
            hits > 1,
            "the client must have retried the 503 route more than once; saw {hits} request(s)"
        );
    }

    /// `/pypi/{NAME}/{VERSION}/json` answers 404: the client reads this as
    /// "declares nothing", and the release still resolves -- with no edges.
    #[test]
    fn requires_dist_404_resolves_the_node_with_no_edges() {
        let rt = tokio::runtime::Runtime::new()
            .expect("fixture: build a runtime for the mock registry");
        let server = rt.block_on(async {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path(format!("/pypi/{NAME}/json")))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(metadata_body().into_bytes(), "application/json"),
                )
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path(format!("/pypi/{NAME}/{VERSION}/json")))
                .respond_with(ResponseTemplate::new(404))
                .mount(&server)
                .await;
            server
        });

        let resolver = resolver_against(&server, rt.handle().clone());
        let roots = vec![parse_requirement(&format!("{NAME}=={VERSION}"))
            .expect("fixture: root requirement must parse")];

        let graph = resolver.resolve(&roots).unwrap_or_else(|e| {
            panic!("a 404 on the per-version route must still resolve, not refuse: {e}")
        });

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].name, NAME);
        assert_eq!(graph.nodes[0].version, VERSION);
        assert!(
            graph.nodes[0].requires.is_empty(),
            "a 404 per-version route declares nothing, so the node must carry no \
             edges; got {:?}",
            graph.nodes[0].requires
        );
    }
}
