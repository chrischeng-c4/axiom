//! MVP release-gate command runner — locking test (closes #2821).
//!
//! Parent: #2775 (MVP release blocking CI profiles).
//!
//! `scripts/release_gate.py` is the deterministic command runner that
//! drives the six MVP release-blocking profiles (smoke, correctness,
//! performance, ecosystem, package_manager, mambalibs) and emits a
//! summary that matches the schema locked by #2820.
//!
//! Acceptance (issue #2821):
//!
//!     1. Runner returns nonzero when any release-blocking profile fails.
//!     2. Runner writes summaries to a predictable output directory.
//!     3. Runner does not require network for default profiles.
//!
//! This file locks the *contract* of that script — its CLI surface and
//! the structural shape of the summary it writes. The script itself is
//! exercised in `--dry-run` mode so the test stays cheap (one Python
//! invocation per case, no cargo build, no network).
//!
//! The tests deliberately do NOT shell out to `cargo bench` or
//! `cargo test` — those are exercised by the dedicated profile
//! manifests and (eventually) by the runner in CI. Here we only need
//! to prove that the runner's plumbing is wired correctly.

use std::path::{Path, PathBuf};

use serde_json::Value;

fn runner_script() -> PathBuf {
    crate::common::project_root().join("scripts").join("release_gate.py")
}

fn unique_output_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-release-gate-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create unique output dir");
    dir
}

