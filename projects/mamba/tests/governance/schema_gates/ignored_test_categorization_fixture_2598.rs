//! Schema gate for the ignored-test categorization fixture — closes
//! #2598.
//!
//! Acceptance (issue #2598):
//!
//!   1. Inventory summary shows counts by category.
//!      `[summary_counts_contract]` pins must_emit_summary_section +
//!      json record format + must_emit_count_for_every_category +
//!      summary_required_fields {category, count, total, profile} +
//!      missing-category-count exit_code=37.
//!   2. Unclassified ignore markers fail validation.
//!      `[validation_contract]` pins must_validate_every_ignored_
//!      marker_has_category + must_validate_blocker_links_work_item
//!      + unclassified exit_code=38 + blocker-without-work-item
//!      exit_code=39, distinct.
//!   3. MVP-required tests cannot be categorized only as opt_in
//!      unless justified. `[mvp_required_guard_contract]` pins
//!      must_block_opt_in_only_for_mvp_required + justification field
//!      + work-item link + exit_code=40 unjustified + exit_code=41
//!      justification missing work item.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("skip_debt")
        .join("ignored_test_categorization")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("ignored_test_categorization"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2598));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2533));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("skip_debt"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("ignored_test_categorization"));
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
fn categories_block_lists_four_required_values() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("categories").and_then(|v| v.as_table()).expect("[categories] missing");
    let values: Vec<&str> = c.get("allowed_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["opt_in", "blocker", "flaky", "obsolete"] {
        assert!(values.contains(v), "categories.allowed_values must include {v}");
    }
    assert_eq!(values.len(), 4, "exactly the four required categories");
    assert_eq!(c.get("required_for_every_ignored_marker").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("case_sensitive").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_be_lowercase").and_then(|v| v.as_bool()), Some(true));
    for v in &values {
        assert!(v.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_'),
            "category value {v} must be lowercase snake_case");
    }
    let exit = c.get("unknown_category_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 36);
}

// Acceptance: "Inventory summary shows counts by category."
#[test]
fn inventory_summary_shows_counts_by_category() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("summary_counts_contract").and_then(|v| v.as_table()).expect(
        "[summary_counts_contract] missing — acceptance: \
         \"Inventory summary shows counts by category.\"",
    );
    assert_eq!(s.get("must_emit_summary_section").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("summary_record_format").and_then(|v| v.as_str()), Some("json"));
    assert_eq!(s.get("must_emit_count_for_every_category").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("must_emit_total_count").and_then(|v| v.as_bool()), Some(true));
    let required: Vec<&str> = s.get("summary_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &["category", "count", "total", "profile"] {
        assert!(required.contains(f), "summary_required_fields must include {f}");
    }
    let exit = s.get("missing_category_count_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 37);
}

// Acceptance: "Unclassified ignore markers fail validation."
#[test]
fn unclassified_ignore_markers_fail_validation() {
    let doc = crate::common::load_toml(&manifest_path());
    let v = doc.get("validation_contract").and_then(|v| v.as_table()).expect(
        "[validation_contract] missing — acceptance: \
         \"Unclassified ignore markers fail validation.\"",
    );
    assert_eq!(v.get("must_validate_every_ignored_marker_has_category").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(v.get("must_validate_category_is_in_allowed_values").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(v.get("must_validate_blocker_links_work_item").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(v.get("unclassified_marker_failure_kind").and_then(|v| v.as_str()), Some("ignore_marker_unclassified"));
    let unclassified = v.get("unclassified_marker_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(unclassified, 38);
    let blocker = v.get("blocker_without_work_item_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(blocker, 39);
    assert_ne!(unclassified, blocker, "unclassified and blocker-link-missing exits must differ");
    assert_eq!(v.get("must_distinguish_unclassified_from_blocker_link_missing").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "MVP-required tests cannot be categorized only as opt_in unless justified."
#[test]
fn mvp_required_tests_cannot_be_only_opt_in_unless_justified() {
    let doc = crate::common::load_toml(&manifest_path());
    let g = doc.get("mvp_required_guard_contract").and_then(|v| v.as_table()).expect(
        "[mvp_required_guard_contract] missing — acceptance: \
         \"MVP-required tests cannot be categorized only as opt_in unless justified.\"",
    );
    assert_eq!(g.get("must_block_opt_in_only_for_mvp_required").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(g.get("must_require_explicit_justification_when_mvp_required_is_opt_in").and_then(|v| v.as_bool()), Some(true));
    let field = g.get("justification_field_name").and_then(|v| v.as_str()).unwrap();
    assert!(field.contains("justification"));
    assert!(field.contains("opt_in"));
    assert_eq!(g.get("must_require_justification_to_link_work_item").and_then(|v| v.as_bool()), Some(true));
    let unjust = g.get("mvp_required_opt_in_without_justification_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(unjust, 40);
    let no_wi = g.get("justification_missing_work_item_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(no_wi, 41);
    assert_ne!(unjust, no_wi);
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case",
        "marker_id", "marker_kind", "category",
        "work_item_link", "mvp_required", "justification_present",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "inventory_summary_shows_counts_by_category",
        "unclassified_ignore_markers_fail_validation",
        "mvp_required_tests_cannot_be_only_opt_in_without_justification",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("deleting_obsolete_tests").and_then(|v| v.as_bool()), Some(true));
}
