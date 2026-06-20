//! MVP performance 10× suite geomean checker lock (closes #2569).
//!
//! Parent: #2530 (performance gate suite).
//!
//! Locks the contract of `scripts/perf_suite_geomean_check.py`, which
//! enforces the suite-level perf rule declared by
//! `validation/profiles/performance.toml`:
//!
//!     geomean(speedup_vs_cpython for required benchmarks) >= 10.0
//!
//! Acceptance (issue #2569):
//!
//!     1. Required suite geomean below 10.0 fails.
//!     2. Required suite geomean at or above 10.0 passes.
//!     3. Summary names benchmark count, averaging method, and CPython
//!        version.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn checker_script() -> PathBuf {
    crate::common::project_root()
        .join("scripts")
        .join("perf_suite_geomean_check.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-perf-geomean-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn write_baseline(dir: &Path, name: &str, value: &Value) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, value.to_string()).expect("write baseline fixture");
    path
}

fn run_checker(args: &[&str]) -> (i32, String, String) {
    let output = crate::common::run_python_script(&checker_script(), args);
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

// ─── Acceptance 1: geomean < 10.0 → fail ────────────────────────────

#[test]
fn suite_geomean_below_floor_fails() {
    let dir = unique_dir("below");
    // geomean(0.5, 1.5, 2.0) ≈ 1.14 — well below 10.0
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "a", "bucket": "required", "speedup_vs_cpython": 0.5,  "kind": "x"},
            {"name": "b", "bucket": "required", "speedup_vs_cpython": 1.5,  "kind": "x"},
            {"name": "c", "bucket": "required", "speedup_vs_cpython": 2.0,  "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, stderr) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(
        code, 1,
        "geomean ≈ 1.14× must fail under 10× floor (stderr={stderr})"
    );
    assert!(stderr.contains("rule:"), "must restate the rule");
    assert!(stderr.contains("#2569"), "must cite the issue");
    assert!(stderr.contains("worst contributors"));
}

#[test]
fn empty_required_set_fails_rather_than_vacuous_pass() {
    // Acceptance #1 edge: an empty required set must NOT silently pass
    // by treating geomean(∅) as ∞ or 1.0. We document it as 0.0× and fail.
    let dir = unique_dir("empty");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "skip_me", "bucket": "optional", "speedup_vs_cpython": 100.0, "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 1, "empty required set must fail");
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["actual"].as_f64(), Some(0.0));
    assert_eq!(v["checked_count"].as_i64(), Some(0));
}

// ─── Acceptance 2: geomean ≥ 10.0 → pass ───────────────────────────

#[test]
fn suite_geomean_at_floor_passes() {
    let dir = unique_dir("exact");
    // geomean(10.0, 10.0, 10.0) = 10.0
    let baseline = json!({
        "version": 1,
        "benchmarks": (0..3).map(|i| json!({
            "name": format!("b{i}"),
            "bucket": "required",
            "speedup_vs_cpython": 10.0,
            "kind": "Workload",
        })).collect::<Vec<_>>(),
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, stderr) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 0, "exact-floor geomean must pass (stderr={stderr})");
    assert!(stderr.contains("clean"));
}

#[test]
fn suite_geomean_well_above_floor_passes() {
    let dir = unique_dir("above");
    // geomean(20, 30, 40, 50) ≈ 33 — well above 10
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "a", "bucket": "required", "speedup_vs_cpython": 20.0, "kind": "x"},
            {"name": "b", "bucket": "required", "speedup_vs_cpython": 30.0, "kind": "x"},
            {"name": "c", "bucket": "required", "speedup_vs_cpython": 40.0, "kind": "x"},
            {"name": "d", "bucket": "required", "speedup_vs_cpython": 50.0, "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, _) = run_checker(&["--baseline", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 0);
}

#[test]
fn non_required_benchmarks_excluded_from_average() {
    // Optional/blocker entries must NOT skew the suite geomean — that
    // is exactly what `[policy].average_includes = ["required"]` pins.
    let dir = unique_dir("excluded");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "ok",   "bucket": "required", "speedup_vs_cpython": 20.0,  "kind": "x"},
            // optional 0.001× must NOT drag the suite geomean below 10×:
            {"name": "slow", "bucket": "optional", "speedup_vs_cpython": 0.001, "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(code, 0, "optional outliers must not gate");
    assert_eq!(v["checked_count"].as_i64(), Some(1));
    assert!(
        v["actual"].as_f64().unwrap() > 15.0,
        "geomean must reflect only required"
    );
}