fn run_runner(args: &[&str]) -> (i32, String, String) {
    let output = crate::common::run_python_script(&runner_script(), args);
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

fn read_summary(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read summary {}: {e}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse summary {}: {e}", path.display()))
}

// ─── Acceptance 1: nonzero exit on blocking failure ─────────────────

#[test]
fn dry_run_clean_exits_zero_and_writes_summary() {
    let out = unique_output_dir("clean");
    let (code, stdout, stderr) = run_runner(&[
        "--dry-run",
        "--release-id", "test-clean",
        "--output-dir", out.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "dry-run with no simulated failures must exit 0 — \
         stdout=<<<{stdout}>>> stderr=<<<{stderr}>>>"
    );
    let summary_path = out.join("test-clean.summary.json");
    assert!(
        summary_path.is_file(),
        "summary file must exist at predictable path: {}",
        summary_path.display(),
    );
    let summary = read_summary(&summary_path);
    assert_eq!(summary["overall"]["pass"], Value::Bool(true));
    assert_eq!(summary["overall"]["blocking_failure_count"], Value::from(0));
}

#[test]
fn simulated_blocking_failure_propagates_to_exit_code() {
    let out = unique_output_dir("fail");
    let (code, _stdout, stderr) = run_runner(&[
        "--simulate", "performance=fail",
        "--release-id", "test-fail-1",
        "--output-dir", out.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 1,
        "one simulated blocking failure must yield exit code 1 (saw {code}) — stderr={stderr}"
    );
    let summary = read_summary(&out.join("test-fail-1.summary.json"));
    assert_eq!(summary["overall"]["pass"], Value::Bool(false));
    assert_eq!(summary["overall"]["blocking_failure_count"], Value::from(1));

    // Acceptance: "Summary groups blockers by MVP objective."
    let by_obj = summary["blockers_by_objective"]
        .as_object()
        .expect("blockers_by_objective is object");
    assert!(
        by_obj.contains_key("performance"),
        "failed profile must surface under its MVP objective key: {by_obj:?}",
    );
}

#[test]
fn exit_code_equals_blocking_failure_count() {
    let out = unique_output_dir("multi-fail");
    let (code, _stdout, _stderr) = run_runner(&[
        "--simulate", "performance=fail",
        "--simulate", "ecosystem=fail",
        "--release-id", "test-fail-2",
        "--output-dir", out.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "two simulated blocking failures must yield exit code 2 (saw {code})"
    );
}

// ─── Acceptance 2: predictable output directory ─────────────────────

#[test]
fn summary_lands_at_release_id_dot_summary_dot_json() {
    let out = unique_output_dir("path");
    let release_id = "mamba-2026-05-20T00-00-00Z";
    let (code, _stdout, _stderr) = run_runner(&[
        "--dry-run",
        "--release-id", release_id,
        "--output-dir", out.to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    let expected = out.join(format!("{release_id}.summary.json"));
    assert!(
        expected.is_file(),
        "summary must be named <release_id>.summary.json: {}",
        expected.display(),
    );
    let summary = read_summary(&expected);
    assert_eq!(
        summary["release_id"],
        Value::from(release_id),
        "release_id field must round-trip from CLI to summary",
    );
    // Artifacts block must name the very file we just wrote. Canonicalize
    // both sides so the macOS `/var` -> `/private/var` symlink does not
    // make the comparison spuriously fail.
    let recorded = summary["artifacts"]["summary_path"]
        .as_str()
        .expect("artifacts.summary_path is string");
    let recorded_canon = Path::new(recorded)
        .canonicalize()
        .expect("canonicalize recorded summary_path");
    let expected_canon = expected
        .canonicalize()
        .expect("canonicalize expected summary path");
    assert_eq!(
        recorded_canon, expected_canon,
        "artifacts.summary_path must point at the summary we just wrote",
    );
}

#[test]
fn output_dir_is_created_when_missing() {
    let base = unique_output_dir("nested");
    let nested = base.join("a").join("b").join("c");
    assert!(!nested.exists());
    let (code, _stdout, _stderr) = run_runner(&[
        "--dry-run",
        "--release-id", "nested",
        "--output-dir", nested.to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    assert!(
        nested.join("nested.summary.json").is_file(),
        "runner must mkdir -p the output dir if it does not exist",
    );
}

// ─── Acceptance 3: no network by default ────────────────────────────

#[test]
fn dry_run_does_not_require_network_env() {
    // Default offline contract has two anchors:
    //
    //   (a) The umbrella inventory `validation/mvp.toml` declares
    //       `default_network = "offline"` — every profile inherits that
    //       baseline unless its own manifest overrides it.
    //   (b) The runner exposes `--include-live-network` as an EXPLICIT
    //       opt-in. It must appear in --help so the contract is
    //       discoverable; the runner must NOT default to live network.
    //
    // Together these prove "Runner does not require network for default
    // profiles" without forcing every per-profile manifest to redeclare
    // the umbrella default.
    let mvp_toml = crate::common::project_root().join("validation").join("mvp.toml");
    let body = std::fs::read_to_string(&mvp_toml)
        .unwrap_or_else(|e| panic!("read {}: {e}", mvp_toml.display()));
    assert!(
        body.contains("default_network = \"offline\""),
        "validation/mvp.toml must declare default_network = \"offline\"",
    );

    let (code, stdout, stderr) = run_runner(&["--help"]);
    assert_eq!(code, 0, "release_gate.py --help must exit 0 (stderr={stderr})");
    assert!(
        stdout.contains("--include-live-network"),
        "help text must surface the live-network opt-in flag: stdout={stdout}",
    );
    // The default arg parser must not turn live-network on automatically.
    assert!(
        !stdout.contains("--include-live-network (default: True)"),
        "live-network must not be on by default: stdout={stdout}",
    );
}

// ─── Schema contract: matches release_summary.schema.json (#2820) ──

#[test]
fn dry_run_summary_matches_release_summary_schema_top_keys() {
    let out = unique_output_dir("schema");
    let (code, _stdout, _stderr) = run_runner(&[
        "--dry-run",
        "--release-id", "schema-check",
        "--output-dir", out.to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    let summary = read_summary(&out.join("schema-check.summary.json"));

    // Acceptance for #2820 (required top-level fields).
    for key in [
        "schema_version",
        "release_id",
        "generated_at",
        "runtime_identity",
        "overall",
        "profiles",
        "blockers_by_objective",
        "artifacts",
    ] {
        assert!(
            summary.get(key).is_some(),
            "summary missing required top-level field {key}: {summary}",
        );
    }
    assert_eq!(summary["schema_version"], Value::from(1));

    // Every profile entry must declare the schema's required keys.
    let profiles = summary["profiles"]
        .as_object()
        .expect("profiles is object");
    assert!(!profiles.is_empty(), "dry-run must emit at least one profile");
    for (pid, entry) in profiles {
        for key in ["profile", "command", "status", "blocking", "counts"] {
            assert!(
                entry.get(key).is_some(),
                "profile {pid} missing required key {key}: {entry}",
            );
        }
        let counts = entry["counts"].as_object().expect("counts is object");
        for key in ["passed", "failed", "missing"] {
            assert!(
                counts.contains_key(key),
                "profile {pid} counts missing required key {key}: {counts:?}",
            );
        }
    }
}

// ─── Filter contract ────────────────────────────────────────────────

#[test]
fn profile_filter_restricts_summary_to_listed_ids() {
    let out = unique_output_dir("filter");
    let (code, _stdout, _stderr) = run_runner(&[
        "--dry-run",
        "--profile", "smoke",
        "--profile", "correctness",
        "--release-id", "filter-1",
        "--output-dir", out.to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    let summary = read_summary(&out.join("filter-1.summary.json"));
    let profiles = summary["profiles"].as_object().unwrap();
    let mut keys: Vec<&str> = profiles.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(
        keys, ["correctness", "smoke"],
        "--profile filter must restrict the rolled-up summary to the listed ids",
    );
}
