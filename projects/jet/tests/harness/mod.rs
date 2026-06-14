// <HANDWRITE gap="codegen:external-tool-benchmark-gate" tracker="jet-tests-harness" reason="Shared gate-test harness for replacement-claim gates; becomes CODEGEN with the gate generator primitive.">
//! Shared harness for the block-owner gate tests under
//! `tests/<block>/*_gate.rs`.
//!
//! Gates verify replacement claims against real incumbents (npm/pnpm,
//! Playwright, Vite/Webpack) by driving the evidence scripts in
//! `projects/jet/scripts/` and asserting on their machine-readable
//! reports. This module owns the shared mechanics so each gate stays a
//! short, readable statement of its claim:
//!
//! - locating the repo root and the release binary (benchmarks must
//!   never measure a debug build),
//! - skip-vs-run decisions for incumbent tooling (real services over
//!   mocks; skip with a message when a comparator is absent),
//! - running a script and parsing its evidence JSON,
//! - the "every contract check is green" assertion.

#![allow(dead_code)] // each gate uses a subset of the helpers.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Repo root, derived from the jet crate's manifest dir.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("projects/jet is two levels under the repo root")
        .to_path_buf()
}

/// True when `tool --version` runs successfully on PATH.
pub fn tool_available(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Skip helper: returns false (and logs) when any tool is missing.
/// Gates `return` early on false, matching the repo skip pattern.
pub fn require_tools(gate: &str, tools: &[&str]) -> bool {
    for tool in tools {
        if !tool_available(tool) {
            eprintln!("[{gate}] skipping: `{tool}` is not available on PATH");
            return false;
        }
    }
    true
}

/// Path to `target/release/jet`, building it if missing. Speed claims
/// are only meaningful for the optimized binary.
pub fn ensure_release_jet(root: &Path) -> PathBuf {
    let jet_bin = root.join("target/release/jet");
    if !jet_bin.exists() {
        eprintln!("[harness] building target/release/jet ...");
        let status = Command::new("cargo")
            .args(["build", "-p", "jet", "--release"])
            .current_dir(root)
            .status()
            .expect("spawning cargo build -p jet --release");
        assert!(status.success(), "release build of jet failed");
    }
    jet_bin
}

/// Outcome of one evidence-script run.
pub struct GateRun {
    pub exit_ok: bool,
    pub report: serde_json::Value,
}

/// Run a node evidence script from the repo root and parse the JSON it
/// writes to `evidence`. Panics with the script status when the
/// evidence file is missing or unparsable — a gate without evidence is
/// a failure, not a skip.
pub fn run_evidence_script(root: &Path, script: &str, args: &[&str], evidence: &Path) -> GateRun {
    let _ = std::fs::remove_file(evidence);
    let status = Command::new("node")
        .arg(script)
        .args(args)
        .current_dir(root)
        .status()
        .unwrap_or_else(|err| panic!("spawning {script}: {err}"));
    let body = std::fs::read_to_string(evidence).unwrap_or_else(|err| {
        panic!(
            "{script} (exit {status:?}) wrote no evidence at {}: {err}",
            evidence.display()
        )
    });
    let report = serde_json::from_str(&body)
        .unwrap_or_else(|err| panic!("parsing {script} evidence JSON: {err}"));
    GateRun {
        exit_ok: status.success(),
        report,
    }
}

/// Unique evidence path under the system temp dir.
pub fn evidence_path(gate: &str) -> PathBuf {
    std::env::temp_dir().join(format!("jet-{gate}-{}.json", std::process::id()))
}

/// Run a node evidence script that prints its JSON report to stdout
/// (e.g. compare-dom-build-corpus.mjs). Parses the trailing JSON object;
/// everything before it is progress logging and is echoed to stderr.
pub fn run_stdout_report_script(root: &Path, script: &str, args: &[&str]) -> GateRun {
    let output = Command::new("node")
        .arg(script)
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("spawning {script}: {err}"));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_start = stdout
        .find("\n{")
        .map(|i| i + 1)
        .or_else(|| stdout.starts_with('{').then_some(0))
        .unwrap_or_else(|| panic!("{script} printed no JSON report:\n{stdout}"));
    let report = serde_json::from_str(&stdout[json_start..])
        .unwrap_or_else(|err| panic!("parsing {script} stdout JSON: {err}\n{stdout}"));
    GateRun {
        exit_ok: output.status.success(),
        report,
    }
}

/// Assert every `{name, ok}` entry in `checks` is ok, and that the
/// named required checks are present (so contract drift in the script
/// fails loudly instead of silently passing a weaker bar).
pub fn assert_checks_green(report: &serde_json::Value, required: &[&str]) {
    let checks = report["checks"]
        .as_array()
        .unwrap_or_else(|| panic!("evidence must carry a checks array: {report:#}"));
    assert!(!checks.is_empty(), "evidence checks array is empty");
    let mut names = Vec::new();
    for check in checks {
        let name = check["name"].as_str().unwrap_or("<unnamed>");
        names.push(name.to_string());
        assert_eq!(
            check["ok"], true,
            "contract check failed: {name}\nreport: {report:#}"
        );
    }
    for req in required {
        assert!(
            names.iter().any(|n| n == req),
            "evidence is missing the {req} check; gate contract drifted: {names:?}"
        );
    }
}
// </HANDWRITE>
