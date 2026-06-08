//! MVP release-gate flaky test quarantine policy lock (closes #2824).
//!
//! Parent: #2775 (MVP release blocking CI profiles).
//!
//! Locks the contract of `validation/flaky_quarantine_policy.toml` and
//! `scripts/flaky_quarantine_check.py`, the policy + validator pair that
//! decide how flaky tests are handled in the release gate without
//! silently hiding release blockers.
//!
//! Acceptance (issue #2824):
//!
//!     1. Required flaky item without issue link fails policy validation.
//!     2. Quarantined item appears in release summary and blocker budget.
//!     3. No flaky item is silently dropped.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn policy_path() -> PathBuf {
    project_root()
        .join("validation")
        .join("flaky_quarantine_policy.toml")
}

fn checker_script() -> PathBuf {
    project_root()
        .join("scripts")
        .join("flaky_quarantine_check.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-flaky-quarantine-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn run_checker(args: &[&str]) -> (i32, String, String) {
    let output = Command::new("python3")
        .arg(checker_script())
        .args(args)
        .current_dir(project_root())
        .output()
        .expect("invoke flaky_quarantine_check.py");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn load_policy_toml() -> toml::Value {
    let raw = std::fs::read_to_string(policy_path()).expect("read flaky_quarantine_policy.toml");
    raw.parse::<toml::Value>().expect("parse policy TOML")
}

/// Helper: write a policy file under `dir` whose body is `header + entries`.
/// The header mirrors the real policy's required structure so test fixtures
/// only need to vary the `[[quarantine]]` entries they care about.
fn write_test_policy(dir: &Path, name: &str, entries: &str) -> PathBuf {
    let header = r#"
version = 1
profile = "release_gate"
parent_issue = 2775
issue = 2824

buckets = ["required", "optional", "xfail", "blocker"]

[status]
enum = ["active", "investigating", "fixed", "released"]
default = "active"
blocker_visible = ["active", "investigating"]
audit_history = ["released"]

[validation]
require_tracking_issue = true
required_buckets = ["required", "blocker"]
issue_link_regex = "^(#[0-9]+|https?://.+)$"
silent_drop_action = "block"
required_fields_required = ["id", "bucket", "status", "owner_profile", "tracking_issue"]
required_fields_optional = ["id", "bucket", "status", "owner_profile"]

[summary_emission]
blocker_kind = "flaky_quarantine"
[summary_emission.objective_map]
smoke = "smoke"
correctness = "correctness"
performance = "performance"
ecosystem = "ecosystem"
package_manager = "package_manager"
mambalibs = "mambalibs"

[summary_emission.never_drop]
status_in = ["active", "investigating", "fixed", "released"]
bucket_in = ["required", "optional", "xfail", "blocker"]
"#;
    let body = format!("{header}\n{entries}\n");
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write test policy");
    path
}

// ─── Header / schema lock ───────────────────────────────────────────

#[test]
fn policy_header_pins_issue_and_parent() {
    let v = load_policy_toml();
    assert_eq!(v["version"].as_integer(), Some(1));
    assert_eq!(v["profile"].as_str(), Some("release_gate"));
    assert_eq!(v["parent_issue"].as_integer(), Some(2775));
    assert_eq!(v["issue"].as_integer(), Some(2824));
}

#[test]
fn policy_declares_four_status_enum_with_visible_subset() {
    let v = load_policy_toml();
    let enum_ = v["status"]["enum"].as_array().expect("status.enum");
    let names: Vec<&str> = enum_.iter().filter_map(|x| x.as_str()).collect();
    assert_eq!(
        names,
        vec!["active", "investigating", "fixed", "released"],
        "status.enum locked: active, investigating, fixed, released",
    );
    let visible = v["status"]["blocker_visible"]
        .as_array()
        .expect("blocker_visible");
    let visible: Vec<&str> = visible.iter().filter_map(|x| x.as_str()).collect();
    assert_eq!(
        visible,
        vec!["active", "investigating"],
        "blocker_visible must be {{active, investigating}} so released entries\
         don't keep blocking after they're cleaned up",
    );
}

#[test]
fn policy_declares_canonical_four_bucket_model() {
    let v = load_policy_toml();
    let buckets = v["buckets"].as_array().expect("buckets array");
    let names: Vec<&str> = buckets.iter().filter_map(|x| x.as_str()).collect();
    assert_eq!(
        names,
        vec!["required", "optional", "xfail", "blocker"],
        "must match the four-bucket model used by every profile manifest",
    );
}

#[test]
fn policy_locks_silent_drop_action_block() {
    let v = load_policy_toml();
    // Acceptance #3: no flaky item is silently dropped.
    assert_eq!(
        v["validation"]["silent_drop_action"].as_str(),
        Some("block"),
        "silent_drop_action must be 'block' to satisfy acceptance #3",
    );
}

#[test]
fn policy_locks_tracking_issue_regex_and_required_buckets() {
    let v = load_policy_toml();
    // Acceptance #1: required flaky item without issue link fails.
    assert_eq!(
        v["validation"]["require_tracking_issue"].as_bool(),
        Some(true),
    );
    let req = v["validation"]["required_buckets"].as_array().unwrap();
    let req: Vec<&str> = req.iter().filter_map(|x| x.as_str()).collect();
    assert!(
        req.iter().any(|b| *b == "required"),
        "required bucket must be in required_buckets",
    );
    assert!(
        req.iter().any(|b| *b == "blocker"),
        "blocker bucket must be in required_buckets (real release blockers)",
    );

    let regex = v["validation"]["issue_link_regex"].as_str().unwrap();
    assert!(
        regex.contains("[0-9]") && regex.contains("https?"),
        "regex must accept both '#NNNN' and 'https://…' forms",
    );
}

#[test]
fn policy_locks_summary_emission_contract() {
    let v = load_policy_toml();
    // Acceptance #2: quarantined item appears in release summary &
    // blocker budget.
    assert_eq!(
        v["summary_emission"]["blocker_kind"].as_str(),
        Some("flaky_quarantine"),
        "blocker_kind locked — summary consumers depend on this string",
    );
    let map = v["summary_emission"]["objective_map"]
        .as_table()
        .expect("objective_map table");
    for needed in [
        "smoke",
        "correctness",
        "performance",
        "ecosystem",
        "package_manager",
        "mambalibs",
    ] {
        assert!(
            map.contains_key(needed),
            "objective_map missing entry for {needed} — every MVP profile must route",
        );
    }
}

#[test]
fn policy_never_drop_block_covers_full_status_and_bucket_enums() {
    // Acceptance #3 reinforcement: the never_drop block must repeat the
    // full enum/bucket lists. If someone shortens this block we want the
    // gate to flag the drift, not silently drop categories.
    let v = load_policy_toml();
    let nd = v["summary_emission"]["never_drop"]
        .as_table()
        .expect("never_drop table");
    let status_in: Vec<&str> = nd["status_in"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    let bucket_in: Vec<&str> = nd["bucket_in"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    assert_eq!(
        status_in,
        vec!["active", "investigating", "fixed", "released"],
    );
    assert_eq!(bucket_in, vec!["required", "optional", "xfail", "blocker"]);
}

// ─── Validator: acceptance #1 — required without issue link fails ───

#[test]
fn required_entry_without_tracking_issue_fails_validation() {
    let dir = unique_dir("missing-issue");
    let entry = r##"
[[quarantine]]
id = "perf::cross_runtime::demo"
bucket = "required"
status = "investigating"
owner_profile = "performance"
summary = "demo flaky bench"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, _, stderr) = run_checker(&["--policy", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 1, "required entry missing tracking_issue must fail");
    assert!(
        stderr.contains("tracking_issue"),
        "stderr must name the missing field: {stderr}",
    );
}

#[test]
fn required_entry_with_malformed_tracking_issue_fails_validation() {
    let dir = unique_dir("bad-issue-link");
    let entry = r##"
[[quarantine]]
id = "perf::cross_runtime::demo"
bucket = "required"
status = "investigating"
owner_profile = "performance"
tracking_issue = "totally-not-a-link"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, _, stderr) = run_checker(&["--policy", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 1, "malformed tracking_issue must fail validation");
    assert!(
        stderr.contains("does not match policy regex"),
        "stderr must explain the regex mismatch: {stderr}",
    );
}

#[test]
fn required_entry_with_valid_issue_passes() {
    let dir = unique_dir("valid-issue");
    let entry = r##"
[[quarantine]]
id = "perf::cross_runtime::demo"
bucket = "required"
status = "investigating"
owner_profile = "performance"
tracking_issue = "#2096"
summary = "demo"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, _, stderr) = run_checker(&["--policy", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 0, "valid entry must pass (stderr={stderr})");
}

#[test]
fn required_entry_with_url_tracking_issue_passes() {
    let dir = unique_dir("valid-url");
    let entry = r##"
[[quarantine]]
id = "perf::cross_runtime::demo"
bucket = "required"
status = "active"
owner_profile = "performance"
tracking_issue = "https://github.com/example/repo/issues/42"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, _, _) = run_checker(&["--policy", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 0, "URL form must satisfy issue_link_regex");
}

// ─── Validator: acceptance #2 — visible entry surfaces in summary ───

#[test]
fn emit_blockers_synthesizes_summary_rows_for_visible_entry() {
    let dir = unique_dir("emit-visible");
    let entry = r##"
[[quarantine]]
id = "perf::cross_runtime::hashlib_blake2"
bucket = "required"
status = "investigating"
owner_profile = "performance"
tracking_issue = "#2096"
summary = "blake2 occasionally dips below 1.0x under thermal load"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, stdout, _) = run_checker(&[
        "--policy",
        path.to_str().unwrap(),
        "--format",
        "json",
        "--emit-blockers",
    ]);
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    let blockers = v["blockers"].as_array().expect("blockers array");
    assert_eq!(
        blockers.len(),
        1,
        "visible entry must produce 1 blocker row"
    );
    let row = &blockers[0];
    assert_eq!(row["kind"], serde_json::json!("flaky_quarantine"));
    assert_eq!(row["profile"], serde_json::json!("performance"));
    assert_eq!(row["objective"], serde_json::json!("performance"));
    assert_eq!(
        row["fixture_id"],
        serde_json::json!("perf::cross_runtime::hashlib_blake2"),
    );
    assert_eq!(row["tracking_issue"], serde_json::json!("#2096"));
}

#[test]
fn emit_blockers_skips_released_entries() {
    // Acceptance #2 refinement: "released" entries are kept as audit
    // history but no longer block. The blocker_visible list pins this;
    // surfacing them would re-introduce the silent drop in reverse.
    let dir = unique_dir("emit-released");
    let entry = r##"
[[quarantine]]
id = "perf::demo"
bucket = "required"
status = "released"
owner_profile = "performance"
tracking_issue = "#2096"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, stdout, _) = run_checker(&[
        "--policy",
        path.to_str().unwrap(),
        "--format",
        "json",
        "--emit-blockers",
    ]);
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    let blockers = v["blockers"].as_array().expect("blockers array");
    assert_eq!(
        blockers.len(),
        0,
        "released entry must not surface as a blocker",
    );
    assert_eq!(
        v["entry_count"].as_i64(),
        Some(1),
        "but it must still be counted in entry_count (audit history)",
    );
}

// ─── Validator: acceptance #3 — no silent drop ──────────────────────

#[test]
fn entry_with_unknown_bucket_is_reported_not_dropped() {
    let dir = unique_dir("bad-bucket");
    let entry = r##"
[[quarantine]]
id = "perf::demo"
bucket = "fictional"
status = "active"
owner_profile = "performance"
tracking_issue = "#2096"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, _, stderr) = run_checker(&["--policy", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("bucket") && stderr.contains("fictional"),
        "stderr must call out the bad bucket: {stderr}",
    );
}

#[test]
fn entry_with_unknown_status_is_reported_not_dropped() {
    let dir = unique_dir("bad-status");
    let entry = r##"
[[quarantine]]
id = "perf::demo"
bucket = "required"
status = "almost-fixed"
owner_profile = "performance"
tracking_issue = "#2096"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, _, stderr) = run_checker(&["--policy", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("status") && stderr.contains("almost-fixed"),
        "stderr must call out the bad status: {stderr}",
    );
}

#[test]
fn duplicate_ids_are_reported() {
    let dir = unique_dir("dup-id");
    let entry = r##"
[[quarantine]]
id = "perf::demo"
bucket = "required"
status = "active"
owner_profile = "performance"
tracking_issue = "#2096"

[[quarantine]]
id = "perf::demo"
bucket = "optional"
status = "active"
owner_profile = "performance"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, _, stderr) = run_checker(&["--policy", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("duplicate"),
        "duplicate id must be reported: {stderr}",
    );
}

#[test]
fn empty_quarantine_list_passes_without_silent_drop() {
    // Acceptance #3: missing quarantine list must NOT silently disappear.
    // Empty (zero entries) is fine, but the validator must still walk
    // through and surface counts.
    let dir = unique_dir("empty-list");
    let path = write_test_policy(&dir, "policy.toml", "");
    let (code, stdout, _) = run_checker(&["--policy", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    assert_eq!(v["entry_count"].as_i64(), Some(0));
    assert_eq!(v["visible_in_summary"].as_i64(), Some(0));
}

#[test]
fn json_format_emits_machine_readable_errors() {
    // Acceptance #3: errors must be machine-consumable so a runner can
    // surface them rather than discard.
    let dir = unique_dir("json-errors");
    let entry = r##"
[[quarantine]]
id = "perf::demo"
bucket = "required"
status = "investigating"
owner_profile = "performance"
"##;
    let path = write_test_policy(&dir, "policy.toml", entry);
    let (code, stdout, _) = run_checker(&["--policy", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 1);
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    let errors = v["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty());
    for e in errors {
        for key in ["entry_index", "entry_id", "message"] {
            assert!(
                e.get(key).is_some(),
                "every error row must include {key}: {e}",
            );
        }
    }
    assert_eq!(v["exit_code"].as_i64(), Some(1));
}

// ─── Live policy walk: bundled file is internally consistent ────────

#[test]
fn shipped_policy_passes_its_own_validation() {
    // The empty bundled policy must validate cleanly — if it ever stops
    // doing so we want the gate to flag it before a release.
    let (code, _, stderr) = run_checker(&["--format", "text"]);
    assert_eq!(
        code, 0,
        "bundled flaky_quarantine_policy.toml must validate (stderr={stderr})",
    );
}

#[test]
fn checker_exits_101_when_policy_missing() {
    let (code, _, _) = run_checker(&[
        "--policy",
        "/nonexistent/path/missing.toml",
        "--format",
        "json",
    ]);
    assert_eq!(code, 101, "missing policy must exit 101 (IO error)");
}

#[test]
fn checker_help_documents_emit_blockers_and_format() {
    let (code, stdout, _) = run_checker(&["--help"]);
    assert_eq!(code, 0);
    for opt in ["--policy", "--format", "--emit-blockers"] {
        assert!(stdout.contains(opt), "help must surface {opt}");
    }
}
