//! MVP baseline.json tier metadata lock (closes #2566).
//!
//! Parent: #2530 (performance gate suite).
//!
//! Locks the contract of the migrated `projects/mamba/baseline.json` and
//! its companion validator `scripts/baseline_validator.py`. After
//! migration, every benchmark entry carries a `tier ∈ {required,
//! optional, xfail, blocker}` and the validator enforces:
//!
//!     1. Missing tier is a validation error.
//!     2. Required entries can be selected without string-matching
//!        fixture names (--select-tier).
//!     3. Existing v1 baselines remain readable under --legacy-compat.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

fn baseline_path() -> PathBuf {
    crate::common::project_root().join("baseline.json")
}

fn validator_script() -> PathBuf {
    crate::common::project_root().join("scripts").join("baseline_validator.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-baseline-tier-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn write_baseline(dir: &Path, name: &str, value: &Value) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, value.to_string()).expect("write baseline fixture");
    path
}

fn run_validator(args: &[&str]) -> (i32, String, String) {
    let output = crate::common::run_python_script(&validator_script(), args);
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

// ─── Shipped baseline carries tier metadata on every entry ──────────

#[test]
fn shipped_baseline_declares_tier_for_every_entry() {
    let raw = std::fs::read_to_string(baseline_path()).expect("read baseline.json");
    let v: Value = serde_json::from_str(&raw).expect("parse baseline.json");
    let benches = v["benchmarks"].as_array().expect("benchmarks array");
    assert!(!benches.is_empty(), "shipped baseline must not be empty");
    for (idx, e) in benches.iter().enumerate() {
        let tier = e.get("tier").and_then(|t| t.as_str());
        assert!(
            tier.is_some(),
            "entry [{idx}] missing 'tier': {e}",
        );
        let tier = tier.unwrap();
        assert!(
            matches!(tier, "required" | "optional" | "xfail" | "blocker"),
            "entry [{idx}] tier {tier:?} not in legal set",
        );
    }
}

#[test]
fn shipped_baseline_has_at_least_one_required_entry() {
    // Acceptance #2: the required tier must be populated — otherwise
    // every downstream gate (#2565 floor, #2569 geomean) operates on an
    // empty set.
    let raw = std::fs::read_to_string(baseline_path()).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    let count_required = v["benchmarks"]
        .as_array().unwrap()
        .iter()
        .filter(|e| e["tier"].as_str() == Some("required"))
        .count();
    assert!(
        count_required >= 1,
        "shipped baseline must declare at least one required entry, got {count_required}",
    );
}

#[test]
fn shipped_baseline_passes_validator() {
    let (code, _, stderr) = run_validator(&["--format", "text"]);
    assert_eq!(code, 0, "shipped baseline must validate cleanly (stderr={stderr})");
}

// ─── Acceptance 1: missing tier fails validation ────────────────────

#[test]
fn missing_tier_is_a_validation_error() {
    let dir = unique_dir("no-tier");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "no_tier", "kind": "Numeric",
             "mamba_ns": 1, "cpython_ns": 1, "speedup_vs_cpython": 1.0},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, stderr) = run_validator(&[
        "--baseline", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 1, "missing tier must fail (stderr={stderr})");
    assert!(stderr.contains("tier"), "stderr must name the missing field");
    assert!(stderr.contains("no_tier"), "stderr must name the offending entry");
}

#[test]
fn unknown_tier_value_is_a_validation_error() {
    let dir = unique_dir("bad-tier");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "weird", "tier": "exploratory",
             "mamba_ns": 1, "cpython_ns": 1, "speedup_vs_cpython": 1.0},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, stderr) = run_validator(&[
        "--baseline", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 1, "non-enum tier value must fail (stderr={stderr})");
    assert!(stderr.contains("exploratory"));
}

#[test]
fn required_entry_missing_speedup_is_an_error() {
    // Acceptance #1 expansion: required entries must carry timings so
    // downstream tools have ground truth.
    let dir = unique_dir("required-no-speedup");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "incomplete", "tier": "required",
             "mamba_ns": 1, "cpython_ns": 1},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, stderr) = run_validator(&[
        "--baseline", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 1);
    assert!(stderr.contains("speedup_vs_cpython"));
}

#[test]
fn required_entry_missing_timing_is_an_error() {
    let dir = unique_dir("required-no-timing");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "no_timing", "tier": "required", "speedup_vs_cpython": 1.5},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, stderr) = run_validator(&[
        "--baseline", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 1);
    assert!(stderr.contains("mamba_ns") || stderr.contains("cpython_ns"));
}

// ─── Acceptance 2: select required entries by tier, not by name ─────

#[test]
fn select_tier_required_lists_names_one_per_line() {
    let dir = unique_dir("select-required");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "a", "tier": "required", "mamba_ns": 1, "cpython_ns": 1, "speedup_vs_cpython": 1.5},
            {"name": "z", "tier": "required", "mamba_ns": 1, "cpython_ns": 1, "speedup_vs_cpython": 1.5},
            {"name": "m", "tier": "blocker",  "mamba_ns": 1, "cpython_ns": 1, "speedup_vs_cpython": 0.1},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, stdout, _) = run_validator(&[
        "--baseline", path.to_str().unwrap(),
        "--select-tier", "required",
    ]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["a", "z"], "must list required names alphabetically, exclude blocker");
}