// ─── Acceptance 3: summary names count, method, cpython version ─────

#[test]
fn json_summary_names_count_method_and_cpython_version() {
    let dir = unique_dir("summary-fields");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "x", "bucket": "required", "speedup_vs_cpython": 15.0, "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (_code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    let v: Value = serde_json::from_str(&stdout).unwrap();
    for key in [
        "checked_count",
        "method",
        "cpython_version",
        "actual",
        "suite_average_floor",
    ] {
        assert!(v.get(key).is_some(), "summary must surface {key}: {v}");
    }
    assert_eq!(v["method"], json!("geometric_mean"));
    assert_eq!(v["cpython_version"], json!("3.12"));
    assert_eq!(v["suite_average_floor"].as_f64(), Some(10.0));
}

#[test]
fn text_summary_names_count_method_and_cpython_version() {
    let dir = unique_dir("text-summary");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "x", "bucket": "required", "speedup_vs_cpython": 15.0, "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (_code, _, stderr) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "text"]);
    assert!(stderr.contains("cpython=3.12"));
    assert!(stderr.contains("method=geometric_mean"));
    assert!(stderr.contains("checked=1"));
}

#[test]
fn failure_lists_worst_contributors_for_triage() {
    // Acceptance #1 expanded: "Summary names benchmark count, averaging
    // method, and CPython version" — but on failure the worker also
    // needs to know WHICH benchmarks dragged the geomean down. Verify
    // the worst contributors are listed and sorted slowest-first.
    let dir = unique_dir("worst-contrib");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "a_slow",  "bucket": "required", "speedup_vs_cpython": 0.001, "kind": "x"},
            {"name": "b_ok",    "bucket": "required", "speedup_vs_cpython": 5.0,   "kind": "x"},
            {"name": "c_medium","bucket": "required", "speedup_vs_cpython": 1.5,   "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 1);
    let v: Value = serde_json::from_str(&stdout).unwrap();
    let worst = v["worst_contributors"].as_array().unwrap();
    assert!(!worst.is_empty(), "must list contributors on failure");
    assert_eq!(worst[0]["name"], json!("a_slow"), "slowest must come first");
}

// ─── Math correctness sanity ────────────────────────────────────────

#[test]
fn geomean_is_correct_for_known_set() {
    // Standard sanity check: geomean(1, 4, 16) = 4.0
    let dir = unique_dir("geomean-math");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "a", "bucket": "required", "speedup_vs_cpython": 1.0,  "kind": "x"},
            {"name": "b", "bucket": "required", "speedup_vs_cpython": 4.0,  "kind": "x"},
            {"name": "c", "bucket": "required", "speedup_vs_cpython": 16.0, "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (_code, stdout, _) =
        run_checker(&["--baseline", path.to_str().unwrap(), "--format", "json"]);
    let v: Value = serde_json::from_str(&stdout).unwrap();
    let actual = v["actual"].as_f64().unwrap();
    assert!(
        (actual - 4.0).abs() < 1e-6,
        "geomean(1, 4, 16) must equal 4.0, got {actual}",
    );
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
    assert_eq!(code, 101);
}

#[test]
fn checker_help_documents_baseline_format_policy_flags() {
    let (code, stdout, _) = run_checker(&["--help"]);
    assert_eq!(code, 0);
    for opt in ["--baseline", "--policy", "--format"] {
        assert!(stdout.contains(opt), "help must surface {opt}");
    }
}

#[test]
fn shipped_baseline_runs_through_checker_without_crash() {
    // Smoke-test: the bundled baseline.json may not currently meet 10×
    // (and that's fine — we have open issues for that). Check that the
    // script runs end-to-end and produces valid JSON.
    let (code, stdout, _) = run_checker(&["--format", "json"]);
    assert!(code == 0 || code == 1, "must exit 0 or 1, got {code}");
    let v: Value = serde_json::from_str(&stdout).expect("must emit JSON");
    assert!(v["actual"].as_f64().is_some());
    assert_eq!(v["method"], json!("geometric_mean"));
    assert_eq!(v["cpython_version"], json!("3.12"));
}
