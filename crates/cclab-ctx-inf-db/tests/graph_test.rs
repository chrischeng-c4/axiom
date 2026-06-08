//! Edge-case semantics for graph algorithms in `src/graph.rs`.
//!
//! These tests pin behaviour that the crate-level integration tests
//! (`engine_test.rs`) only touch tangentially: empty graphs, cycles,
//! `max_hops == 0`, direction asymmetry, source == target, pruning,
//! and disconnected components. Scope is intentionally limited to the
//! five public graph methods; DashMap iteration order is non-deterministic
//! and is therefore not asserted.
//!
//! Style mirrors `engine_test.rs` — `#[test]` fns, `chrono::TimeZone`
//! for deterministic timestamps, and a shared `setup_graph()` helper.

use std::collections::HashMap;

use cclab_ctx_inf_db::engine::*;
use cclab_ctx_inf_db::*;
use chrono::{TimeZone, Utc};

/// Create an engine populated with `names` as `Person` entities, each
/// carrying a deterministic `valid_from`. Returns the engine plus a
/// lookup map so tests can refer to entities by short string labels.
fn setup_graph(names: &[&str]) -> (CtxInfEngine, HashMap<String, EntityId>) {
    let db = CtxInfEngine::new();
    let mut ids = HashMap::new();
    let base = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    for name in names {
        let entity =
            Entity::new(EntityType::Person, *name).with_temporal(TemporalRange::from(base));
        let created = db.create_entity(entity).unwrap();
        ids.insert((*name).to_string(), created.id);
    }
    (db, ids)
}

/// Convenience: create a default-confidence `MetWith` edge src→tgt.
fn edge(db: &CtxInfEngine, src: EntityId, tgt: EntityId) {
    db.create_relation(Relation::new(RelationType::MetWith, src, tgt))
        .unwrap();
}

// ── reachable ────────────────────────────────────────────────────────

#[test]
fn test_reachable_max_hops_zero() {
    let (db, ids) = setup_graph(&["A", "B"]);
    edge(&db, ids["A"], ids["B"]);

    let reach = db.reachable(ids["A"], 0, Direction::Outgoing).unwrap();
    assert!(reach.is_empty(), "max_hops=0 must not visit any neighbor");
}

#[test]
fn test_reachable_direction_outgoing_vs_incoming() {
    let (db, ids) = setup_graph(&["A", "B"]);
    edge(&db, ids["A"], ids["B"]);

    let a_out = db.reachable(ids["A"], 5, Direction::Outgoing).unwrap();
    let a_in = db.reachable(ids["A"], 5, Direction::Incoming).unwrap();
    assert_eq!(
        a_out.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
        vec![ids["B"]]
    );
    assert!(a_in.is_empty());

    let b_out = db.reachable(ids["B"], 5, Direction::Outgoing).unwrap();
    let b_in = db.reachable(ids["B"], 5, Direction::Incoming).unwrap();
    assert!(b_out.is_empty());
    assert_eq!(
        b_in.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
        vec![ids["A"]]
    );
}

#[test]
fn test_reachable_direction_both() {
    let (db, ids) = setup_graph(&["A", "B"]);
    edge(&db, ids["A"], ids["B"]);

    let a = db.reachable(ids["A"], 5, Direction::Both).unwrap();
    let b = db.reachable(ids["B"], 5, Direction::Both).unwrap();
    assert_eq!(
        a.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
        vec![ids["B"]]
    );
    assert_eq!(
        b.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
        vec![ids["A"]]
    );
}

#[test]
fn test_reachable_cycle_terminates() {
    let (db, ids) = setup_graph(&["A", "B"]);
    edge(&db, ids["A"], ids["B"]);
    edge(&db, ids["B"], ids["A"]);

    // Must terminate (no infinite loop) and report B at depth 1 — A is
    // the start and is excluded from results.
    let reach = db.reachable(ids["A"], 5, Direction::Both).unwrap();
    assert_eq!(reach.len(), 1);
    assert_eq!(reach[0], (ids["B"], 1));
}

