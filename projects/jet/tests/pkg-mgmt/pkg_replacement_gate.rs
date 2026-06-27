// <HANDWRITE gap="codegen:external-tool-benchmark-gate" tracker="jet-pkg-replacement-gate" reason="Gate test orchestrating node + npm/pnpm comparator benchmarks has no generator primitive yet; feed back into Agentic Workflow until it can become CODEGEN.">
//! Replacement + speed gate: `jet install` fully replaces npm/pnpm and
//! stays within the configured incumbent performance envelope.
//!
//! This test is the tests-tree owner of the block claim in
//! `tests/pkg-mgmt/README.md`. It drives
//! `scripts/compare-pkg-management.mjs` with required npm/pnpm baselines
//! and asserts, from the machine-readable evidence:
//!
//! 1. **Replacement** — the overall contract is green: fixture
//!    hydration from `jet-lock.yaml`, mutation contract
//!    (add/remove/update), workspace contract, bin links, and the
//!    "npm/pnpm never manage fixtures" executor rules all pass.
//! 2. **Performance** — for every benchmark fixture, jet's cold and warm
//!    install stay within the script's configured ratio to the fastest
//!    incumbent baseline (npm or pnpm, whichever won that fixture),
//!    measured in the same run on the same machine against isolated
//!    baseline copies.
//!
//! Skips (with a message) when node/npm/pnpm are not on PATH, matching
//! the repo's real-services-over-mocks testing policy. Builds
//! `target/release/jet` if missing — the benchmark must measure the
//! optimized binary, never a debug build.

#[path = "../harness/mod.rs"]
mod harness;

#[test]
fn jet_pkg_management_replaces_npm_pnpm_with_bounded_baseline_performance() {
    const GATE: &str = "pkg-replacement-gate";
    let root = harness::repo_root();
    if !harness::require_tools(GATE, &["node", "npm", "pnpm"]) {
        return;
    }
    // The speed claim is only meaningful for the optimized binary.
    harness::ensure_release_jet(&root);

    let evidence = harness::evidence_path(GATE);
    let run = harness::run_evidence_script(
        &root,
        "projects/jet/scripts/compare-pkg-management.mjs",
        &[
            "--require-baselines",
            "--evidence",
            evidence.to_str().expect("utf-8 evidence path"),
        ],
        &evidence,
    );
    let report = &run.report;

    // 1. Replacement contract: every top-level check green.
    harness::assert_checks_green(
        report,
        &[
            "no_npm_pnpm_yarn_bun_executor_commands",
            "no_npm_ci_anywhere",
            "required_baseline_benchmarks_green",
            "required_baseline_performance_green",
        ],
    );
    assert_eq!(
        report["result"], "green",
        "pkg-management contract is not green: {report:#}"
    );
    assert!(
        run.exit_ok,
        "compare-pkg-management.mjs exited non-zero while reporting green — script/report drift"
    );

    // 2. Speed claim: within the configured incumbent-ratio envelope on
    //    every fixture, cold and warm.
    let fixtures = report["fixtures"]
        .as_array()
        .expect("evidence must carry per-fixture entries");
    assert!(
        fixtures.len() >= 5,
        "benchmark fixture breadth shrank to {}; the speed claim needs the full corpus",
        fixtures.len()
    );
    eprintln!("[{GATE}] fixture timings (jet vs fastest of npm/pnpm):");
    for fixture in fixtures {
        let name = fixture["fixture"]
            .as_str()
            .map(|s| s.rsplit('/').next().unwrap_or(s).to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        let metrics = &fixture["baseline_performance"]["metrics"];
        let jet_cold = metrics["jet_cold_install_ms"].as_f64();
        let jet_warm = metrics["jet_warm_install_ms"].as_f64();
        let base_cold = metrics["fastest_baseline_cold_install_ms"].as_f64();
        let base_warm = metrics["fastest_baseline_warm_install_ms"].as_f64();
        let max_ratio = fixture["baseline_performance"]["thresholds"]["max_install_time_ratio"]
            .as_f64()
            .expect("fixture baseline performance must carry max_install_time_ratio");
        let (Some(jet_cold), Some(jet_warm), Some(base_cold), Some(base_warm)) =
            (jet_cold, jet_warm, base_cold, base_warm)
        else {
            panic!("fixture {name} is missing benchmark metrics: {fixture:#}");
        };
        eprintln!(
            "  {name:30} cold {jet_cold:7.1}ms vs {base_cold:7.1}ms | warm {jet_warm:7.1}ms vs {base_warm:7.1}ms | ratio <= {max_ratio:.2}"
        );
        assert!(
            jet_cold <= base_cold * max_ratio,
            "{name}: jet cold install {jet_cold:.1}ms exceeds baseline envelope {base_cold:.1}ms * {max_ratio:.2}"
        );
        assert!(
            jet_warm <= base_warm * max_ratio,
            "{name}: jet warm install {jet_warm:.1}ms exceeds baseline envelope {base_warm:.1}ms * {max_ratio:.2}"
        );
        assert_eq!(
            fixture["baseline_performance"]["result"], "green",
            "{name}: baseline performance contract not green"
        );
        assert_eq!(
            fixture["result"], "green",
            "{name}: fixture replacement contract not green"
        );
    }

    let _ = std::fs::remove_file(&evidence);
}
// </HANDWRITE>
