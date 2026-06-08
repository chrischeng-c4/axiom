//! MVP performance per-benchmark floor checker lock (closes #2565).
//!
//! Parent: #2530 (performance gate suite).
//!
//! Locks the contract of `scripts/perf_floor_check.py`, the script that
//! reads a baseline JSON (the perf source of truth — see #2823) and
//! enforces the per-benchmark floor declared in
//! `validation/profiles/performance.toml`:
//!
//!     every required benchmark must report speedup_vs_cpython >= 1.0.
//!
//! Acceptance (issue #2565):
//!
//!     1. A required benchmark below 1.0 fails the checker.
//!     2. A blocked benchmark below 1.0 is reported but does not count
//!        as a pass.
//!     3. The checker can run without re-running benchmarks (reads
//!        baseline JSON only).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde_json::{json, Value};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn checker_script() -> PathBuf {
    project_root().join("scripts").join("perf_floor_check.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-perf-floor-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn write_baseline(dir: &Path, name: &str, value: &Value) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, value.to_string()).expect("write baseline fixture");
    path
}

fn run_checker(args: &[&str]) -> (i32, String, String) {
    let output = Command::new("python3")
        .arg(checker_script())
        .args(args)
        .current_dir(project_root())
        .output()
        .expect("invoke perf_floor_check.py");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

// ─── Acceptance 1: required below floor → fail ──────────────────────

#[test]
fn required_benchmark_below_floor_fails_checker() {
    let dir = unique_dir("required-fail");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "string_concat", "kind": "Workload",
             "bucket": "required", "speedup_vs_cpython": 0.42},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, stderr) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 1, "required below floor must fail (stderr={stderr})");
    assert!(
        stderr.contains("string_concat"),
        "must name offending benchmark"
    );
    assert!(stderr.contains("rule:"), "must restate the rule for triage");
    assert!(stderr.contains("#2565"), "must cite tracking issue");
}

#[test]
fn unbucketed_entries_default_to_required() {
    // Acceptance #1 reinforcement: a freshly recorded benchmark missing
    // a `bucket` field must NOT silently land in optional — it must be
    // gated by default so it can't slip past the floor unnoticed.
    let dir = unique_dir("default-required");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "loose_entry", "kind": "Workload",
             "speedup_vs_cpython": 0.5},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, _) = run_checker(&["--baseline", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 1, "missing-bucket entries default to required");
}

#[test]
fn required_benchmark_at_or_above_floor_passes() {
    let dir = unique_dir("required-pass");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "int_sum_loop", "kind": "Numeric",
             "bucket": "required", "speedup_vs_cpython": 3.17},
            {"name": "exactly_floor", "kind": "Numeric",
             "bucket": "required", "speedup_vs_cpython": 1.0},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, stderr) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 0, "speedup ≥ 1.0 must pass (stderr={stderr})");
    assert!(stderr.contains("clean"), "text mode must announce clean");
}

// ─── Acceptance 2: blocked entries reported, not gating ─────────────

#[test]
fn blocked_benchmark_below_floor_is_reported_but_not_gating() {
    let dir = unique_dir("blocked-report");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "ok_required", "kind": "Numeric",
             "bucket": "required", "speedup_vs_cpython": 1.5},
            {"name": "slow_blocked", "kind": "Workload",
             "bucket": "blocker", "speedup_vs_cpython": 0.001},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 0, "blocked entry must NOT cause non-zero exit");
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    assert_eq!(v["violations"].as_array().unwrap().len(), 0);
    let blocked = v["blocked_below_floor"].as_array().unwrap();
    assert_eq!(
        blocked.len(),
        1,
        "blocked entry must surface in dedicated section"
    );
    assert_eq!(blocked[0]["name"], json!("slow_blocked"));
    assert_eq!(blocked[0]["bucket"], json!("blocker"));
}

#[test]
fn xfail_and_optional_buckets_treated_as_report_only() {
    let dir = unique_dir("xfail-optional");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "xfail_slow", "kind": "Workload",
             "bucket": "xfail", "speedup_vs_cpython": 0.1},
            {"name": "optional_slow", "kind": "Workload",
             "bucket": "optional", "speedup_vs_cpython": 0.2},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    let blocked = v["blocked_below_floor"].as_array().unwrap();
    assert_eq!(blocked.len(), 2, "both xfail and optional must report");
}

