// <HANDWRITE gap="codegen:external-tool-benchmark-gate" tracker="jet-build-parity-gate" reason="Gate test orchestrating node + Vite/Webpack comparator builds has no generator primitive yet; feed back into Agentic Workflow until it can become CODEGEN.">
//! Replacement gate: `jet build` is interchangeable with a Vite/Webpack
//! production build — and wins where it counts.
//!
//! This test is the tests-tree owner of the block claim in
//! `tests/build/README.md`. It drives
//! `scripts/compare-dom-build-corpus.mjs` over the production-build
//! corpus (react-bench, asset fixture, MUI, AntD, Tailwind,
//! styled-components) and asserts, per fixture, in priority order:
//!
//! 1. **Correctness** — the jet-built app passes the same runtime smoke
//!    (driven through `jet bb`, never Playwright) as the Vite reference,
//!    and the static functional checks are green.
//! 2. **Bundle size** — jet's gzip total is no larger than the smallest
//!    incumbent baseline (ratio ≤ 1.0 within a small measurement
//!    tolerance).
//! 3. **Speed** — jet's build wall-clock beats the fastest incumbent
//!    baseline (ratio < 1.0).
//!
//! Skips (with a message) when node is not on PATH. The corpus script
//! hydrates fixtures with `jet install` and runs Vite/Webpack only as
//! isolated comparators.

#[path = "../harness/mod.rs"]
mod harness;

/// Allow 2% gzip measurement noise; "not larger" must not flake on a
/// single recompressed byte boundary.
const GZIP_TOLERANCE: f64 = 1.02;

#[test]
fn jet_build_matches_vite_webpack_and_is_faster() {
    const GATE: &str = "build-parity-gate";
    let root = harness::repo_root();
    if !harness::require_tools(GATE, &["node"]) {
        return;
    }
    harness::ensure_release_jet(&root);

    let run = harness::run_stdout_report_script(
        &root,
        "projects/jet/scripts/compare-dom-build-corpus.mjs",
        &["--runtime-smoke", "required"],
    );
    let report = &run.report;

    let cases = report["cases"]
        .as_array()
        .expect("corpus report must carry cases");
    assert!(
        cases.len() >= 6,
        "corpus breadth shrank to {}; the replacement claim needs the full corpus",
        cases.len()
    );

    eprintln!("[{GATE}] per-fixture comparison (jet vs fastest/smallest of vite+webpack):");
    let mut failures = Vec::new();
    for case in cases {
        let name = case["name"].as_str().unwrap_or("<unknown>");
        let comparison = &case["comparison"];
        if comparison.is_null() {
            failures.push(format!("{name}: jet build failed outright: {case:#}"));
            continue;
        }
        let dur = comparison["jet_duration_ratio_to_fastest_baseline"]
            .as_f64()
            .unwrap_or(f64::INFINITY);
        let gzip = comparison["jet_gzip_ratio_to_smallest_baseline"]
            .as_f64()
            .unwrap_or(f64::INFINITY);
        let smoke = comparison["runtime_smoke"]["result"]
            .as_str()
            .unwrap_or("missing");
        let functional = comparison["static_functional_result"]
            .as_str()
            .unwrap_or("missing");
        eprintln!(
            "  {name:34} duration x{dur:5.2} | gzip x{gzip:5.2} | smoke {smoke} | static {functional}"
        );

        // 1. Correctness first: the built app must work.
        if smoke != "green" {
            failures.push(format!(
                "{name}: runtime smoke is {smoke} ({})",
                comparison["runtime_smoke"]
            ));
        }
        if functional != "green" {
            failures.push(format!("{name}: static functional checks are {functional}"));
        }
        // 2. Bundle size must not be larger.
        if gzip > GZIP_TOLERANCE {
            failures.push(format!(
                "{name}: jet gzip output is {gzip:.3}x the smallest baseline (must be ≤ 1.0)"
            ));
        }
        // 3. Build must be faster.
        if dur >= 1.0 {
            failures.push(format!(
                "{name}: jet build is {dur:.3}x the fastest baseline (must be < 1.0)"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "[{GATE}] {} fixture(s) violate the jet build === vite/webpack claim:\n  - {}",
        failures.len(),
        failures.join("\n  - ")
    );
}
// </HANDWRITE>
