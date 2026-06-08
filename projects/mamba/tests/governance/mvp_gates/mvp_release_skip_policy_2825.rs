//! MVP release-gate skip/xfail/Stub/ImportPass policy check (closes #2825).
//!
//! Parent: #2775 (MVP release blocking CI profiles).
//!
//! Locks the contract of `scripts/skip_policy_check.py`, the enforcer
//! that walks a release-gate summary (#2820 shape) and ensures every
//! release-required profile reports zero in the four non-passing
//! outcome buckets:
//!
//!     skipped, xfail, stub, import_pass
//!
//! Acceptance (issue #2825):
//!
//!     1. Synthetic summary with a skipped required test fails.
//!     2. Synthetic summary with all AssertionPass items passes.
//!     3. Failure output is actionable for worker triage.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn checker_script() -> PathBuf {
    crate::common::project_root().join("scripts").join("skip_policy_check.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-skip-policy-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn write_summary(dir: &Path, name: &str, value: &Value) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, value.to_string()).expect("write summary fixture");
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

/// Build a release summary with explicit per-profile counts. `counts` is
/// a list of (profile, passed, failed, missing, skipped, xfail, stub,
/// import_pass) tuples for release-required profiles.
fn build_summary(
    release_id: &str,
    counts: &[(&str, i64, i64, i64, i64, i64, i64, i64)],
) -> Value {
    let mut profiles = serde_json::Map::new();
    let mut required: Vec<&str> = Vec::new();
    for (pid, passed, failed, missing, skipped, xfail, stub, import_pass) in counts {
        required.push(pid);
        profiles.insert(pid.to_string(), json!({
            "profile": pid,
            "command": "x",
            "status": if *failed > 0 { "fail" } else { "pass" },
            "blocking": true,
            "counts": {
                "passed": passed,
                "failed": failed,
                "missing": missing,
                "skipped": skipped,
                "xfail": xfail,
                "stub": stub,
                "import_pass": import_pass,
            },
        }));
    }
    json!({
        "schema_version": 1,
        "release_id": release_id,
        "generated_at": "2026-05-20T00:00:00Z",
        "runtime_identity": {"cpython": "3.12", "mamba_edition": "py312"},
        "overall": {
            "pass": counts.iter().all(|c| c.2 == 0),
            "blocking_failure_count": counts.iter().map(|c| c.2).sum::<i64>(),
            "release_required_profiles": required,
            "report_only_profiles": [],
        },
        "profiles": Value::Object(profiles),
        "blockers_by_objective": {},
        "artifacts": {"summary_path": "/tmp/x"},
    })
}

// ─── Acceptance 1: skip / xfail / stub / import_pass on required → fail

#[test]
fn skipped_count_on_required_profile_fails() {
    let dir = unique_dir("skipped");
    let summary = build_summary(
        "rel-skip",
        &[("smoke", 5, 0, 0, /*skipped*/ 3, 0, 0, 0)],
    );
    let path = write_summary(&dir, "summary.json", &summary);
    let (code, _, stderr) = run_checker(&[
        "--summary", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 1, "skipped > 0 on required profile must fail (stderr={stderr})");
    assert!(stderr.contains("skipped"), "stderr must name the skipped outcome");
    assert!(stderr.contains("smoke"), "stderr must name the offending profile");
}

#[test]
fn xfail_count_on_required_profile_fails() {
    let dir = unique_dir("xfail");
    let summary = build_summary(
        "rel-xfail",
        &[("correctness", 100, 0, 0, 0, /*xfail*/ 4, 0, 0)],
    );
    let path = write_summary(&dir, "summary.json", &summary);
    let (code, _, stderr) = run_checker(&[
        "--summary", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 1, "xfail > 0 on required profile must fail");
    assert!(stderr.contains("xfail"));
    assert!(stderr.contains("correctness"));
}

#[test]
fn stub_count_on_required_profile_fails() {
    let dir = unique_dir("stub");
    let summary = build_summary(
        "rel-stub",
        &[("correctness", 100, 0, 0, 0, 0, /*stub*/ 2, 0)],
    );
    let path = write_summary(&dir, "summary.json", &summary);
    let (code, _, stderr) = run_checker(&[
        "--summary", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 1, "stub > 0 on required profile must fail");
    assert!(stderr.contains("stub"));
}

#[test]
fn import_pass_count_on_required_profile_fails() {
    let dir = unique_dir("import-pass");
    let summary = build_summary(
        "rel-import",
        &[("correctness", 100, 0, 0, 0, 0, 0, /*import_pass*/ 8)],
    );
    let path = write_summary(&dir, "summary.json", &summary);
    let (code, _, stderr) = run_checker(&[
        "--summary", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 1, "import_pass > 0 on required profile must fail");
    assert!(stderr.contains("import_pass"));
}

// ─── Acceptance 2: clean summary passes ─────────────────────────────

#[test]
fn clean_summary_with_only_passed_counts_exits_zero() {
    let dir = unique_dir("clean");
    let summary = build_summary(
        "rel-clean",
        &[
            ("smoke", 5, 0, 0, 0, 0, 0, 0),
            ("correctness", 100, 0, 0, 0, 0, 0, 0),
            ("performance", 20, 0, 0, 0, 0, 0, 0),
            ("ecosystem", 28, 0, 0, 0, 0, 0, 0),
            ("package_manager", 8, 0, 0, 0, 0, 0, 0),
            ("mambalibs", 12, 0, 0, 0, 0, 0, 0),
        ],
    );
    let path = write_summary(&dir, "summary.json", &summary);
    let (code, _, stderr) = run_checker(&[
        "--summary", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 0, "all-passed summary must exit 0 (stderr={stderr})");
    assert!(stderr.contains("clean"), "text mode must announce clean");
}

// ─── Acceptance 3: actionable failure output ────────────────────────

#[test]
fn json_format_emits_machine_readable_violations() {
    let dir = unique_dir("json");
    let summary = build_summary(
        "rel-multi",
        &[
            ("smoke", 5, 0, 0, /*skipped*/ 1, 0, 0, 0),
            ("correctness", 100, 0, 0, 0, /*xfail*/ 4, /*stub*/ 2, /*import_pass*/ 8),
        ],
    );
    let path = write_summary(&dir, "summary.json", &summary);
    let (code, stdout, _) = run_checker(&[
        "--summary", path.to_str().unwrap(),
        "--format", "json",
    ]);
    assert_eq!(code, 1);
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    assert_eq!(v["release_id"], json!("rel-multi"));
    let violations = v["violations"].as_array().expect("violations array");

    // Four buckets across two profiles → exactly 4 violations.
    assert_eq!(violations.len(), 4, "expected 4 violations, got {violations:?}");

    // Every violation row must name profile + outcome + count + objective.
    for entry in violations {
        for key in ["profile", "outcome", "count", "objective"] {
            assert!(
                entry.get(key).is_some(),
                "violation must surface {key}: {entry}",
            );
        }
    }

    // Stable sort order: (profile alpha, outcome alpha). Critical for
    // diff-friendly worker output (acceptance #3 — "actionable").
    let labels: Vec<String> = violations
        .iter()
        .map(|v| format!("{}/{}", v["profile"].as_str().unwrap(), v["outcome"].as_str().unwrap()))
        .collect();
    let mut sorted = labels.clone();
    sorted.sort();
    assert_eq!(labels, sorted, "violations must be sorted by (profile, outcome)");
}

#[test]
fn text_output_names_rule_and_issue() {
    let dir = unique_dir("rule-cite");
    let summary = build_summary("rel-x", &[("smoke", 5, 0, 0, 1, 0, 0, 0)]);
    let path = write_summary(&dir, "summary.json", &summary);
    let (_code, _, stderr) = run_checker(&[
        "--summary", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert!(stderr.contains("#2825"), "text output must cite tracking issue for triage");
    assert!(stderr.contains("rule:"), "text output must restate the rule");
}

// ─── Missing-profile rendering ──────────────────────────────────────

#[test]
fn release_required_profile_missing_from_summary_is_violation() {
    // The runner may have crashed before recording one profile. If the
    // overall block claims it as release-required but `profiles` has no
    // entry, the checker must surface that as a violation, not silently
    // ignore it.
    let dir = unique_dir("missing");
    let summary = json!({
        "schema_version": 1,
        "release_id": "rel-missing-profile",
        "generated_at": "2026-05-20T00:00:00Z",
        "runtime_identity": {"cpython": "3.12", "mamba_edition": "py312"},
        "overall": {
            "pass": false,
            "blocking_failure_count": 0,
            "release_required_profiles": ["smoke", "performance"],
            "report_only_profiles": [],
        },
        "profiles": {
            "smoke": {"profile":"smoke","command":"x","status":"pass","blocking":true,
                      "counts":{"passed":1,"failed":0,"missing":0}}
            // performance entry deliberately absent
        },
        "blockers_by_objective": {},
        "artifacts": {"summary_path": "/tmp/x"},
    });
    let path = write_summary(&dir, "summary.json", &summary);
    let (code, stdout, _) = run_checker(&[
        "--summary", path.to_str().unwrap(),
        "--format", "json",
    ]);
    assert_eq!(code, 1, "missing required profile must be a violation");
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    let violations = v["violations"].as_array().expect("violations");
    assert!(
        violations.iter().any(|v|
            v["profile"] == "performance"
            && v["outcome"] == "missing"
        ),
        "violation for missing required profile must be reported: {violations:?}",
    );
}

// ─── Checker discoverability ────────────────────────────────────────

#[test]
fn checker_help_documents_summary_and_format_flags() {
    let (code, stdout, _) = run_checker(&["--help"]);
    assert_eq!(code, 0);
    for opt in ["--summary", "--format"] {
        assert!(stdout.contains(opt), "help must surface {opt}");
    }
}

#[test]
fn checker_exits_101_when_summary_missing() {
    let (code, _, _) = run_checker(&[
        "--summary", "/nonexistent/path/missing.json",
        "--format", "json",
    ]);
    assert_eq!(code, 101, "missing summary must exit 101");
}
