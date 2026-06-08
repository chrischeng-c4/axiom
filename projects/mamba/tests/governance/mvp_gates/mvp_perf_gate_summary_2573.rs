//! MVP perf gate machine-readable summary (closes #2573).
//!
//! Parent: #2530 (performance gate suite).
//!
//! Locks the schema of `scripts/perf_gate_summary.py`. The summary
//! stitches together baseline tier metadata (#2566), per-benchmark
//! floor (#2565), suite geomean (#2569), and CPython identity
//! (#2572) into one JSON document a CI worker can parse without
//! scraping logs.
//!
//! Acceptance (issue #2573):
//!
//!     1. CI or worker scripts can parse the JSON without scraping
//!        logs.
//!     2. Summary includes enough data to identify the slowest
//!        blockers.
//!     3. A regression test covers the JSON shape.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn summary_script() -> PathBuf {
    project_root().join("scripts").join("perf_gate_summary.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-perf-gate-summary-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn write_baseline(dir: &Path, name: &str, body: &Value) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body.to_string()).unwrap();
    p
}

fn write_identity(dir: &Path, name: &str, body: &Value) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body.to_string()).unwrap();
    p
}

fn run_summary(args: &[&str]) -> (i32, String, String) {
    let output = Command::new("python3")
        .arg(summary_script())
        .args(args)
        .current_dir(project_root())
        .env_remove("MAMBA_PERF_LOCAL_DEBUG_OVERRIDE")
        .output()
        .expect("invoke perf_gate_summary.py");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn run_summary_json(args: &[&str]) -> (i32, Value) {
    let (code, stdout, stderr) = run_summary(args);
    let payload: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "summary JSON parse failed (code={code}): {e}\n--stdout--\n{stdout}\n--stderr--\n{stderr}"
            )
        });
    (code, payload)
}

fn passing_identity_for(dir: &Path) -> PathBuf {
    write_identity(
        dir,
        "id.json",
        &json!({
            "executable": "/fake/python3.12",
            "version": "3.12.4 (main, ...)",
            "version_major_minor": "3.12",
            "implementation_name": "cpython",
        }),
    )
}

// ─── Acceptance 1: JSON is parseable + complete ──────────────────