#[test]
fn select_tier_blocker_picks_known_blockers_in_shipped_baseline() {
    // Acceptance #2: selection works on the bundled file without
    // string-matching fixture names. The migration assigns the known
    // perf regressions (#2512 string_concat, #2513 list_sort_builtin) to
    // tier=blocker; verify they fall out of --select-tier blocker.
    let (code, stdout, _) = run_validator(&["--select-tier", "blocker"]);
    assert_eq!(code, 0);
    let names: Vec<&str> = stdout.lines().collect();
    assert!(names.contains(&"string_concat"), "blocker tier must include string_concat: {names:?}");
    assert!(names.contains(&"list_sort_builtin"), "blocker tier must include list_sort_builtin: {names:?}");
}

#[test]
fn select_tier_with_unknown_tier_value_exits_usage_error() {
    let (code, _, stderr) = run_validator(&["--select-tier", "fictional"]);
    assert_eq!(code, 100, "unknown tier must exit 100");
    assert!(stderr.contains("fictional"));
}

// ─── Acceptance 3: legacy v1 baselines remain readable ──────────────

#[test]
fn legacy_v1_baseline_loads_under_legacy_compat() {
    // V1 baselines pre-#2566 had no `tier` field. The validator must
    // accept them under --legacy-compat without crashing.
    let dir = unique_dir("legacy-v1");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "v1_entry", "kind": "Numeric",
             "mamba_ns": 1, "cpython_ns": 1, "speedup_vs_cpython": 3.0},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, stdout, _) = run_validator(&[
        "--baseline", path.to_str().unwrap(),
        "--format", "json",
        "--legacy-compat",
    ]);
    assert_eq!(code, 0, "legacy v1 baseline must validate under --legacy-compat");
    let v: Value = serde_json::from_str(&stdout).unwrap();
    let drift = v["drift"].as_array().expect("drift array");
    assert!(!drift.is_empty(), "missing-tier entries must surface as drift, not silently disappear");
}

#[test]
fn legacy_v1_baseline_fails_without_legacy_compat() {
    // Acceptance #3 dual: missing tier WITHOUT --legacy-compat must be
    // a hard error so a CI run doesn't silently accept an un-migrated
    // file.
    let dir = unique_dir("legacy-no-flag");
    let baseline = json!({
        "version": 1,
        "benchmarks": [
            {"name": "v1_entry", "mamba_ns": 1, "cpython_ns": 1,
             "speedup_vs_cpython": 3.0},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let (code, _, _) = run_validator(&[
        "--baseline", path.to_str().unwrap(),
        "--format", "text",
    ]);
    assert_eq!(code, 1, "v1 baseline must fail without --legacy-compat");
}

// ─── Downstream tools still read the migrated baseline ──────────────

#[test]
fn perf_floor_check_reads_tier_from_migrated_baseline() {
    // Cross-tool contract: the floor checker shipped in #2565 must read
    // `tier` on the new v2 baseline. Build a baseline with both tier
    // levels and verify only required entries trigger violations.
    let dir = unique_dir("floor-tier");
    let baseline = json!({
        "version": 2,
        "benchmarks": [
            {"name": "req_slow",   "tier": "required", "mamba_ns": 1, "cpython_ns": 1, "speedup_vs_cpython": 0.5, "kind": "x"},
            {"name": "block_slow", "tier": "blocker",  "mamba_ns": 1, "cpython_ns": 1, "speedup_vs_cpython": 0.1, "kind": "x"},
        ],
    });
    let path = write_baseline(&dir, "baseline.json", &baseline);
    let output = Command::new("python3")
        .arg(crate::common::project_root().join("scripts").join("perf_floor_check.py"))
        .args(["--baseline", path.to_str().unwrap(), "--format", "json"])
        .current_dir(crate::common::project_root())
        .output()
        .expect("invoke perf_floor_check");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(code, 1, "required slow entry must fail floor");
    let v: Value = serde_json::from_str(&stdout).unwrap();
    let viol = v["violations"].as_array().unwrap();
    assert_eq!(viol.len(), 1);
    assert_eq!(viol[0]["name"], json!("req_slow"));
    let blocked = v["blocked_below_floor"].as_array().unwrap();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0]["name"], json!("block_slow"));
}

#[test]
fn baseline_version_bumped_to_2_for_tier_migration() {
    let raw = std::fs::read_to_string(baseline_path()).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    // The tier migration bumped the schema to 2; the v3 memory axis
    // (`schema_notes_v3`) bumped it again. Both changes are additive, so
    // this gate pins a version floor rather than an exact value.
    assert!(
        v["version"].as_i64() >= Some(2),
        "tier migration is a schema-additive change — version must be >= 2, got {:?}",
        v["version"],
    );
}

// ─── IO / discoverability ───────────────────────────────────────────

#[test]
fn validator_exits_101_when_baseline_missing() {
    let (code, _, _) = run_validator(&[
        "--baseline", "/nonexistent/path/missing.json",
        "--format", "json",
    ]);
    assert_eq!(code, 101);
}

#[test]
fn validator_help_documents_select_tier_and_legacy_compat() {
    let (code, stdout, _) = run_validator(&["--help"]);
    assert_eq!(code, 0);
    for opt in ["--baseline", "--format", "--select-tier", "--legacy-compat"] {
        assert!(stdout.contains(opt), "help must surface {opt}");
    }
}