#[test]
fn test_reachable_sorted_by_depth() {
    let (db, ids) = setup_graph(&["A", "B", "C", "D"]);
    edge(&db, ids["A"], ids["B"]);
    edge(&db, ids["B"], ids["C"]);
    edge(&db, ids["C"], ids["D"]);

    let reach = db.reachable(ids["A"], 5, Direction::Outgoing).unwrap();
    let depths: Vec<usize> = reach.iter().map(|(_, d)| *d).collect();
    assert_eq!(
        depths,
        vec![1, 2, 3],
        "results must be sorted ascending by depth"
    );
}

// ── shortest_path ────────────────────────────────────────────────────

#[test]
fn test_shortest_path_source_equals_target() {
    let (db, ids) = setup_graph(&["A"]);

    let path = db.shortest_path(ids["A"], ids["A"], 5).unwrap().unwrap();
    assert_eq!(path.hop_count, 0);
    assert_eq!(path.min_confidence, 1.0);
    assert_eq!(path.nodes, vec![ids["A"]]);
    assert!(path.edges.is_empty());
}

#[test]
fn test_shortest_path_disconnected_returns_none() {
    let (db, ids) = setup_graph(&["A", "B"]);
    // No edges.
    let result = db.shortest_path(ids["A"], ids["B"], 5).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_shortest_path_max_hops_prunes() {
    let (db, ids) = setup_graph(&["A", "B", "C"]);
    edge(&db, ids["A"], ids["B"]);
    edge(&db, ids["B"], ids["C"]);

    assert!(db.shortest_path(ids["A"], ids["C"], 1).unwrap().is_none());
    let path = db.shortest_path(ids["A"], ids["C"], 2).unwrap().unwrap();
    assert_eq!(path.hop_count, 2);
}

// ── all_paths ────────────────────────────────────────────────────────

#[test]
fn test_all_paths_source_equals_target_empty() {
    let (db, ids) = setup_graph(&["A"]);

    // NOTE: documented asymmetry with shortest_path — all_paths(A, A, _)
    // returns an empty Vec because the DFS never enqueues a zero-length
    // path. shortest_path handles source==target with an explicit early
    // return. See graph.rs L51-58 vs L105-128.
    let paths = db.all_paths(ids["A"], ids["A"], 5).unwrap();
    assert!(paths.is_empty(), "all_paths(X, X, _) currently returns []");
}

#[test]
fn test_all_paths_cycle_is_avoided() {
    let (db, ids) = setup_graph(&["A", "B", "C"]);
    edge(&db, ids["A"], ids["B"]);
    edge(&db, ids["B"], ids["A"]);
    edge(&db, ids["A"], ids["C"]);

    let paths = db.all_paths(ids["A"], ids["C"], 5).unwrap();
    // Every returned path must be simple — no node repeats.
    for p in &paths {
        let mut seen = std::collections::HashSet::new();
        for n in &p.nodes {
            assert!(seen.insert(*n), "cycle leaked into all_paths result");
        }
    }
    assert!(paths.iter().any(|p| p.nodes == vec![ids["A"], ids["C"]]));
}

// ── degree_centrality ────────────────────────────────────────────────

#[test]
fn test_degree_centrality_empty_graph_returns_empty() {
    let db = CtxInfEngine::new();
    let centrality = db.degree_centrality();
    assert!(centrality.is_empty());
}

#[test]
fn test_degree_centrality_single_entity_is_zero() {
    let (db, ids) = setup_graph(&["A"]);
    // Hits the `n <= 1` branch at graph.rs L133.
    let centrality = db.degree_centrality();
    assert_eq!(centrality.len(), 1);
    assert_eq!(centrality[&ids["A"]], 0.0);
}

// ── connected_components ─────────────────────────────────────────────

#[test]
fn test_connected_components_multi_disjoint() {
    let (db, _ids) = setup_graph(&["A", "B", "C"]);
    let components = db.connected_components();
    assert_eq!(components.len(), 3);
    for c in &components {
        assert_eq!(c.len(), 1);
    }
}

#[test]
fn test_connected_components_single_cluster() {
    let (db, ids) = setup_graph(&["A", "B", "C"]);
    edge(&db, ids["A"], ids["B"]);
    edge(&db, ids["B"], ids["C"]);
    let components = db.connected_components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].len(), 3);
}
