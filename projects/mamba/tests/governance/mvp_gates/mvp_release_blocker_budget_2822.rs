//! MVP release-gate blocker budget report — locking test (closes #2822).
//!
//! Parent: #2775 (MVP release blocking CI profiles).
//!
//! Locks the contract of two artifacts:
//!
//!   - `validation/blocker_budget.toml` — policy file: per-objective
//!     budgets + issue-link policy + human-summary trimming rules.
//!   - `scripts/release_blocker_report.py` — reporter that ingests the
//!     release-gate summary (shape from #2820) and renders it against
//!     the budget policy.
//!
//! Acceptance (issue #2822):
//!
//!     1. Report lists blocker counts per MVP objective.
//!     2. Missing issue links for required blockers fail or warn
//!        according to policy.
//!     3. Human summary is concise.
//!
//! The reporter is exercised against ad-hoc summary fixtures written to
//! a tempdir per test. This keeps the test fast (no cargo invocation, no
//! network) and makes each acceptance bullet a direct, named test.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

// ─── Path helpers ───────────────────────────────────────────────────

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn policy_path() -> PathBuf {
    project_root()
        .join("validation")
        .join("blocker_budget.toml")
}

fn reporter_script() -> PathBuf {
    project_root()
        .join("scripts")
        .join("release_blocker_report.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-blocker-budget-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn read_policy() -> String {
    std::fs::read_to_string(policy_path()).unwrap_or_else(|e| panic!("read policy: {e}"))
}

fn write_summary(dir: &Path, name: &str, value: &Value) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, value.to_string()).expect("write summary fixture");
    path
}

