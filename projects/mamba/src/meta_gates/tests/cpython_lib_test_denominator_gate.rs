#![cfg(test)]

// Locks the shape of the CPython Lib/test denominator fixture
// pinned by tests/cpython/
// cpython_lib_test_denominator_gate/manifest.toml. Closes #1396.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/cpython_lib_test_denominator_gate/manifest.toml")
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
        Some("cpython_lib_test_denominator_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(1396));
    assert_eq!(m["profile"].as_str(), Some("conformance"));
    assert_eq!(
        m["family"].as_str(),
        Some("cpython_lib_test_denominator_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
    let related: Vec<_> = m["related_issues"]
        .as_array()
        .expect("related_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(related, vec![1265, 1397]);
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
fn surface_pins_lib_test_iteration_and_tiered_monitoring() {
    let s = &manifest()["surface"];
    for key in [
        "must_iterate_cpython_lib_test_test_files",
        "must_record_pass_fail_skip_error_per_module",
        "must_tier_modules_t1_t2_t3",
        "must_replace_self_defined_percentage_in_monitoring",
        "must_be_offline",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn denominator_source_is_vendored_cpython_lib_test() {
    let c = &manifest()["denominator_source_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("denominator_source_is_vendored_cpython_lib_test")
    );
    assert_eq!(
        c["denominator_source_relative_path"].as_str(),
        Some("projects/mamba/conformance/cpython-lib-test")
    );
    assert_eq!(
        c["denominator_source_relative_path_field_name"].as_str(),
        Some("denominator_source_path")
    );
    for key in [
        "must_pin_denominator_source_relative_path",
        "forbid_denominator_source_outside_project",
        "forbid_using_self_defined_fixtures_as_denominator",
        "must_distinguish_source_missing_from_substitution",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["self_defined_denominator_field_name"].as_str(),
        Some("self_defined_fixture_pass_rate")
    );
    assert_eq!(
        c["canonical_denominator_field_name"].as_str(),
        Some("cpython_lib_test_denominator")
    );
    assert_eq!(
        c["denominator_source_missing_failure_kind"].as_str(),
        Some("cpython_lib_test_denominator_source_missing")
    );
    assert_eq!(
        c["denominator_source_missing_exit_code"].as_integer(),
        Some(255)
    );
    assert_eq!(
        c["denominator_substituted_with_self_defined_failure_kind"].as_str(),
        Some("cpython_lib_test_denominator_substituted_with_self_defined")
    );
    assert_eq!(
        c["denominator_substituted_with_self_defined_exit_code"].as_integer(),
        Some(256)
    );
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
        Some("projects/mamba/conformance/cpython_lib_test_runner.rs")
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
        Some("cpython_lib_test_runner_missing")
    );
    assert_eq!(c["runner_missing_exit_code"].as_integer(), Some(257));
    assert_eq!(
        c["runner_outside_project_failure_kind"].as_str(),
        Some("cpython_lib_test_runner_outside_project")
    );
    assert_eq!(
        c["runner_outside_project_exit_code"].as_integer(),
        Some(258)
    );
}

#[test]
fn per_module_outcome_records_pass_fail_skip_error() {
    let c = &manifest()["per_module_outcome_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("per_module_outcome_records_pass_fail_skip_error")
    );
    for key in [
        "must_emit_per_module_outcome_json",
        "must_record_module_name_in_each_record",
        "must_record_outcome_in_each_record",
        "must_record_tier_in_each_record",
        "forbid_silently_collapsing_pass_and_skip",
        "forbid_silently_collapsing_fail_and_error",
        "forbid_dropping_modules_with_zero_passes",
        "must_distinguish_pass_skip_collapse_from_fail_error_collapse",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let outcomes: Vec<_> = c["required_outcome_values"]
        .as_array()
        .expect("required_outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(outcomes, vec!["pass", "fail", "skip", "error"]);
    assert_eq!(c["outcome_field_name"].as_str(), Some("outcome"));
    assert_eq!(
        c["per_module_record_field_name"].as_str(),
        Some("per_module_outcome")
    );
    let fields: Vec<_> = c["required_per_module_record_fields"]
        .as_array()
        .expect("required_per_module_record_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec!["module_name", "tier", "outcome", "duration_ms"]
    );
    assert_eq!(
        c["pass_collapsed_with_skip_failure_kind"].as_str(),
        Some("cpython_lib_test_pass_collapsed_with_skip")
    );
    assert_eq!(
        c["pass_collapsed_with_skip_exit_code"].as_integer(),
        Some(259)
    );
    assert_eq!(
        c["fail_collapsed_with_error_failure_kind"].as_str(),
        Some("cpython_lib_test_fail_collapsed_with_error")
    );
    assert_eq!(
        c["fail_collapsed_with_error_exit_code"].as_integer(),
        Some(260)
    );
}

#[test]
fn tier_definition_pins_t1_t2_t3_modules() {
    let c = &manifest()["tier_definition_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("tier_definition_t1_language_core_t2_frequent_stdlib_t3_long_tail")
    );
    let tiers: Vec<_> = c["required_tiers"]
        .as_array()
        .expect("required_tiers")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(tiers, vec!["t1", "t2", "t3"]);
    assert_eq!(c["tier_field_name"].as_str(), Some("tier"));
    let t1: Vec<_> = c["t1_language_core_modules"]
        .as_array()
        .expect("t1_language_core_modules")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(t1.len(), 25);
    assert!(t1.contains(&"test_grammar"));
    assert!(t1.contains(&"test_yield_from"));
    let t2: Vec<_> = c["t2_frequent_stdlib_modules"]
        .as_array()
        .expect("t2_frequent_stdlib_modules")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(t2.len(), 19);
    assert!(t2.contains(&"test_asyncio"));
    assert!(t2.contains(&"test_unittest"));
    for key in [
        "t3_long_tail_is_everything_else",
        "forbid_collapsing_tiers_into_overall_only",
        "forbid_omitting_tier_aggregate_in_monitoring",
        "must_aggregate_per_tier_pass_total",
        "must_distinguish_tier_missing_from_unknown_tier",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["tier_aggregate_field_name"].as_str(),
        Some("tier_aggregate")
    );
    let agg: Vec<_> = c["required_tier_aggregate_fields"]
        .as_array()
        .expect("required_tier_aggregate_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(agg, vec!["tier", "pass_count", "total_count"]);
    assert_eq!(
        c["tier_missing_failure_kind"].as_str(),
        Some("cpython_lib_test_tier_missing")
    );
    assert_eq!(c["tier_missing_exit_code"].as_integer(), Some(261));
    assert_eq!(
        c["unknown_tier_failure_kind"].as_str(),
        Some("cpython_lib_test_unknown_tier")
    );
    assert_eq!(c["unknown_tier_exit_code"].as_integer(), Some(262));
}

#[test]
fn monitoring_replacement_pins_tiered_denominator_in_monitoring() {
    let c = &manifest()["monitoring_replacement_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("monitoring_loop_replaces_self_defined_percent_with_tiered_denominator")
    );
    for key in [
        "must_emit_tier_aggregate_for_monitoring",
        "must_omit_self_defined_percent_as_conformance_metric",
        "forbid_self_defined_percent_being_used_as_conformance",
        "forbid_silently_omitting_t1_or_t2_or_t3_in_monitoring",
        "must_distinguish_self_defined_percent_used_as_conformance_from_tier_omitted",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["monitoring_report_field_name"].as_str(),
        Some("monitoring_report")
    );
    let fields: Vec<_> = c["required_monitoring_report_fields"]
        .as_array()
        .expect("required_monitoring_report_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "t1_pass_count",
            "t1_total_count",
            "t2_pass_count",
            "t2_total_count",
            "t3_pass_count",
            "t3_total_count",
        ]
    );
    assert_eq!(
        c["monitoring_self_defined_percent_used_as_conformance_failure_kind"].as_str(),
        Some("cpython_lib_test_monitoring_self_defined_percent_used_as_conformance")
    );
    assert_eq!(
        c["monitoring_self_defined_percent_used_as_conformance_exit_code"].as_integer(),
        Some(263)
    );
    assert_eq!(
        c["monitoring_tier_omitted_failure_kind"].as_str(),
        Some("cpython_lib_test_monitoring_tier_omitted")
    );
    assert_eq!(
        c["monitoring_tier_omitted_exit_code"].as_integer(),
        Some(264)
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
            "denominator_source_path",
            "runner_relative_path",
            "per_module_outcome",
            "tier_aggregate",
            "monitoring_report",
            "module_name",
            "tier",
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
    assert_eq!(outcomes, vec!["pass", "fail", "skip", "error", "missing"]);
    let cases: Vec<_> = r["case_values"]
        .as_array()
        .expect("case_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        cases,
        vec![
            "denominator_source_is_vendored_cpython_lib_test",
            "runner_lives_under_projects_mamba_conformance",
            "per_module_outcome_records_pass_fail_skip_error",
            "tier_definition_t1_language_core_t2_frequent_stdlib_t3_long_tail",
            "monitoring_loop_replaces_self_defined_percent_with_tiered_denominator",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "behavioural_correctness_of_individual_apis",
        "surface_coverage_per_module",
        "upstreaming_back_to_cpython",
        "runtime_implementation_of_lib_test_runner",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
