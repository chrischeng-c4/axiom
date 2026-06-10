// <HANDWRITE gap="codegen:external-tool-benchmark-gate" tracker="jet-pkg-replacement-gate" reason="Gate test orchestrating node + npm/pnpm comparator benchmarks has no generator primitive yet; feed back into Agentic Workflow until it can become CODEGEN.">
//! Replacement + speed gate: `jet install` fully replaces npm/pnpm and
//! is faster.
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
//! 2. **Faster** — for every benchmark fixture, jet's cold AND warm
//!    install are strictly faster than the fastest incumbent baseline
//!    (npm or pnpm, whichever won that fixture), measured in the same
//!    run on the same machine against isolated baseline copies.
//!
//! Skips (with a message) when node/npm/pnpm are not on PATH, matching
//! the repo's real-services-over-mocks testing policy. Builds
//! `target/release/jet` if missing — the benchmark must measure the
//! optimized binary, never a debug build.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("projects/jet is two levels under the repo root")
        .to_path_buf()
}

fn tool_available(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

#[test]
fn jet_pkg_management_replaces_npm_pnpm_and_is_faster() {
    let root = repo_root();

    for tool in ["node", "npm", "pnpm"] {
        if !tool_available(tool) {
            eprintln!("[pkg-replacement-gate] skipping: `{tool}` is not available on PATH");
            return;
        }
    }

    // The speed claim is only meaningful for the optimized binary.
    let jet_bin = root.join("target/release/jet");
    if !jet_bin.exists() {
        eprintln!("[pkg-replacement-gate] building target/release/jet ...");
        let status = Command::new("cargo")
            .args(["build", "-p", "jet", "--release"])
            .current_dir(&root)
            .status()
            .expect("spawning cargo build -p jet --release");
        assert!(status.success(), "release build of jet failed");
    }

    let evidence = std::env::temp_dir().join(format!(
        "jet-pkg-replacement-gate-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&evidence);

    let status = Command::new("node")
        .arg("projects/jet/scripts/compare-pkg-management.mjs")
        .arg("--require-baselines")
        .arg("--evidence")
        .arg(&evidence)
        .current_dir(&root)
        .status()
        .expect("spawning compare-pkg-management.mjs");

    let body = std::fs::read_to_string(&evidence).unwrap_or_else(|err| {
        panic!(
            "compare-pkg-management.mjs (exit {status:?}) wrote no evidence at {}: {err}",
            evidence.display()
        )
    });
    let report: serde_json::Value =
        serde_json::from_str(&body).expect("parsing pkg-management evidence JSON");

    // 1. Replacement contract: every top-level check green.
    let checks = report["checks"]
        .as_array()
        .expect("evidence must carry a checks array");
    assert!(!checks.is_empty(), "evidence checks array is empty");
    let mut check_names = Vec::new();
    for check in checks {
        let name = check["name"].as_str().unwrap_or("<unnamed>");
        check_names.push(name.to_string());
        assert_eq!(
            check["ok"], true,
            "replacement contract check failed: {name}\nreport: {report:#}"
        );
    }
    for required in [
        "no_npm_pnpm_yarn_bun_executor_commands",
        "no_npm_ci_anywhere",
        "required_baseline_benchmarks_green",
        "required_baseline_performance_green",
    ] {
        assert!(
            check_names.iter().any(|n| n == required),
            "evidence is missing the {required} check; gate contract drifted: {check_names:?}"
        );
    }
    assert_eq!(
        report["result"], "green",
        "pkg-management contract is not green: {report:#}"
    );
    assert!(
        status.success(),
        "compare-pkg-management.mjs exited non-zero while reporting green — script/report drift"
    );

    // 2. Speed claim: strictly faster than the fastest incumbent on every
    //    fixture, cold and warm.
    let fixtures = report["fixtures"]
        .as_array()
        .expect("evidence must carry per-fixture entries");
    assert!(
        fixtures.len() >= 5,
        "benchmark fixture breadth shrank to {}; the speed claim needs the full corpus",
        fixtures.len()
    );
    eprintln!("[pkg-replacement-gate] fixture timings (jet vs fastest of npm/pnpm):");
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
        let (Some(jet_cold), Some(jet_warm), Some(base_cold), Some(base_warm)) =
            (jet_cold, jet_warm, base_cold, base_warm)
        else {
            panic!("fixture {name} is missing benchmark metrics: {fixture:#}");
        };
        eprintln!(
            "  {name:30} cold {jet_cold:7.1}ms vs {base_cold:7.1}ms | warm {jet_warm:7.1}ms vs {base_warm:7.1}ms"
        );
        assert!(
            jet_cold < base_cold,
            "{name}: jet cold install {jet_cold:.1}ms is not faster than the fastest baseline {base_cold:.1}ms"
        );
        assert!(
            jet_warm < base_warm,
            "{name}: jet warm install {jet_warm:.1}ms is not faster than the fastest baseline {base_warm:.1}ms"
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
