#![cfg(test)]

// Locks the shape of the typeshed-based per-module surface coverage
// fixture pinned by tests/governance/gates/typeshed_surface_gate/
// manifest.toml. Closes #1397.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/typeshed_surface_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("typeshed_surface_gate"));
    assert_eq!(m["issue"].as_integer(), Some(1397));
    assert_eq!(m["profile"].as_str(), Some("conformance"));
    assert_eq!(m["family"].as_str(), Some("typeshed_surface_gate"));
    assert_eq!(m["network"].as_str(), Some("offline"));
    let related: Vec<_> = m["related_issues"]
        .as_array()
        .expect("related_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(related, vec![1396, 1265]);
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
fn surface_pins_stub_parse_introspection_per_module_and_tier() {
    let s = &manifest()["surface"];
    for key in [
        "must_parse_typeshed_stub_for_each_module",
        "must_introspect_mamba_module_for_each_module",
        "must_report_per_module_attribute_coverage",
        "must_aggregate_coverage_by_tier",
        "must_be_offline",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn runner_lives_under_projects_mamba_conformance() {
    let c = &manifest()["runner_location_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("runner_lives_under_projects_mamba_conformance")
    );
    assert_eq!(
        c["runner_relative_path"].as_str(),
        Some("projects/mamba/conformance/typeshed_surface.rs")
    );
    assert_eq!(
        c["runner_relative_path_field_name"].as_str(),
        Some("runner_relative_path")
    );
    for key in [
        "must_pin_runner_relative_path",
        "forbid_runner_outside_projects_mamba_conformance",
        "must_distinguish_runner_missing_from_runner_outside_project",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["runner_missing_failure_kind"].as_str(),
        Some("typeshed_surface_runner_missing")
    );
    assert_eq!(c["runner_missing_exit_code"].as_integer(), Some(247));
    assert_eq!(
        c["runner_outside_project_failure_kind"].as_str(),
        Some("typeshed_surface_runner_outside_project")
    );
    assert_eq!(
        c["runner_outside_project_exit_code"].as_integer(),
        Some(248)
    );
}

#[test]
fn per_module_attribute_coverage_json_is_emitted() {
    let c = &manifest()["per_module_coverage_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("per_module_attribute_coverage_json_is_emitted")
    );
    for key in [
        "must_emit_per_module_coverage_json",
        "must_include_module_name_in_each_record",
        "must_include_declared_public_names_count_in_each_record",
        "must_include_present_public_names_count_in_each_record",
        "must_include_missing_public_names_list_in_each_record",
        "must_include_tier_in_each_record",
        "forbid_collapsing_module_records_into_aggregate_only",
        "forbid_silently_dropping_modules_with_zero_coverage",
        "must_distinguish_record_missing_from_zero_coverage_module_dropped",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_per_module_record_fields"]
        .as_array()
        .expect("required_per_module_record_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "module_name",
            "tier",
            "declared_public_names_count",
            "present_public_names_count",
            "missing_public_names",
            "coverage_ratio",
        ]
    );
    assert_eq!(
        c["coverage_record_field_name"].as_str(),
        Some("per_module_coverage")
    );
    assert_eq!(
        c["per_module_record_missing_failure_kind"].as_str(),
        Some("typeshed_surface_per_module_record_missing")
    );
    assert_eq!(
        c["per_module_record_missing_exit_code"].as_integer(),
        Some(249)
    );
    assert_eq!(
        c["zero_coverage_module_dropped_failure_kind"].as_str(),
        Some("typeshed_surface_zero_coverage_module_dropped")
    );
    assert_eq!(
        c["zero_coverage_module_dropped_exit_code"].as_integer(),
        Some(250)
    );
}

#[test]
fn tier_aggregation_t1_t2_t3_alongside_lib_test_pass_rate() {
    let c = &manifest()["tier_aggregation_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("tier_aggregation_t1_t2_t3_alongside_lib_test_pass_rate")
    );
    for key in [
        "must_report_t1_aggregate",
        "must_report_t2_aggregate",
        "must_report_t3_aggregate",
        "must_report_alongside_lib_test_pass_rate",
        "forbid_collapsing_tiers_into_overall_only",
        "forbid_omitting_lib_test_pass_rate_neighbor",
        "must_distinguish_tier_aggregate_missing_from_lib_test_neighbor_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let tiers: Vec<_> = c["required_tier_names"]
        .as_array()
        .expect("required_tier_names")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(tiers, vec!["t1", "t2", "t3"]);
    assert_eq!(c["tier_field_name"].as_str(), Some("tier"));
    let fields: Vec<_> = c["required_aggregate_fields"]
        .as_array()
        .expect("required_aggregate_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "tier",
            "modules_total",
            "declared_public_names_total",
            "present_public_names_total",
            "coverage_ratio",
            "lib_test_pass_count",
            "lib_test_total_count",
        ]
    );
    assert_eq!(c["aggregate_field_name"].as_str(), Some("tier_aggregate"));
    assert_eq!(
        c["tier_aggregate_missing_failure_kind"].as_str(),
        Some("typeshed_surface_tier_aggregate_missing")
    );
    assert_eq!(
        c["tier_aggregate_missing_exit_code"].as_integer(),
        Some(251)
    );
    assert_eq!(
        c["lib_test_neighbor_missing_failure_kind"].as_str(),
        Some("typeshed_surface_lib_test_neighbor_missing")
    );
    assert_eq!(
        c["lib_test_neighbor_missing_exit_code"].as_integer(),
        Some(252)
    );
}

#[test]
fn partial_module_must_not_be_counted_as_full() {
    let c = &manifest()["partial_vs_full_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("partial_module_must_not_be_counted_as_full")
    );
    for key in [
        "must_compute_coverage_ratio_as_present_over_declared",
        "must_treat_missing_public_names_as_uncovered",
        "forbid_treating_module_present_as_full_coverage",
        "forbid_implicit_coverage_inflation",
        "must_distinguish_coverage_inflation_from_partial_misreported_as_full",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["coverage_ratio_field_name"].as_str(),
        Some("coverage_ratio")
    );
    assert_eq!(
        c["coverage_inflation_failure_kind"].as_str(),
        Some("typeshed_surface_coverage_inflation")
    );
    assert_eq!(c["coverage_inflation_exit_code"].as_integer(), Some(253));
    assert_eq!(
        c["module_present_but_partial_misreported_as_full_failure_kind"].as_str(),
        Some("typeshed_surface_module_partial_misreported_as_full")
    );
    assert_eq!(
        c["module_present_but_partial_misreported_as_full_exit_code"].as_integer(),
        Some(254)
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
            "runner_relative_path",
            "per_module_coverage",
            "tier_aggregate",
            "coverage_ratio",
            "tier",
            "module_name",
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
            "runner_lives_under_projects_mamba_conformance",
            "per_module_attribute_coverage_json_is_emitted",
            "tier_aggregation_t1_t2_t3_alongside_lib_test_pass_rate",
            "partial_module_must_not_be_counted_as_full",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "behavioural_correctness_of_named_public_apis",
        "typeshed_upstream_curation",
        "runtime_implementation_of_stub_parser",
        "runtime_implementation_of_introspection_runner",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
