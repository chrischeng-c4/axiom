// Locks the shape of the MVP ignored/xfail/skip debt manifest gate
// pinned by tests/governance/gates/mvp/ignored_xfail_skip_debt_manifest/
// manifest.toml. Closes #2533. Parent: #2526.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/ignored_xfail_skip_debt_manifest/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(
        m["fixture"].as_str(),
        Some("ignored_xfail_skip_debt_manifest")
    );
    assert_eq!(m["issue"].as_integer(), Some(2533));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(
        m["family"].as_str(),
        Some("ignored_xfail_skip_debt_manifest")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let iso = &manifest()["isolation"];
    for key in [
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(iso[key].as_bool(), Some(true), "isolation.{key}");
    }
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let py = &manifest()["python_target"];
    assert_eq!(py["python_major"].as_integer(), Some(3));
    assert_eq!(py["python_minor"].as_integer(), Some(12));
    assert_eq!(py["must_be_python_3_12"].as_bool(), Some(true));
}

#[test]
fn surface_pins_required_coverage_and_marker_kinds() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_ignored_xfail_skip_inventory",
        "must_cover_required_mvp_profile_skip_block",
        "must_cover_new_marker_requires_work_item_reference",
        "must_group_counts_by_marker_kind",
        "must_emit_inventory_as_machine_readable",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
    let kinds: Vec<_> = s["recognized_marker_kinds"]
        .as_array()
        .expect("recognized_marker_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(kinds, vec!["ignored", "xfail", "skip"]);
}

#[test]
fn atomic_queue_pins_six_children() {
    let q = &manifest()["atomic_queue"];
    for key in [
        "issue_2598_categorize_ignored_tests_by_opt_in_vs_blocker",
        "issue_2599_block_ignored_tests_in_required_mvp_profile",
        "issue_2600_add_skip_debt_counts_to_smoke_summary",
        "issue_2601_require_issue_references_in_ignored_tests",
        "issue_2602_add_ignore_xfail_skip_inventory_command",
        "issue_2603_fail_new_ignore_markers_without_linked_work_item",
    ] {
        assert_eq!(q[key].as_bool(), Some(true), "atomic_queue.{key}");
    }
}

#[test]
fn ignore_xfail_and_skip_counts_are_inventoried_and_grouped() {
    let c = &manifest()["inventory_and_grouping_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("ignore_xfail_and_skip_counts_are_inventoried_and_grouped")
    );
    for key in [
        "must_inventory_ignored_count",
        "must_inventory_xfail_count",
        "must_inventory_skip_count",
        "must_group_counts_by_marker_kind",
        "must_emit_per_marker_subtotals",
        "must_emit_grand_total",
        "forbid_silently_dropping_marker_kinds",
        "must_distinguish_missing_inventory_from_ungrouped_inventory",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_inventory_fields"]
        .as_array()
        .expect("required_inventory_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "ignored_count",
            "xfail_count",
            "skip_count",
            "grand_total_marker_count",
        ]
    );
    assert_eq!(c["inventory_field_name"].as_str(), Some("marker_inventory"));
    assert_eq!(
        c["missing_inventory_failure_kind"].as_str(),
        Some("mvp_marker_inventory_missing")
    );
    assert_eq!(c["missing_inventory_exit_code"].as_integer(), Some(185));
    assert_eq!(
        c["inventory_grouping_failure_kind"].as_str(),
        Some("mvp_marker_inventory_not_grouped_by_kind")
    );
    assert_eq!(c["inventory_grouping_exit_code"].as_integer(), Some(186));
}

#[test]
fn required_mvp_profile_cannot_pass_with_required_tests_skipped() {
    let c = &manifest()["required_profile_skip_block_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("required_mvp_profile_cannot_pass_with_required_tests_skipped")
    );
    for key in [
        "must_fail_required_mvp_profile_when_required_test_skipped",
        "must_fail_required_mvp_profile_when_required_test_ignored",
        "must_fail_required_mvp_profile_when_required_test_xfailed",
        "forbid_silently_passing_with_required_test_skipped",
        "forbid_skip_marker_being_treated_as_pass_in_required_profile",
        "must_distinguish_required_test_skip_from_opt_in_skip",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["required_profile_name"].as_str(), Some("mvp_required"));
    assert_eq!(
        c["required_test_skipped_failure_kind"].as_str(),
        Some("mvp_required_test_skipped")
    );
    assert_eq!(c["required_test_skipped_exit_code"].as_integer(), Some(187));
    assert_eq!(c["opt_in_marker_classification"].as_str(), Some("opt_in"));
    assert_eq!(c["blocker_marker_classification"].as_str(), Some("blocker"));
    assert_eq!(
        c["required_classification_field_name"].as_str(),
        Some("marker_classification")
    );
    let allowed: Vec<_> = c["allowed_classification_values"]
        .as_array()
        .expect("allowed_classification_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(allowed, vec!["opt_in", "blocker"]);
}

#[test]
fn new_ignored_or_xfailed_tests_require_linked_work_item_references() {
    let c = &manifest()["work_item_reference_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("new_ignored_or_xfailed_tests_require_linked_work_item_references")
    );
    for key in [
        "must_require_work_item_reference_on_ignored_marker",
        "must_require_work_item_reference_on_xfailed_marker",
        "must_validate_work_item_reference_format",
        "forbid_unreferenced_ignored_marker",
        "forbid_unreferenced_xfailed_marker",
        "forbid_placeholder_or_stub_work_item_reference",
        "must_distinguish_missing_reference_from_invalid_reference",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["work_item_reference_field_name"].as_str(),
        Some("work_item_reference")
    );
    let kinds: Vec<_> = c["allowed_work_item_reference_kinds"]
        .as_array()
        .expect("allowed_work_item_reference_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(kinds, vec!["issue", "epic", "tracker"]);
    assert_eq!(c["work_item_reference_regex"].as_str(), Some("^#[0-9]+$"));
    assert_eq!(
        c["missing_work_item_reference_failure_kind"].as_str(),
        Some("mvp_marker_missing_work_item_reference")
    );
    assert_eq!(
        c["missing_work_item_reference_exit_code"].as_integer(),
        Some(188)
    );
    assert_eq!(
        c["invalid_work_item_reference_failure_kind"].as_str(),
        Some("mvp_marker_invalid_work_item_reference")
    );
    assert_eq!(
        c["invalid_work_item_reference_exit_code"].as_integer(),
        Some(189)
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let r = &manifest()["runner_contract"];
    let keys: Vec<_> = r["keys"]
        .as_array()
        .expect("keys")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        keys,
        vec![
            "outcome",
            "case",
            "marker_inventory",
            "ignored_count",
            "xfail_count",
            "skip_count",
            "grand_total_marker_count",
            "required_profile_name",
            "marker_classification",
            "work_item_reference",
            "failure_kind",
            "exit_code",
        ]
    );
    let outcomes: Vec<_> = r["outcome_values"]
        .as_array()
        .expect("outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(outcomes, vec!["pass", "fail", "missing", "skip"]);
    let cases: Vec<_> = r["case_values"]
        .as_array()
        .expect("case_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        cases,
        vec![
            "ignore_xfail_and_skip_counts_are_inventoried_and_grouped",
            "required_mvp_profile_cannot_pass_with_required_tests_skipped",
            "new_ignored_or_xfailed_tests_require_linked_work_item_references",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_of_inventory_logic",
        "runtime_implementation_of_required_profile_blocking",
        "runtime_implementation_of_work_item_reference_validation",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