#[test]
fn json_report_surfaces_required_speedup_and_kind() {
    // Acceptance #1: report must include benchmark id, current speedup,
    // and the floor — enough for a worker to triage without re-running.
    let dir = unique_dir("triage-fields");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "list_sort_builtin", "kind": "Workload",
             "bucket": "required", "speedup_vs_cpython": 0.001},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 1);
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    assert_eq!(
        v["floor"].as_f64(),
        Some(1.0),
        "floor must surface in report"
    );
    let violations = v["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 1);
    for key in ["name", "bucket", "speedup", "kind"] {
        assert!(
            violations[0].get(key).is_some(),
            "violation must include {key}: {}",
            violations[0],
        );
    }
}

#[test]
fn violations_sorted_ascending_speedup_for_diff_stability() {
    let dir = unique_dir("sort-order");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "b", "kind": "x", "bucket": "required",
             "speedup_vs_cpython": 0.5},
            {"name": "a", "kind": "x", "bucket": "required",
             "speedup_vs_cpython": 0.1},
            {"name": "c", "kind": "x", "bucket": "required",
             "speedup_vs_cpython": 0.9},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (_code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    let v: Value = serde_json::from_str(&stdout).unwrap();
    let names: Vec<&str> = v["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["a", "b", "c"],
        "ascending speedup → slowest first"
    );
}

// ─── Acceptance 3: runs without re-executing benchmarks ─────────────

#[test]
fn checker_runs_in_under_two_seconds_on_baseline() {
    // Acceptance #3: "can run without re-running all benchmarks". A
    // full perf rerun takes minutes. The checker must read the baseline
    // and answer in well under that.
    let dir = unique_dir("speed");
    let baseline = json!({
        "version": 1,
        "benchmarks": (0..32).map(|i| json!({
            "name": format!("b{i}"),
            "kind": "Workload",
            "bucket": "required",
            "speedup_vs_cpython": 1.5,
        })).collect::<Vec<_>>(),
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let started = Instant::now();
    let (code, _, _) = run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    let elapsed = started.elapsed();
    assert_eq!(code, 0);
    assert!(
        elapsed.as_secs() < 2,
        "checker must answer in <2s, took {elapsed:?}",
    );
}

#[test]
fn checker_reports_checked_count_for_audit() {
    let dir = unique_dir("checked-count");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "r1", "bucket": "required", "speedup_vs_cpython": 1.5, "kind": "x"},
            {"name": "r2", "bucket": "required", "speedup_vs_cpython": 2.5, "kind": "x"},
            {"name": "o1", "bucket": "optional", "speedup_vs_cpython": 0.1, "kind": "x"},
            {"name": "b1", "bucket": "blocker",  "speedup_vs_cpython": 0.1, "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (_code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        v["checked_count"].as_i64(),
        Some(2),
        "only required entries should be counted as 'checked'",
    );
}

// ─── Cross-reference: performance.toml drives the floor ─────────────

#[test]
fn checker_picks_up_floor_from_performance_toml() {
    // The default policy file ships `per_benchmark_floor = 1.0`. If
    // someone later weakens it to 0.5, every existing required entry
    // would suddenly pass — the gate must surface that.
    let dir = unique_dir("policy-floor");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "edge", "bucket": "required",
             "speedup_vs_cpython": 0.6, "kind": "Numeric"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 1, "0.6 must fail under default 1.0 floor");
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["floor"].as_f64(), Some(1.0));
}

// ─── IO / discoverability ───────────────────────────────────────────

#[test]
fn checker_exits_101_when_baseline_missing() {
    let (code, _, _) = run_checker(&[
        "--baseline",
        "/nonexistent/path/missing.json",
        "--format",
        "json",
    ]);
    assert_eq!(code, 101, "missing baseline must exit 101 (IO error)");
}

#[test]
fn checker_help_documents_baseline_format_policy_flags() {
    let (code, stdout, _) = run_checker(&["--help"]);
    assert_eq!(code, 0);
    for opt in ["--baseline", "--format", "--policy"] {
        assert!(stdout.contains(opt), "help must surface {opt}");
    }
}

#[test]
fn shipped_baseline_runs_through_checker_without_crash() {
    // Smoke-test the bundled baseline.json: the checker must run end-
    // to-end on whatever ships today. The current baseline contains
    // benchmarks below 1.0 by design (#2096, #2512, #2513 etc.) — so we
    // assert the checker terminates with a known exit code (0 or 1)
    // and produces valid JSON, not that it exits 0.
    let (code, stdout, _) = run_checker(&["--format", "json"]);
    assert!(
        code == 0 || code == 1,
        "shipped baseline run must exit 0/1, got {code}",
    );
    let v: Value = serde_json::from_str(&stdout).expect("checker must emit JSON");
    assert!(v["floor"].as_f64().is_some());
    assert!(v["violations"].as_array().is_some());
    assert!(v["blocked_below_floor"].as_array().is_some());
    assert!(v["checked_count"].as_i64().is_some());
}