#[test]
fn summary_top_level_field_set_is_locked() {
    let dir = unique_dir("locked-fields");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "tier": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (_, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    for key in &[
        "schema_version",
        "benchmark_count",
        "by_tier",
        "release_required_buckets",
        "pass_count",
        "fail_count",
        "required_checked_count",
        "per_benchmark",
        "floor_result",
        "suite_result",
        "cpython_identity",
        "gates",
        "overall_passed",
        "exit_code",
    ] {
        assert!(
            payload.get(key).is_some(),
            "summary missing locked top-level field {key}; payload={payload}"
        );
    }
}

#[test]
fn floor_result_carries_threshold_violations_count_slowest_blockers_and_passed() {
    let dir = unique_dir("floor-shape");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "tier": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (_, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    for key in &[
        "threshold",
        "violations_count",
        "slowest_blockers",
        "passed",
    ] {
        assert!(
            payload["floor_result"].get(key).is_some(),
            "floor_result missing {key}"
        );
    }
}

#[test]
fn suite_result_carries_floor_method_actual_passed_checked_count() {
    let dir = unique_dir("suite-shape");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "tier": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (_, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    for key in &[
        "floor",
        "method",
        "actual_geomean",
        "passed",
        "checked_count",
    ] {
        assert!(
            payload["suite_result"].get(key).is_some(),
            "suite_result missing {key}"
        );
    }
}

#[test]
fn cpython_identity_block_carries_executable_version_and_implementation() {
    let dir = unique_dir("cpython-shape");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "tier": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (_, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    let ci = &payload["cpython_identity"];
    for key in &[
        "executable",
        "version",
        "version_major_minor",
        "implementation_name",
        "required_cpython",
        "matches",
        "override_active",
    ] {
        assert!(ci.get(key).is_some(), "cpython_identity missing {key}");
    }
    assert_eq!(ci["version_major_minor"], "3.12");
    assert_eq!(ci["implementation_name"], "cpython");
    assert_eq!(ci["matches"], true);
}

#[test]
fn per_benchmark_array_has_name_speedup_tier_and_passed_floor_bit() {
    let dir = unique_dir("perbench-shape");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "tier": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
            {"name": "b", "kind": "Numeric", "tier": "blocker",
             "tracking_issue": "#1",
             "mamba_ns": 1000, "cpython_ns": 100, "speedup_vs_cpython": 0.1},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (_, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    let per = payload["per_benchmark"].as_array().unwrap();
    assert_eq!(per.len(), 2);
    for entry in per {
        for key in &["name", "speedup_vs_cpython", "tier", "passed_floor", "kind"] {
            assert!(entry.get(key).is_some(), "per_benchmark missing {key}");
        }
    }
    // The required entry passed its floor.
    let a = per.iter().find(|e| e["name"] == "a").unwrap();
    assert_eq!(a["passed_floor"], true);
    // The blocker entry's floor result is null (not counted, not gating).
    let b = per.iter().find(|e| e["name"] == "b").unwrap();
    assert!(b["passed_floor"].is_null());
}

// ─── Acceptance 2: slowest blockers identify worst contributors ───

#[test]
fn slowest_blockers_lists_required_entries_below_floor_ascending() {
    let dir = unique_dir("slowest");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "fast", "kind": "Numeric", "tier": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
            {"name": "mid_slow", "kind": "Numeric", "tier": "required",
             "mamba_ns": 1000, "cpython_ns": 500, "speedup_vs_cpython": 0.5},
            {"name": "very_slow", "kind": "Numeric", "tier": "required",
             "mamba_ns": 10000, "cpython_ns": 100, "speedup_vs_cpython": 0.01},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (code, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    assert_eq!(code, 1, "floor failures must gate");
    let slowest = payload["floor_result"]["slowest_blockers"]
        .as_array()
        .unwrap();
    assert_eq!(slowest.len(), 2);
    assert_eq!(slowest[0]["name"], "very_slow");
    assert_eq!(slowest[1]["name"], "mid_slow");
    assert_eq!(payload["floor_result"]["violations_count"], 2);
    assert_eq!(payload["fail_count"], 2);
    assert_eq!(payload["pass_count"], 1);
}

#[test]
fn slowest_blockers_truncates_to_at_most_five() {
    let dir = unique_dir("top5");
    let mut entries = vec![];
    for i in 0..10 {
        entries.push(json!({
            "name": format!("slow_{i:02}"), "kind": "Numeric", "tier": "required",
            "mamba_ns": 1000, "cpython_ns": 100,
            "speedup_vs_cpython": 0.1 - 0.01 * (i as f64),
        }));
    }
    let baseline = json!({"version": 2, "benchmarks": entries});
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (_, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    let slowest = payload["floor_result"]["slowest_blockers"]
        .as_array()
        .unwrap();
    assert_eq!(slowest.len(), 5, "slowest_blockers truncates to top 5");
}

// ─── Gate composition: every constituent gate surfaces ────────────

#[test]
fn gates_block_includes_each_constituent_gate_bit() {
    let dir = unique_dir("gates");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "tier": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (code, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    let gates = &payload["gates"];
    for key in &[
        "baseline_version_ok",
        "cpython_identity_ok",
        "per_benchmark_floor_ok",
        "suite_geomean_ok",
    ] {
        assert!(gates.get(key).is_some(), "gates missing {key}");
    }
    assert_eq!(gates["baseline_version_ok"], true);
    assert_eq!(gates["cpython_identity_ok"], true);
    assert_eq!(gates["per_benchmark_floor_ok"], true);
    assert_eq!(gates["suite_geomean_ok"], true);
    assert_eq!(payload["overall_passed"], true);
}

#[test]
fn legacy_v1_baseline_marks_baseline_version_ok_false() {
    let dir = unique_dir("v1");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "bucket": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (code, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    assert_eq!(code, 1, "legacy v1 must fail gates");
    assert_eq!(payload["gates"]["baseline_version_ok"], false);
    assert_eq!(payload["overall_passed"], false);
}

#[test]
fn suite_geomean_below_floor_blocks_overall_pass() {
    let dir = unique_dir("suite-fail");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "tier": "required",
             "mamba_ns": 500, "cpython_ns": 1000, "speedup_vs_cpython": 2.0},
            {"name": "b", "kind": "Numeric", "tier": "required",
             "mamba_ns": 800, "cpython_ns": 1000, "speedup_vs_cpython": 1.25},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (code, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    assert_eq!(code, 1, "suite below 10x must gate");
    // floor passes (both ≥ 1.0×) but suite geomean fails (sqrt(2 * 1.25) ≈ 1.58).
    assert_eq!(payload["gates"]["per_benchmark_floor_ok"], true);
    assert_eq!(payload["gates"]["suite_geomean_ok"], false);
    let geomean = payload["suite_result"]["actual_geomean"].as_f64().unwrap();
    assert!(
        geomean > 1.5 && geomean < 1.7,
        "geomean(2, 1.25) should be ~1.58; got {geomean}"
    );
}

#[test]
fn cpython_mismatch_blocks_gate_without_override() {
    let dir = unique_dir("cpython-mismatch");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "tier": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = write_identity(
        &dir,
        "id.json",
        &json!({
            "executable": "/fake/python3.11",
            "version": "3.11.7",
            "version_major_minor": "3.11",
            "implementation_name": "cpython",
        }),
    );
    let (code, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    assert_eq!(code, 1);
    assert_eq!(payload["gates"]["cpython_identity_ok"], false);
    assert_eq!(payload["cpython_identity"]["matches"], false);
}

#[test]
fn cpython_mismatch_with_local_debug_override_still_passes_cpython_gate() {
    let dir = unique_dir("cpython-override");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "kind": "Numeric", "tier": "required",
             "mamba_ns": 100, "cpython_ns": 1000, "speedup_vs_cpython": 10.0},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = write_identity(
        &dir,
        "id.json",
        &json!({
            "executable": "/fake/python3.11",
            "version": "3.11.7",
            "version_major_minor": "3.11",
            "implementation_name": "cpython",
        }),
    );
    let (code, payload) = run_summary_json(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
        "--local-debug-override",
    ]);
    assert_eq!(code, 0);
    assert_eq!(payload["gates"]["cpython_identity_ok"], true);
    assert_eq!(payload["cpython_identity"]["matches"], false);
    assert_eq!(payload["cpython_identity"]["override_active"], true);
}

// ─── CLI surface + robustness ────────────────────────────────────

#[test]
fn summary_exits_101_when_baseline_missing() {
    let dir = unique_dir("missing");
    let id_path = passing_identity_for(&dir);
    let (code, _stdout, stderr) = run_summary(&[
        "--baseline",
        "/tmp/perf-summary-baseline-missing.json",
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
    ]);
    assert_eq!(code, 101);
    assert!(stderr.contains("baseline missing"));
}

#[test]
fn summary_text_mode_lists_slowest_blockers_and_gate_status() {
    let dir = unique_dir("text-mode");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "rogue", "kind": "Numeric", "tier": "required",
             "mamba_ns": 1000, "cpython_ns": 100, "speedup_vs_cpython": 0.1},
        ],
    });
    let baseline_path = write_baseline(&dir, "baseline.json", &baseline);
    let id_path = passing_identity_for(&dir);
    let (code, _stdout, stderr) = run_summary(&[
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--cpython-identity-json",
        id_path.to_str().unwrap(),
        "--format",
        "text",
    ]);
    assert_eq!(code, 1);
    assert!(stderr.contains("perf_gate_summary"));
    assert!(stderr.contains("overall=FAIL"));
    assert!(
        stderr.contains("slowest blockers"),
        "text output must list slowest blockers; got {stderr}"
    );
    assert!(stderr.contains("name=rogue"));
}

#[test]
fn help_documents_baseline_policy_identity_and_override_flags() {
    let (code, stdout, _stderr) = run_summary(&["--help"]);
    assert_eq!(code, 0);
    for flag in &[
        "--baseline",
        "--policy",
        "--cpython-identity-json",
        "--local-debug-override",
        "--format",
    ] {
        assert!(
            stdout.contains(flag),
            "--help must document {flag}; got {stdout}"
        );
    }
}
