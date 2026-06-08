//! Schema gate for the stdlib required module manifest fixture —
//! closes #2624.
//!
//! Acceptance (issue #2624):
//!
//!   1. Missing fixture reference for a required stdlib module fails
//!      validation. `[required_fixture_reference_contract]` pins
//!      must_validate_every_required_module_has_fixture_reference +
//!      required_module_required_fields + missing-fixture exit_
//!      code=59 + fixture_issue_zero_is_treated_as_missing_for_
//!      required.
//!   2. Optional modules are reported separately.
//!      `[optional_modules_reporting_contract]` pins
//!      must_separate_optional_from_required_in_report +
//!      must_not_fail_validation_for_optional_without_fixture +
//!      json record + missing-optional-section exit_code=60.
//!   3. Manifest is consumed by the ecosystem summary.
//!      `[ecosystem_summary_consumption_contract]` pins
//!      must_be_consumable_by_ecosystem_summary +
//!      must_emit_consumption_artifact + json +
//!      consumption_artifact_required_fields +
//!      missing-consumption-artifact exit_code=61.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("stdlib")
        .join("required_module_manifest")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_required_module_manifest"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2624));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_required_module_manifest"));
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
fn python_target_is_pinned_to_3_12() {
    let doc = crate::common::load_toml(&manifest_path());
    let p = doc.get("python_target").and_then(|v| v.as_table()).expect("[python_target] missing");
    assert_eq!(p.get("python_major").and_then(|v| v.as_integer()), Some(3));
    assert_eq!(p.get("python_minor").and_then(|v| v.as_integer()), Some(12));
    assert_eq!(p.get("must_be_python_3_12").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn modules_table_array_declares_required_and_optional_with_consistent_fields() {
    let doc = crate::common::load_toml(&manifest_path());
    let modules = doc.get("modules").and_then(|v| v.as_array()).expect("[[modules]] missing");
    assert!(modules.len() >= 2, "must declare at least one required and one optional entry");

    let mut required_count = 0;
    let mut optional_count = 0;
    let mut seen_names: Vec<&str> = Vec::new();
    for m in modules {
        let t = m.as_table().expect("module entry must be a table");
        let name = t.get("name").and_then(|v| v.as_str()).expect("module.name missing");
        let fixture_issue = t.get("fixture_issue").and_then(|v| v.as_integer())
            .expect("module.fixture_issue missing");
        let required = t.get("required").and_then(|v| v.as_bool()).expect("module.required missing");
        let mvp = t.get("mvp_objective").and_then(|v| v.as_str()).expect("module.mvp_objective missing");
        assert!(!name.is_empty(), "module.name must be non-empty");
        assert!(!mvp.is_empty(), "module.mvp_objective must be non-empty");
        assert!(!seen_names.contains(&name), "duplicate module name {name}");
        seen_names.push(name);

        if required {
            assert!(fixture_issue > 0, "required module {name} must point to a real fixture_issue, got {fixture_issue}");
            required_count += 1;
        } else {
            optional_count += 1;
        }
    }
    assert!(required_count >= 1, "must declare at least one required module");
    assert!(optional_count >= 1, "must declare at least one optional module");
}

// Acceptance: "Missing fixture reference for a required stdlib module fails validation."
#[test]
fn required_module_without_fixture_reference_fails_validation() {
    let doc = crate::common::load_toml(&manifest_path());
    let r = doc.get("required_fixture_reference_contract").and_then(|v| v.as_table()).expect(
        "[required_fixture_reference_contract] missing — acceptance: \
         \"Missing fixture reference for a required stdlib module fails validation.\"",
    );
    assert_eq!(r.get("must_validate_every_required_module_has_fixture_reference").and_then(|v| v.as_bool()), Some(true));
    let required: Vec<&str> = r.get("required_module_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &["name", "fixture_issue", "required", "mvp_objective"] {
        assert!(required.contains(f), "required_module_required_fields must include {f}");
    }
    let exit = r.get("missing_fixture_reference_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 59);
    assert_eq!(r.get("must_emit_module_name_in_diagnostic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(r.get("must_emit_owning_mvp_objective_in_diagnostic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(r.get("fixture_issue_zero_is_treated_as_missing_for_required").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Optional modules are reported separately."
#[test]
fn optional_modules_are_reported_separately() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("optional_modules_reporting_contract").and_then(|v| v.as_table()).expect(
        "[optional_modules_reporting_contract] missing — acceptance: \
         \"Optional modules are reported separately.\"",
    );
    assert_eq!(o.get("must_separate_optional_from_required_in_report").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(o.get("must_not_fail_validation_for_optional_without_fixture").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(o.get("optional_report_record_format").and_then(|v| v.as_str()), Some("json"));
    let required: Vec<&str> = o.get("optional_report_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &["name", "required", "fixture_issue", "mvp_objective"] {
        assert!(required.contains(f), "optional_report_required_fields must include {f}");
    }
    let exit = o.get("missing_optional_section_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 60);
    assert_eq!(o.get("must_distinguish_required_from_optional_validation_path").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Manifest is consumed by the ecosystem summary."
#[test]
fn manifest_is_consumed_by_ecosystem_summary() {
    let doc = crate::common::load_toml(&manifest_path());
    let e = doc.get("ecosystem_summary_consumption_contract").and_then(|v| v.as_table()).expect(
        "[ecosystem_summary_consumption_contract] missing — acceptance: \
         \"Manifest is consumed by the ecosystem summary.\"",
    );
    assert_eq!(e.get("must_be_consumable_by_ecosystem_summary").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(e.get("must_emit_consumption_artifact").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(e.get("consumption_artifact_format").and_then(|v| v.as_str()), Some("json"));
    let path = e.get("consumption_artifact_relative_path_within_repo").and_then(|v| v.as_str()).unwrap();
    assert!(path.starts_with("projects/mamba/tests/"));
    assert!(path.ends_with(".json"));
    let required: Vec<&str> = e.get("consumption_artifact_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &["manifest_path", "schema_version", "checksum", "required_count", "optional_count"] {
        assert!(required.contains(f), "consumption_artifact_required_fields must include {f}");
    }
    assert_eq!(e.get("must_be_stable_across_runs_on_identical_input").and_then(|v| v.as_bool()), Some(true));
    let exit = e.get("missing_consumption_artifact_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 61);
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case",
        "module_name", "fixture_issue", "required", "mvp_objective",
        "manifest_path", "schema_version", "checksum",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "required_module_without_fixture_reference_fails_validation",
        "optional_modules_are_reported_separately",
        "manifest_is_consumed_by_ecosystem_summary",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("implementing_every_module_fixture").and_then(|v| v.as_bool()), Some(true));
}
