#![cfg(test)]

// Locks the shape of the MVP Py3.12 ecosystem umbrella fixture
// pinned by tests/governance/gates/mvp/mvp_py312_ecosystem_umbrella_gate/
// manifest.toml. Closes #1265.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/mvp_py312_ecosystem_umbrella_gate/manifest.toml")
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
        Some("mvp_py312_ecosystem_umbrella_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(1265));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(
        m["family"].as_str(),
        Some("mvp_py312_ecosystem_umbrella_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
    let children: Vec<_> = m["child_issues"]
        .as_array()
        .expect("child_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(children, vec![1234, 1257, 1259, 1263, 1396, 1397, 1537]);
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
fn surface_pins_target_3p_denominator_blockers_and_exclusions() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_py312_language_and_runtime_target",
        "must_cover_three_canonical_3p_programs_run_unmodified",
        "must_cover_stdlib_surface_has_explicit_denominator",
        "must_cover_no_p0_p1_conformance_blockers_remain_open",
        "must_cover_future_versions_and_other_epics_are_excluded",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_py312_is_the_target() {
    let c = &manifest()["r1_py312_is_the_target_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("py312_language_and_runtime_behavior_is_the_target_for_mvp")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_pin_python_target_to_3_12",
        "must_run_on_real_py312_grammar_and_runtime_behavior",
        "forbid_silently_regressing_to_3_10_or_3_11",
        "forbid_treating_py313_features_as_required_under_mvp",
        "must_distinguish_silent_regression_from_py313_required",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["python_target_major_field_name"].as_str(),
        Some("python_target_major")
    );
    assert_eq!(
        c["python_target_minor_field_name"].as_str(),
        Some("python_target_minor")
    );
    assert_eq!(c["expected_python_target_major"].as_integer(), Some(3));
    assert_eq!(c["expected_python_target_minor"].as_integer(), Some(12));
    assert_eq!(
        c["silent_regression_failure_kind"].as_str(),
        Some("mvp_ecosystem_silent_regression_to_pre_3_12")
    );
    assert_eq!(c["silent_regression_exit_code"].as_integer(), Some(323));
    assert_eq!(
        c["py313_required_failure_kind"].as_str(),
        Some("mvp_ecosystem_py313_feature_required_under_mvp")
    );
    assert_eq!(c["py313_required_exit_code"].as_integer(), Some(324));
}

#[test]
fn r2_three_canonical_3p_programs() {
    let c = &manifest()["r2_three_canonical_3p_programs_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("three_canonical_3p_programs_pytest_flask_requests_run_unmodified")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_wire_pytest_hello_program",
        "must_wire_flask_hello_program",
        "must_wire_requests_https_get_program",
        "must_wire_3p_libs_umbrella_under_1263",
        "forbid_dropping_any_canonical_3p_program",
        "forbid_silently_substituting_canonical_3p_program",
        "must_distinguish_program_missing_from_substituted",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let progs: Vec<_> = c["required_canonical_3p_programs"]
        .as_array()
        .expect("required_canonical_3p_programs")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        progs,
        vec![
            "pytest_hello_world",
            "flask_hello_world",
            "requests_https_get",
        ]
    );
    assert_eq!(
        c["canonical_3p_programs_field_name"].as_str(),
        Some("canonical_3p_programs")
    );
    assert_eq!(
        c["threep_umbrella_relative_path"].as_str(),
        Some("projects/mamba/tests/governance/gates/third_party/c3_3p_libs_conformance_umbrella_gate/manifest.toml")
    );
    assert_eq!(
        c["threep_umbrella_relative_path_field_name"].as_str(),
        Some("threep_umbrella_relative_path")
    );
    assert_eq!(
        c["canonical_3p_program_missing_failure_kind"].as_str(),
        Some("mvp_ecosystem_canonical_3p_program_missing")
    );
    assert_eq!(
        c["canonical_3p_program_missing_exit_code"].as_integer(),
        Some(325)
    );
    assert_eq!(
        c["canonical_3p_program_substituted_failure_kind"].as_str(),
        Some("mvp_ecosystem_canonical_3p_program_silently_substituted")
    );
    assert_eq!(
        c["canonical_3p_program_substituted_exit_code"].as_integer(),
        Some(326)
    );
}

#[test]
fn r3_stdlib_denominator_is_explicit() {
    let c = &manifest()["r3_stdlib_denominator_is_explicit_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("stdlib_surface_has_explicit_denominator_via_lib_test_and_typeshed")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_pin_cpython_lib_test_denominator_under_1396",
        "must_pin_typeshed_surface_denominator_under_1397",
        "must_report_per_module_pass_fail_in_ci",
        "forbid_relying_on_implicit_denominator",
        "forbid_dropping_either_denominator_dimension",
        "must_distinguish_implicit_denominator_from_dimension_dropped",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["cpython_lib_test_gate_relative_path"].as_str(),
        Some("projects/mamba/tests/governance/gates/cpython_lib_test_denominator_gate/manifest.toml")
    );
    assert_eq!(
        c["typeshed_surface_gate_relative_path"].as_str(),
        Some("projects/mamba/tests/governance/gates/typeshed_surface_gate/manifest.toml")
    );
    assert_eq!(
        c["cpython_lib_test_gate_relative_path_field_name"].as_str(),
        Some("cpython_lib_test_gate_relative_path")
    );
    assert_eq!(
        c["typeshed_surface_gate_relative_path_field_name"].as_str(),
        Some("typeshed_surface_gate_relative_path")
    );
    assert_eq!(
        c["denominator_implicit_failure_kind"].as_str(),
        Some("mvp_ecosystem_stdlib_denominator_is_implicit")
    );
    assert_eq!(
        c["denominator_implicit_exit_code"].as_integer(),
        Some(327)
    );
    assert_eq!(
        c["denominator_dimension_dropped_failure_kind"].as_str(),
        Some("mvp_ecosystem_stdlib_denominator_dimension_dropped")
    );
    assert_eq!(
        c["denominator_dimension_dropped_exit_code"].as_integer(),
        Some(328)
    );
}

#[test]
fn r4_no_p0_p1_blockers_remain_open() {
    let c = &manifest()["r4_no_p0_p1_blockers_remain_open_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("no_p0_or_p1_conformance_blockers_remain_open_for_py312_subset")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_track_p0_blocker_count",
        "must_track_p1_blocker_count",
        "must_require_zero_p0_blockers_for_umbrella_green",
        "must_require_zero_p1_blockers_for_umbrella_green",
        "forbid_treating_p0_blocker_as_p2_to_pass",
        "forbid_silently_closing_blocker_without_fix",
        "must_distinguish_p0_from_p1_blocker_open",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let tiers: Vec<_> = c["required_priority_tiers"]
        .as_array()
        .expect("required_priority_tiers")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(tiers, vec!["p0", "p1", "p2"]);
    assert_eq!(
        c["priority_tier_field_name"].as_str(),
        Some("priority_tier")
    );
    assert_eq!(
        c["p0_blocker_count_field_name"].as_str(),
        Some("p0_blocker_count")
    );
    assert_eq!(
        c["p1_blocker_count_field_name"].as_str(),
        Some("p1_blocker_count")
    );
    assert_eq!(
        c["expected_p0_blocker_count_for_green"].as_integer(),
        Some(0)
    );
    assert_eq!(
        c["expected_p1_blocker_count_for_green"].as_integer(),
        Some(0)
    );
    assert_eq!(
        c["p0_blocker_open_failure_kind"].as_str(),
        Some("mvp_ecosystem_p0_blocker_remains_open")
    );
    assert_eq!(c["p0_blocker_open_exit_code"].as_integer(), Some(329));
    assert_eq!(
        c["p1_blocker_open_failure_kind"].as_str(),
        Some("mvp_ecosystem_p1_blocker_remains_open")
    );
    assert_eq!(c["p1_blocker_open_exit_code"].as_integer(), Some(330));
}

#[test]
fn r5_other_epics_explicitly_excluded() {
    let c = &manifest()["r5_other_epics_explicitly_excluded_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("future_versions_perf_mambalibs_pkg_manager_are_excluded_from_umbrella")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_exclude_py313_conformance_from_umbrella",
        "must_exclude_py314_conformance_from_umbrella",
        "must_exclude_10x_perf_target_from_umbrella",
        "must_exclude_mambalibs_from_umbrella",
        "must_exclude_uv_like_package_manager_from_umbrella",
        "forbid_folding_other_epic_into_umbrella_to_pass",
        "must_distinguish_epic_folded_in_from_excluded_marked_required",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["excluded_epic_issues_field_name"].as_str(),
        Some("excluded_epic_issues")
    );
    let excluded: Vec<_> = c["excluded_epic_issues"]
        .as_array()
        .expect("excluded_epic_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(excluded, vec![1266, 1267, 1260, 2459, 751]);
    assert_eq!(
        c["epic_folded_into_umbrella_failure_kind"].as_str(),
        Some("mvp_ecosystem_other_epic_folded_into_umbrella")
    );
    assert_eq!(
        c["epic_folded_into_umbrella_exit_code"].as_integer(),
        Some(331)
    );
    assert_eq!(
        c["excluded_epic_marked_required_failure_kind"].as_str(),
        Some("mvp_ecosystem_excluded_epic_marked_required")
    );
    assert_eq!(
        c["excluded_epic_marked_required_exit_code"].as_integer(),
        Some(332)
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
            "requirement_id",
            "python_target_major",
            "python_target_minor",
            "canonical_3p_programs",
            "threep_umbrella_relative_path",
            "cpython_lib_test_gate_relative_path",
            "typeshed_surface_gate_relative_path",
            "priority_tier",
            "p0_blocker_count",
            "p1_blocker_count",
            "excluded_epic_issues",
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
            "py312_language_and_runtime_behavior_is_the_target_for_mvp",
            "three_canonical_3p_programs_pytest_flask_requests_run_unmodified",
            "stdlib_surface_has_explicit_denominator_via_lib_test_and_typeshed",
            "no_p0_or_p1_conformance_blockers_remain_open_for_py312_subset",
            "future_versions_perf_mambalibs_pkg_manager_are_excluded_from_umbrella",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "python_3_13_conformance",
        "python_3_14_conformance",
        "ten_x_performance_target",
        "mambalibs_import_path",
        "uv_like_package_management",
        "c_extension_fast_paths",
        "runtime_implementation_per_child_program",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
