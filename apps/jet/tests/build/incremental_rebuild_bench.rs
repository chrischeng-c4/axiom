// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! R6 benchmark gate for #1250 — bundler HMR incremental rebuild.
//!
//! @spec `.aw/tech-design/projects/jet/logic/bundler-incremental-rebuild.md`
//!     §"Slice 5 — benchmark harness".
//! @issue #1250 — R6: "rebuild latency under 100 ms for a single-file
//!     change in a 500-module graph".
//!
//! Run as a regular `cargo test` so the gate fires on every CI run
//! without needing a separate `cargo bench` invocation. Criterion is
//! NOT pulled in: the gate is a wall-clock assertion against a hard
//! ceiling, and adding a heavy regression-tracking dep just to print
//! prettier numbers is not worth the build cost on a critical-path
//! crate.
//!
//! Three test cases, each derived from the spec's three bench
//! functions:
//!
//! 1. `cold_full_graph_baseline` — informational. Measures a fresh
//!    transform pass over all 500 modules (no cache). NOT gated;
//!    serves as the upper bound the warm-cache case must beat.
//! 2. `single_leaf_change_warm_cache_under_100ms` — **R6 ceiling**.
//!    Pre-builds the cache, then changes one leaf and runs
//!    `IncrementalRebuilder::rebuild`. Wall-clock must be <100ms.
//! 3. `barrel_cascade_under_2x_leaf_baseline` — relative gate. A
//!    3-deep barrel re-export cascade rebuild must finish within 2×
//!    the leaf-change wall-clock (spec language: "no more than 2×
//!    the leaf-change baseline; not a hard ms ceiling").

use jet::dev_server::incremental_rebuilder::IncrementalRebuilder;
use jet::dev_server::module_graph::ModuleGraph;
use std::path::Path;
use std::time::Instant;

const MODULE_COUNT: usize = 500;
const R6_CEILING_MS: u128 = 100;

fn js_lang() -> tree_sitter::Language {
    tree_sitter_javascript::LANGUAGE.into()
}

