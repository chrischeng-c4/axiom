//! Schema gate for the require-issue-references-in-ignored fixture —
//! closes #2601.
//!
//! Acceptance (issue #2601):
//!
//!   1. An ignored test without a linked issue fails validation.
//!      `[unlinked_ignore_failure_contract]` pins must_validate_every_
//!      ignore_attribute + must_require_either_issue_reference_or_
//!      opt_in_category + must_emit_exact_file/line +
//!      unlinked_ignore_exit_code=49.
//!   2. Existing ignored tests are either annotated or listed as
//!      current violations. `[existing_inventory_contract]` pins
//!      must_emit_existing_violations_inventory + json record +
//!      must_separate_annotated_from_violations +
//!      missing-inventory exit_code=50 +
//!      inventory_must_be_committed_to_repo.
//!   3. Validation can run without compiling the project.
//!      `[compile_free_contract]` pins must_run_without_cargo_check/
//!      build/test + allowed_scan_modes regex/syn + compile_free_
//!      violation_exit_code=51.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("skip_debt")
        .join("require_issue_refs_in_ignored")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path).unwrap();
    raw.parse().unwrap()
}

#[test]
fn header_is_well_formed() {
    let doc = load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("require_issue_refs_in_ignored")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2601));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2533)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("skip_debt")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("require_issue_refs_in_ignored")
    );
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = load_toml(&manifest_path());
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
fn categorization_cross_references_fixture_2598() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("categorization_cross_reference")
        .and_then(|v| v.as_table())
        .expect("[categorization_cross_reference] missing");
    assert_eq!(
        c.get("fixture_issue").and_then(|v| v.as_integer()),
        Some(2598)
    );
    let values: Vec<&str> = c
        .get("inherited_category_allowed_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &["opt_in", "blocker", "flaky", "obsolete"] {
        assert!(
            values.contains(v),
            "inherited_category_allowed_values must include {v}"
        );
    }
    assert_eq!(
        c.get("must_share_category_allowed_values")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn reference_rule_pins_nearby_window_and_allowed_kinds() {
    let doc = load_toml(&manifest_path());
    let r = doc
        .get("reference_rule")
        .and_then(|v| v.as_table())
        .expect("[reference_rule] missing");
    assert_eq!(
        r.get("must_validate_reference_within_same_attribute_block")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let win = r
        .get("nearby_window_in_lines")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert!(
        win > 0 && win <= 20,
        "nearby_window_in_lines must be in (0,20], got {win}"
    );
    let kinds: Vec<&str> = r
        .get("allowed_reference_kinds")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for k in &["issue_number", "documented_opt_in_category"] {
        assert!(
            kinds.contains(k),
            "allowed_reference_kinds must include {k}"
        );
    }
    assert_eq!(
        r.get("issue_number_pattern_must_match_project_mamba_only")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        r.get("project_mamba_label_required")
            .and_then(|v| v.as_str()),
        Some("project:mamba")
    );
    assert_eq!(
        r.get("forbid_external_repository_references")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "An ignored test without a linked issue fails validation."
#[test]
fn unlinked_ignore_fails_validation() {
    let doc = load_toml(&manifest_path());
    let u = doc
        .get("unlinked_ignore_failure_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[unlinked_ignore_failure_contract] missing — acceptance: \
         \"An ignored test without a linked issue fails validation.\"",
        );
    assert_eq!(
        u.get("must_validate_every_ignore_attribute")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        u.get("must_require_either_issue_reference_or_opt_in_category")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        u.get("unlinked_ignore_failure_kind")
            .and_then(|v| v.as_str()),
        Some("ignore_attribute_without_linked_reference")
    );
    let exit = u
        .get("unlinked_ignore_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 49);
    assert_eq!(
        u.get("must_emit_exact_file_for_violation")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        u.get("must_emit_exact_line_for_violation")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        u.get("violation_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    let required: Vec<&str> = u
        .get("violation_required_fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for f in &[
        "file",
        "line",
        "ignore_kind",
        "reference_kind",
        "reference_value",
        "failure_kind",
        "exit_code",
    ] {
        assert!(
            required.contains(f),
            "violation_required_fields must include {f}"
        );
    }
}

// Acceptance: "Existing ignored tests are either annotated or listed as current violations."
#[test]
fn existing_ignored_tests_are_annotated_or_listed() {
    let doc = load_toml(&manifest_path());
    let e = doc
        .get("existing_inventory_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[existing_inventory_contract] missing — acceptance: \
         \"Existing ignored tests are either annotated or listed as current violations.\"",
        );
    assert_eq!(
        e.get("must_emit_existing_violations_inventory")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        e.get("existing_inventory_record_format")
            .and_then(|v| v.as_str()),
        Some("json")
    );
    let required: Vec<&str> = e
        .get("existing_inventory_required_fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for f in &["file", "line", "category", "reference", "annotated"] {
        assert!(
            required.contains(f),
            "existing_inventory_required_fields must include {f}"
        );
    }
    assert_eq!(
        e.get("must_separate_annotated_from_violations")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        e.get("must_not_silently_pass_unannotated")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let exit = e
        .get("missing_inventory_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 50);
    assert_eq!(
        e.get("inventory_must_be_committed_to_repo")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let path = e
        .get("inventory_relative_path_within_repo")
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(path.starts_with("projects/mamba/tests/"));
    assert!(path.ends_with(".json"));
}

// Acceptance: "Validation can run without compiling the project."
#[test]
fn validation_runs_without_compiling_project() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("compile_free_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[compile_free_contract] missing — acceptance: \
         \"Validation can run without compiling the project.\"",
        );
    for f in &[
        "must_run_without_cargo_check",
        "must_run_without_cargo_build",
        "must_run_without_cargo_test",
        "must_be_pure_source_scan",
    ] {
        assert_eq!(
            c.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    let allowed: Vec<&str> = c
        .get("allowed_scan_modes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for m in &["regex_scan_of_test_sources", "syn_parse_without_typeck"] {
        assert!(allowed.contains(m), "allowed_scan_modes must include {m}");
    }
    let forbidden: Vec<&str> = c
        .get("forbidden_scan_modes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for m in &[
        "cargo_check",
        "cargo_build",
        "cargo_test",
        "rustc_invocation_with_codegen",
    ] {
        assert!(
            forbidden.contains(m),
            "forbidden_scan_modes must include {m}"
        );
    }
    let exit = c
        .get("compile_free_violation_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 51);
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .unwrap();
    let keys: Vec<&str> = c
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "outcome",
        "case",
        "file",
        "line",
        "ignore_kind",
        "reference_kind",
        "reference_value",
        "annotated",
        "category",
        "failure_kind",
        "exit_code",
    ] {
        assert!(
            keys.contains(required),
            "runner_contract.keys must include {required}"
        );
    }
    let cases: Vec<&str> = c
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "unlinked_ignore_fails_validation",
        "existing_ignored_tests_are_annotated_or_listed",
        "validation_runs_without_compiling_project",
    ] {
        assert!(
            cases.contains(required),
            "runner_contract.case_values must include {required}"
        );
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("resolving_ignored_tests").and_then(|v| v.as_bool()),
        Some(true)
    );
}