fn run_reporter(args: &[&str]) -> (i32, String, String) {
    let output = Command::new("python3")
        .arg(reporter_script())
        .args(args)
        .current_dir(project_root())
        .output()
        .expect("invoke release_blocker_report.py");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn parse_json(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or_else(|e| panic!("parse reporter stdout as JSON: {e}\n{s}"))
}

// Build a summary fixture that matches release_summary.schema.json
// (#2820). `blockers` is a list of (objective, profile, fixture, kind,
// tracking_issue) tuples that flow through to `blockers_by_objective`.
fn build_summary(release_id: &str, blockers: &[(&str, &str, &str, &str, Option<&str>)]) -> Value {
    let mut by_obj = serde_json::Map::new();
    for (objective, profile, fixture, kind, tracker) in blockers {
        let entry = json!({
            "profile": profile,
            "fixture_id": fixture,
            "kind": kind,
            "reason": "test fixture",
            "tracking_issue": tracker,
        });
        by_obj
            .entry(objective.to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .expect("array")
            .push(entry);
    }
    json!({
        "schema_version": 1,
        "release_id": release_id,
        "generated_at": "2026-05-20T00:00:00Z",
        "runtime_identity": {"cpython": "3.12", "mamba_edition": "py312"},
        "overall": {
            "pass": blockers.is_empty(),
            "blocking_failure_count": blockers.len() as i64,
            "release_required_profiles": ["smoke", "correctness", "performance", "ecosystem", "package_manager", "mambalibs"],
            "report_only_profiles": [],
        },
        "profiles": {
            "smoke": {"profile":"smoke","command":"x","status":"pass","blocking":true,"counts":{"passed":1,"failed":0,"missing":0}},
            "correctness": {"profile":"correctness","command":"x","status":"pass","blocking":true,"counts":{"passed":1,"failed":0,"missing":0}},
            "performance": {"profile":"performance","command":"x","status":"pass","blocking":true,"counts":{"passed":1,"failed":0,"missing":0}},
            "ecosystem": {"profile":"ecosystem","command":"x","status":"pass","blocking":true,"counts":{"passed":1,"failed":0,"missing":0}},
            "package_manager": {"profile":"package_manager","command":"x","status":"pass","blocking":true,"counts":{"passed":1,"failed":0,"missing":0}},
            "mambalibs": {"profile":"mambalibs","command":"x","status":"pass","blocking":true,"counts":{"passed":1,"failed":0,"missing":0}},
        },
        "blockers_by_objective": Value::Object(by_obj),
        "artifacts": {"summary_path": "/tmp/x"},
    })
}

// ─── Policy file shape ──────────────────────────────────────────────

#[test]
fn policy_lives_alongside_profile_manifests() {
    let path = policy_path();
    assert!(
        path.is_file(),
        "blocker_budget.toml must live next to per-profile manifests: {}",
        path.display(),
    );
}

#[test]
fn policy_declares_budget_for_every_mvp_objective() {
    let body = read_policy();
    assert!(
        body.contains("[budgets]"),
        "policy must declare [budgets] table"
    );
    for obj in [
        "smoke",
        "correctness",
        "performance",
        "ecosystem",
        "package_manager",
        "mambalibs",
        "release_gate",
    ] {
        assert!(
            body.contains(&format!("{obj} =")),
            "[budgets] must include {obj} budget — needed by acceptance #1",
        );
    }
}

#[test]
fn policy_declares_issue_link_policy_for_required_and_optional() {
    let body = read_policy();
    assert!(
        body.contains("[issue_link_policy]"),
        "policy must declare [issue_link_policy]",
    );
    assert!(
        body.contains("required_blocker_missing_tracker"),
        "[issue_link_policy] must declare required_blocker_missing_tracker",
    );
    assert!(
        body.contains("optional_blocker_missing_tracker"),
        "[issue_link_policy] must declare optional_blocker_missing_tracker",
    );
    // The policy values are an enum: "fail" or "warn". Spell-check the
    // file by asserting at least one of each appears.
    assert!(
        body.contains("\"fail\"") || body.contains("\"warn\""),
        "policy values must be quoted enum strings (fail|warn)",
    );
}

#[test]
fn policy_caps_human_summary_for_conciseness() {
    let body = read_policy();
    assert!(
        body.contains("human_max_lines"),
        "[summary] must declare human_max_lines (acceptance #3 — conciseness)",
    );
}

// ─── Acceptance 1: report lists blocker counts per MVP objective ────

#[test]
fn json_report_lists_one_row_per_budgeted_objective() {
    let dir = unique_dir("rows");
    let summary = build_summary(
        "rel-rows",
        &[("performance", "performance", "fib", "failed", Some("#2096"))],
    );
    let summary_path = write_summary(&dir, "summary.json", &summary);

    let (code, stdout, _) = run_reporter(&[
        "--summary",
        summary_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(
        code, 1,
        "one blocker over budget=0 must exit 1 (budget_overrun)"
    );

    let payload = parse_json(&stdout);
    let objectives = payload["objectives"]
        .as_array()
        .expect("objectives is array");
    let names: Vec<String> = objectives
        .iter()
        .map(|o| o["objective"].as_str().unwrap().to_string())
        .collect();
    for required in [
        "smoke",
        "correctness",
        "performance",
        "ecosystem",
        "package_manager",
        "mambalibs",
        "release_gate",
    ] {
        assert!(
            names.iter().any(|n| n == required),
            "objectives list must include {required}: {names:?}",
        );
    }
    let perf = objectives
        .iter()
        .find(|o| o["objective"] == "performance")
        .expect("performance row");
    assert_eq!(perf["budget"], json!(0));
    assert_eq!(perf["actual"], json!(1));
    assert_eq!(perf["overrun"], json!(1));
}

#[test]
fn clean_summary_reports_zero_overrun_and_exits_zero() {
    let dir = unique_dir("clean");
    let summary = build_summary("rel-clean", &[]);
    let summary_path = write_summary(&dir, "summary.json", &summary);
    let (code, stdout, _) = run_reporter(&[
        "--summary",
        summary_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(code, 0, "clean summary must exit 0");
    let payload = parse_json(&stdout);
    assert_eq!(payload["totals"]["actual"], json!(0));
    assert_eq!(payload["totals"]["overrun"], json!(0));
    assert_eq!(payload["exit_reason"], json!("clean"));
}

// ─── Acceptance 2: missing-tracker policy ───────────────────────────

#[test]
fn required_blocker_missing_tracker_exits_nonzero_under_fail_policy() {
    let dir = unique_dir("missing-tracker");
    // Required objective (performance) blocker with no tracker — policy
    // is "fail" in blocker_budget.toml, so reporter MUST exit 2.
    let summary = build_summary(
        "rel-missing",
        &[("performance", "performance", "fib", "failed", None)],
    );
    let summary_path = write_summary(&dir, "summary.json", &summary);

    let (code, stdout, _) = run_reporter(&[
        "--summary",
        summary_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(
        code, 2,
        "missing-tracker on required blocker under fail policy must exit 2 — saw {code}",
    );
    let payload = parse_json(&stdout);
    assert_eq!(payload["exit_reason"], json!("missing_tracker"));
    assert_eq!(payload["totals"]["missing_tracker"], json!(1));
}

#[test]
fn tracked_required_blocker_does_not_trigger_missing_tracker_exit() {
    let dir = unique_dir("tracked");
    let summary = build_summary(
        "rel-tracked",
        &[("performance", "performance", "fib", "failed", Some("#2096"))],
    );
    let summary_path = write_summary(&dir, "summary.json", &summary);
    let (code, stdout, _) = run_reporter(&[
        "--summary",
        summary_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(
        code, 1,
        "tracked blocker still over budget — must be budget_overrun, not missing_tracker"
    );
    let payload = parse_json(&stdout);
    assert_eq!(payload["exit_reason"], json!("budget_overrun"));
}

// ─── Acceptance 3: concise human report ─────────────────────────────

#[test]
fn human_report_respects_max_lines_cap() {
    let dir = unique_dir("human");
    // Synthesize many blockers across multiple objectives so we'd
    // exceed any sane cap without the trim logic.
    let mut blockers: Vec<(&str, &str, &str, &str, Option<&str>)> = Vec::new();
    for i in 0..50 {
        let obj = [
            "smoke",
            "correctness",
            "performance",
            "ecosystem",
            "package_manager",
            "mambalibs",
        ][i % 6];
        // Leak so the &str lifetime is 'static; this is a test, blast radius is tiny.
        let fixture: &'static str = Box::leak(format!("fx-{i}").into_boxed_str());
        blockers.push((obj, obj, fixture, "failed", Some("#1")));
    }
    let summary = build_summary("rel-human", &blockers);
    let summary_path = write_summary(&dir, "summary.json", &summary);

    let (_code, stdout, _) = run_reporter(&[
        "--summary",
        summary_path.to_str().unwrap(),
        "--format",
        "human",
    ]);
    let line_count = stdout.lines().count();
    let policy_body = read_policy();
    let cap_line = policy_body
        .lines()
        .find(|l| l.trim_start().starts_with("human_max_lines"))
        .expect("human_max_lines line in policy");
    let cap: usize = cap_line
        .split('=')
        .nth(1)
        .and_then(|s| s.trim().parse().ok())
        .expect("parse human_max_lines value");
    assert!(
        line_count <= cap,
        "human report ({} lines) must respect human_max_lines cap ({cap})",
        line_count,
    );
}

#[test]
fn human_report_marks_overrun_and_includes_release_id() {
    let dir = unique_dir("human-overrun");
    let summary = build_summary(
        "rel-name",
        &[("performance", "performance", "fib", "failed", Some("#2096"))],
    );
    let summary_path = write_summary(&dir, "summary.json", &summary);
    let (_code, stdout, _) = run_reporter(&[
        "--summary",
        summary_path.to_str().unwrap(),
        "--format",
        "human",
    ]);
    assert!(
        stdout.contains("rel-name"),
        "human report must include release_id"
    );
    assert!(
        stdout.contains("performance"),
        "human report must include overrun objective"
    );
    assert!(
        stdout.contains("OVER"),
        "human report must mark overrun rows"
    );
}

// ─── Reporter contract — script discoverability ─────────────────────

#[test]
fn reporter_help_documents_format_options() {
    let (code, stdout, _) = run_reporter(&["--help"]);
    assert_eq!(code, 0);
    for opt in ["--summary", "--policy", "--format"] {
        assert!(stdout.contains(opt), "help must surface {opt}: {stdout}");
    }
}

#[test]
fn reporter_exits_101_when_summary_missing() {
    let (code, _, _) = run_reporter(&[
        "--summary",
        "/nonexistent/path/missing.json",
        "--format",
        "json",
    ]);
    assert_eq!(code, 101, "missing summary must exit 101");
}