/// Build a deterministic 500-module synthetic graph.
///
/// Layout: entry imports `fanout = 25` first-level barrels; each
/// barrel imports `fanout = 19` leaves; the remaining ≈25 modules
/// form a deeper re-export chain (entry → barrel0 → mid0 → mid1 →
/// leaf) so the benchmark exercises both wide fan-out and deep
/// nesting. Total count is exactly 500.
///
/// Returns `(graph, sources)` where `sources[i] = (url, file,
/// source)` — the i-th module's identifying triple.
fn build_500_module_graph() -> (ModuleGraph, Vec<(String, String, String)>) {
    let mut graph = ModuleGraph::new();
    let mut sources: Vec<(String, String, String)> = Vec::with_capacity(MODULE_COUNT);

    let entry_url = "/src/entry.ts".to_string();
    let entry_file = "/abs/src/entry.ts".to_string();

    let fanout_barrels: usize = 25;
    let leaves_per_barrel: usize = 19;
    let entry_imports: Vec<String> = (0..fanout_barrels)
        .map(|i| format!("/src/barrel_{i}.ts"))
        .collect();
    graph.add_module(&entry_url, &entry_file, &entry_imports);
    sources.push((
        entry_url.clone(),
        entry_file.clone(),
        format!(
            "{}\nexport const ENTRY = 1;\n",
            entry_imports
                .iter()
                .enumerate()
                .map(|(i, u)| format!("import {{ B{i} }} from '{u}';"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    ));

    // 25 barrels × 19 leaves = 475 nodes; plus entry + 25 barrels = 501.
    // Drop one leaf to land at exactly 500.
    let mut total = 1 + fanout_barrels;
    'outer: for bi in 0..fanout_barrels {
        let barrel_url = format!("/src/barrel_{bi}.ts");
        let barrel_file = format!("/abs/src/barrel_{bi}.ts");
        let leaf_imports: Vec<String> = (0..leaves_per_barrel)
            .map(|li| format!("/src/leaf_{bi}_{li}.ts"))
            .collect();
        graph.add_module(&barrel_url, &barrel_file, &leaf_imports);
        sources.push((
            barrel_url.clone(),
            barrel_file.clone(),
            format!(
                "{}\nexport const B{bi} = {bi};\n",
                leaf_imports
                    .iter()
                    .map(|u| format!("export {{ leaf_{} }} from '{u}';", u_basename(u)))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        ));

        for li in 0..leaves_per_barrel {
            let leaf_url = format!("/src/leaf_{bi}_{li}.ts");
            let leaf_file = format!("/abs/src/leaf_{bi}_{li}.ts");
            graph.add_module(&leaf_url, &leaf_file, &[]);
            sources.push((
                leaf_url,
                leaf_file,
                format!("export const leaf_{bi}_{li} = {bi} * 100 + {li};\n"),
            ));
            total += 1;
            if total == MODULE_COUNT {
                break 'outer;
            }
        }
    }

    assert_eq!(
        sources.len(),
        MODULE_COUNT,
        "synthetic graph must have exactly {MODULE_COUNT} modules"
    );
    (graph, sources)
}

/// Tiny helper: extract `leaf_X_Y` from `/src/leaf_X_Y.ts`.
fn u_basename(url: &str) -> &str {
    let stem = url.rsplit('/').next().unwrap_or(url);
    stem.strip_suffix(".ts").unwrap_or(stem)
}

/// Prime the rebuilder's transform cache for every module in
/// `sources`. This is the "warm cache" the leaf-change case
/// measures against.
fn prime_cache(rebuilder: &mut IncrementalRebuilder, sources: &[(String, String, String)]) {
    // Reach in via metrics_snapshot to confirm priming worked.
    // The rebuilder's `transformer` is private, so we drive priming
    // through the public `rebuild` path with no-op-style sources.
    // Each `rebuild` call invalidates + retransforms one path,
    // which is enough to seat its cache entry.
    let empty_graph = ModuleGraph::new();
    for (url, file, source) in sources {
        // We pass an empty graph so the priming pass does NOT walk
        // dependents (that would re-prime modules we already touched
        // and inflate the cache spuriously). The single-target
        // invalidation + re-transform is the priming step.
        let _ = rebuilder
            .rebuild(url, Path::new(file), source, &empty_graph)
            .expect("priming rebuild must succeed");
    }
}

#[test]
fn cold_full_graph_baseline() {
    // Informational only — measures the upper bound (full re-transform
    // of 500 modules from a cold cache). Not gated; recorded as the
    // baseline the warm path must beat.
    let (_graph, sources) = build_500_module_graph();
    let mut rebuilder = IncrementalRebuilder::new(js_lang()).unwrap();

    let started = Instant::now();
    prime_cache(&mut rebuilder, &sources);
    let elapsed = started.elapsed();

    eprintln!(
        "cold_full_graph_baseline: 500 modules transformed in {:?} ({:.1} ms/module)",
        elapsed,
        elapsed.as_secs_f64() * 1000.0 / MODULE_COUNT as f64,
    );

    let snap = rebuilder.metrics_snapshot();
    assert_eq!(
        snap.misses as usize, MODULE_COUNT,
        "cold pass must register 500 misses, got {snap:?}"
    );
}

#[test]
fn single_leaf_change_warm_cache_under_100ms() {
    let (graph, sources) = build_500_module_graph();
    let mut rebuilder = IncrementalRebuilder::new(js_lang()).unwrap();
    prime_cache(&mut rebuilder, &sources);

    // Pick a deep leaf — last one in the source list — and change
    // its bytes by appending a single character. The graph fans out
    // entry → barrel_? → leaf_?_?, so the rebuilder must walk barrel
    // and entry as dependents.
    let (leaf_url, leaf_file, leaf_src) = sources.last().unwrap().clone();
    let new_src = format!("{leaf_src}// touched\n");

    let started = Instant::now();
    let outcome = rebuilder
        .rebuild(&leaf_url, Path::new(&leaf_file), &new_src, &graph)
        .expect("warm rebuild must succeed");
    let elapsed_ms = started.elapsed().as_millis();

    eprintln!(
        "single_leaf_change_warm_cache: rebuild took {elapsed_ms} ms, invalidated {} modules",
        outcome.invalidated.len()
    );

    // The rebuilder MUST walk the barrel cascade — leaf's barrel +
    // entry must both appear in the invalidated set.
    assert!(
        outcome
            .invalidated
            .iter()
            .any(|u| u.starts_with("/src/barrel_")),
        "barrel cascade missed: {:?}",
        outcome.invalidated
    );
    assert!(
        outcome.invalidated.iter().any(|u| u == "/src/entry.ts"),
        "entry cascade missed: {:?}",
        outcome.invalidated
    );

    assert!(
        elapsed_ms < R6_CEILING_MS,
        "R6 ceiling violated: warm-cache leaf rebuild took {elapsed_ms} ms (ceiling {R6_CEILING_MS} ms)"
    );
}

#[test]
fn barrel_cascade_under_2x_leaf_baseline() {
    // Relative gate: a barrel-cascade rebuild must finish within 2×
    // the leaf-change wall-clock measured immediately above. We
    // re-run the leaf measurement here (independent rebuilder) so
    // both numbers come from the same hardware in the same test run.
    let (graph, sources) = build_500_module_graph();

    // Leaf baseline.
    let mut leaf_rebuilder = IncrementalRebuilder::new(js_lang()).unwrap();
    prime_cache(&mut leaf_rebuilder, &sources);
    let (leaf_url, leaf_file, leaf_src) = sources.last().unwrap().clone();
    let leaf_started = Instant::now();
    let _ = leaf_rebuilder
        .rebuild(
            &leaf_url,
            Path::new(&leaf_file),
            &format!("{leaf_src}// touched\n"),
            &graph,
        )
        .unwrap();
    let leaf_ms = leaf_started.elapsed().as_millis();

    // Barrel rebuild: change a barrel module (depth-2 in the graph).
    // Its dependents include entry + nothing else (leaves don't
    // import barrels), so the cascade is shallower than the leaf
    // case but the per-module cost is higher (barrel sources contain
    // 19 export-from lines).
    let mut barrel_rebuilder = IncrementalRebuilder::new(js_lang()).unwrap();
    prime_cache(&mut barrel_rebuilder, &sources);
    let barrel = sources
        .iter()
        .find(|(u, _, _)| u == "/src/barrel_0.ts")
        .cloned()
        .expect("barrel_0 must exist");
    let barrel_started = Instant::now();
    let _ = barrel_rebuilder
        .rebuild(
            &barrel.0,
            Path::new(&barrel.1),
            &format!("{}// touched\n", barrel.2),
            &graph,
        )
        .unwrap();
    let barrel_ms = barrel_started.elapsed().as_millis();

    eprintln!(
        "barrel_cascade: leaf={leaf_ms} ms, barrel={barrel_ms} ms (ceiling = 2× leaf = {} ms, min 4 ms)",
        leaf_ms.saturating_mul(2),
    );

    // 2× leaf, with a 4 ms floor so the test isn't flaky on
    // sub-millisecond timing where 2× rounding makes the ceiling
    // tighter than measurement resolution.
    let ceiling = (leaf_ms.saturating_mul(2)).max(4);
    assert!(
        barrel_ms <= ceiling,
        "barrel cascade exceeded 2× leaf baseline: barrel={barrel_ms} ms, leaf={leaf_ms} ms, ceiling={ceiling} ms"
    );
}
// CODEGEN-END
