//! MVP release-gate baseline update policy — locking test (closes #2823).
//!
//! Parent: #2775 (MVP release blocking CI profiles).
//!
//! Locks three things:
//!
//!   - `validation/baseline_update_policy.toml` — the policy file:
//!     [baselines.*] families, [direction.weaker]/[direction.stronger]
//!     required-fields lists, and [validation] enforcement settings.
//!   - Cross-references: every release profile that owns a governed
//!     baseline (performance / correctness / ecosystem / smoke) cites
//!     this policy in its own manifest.
//!   - `scripts/baseline_policy_check.py` — the validator behavior:
//!     weaker-without-reason fails, stronger-with-summary passes,
//!     missing changelog warns (non-fatal by default).
//!
//! Acceptance (issue #2823):
//!
//!     1. Baseline changes without reason or issue link fail validation.
//!     2. Stronger baselines are allowed with summary.
//!     3. Policy is referenced by release profile manifests.

use std::path::PathBuf;

use serde_json::Value;

fn policy_path() -> PathBuf {
    crate::common::project_root()
        .join("validation")
        .join("baseline_update_policy.toml")
}

fn validator_script() -> PathBuf {
    crate::common::project_root()
        .join("scripts")
        .join("baseline_policy_check.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-baseline-policy-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn read_policy() -> String {
    std::fs::read_to_string(policy_path()).unwrap_or_else(|e| panic!("read policy: {e}"))
}

fn run_validator(args: &[&str]) -> (i32, String, String) {
    let output = crate::common::run_python_script(&validator_script(), args);
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

// ─── Policy file shape ──────────────────────────────────────────────

#[test]
fn policy_file_lives_alongside_release_artifacts() {
    let path = policy_path();
    assert!(
        path.is_file(),
        "baseline_update_policy.toml must live next to per-profile manifests: {}",
        path.display(),
    );
}

#[test]
fn policy_declares_every_governed_baseline_family() {
    let body = read_policy();
    for fam in ["performance", "cpython_seeds", "ecosystem", "skip_debt"] {
        assert!(
            body.contains(&format!("[baselines.{fam}]")),
            "policy must declare [baselines.{fam}] block",
        );
    }
}

#[test]
fn policy_separates_weaker_and_stronger_directions() {
    let body = read_policy();
    assert!(
        body.contains("[direction.weaker]"),
        "policy missing [direction.weaker]"
    );
    assert!(
        body.contains("[direction.stronger]"),
        "policy missing [direction.stronger]"
    );
    // The section header appears as a line on its own ("\n[direction.X]\n").
    // The header also appears INSIDE comment lines as documentation; split on
    // the newline-anchored form so we only see the real TOML section.
    let weaker_block = body
        .split("\n[direction.weaker]\n")
        .last()
        .unwrap_or("")
        .split("\n[direction.stronger]\n")
        .next()
        .unwrap_or("");
    for fld in ["reason", "tracking_issue", "before", "after"] {
        assert!(
            weaker_block.contains(&format!("\"{fld}\"")),
            "weaker required_fields must include {fld:?}; block was: <<<{weaker_block}>>>",
        );
    }
    let stronger_block = body.split("\n[direction.stronger]\n").last().unwrap_or("");
    assert!(
        stronger_block.contains("\"summary\""),
        "stronger required_fields must include \"summary\"; block was: <<<{stronger_block}>>>",
    );
}

#[test]
fn policy_validation_block_pins_issue_link_regex_and_changelog_path() {
    let body = read_policy();
    assert!(
        body.contains("[validation]"),
        "policy must declare [validation] enforcement block",
    );
    assert!(
        body.contains("changelog_path"),
        "[validation] must declare changelog_path",
    );
    assert!(
        body.contains("issue_link_regex"),
        "[validation] must declare issue_link_regex (acceptance #1)",
    );
}

// ─── Acceptance 3: profile manifests reference this policy ──────────

#[test]
fn each_governed_profile_manifest_references_the_policy() {
    let profiles_dir = crate::common::project_root()
        .join("validation")
        .join("profiles");
    for profile in ["performance", "correctness", "ecosystem", "smoke"] {
        let path = profiles_dir.join(format!("{profile}.toml"));
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert!(
            body.contains("[references.baseline_update_policy]"),
            "{profile}.toml must declare [references.baseline_update_policy] \
             so the policy is mutually visible from both sides (acceptance #3)",
        );
        assert!(
            body.contains("issue = 2823"),
            "{profile}.toml reference block must cite #2823",
        );
    }
}

// ─── Acceptance 1: weaker entry without reason/issue fails ──────────

fn entry(direction: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::from("[[entries]]\n");
    out.push_str(&format!("direction = \"{direction}\"\n"));
    for (k, v) in fields {
        out.push_str(&format!("{k} = \"{v}\"\n"));
    }
    out
}

fn write_changelog(dir: &PathBuf, body: &str) -> PathBuf {
    let path = dir.join("changelog.toml");
    std::fs::write(&path, body).expect("write changelog fixture");
    path
}

#[test]
fn weaker_entry_missing_reason_fails_validation() {
    let dir = unique_dir("weaker-missing-reason");
    // reason intentionally absent
    let body = entry(
        "weaker",
        &[
            ("family", "performance"),
            ("before", "1.0x"),
            ("after", "0.5x"),
            ("tracking_issue", "#2096"),
        ],
    );
    let path = write_changelog(&dir, &body);
    let (code, _, stderr) =
        run_validator(&["--changelog", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(
        code, 1,
        "weaker entry without `reason` must fail (stderr={stderr})"
    );
    assert!(
        stderr.contains("reason"),
        "error message must name the missing field"
    );
}

#[test]
fn weaker_entry_missing_tracking_issue_fails_validation() {
    let dir = unique_dir("weaker-missing-issue");
    let body = entry(
        "weaker",
        &[
            ("family", "performance"),
            ("before", "1.0x"),
            ("after", "0.5x"),
            ("reason", "Lowered floor for new arm64 baseline"),
        ],
    );
    let path = write_changelog(&dir, &body);
    let (code, _, stderr) =
        run_validator(&["--changelog", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(
        code, 1,
        "weaker entry without tracking_issue must fail (stderr={stderr})"
    );
    assert!(
        stderr.contains("tracking_issue"),
        "error message must name the missing tracking_issue field",
    );
}

#[test]
fn weaker_entry_with_invalid_issue_link_fails_validation() {
    let dir = unique_dir("weaker-bad-issue");
    let body = entry(
        "weaker",
        &[
            ("family", "performance"),
            ("before", "1.0x"),
            ("after", "0.5x"),
            ("reason", "Floor lowered while we investigate"),
            ("tracking_issue", "TODO"),
        ],
    );
    let path = write_changelog(&dir, &body);
    let (code, stdout, _) =
        run_validator(&["--changelog", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 1, "tracking_issue=TODO must fail the policy regex");
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    let errs = v["errors"].as_array().expect("errors is array");
    assert!(
        errs.iter()
            .any(|e| e["message"].as_str().unwrap_or("").contains("regex")),
        "error report must explain regex mismatch: {errs:?}",
    );
}

// ─── Acceptance 2: stronger entry with summary passes ───────────────

#[test]
fn stronger_entry_with_summary_passes() {
    let dir = unique_dir("stronger-ok");
    let body = entry(
        "stronger",
        &[
            ("family", "performance"),
            (
                "summary",
                "Raised per-benchmark floor to 1.5x after JIT improvements",
            ),
        ],
    );
    let path = write_changelog(&dir, &body);
    let (code, _, stderr) =
        run_validator(&["--changelog", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(
        code, 0,
        "stronger entry with summary must pass (stderr={stderr})"
    );
}

#[test]
fn stronger_entry_without_summary_fails() {
    let dir = unique_dir("stronger-missing");
    let body = entry("stronger", &[("family", "ecosystem")]);
    let path = write_changelog(&dir, &body);
    let (code, _, stderr) =
        run_validator(&["--changelog", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(
        code, 1,
        "stronger entry without summary must fail (stderr={stderr})"
    );
    assert!(stderr.contains("summary"));
}

// ─── Validator robustness ───────────────────────────────────────────

#[test]
fn missing_changelog_warns_but_does_not_fail() {
    let (code, _, _) = run_validator(&[
        "--changelog",
        "/nonexistent/path/missing.toml",
        "--format",
        "json",
    ]);
    assert_eq!(
        code, 0,
        "missing changelog under warn policy must exit 0 (fresh-repo case)",
    );
}

#[test]
fn validator_help_documents_policy_and_changelog_flags() {
    let (code, stdout, _) = run_validator(&["--help"]);
    assert_eq!(code, 0);
    for opt in ["--policy", "--changelog", "--format"] {
        assert!(stdout.contains(opt), "help must surface {opt}");
    }
}

#[test]
fn unknown_family_fails_validation() {
    let dir = unique_dir("unknown-family");
    let body = entry(
        "weaker",
        &[
            ("family", "made_up_family"),
            ("reason", "x"),
            ("tracking_issue", "#1"),
            ("before", "0"),
            ("after", "1"),
        ],
    );
    let path = write_changelog(&dir, &body);
    let (code, _, _) = run_validator(&["--changelog", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(code, 1, "entries with unknown family must be rejected");
}

#[test]
fn duplicate_entries_with_same_after_hash_are_rejected() {
    let dir = unique_dir("dup");
    let mut body = entry(
        "weaker",
        &[
            ("family", "performance"),
            ("reason", "first waiver"),
            ("tracking_issue", "#2096"),
            ("before", "1.0x"),
            ("after", "0.5x"),
            ("after_hash", "deadbeef"),
        ],
    );
    body.push('\n');
    body.push_str(&entry(
        "weaker",
        &[
            ("family", "performance"),
            ("reason", "duplicate of above"),
            ("tracking_issue", "#2096"),
            ("before", "1.0x"),
            ("after", "0.5x"),
            ("after_hash", "deadbeef"),
        ],
    ));
    let path = write_changelog(&dir, &body);
    let (code, stdout, _) =
        run_validator(&["--changelog", path.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 1, "duplicate after_hash must be rejected");
    let v: Value = serde_json::from_str(&stdout).expect("parse JSON");
    let errs = v["errors"].as_array().expect("errors");
    assert!(
        errs.iter()
            .any(|e| e["message"].as_str().unwrap_or("").contains("duplicate")),
        "error must call out duplicate entry: {errs:?}",
    );
}

#[test]
fn real_policy_file_is_consumed_without_changelog_present() {
    // Default run against the live policy file. The real repo has no
    // changelog yet, so missing-changelog warn path applies and the
    // validator exits 0. Guards against defaults flipping to "block".
    let (code, _stdout, _stderr) = run_validator(&[]);
    assert_eq!(
        code, 0,
        "default run against the live policy must exit 0 until a changelog lands",
    );
}
