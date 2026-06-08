//! Schema gate for the fail-new-ignore-without-work-item fixture —
//! closes #2603.
//!
//! Acceptance (issue #2603):
//!
//!   1. Adding #[ignore] without a work item fails the guard.
//!      `[new_ignore_without_work_item_contract]` pins must_compare_
//!      current_inventory_to_baseline + must_emit_added_markers_diff
//!      + json record + must_fail_when_new_ignore_lacks_work_item
//!      + new_ignore_without_work_item_exit_code=56.
//!   2. Adding xfail without a work item fails the guard.
//!      `[new_xfail_without_work_item_contract]` pins
//!      must_fail_when_new_xfail_lacks_work_item +
//!      new_xfail_without_work_item_exit_code=57 +
//!      must_distinguish_new_xfail_from_new_ignore + same rule
//!      applies to all marker kinds.
//!   3. Updating the baseline remains an explicit review step.
//!      `[baseline_review_contract]` pins
//!      must_require_explicit_review_to_update_baseline +
//!      codeowners_review_required + forbid_silent_baseline_update
//!      + silent_baseline_update_exit_code=58 +
//!      must_emit_review_required_message_on_baseline_diff.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("skip_debt")
        .join("fail_new_ignore_without_work_item")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("fail_new_ignore_without_work_item"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2603));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2533));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("skip_debt"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("fail_new_ignore_without_work_item"));
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("isolation").and_then(|v| v.as_table()).unwrap();
    for f in &[
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(i.get(*f).and_then(|v| v.as_bool()), Some(true));
    }
}

#[test]
fn cross_references_inventory_and_reference_fixtures() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("inventory_cross_reference").and_then(|v| v.as_table())
        .expect("[inventory_cross_reference] missing");
    assert_eq!(i.get("fixture_issue").and_then(|v| v.as_integer()), Some(2602));
    assert_eq!(i.get("must_consume_inventory_output").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(i.get("must_be_consistent_with_inventory_marker_kinds").and_then(|v| v.as_bool()), Some(true));
    let shared: Vec<&str> = i.get("shared_marker_kinds").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for k in &["ignore", "xfail", "skip"] {
        assert!(shared.contains(k), "shared_marker_kinds must include {k}");
    }

    let r = doc.get("reference_cross_reference").and_then(|v| v.as_table())
        .expect("[reference_cross_reference] missing");
    assert_eq!(r.get("fixture_issue").and_then(|v| v.as_integer()), Some(2601));
    assert_eq!(r.get("must_share_work_item_reference_definition").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn baseline_block_is_committed_versioned_and_checksummed() {
    let doc = crate::common::load_toml(&manifest_path());
    let b = doc.get("baseline").and_then(|v| v.as_table()).expect("[baseline] missing");
    let path = b.get("baseline_relative_path_within_repo").and_then(|v| v.as_str()).unwrap();
    assert!(path.starts_with("projects/mamba/tests/"));
    assert!(path.ends_with("baseline.json"));
    assert_eq!(b.get("baseline_format").and_then(|v| v.as_str()), Some("json"));
    assert_eq!(b.get("must_be_committed_to_repo").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(b.get("must_be_versioned").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(b.get("schema_version_field_name").and_then(|v| v.as_str()), Some("schema_version"));
    assert_eq!(b.get("must_emit_baseline_checksum").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(b.get("checksum_field_name").and_then(|v| v.as_str()), Some("checksum"));
}

// Acceptance: "Adding #[ignore] without a work item fails the guard."
#[test]
fn new_ignore_without_work_item_fails_guard() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("new_ignore_without_work_item_contract").and_then(|v| v.as_table()).expect(
        "[new_ignore_without_work_item_contract] missing — acceptance: \
         \"Adding #[ignore] without a work item fails the guard.\"",
    );
    assert_eq!(n.get("must_compare_current_inventory_to_baseline").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(n.get("must_emit_added_markers_diff").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(n.get("diff_record_format").and_then(|v| v.as_str()), Some("json"));
    let required: Vec<&str> = n.get("diff_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &[
        "file", "line", "marker_kind", "work_item_link",
        "baseline_status", "failure_kind", "exit_code",
    ] {
        assert!(required.contains(f), "diff_required_fields must include {f}");
    }
    assert_eq!(n.get("must_fail_when_new_ignore_lacks_work_item").and_then(|v| v.as_bool()), Some(true));
    let exit = n.get("new_ignore_without_work_item_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 56);
}

// Acceptance: "Adding xfail without a work item fails the guard."
#[test]
fn new_xfail_without_work_item_fails_guard() {
    let doc = crate::common::load_toml(&manifest_path());
    let x = doc.get("new_xfail_without_work_item_contract").and_then(|v| v.as_table()).expect(
        "[new_xfail_without_work_item_contract] missing — acceptance: \
         \"Adding xfail without a work item fails the guard.\"",
    );
    assert_eq!(x.get("must_fail_when_new_xfail_lacks_work_item").and_then(|v| v.as_bool()), Some(true));
    let exit = x.get("new_xfail_without_work_item_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 57);
    assert_eq!(x.get("must_distinguish_new_xfail_from_new_ignore").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(x.get("must_apply_same_work_item_rule_to_all_marker_kinds").and_then(|v| v.as_bool()), Some(true));
    let ignore_exit = doc.get("new_ignore_without_work_item_contract")
        .and_then(|v| v.get("new_ignore_without_work_item_exit_code"))
        .and_then(|v| v.as_integer()).unwrap();
    assert_ne!(exit, ignore_exit, "new-xfail and new-ignore exit codes must differ");
}

// Acceptance: "Updating the baseline remains an explicit review step."
#[test]
fn baseline_update_is_explicit_review_step() {
    let doc = crate::common::load_toml(&manifest_path());
    let b = doc.get("baseline_review_contract").and_then(|v| v.as_table()).expect(
        "[baseline_review_contract] missing — acceptance: \
         \"Updating the baseline remains an explicit review step.\"",
    );
    for f in &[
        "must_require_explicit_review_to_update_baseline",
        "must_require_codeowners_review_for_baseline_path",
        "codeowners_review_required",
        "baseline_must_be_in_codeowners_protected_path",
        "forbid_silent_baseline_update",
        "must_emit_review_required_message_on_baseline_diff",
    ] {
        assert_eq!(b.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
    let exit = b.get("silent_baseline_update_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 58);
    assert_eq!(b.get("review_required_message_field_name").and_then(|v| v.as_str()), Some("review_required_message"));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "file", "line",
        "marker_kind", "work_item_link", "baseline_status",
        "review_required_message", "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "new_ignore_without_work_item_fails_guard",
        "new_xfail_without_work_item_fails_guard",
        "baseline_update_is_explicit_review_step",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("making_all_existing_markers_pass_immediately").and_then(|v| v.as_bool()), Some(true));
}
