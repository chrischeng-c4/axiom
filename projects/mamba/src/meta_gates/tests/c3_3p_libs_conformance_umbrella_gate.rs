#![cfg(test)]

// Locks the shape of the C3 3P Libs conformance umbrella fixture
// pinned by tests/governance/gates/third_party/
// c3_3p_libs_conformance_umbrella_gate/manifest.toml. Closes #1263.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "tests/governance/gates/third_party/c3_3p_libs_conformance_umbrella_gate/manifest.toml",
    )
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
        Some("c3_3p_libs_conformance_umbrella_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(1263));
    assert_eq!(m["parent_issue"].as_integer(), Some(1265));
    assert_eq!(m["profile"].as_str(), Some("third_party"));
    assert_eq!(
        m["family"].as_str(),
        Some("c3_3p_libs_conformance_umbrella_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
    let children: Vec<_> = m["child_issues"]
        .as_array()
        .expect("child_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(children, vec![1234, 1257, 1259]);
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
fn surface_pins_umbrella_aggregation_and_blocker_reporting() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_required_programs_set",
        "must_cover_child_gate_wiring",
        "must_cover_umbrella_green_only_when_all_children_green",
        "must_cover_per_child_blocker_tier_reporting",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn exactly_three_required_programs_pytest_flask_requests() {
    let c = &manifest()["required_programs_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("exactly_three_required_programs_pytest_flask_requests")
    );
    let progs: Vec<_> = c["required_programs"]
        .as_array()
        .expect("required_programs")
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
        c["required_programs_field_name"].as_str(),
        Some("required_programs")
    );
    for key in [
        "must_pin_exactly_three_required_programs",
        "forbid_adding_or_dropping_required_program",
        "forbid_implicit_required_program",
        "must_distinguish_required_program_missing_from_unknown_required_program",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["required_program_missing_failure_kind"].as_str(),
        Some("c3_umbrella_required_program_missing")
    );
    assert_eq!(
        c["required_program_missing_exit_code"].as_integer(),
        Some(295)
    );
    assert_eq!(
        c["unknown_required_program_failure_kind"].as_str(),
        Some("c3_umbrella_unknown_required_program")
    );
    assert_eq!(
        c["unknown_required_program_exit_code"].as_integer(),
        Some(296)
    );
}

#[test]
fn each_child_program_is_wired_to_its_own_fixture() {
    let c = &manifest()["child_gate_wiring_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("each_child_program_is_wired_to_its_own_fixture")
    );
    for key in [
        "must_wire_pytest_hello_to_child_gate",
        "must_wire_flask_hello_to_child_gate",
        "must_wire_requests_get_to_child_gate",
        "forbid_collapsing_children_into_a_single_combined_gate",
        "forbid_silently_dropping_a_child_from_the_wiring",
        "must_distinguish_child_gate_missing_from_child_gate_collapsed",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["child_gate_wiring_field_name"].as_str(),
        Some("child_gate_wiring")
    );
    assert_eq!(
        c["pytest_child_gate_relative_path"].as_str(),
        Some("projects/mamba/tests/governance/gates/third_party/c3_pytest_runs_unmodified_gate/manifest.toml")
    );
    assert_eq!(
        c["flask_child_gate_relative_path"].as_str(),
        Some("projects/mamba/tests/governance/gates/third_party/c3_flask_runs_unmodified_gate/manifest.toml")
    );
    assert_eq!(
        c["requests_child_gate_relative_path"].as_str(),
        Some("projects/mamba/tests/governance/gates/third_party/c3_requests_runs_unmodified_gate/manifest.toml")
    );
    assert_eq!(c["pytest_child_issue"].as_integer(), Some(1234));
    assert_eq!(c["flask_child_issue"].as_integer(), Some(1257));
    assert_eq!(c["requests_child_issue"].as_integer(), Some(1259));
    assert_eq!(
        c["child_gate_missing_failure_kind"].as_str(),
        Some("c3_umbrella_child_gate_missing")
    );
    assert_eq!(c["child_gate_missing_exit_code"].as_integer(), Some(297));
    assert_eq!(
        c["child_gate_collapsed_failure_kind"].as_str(),
        Some("c3_umbrella_child_gates_collapsed_into_single_gate")
    );
    assert_eq!(c["child_gate_collapsed_exit_code"].as_integer(), Some(298));
}

#[test]
fn umbrella_is_green_only_when_all_three_children_are_green() {
    let c = &manifest()["umbrella_green_aggregation_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("umbrella_is_green_only_when_all_three_children_are_green")
    );
    for key in [
        "must_aggregate_children_into_umbrella_outcome",
        "must_report_parent_fail_with_missing_child_names_when_partial",
        "must_report_per_child_outcome_alongside_umbrella_outcome",
        "forbid_umbrella_passing_when_any_child_failing",
        "forbid_umbrella_passing_when_any_child_missing",
        "forbid_silently_passing_umbrella_via_2_of_3",
        "must_distinguish_passing_with_child_fail_from_passing_with_child_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_per_child_outcome_fields"]
        .as_array()
        .expect("required_per_child_outcome_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec!["child_program", "child_outcome", "child_failure_kind"]
    );
    assert_eq!(
        c["per_child_outcome_field_name"].as_str(),
        Some("per_child_outcome")
    );
    assert_eq!(
        c["umbrella_outcome_field_name"].as_str(),
        Some("umbrella_outcome")
    );
    let vals: Vec<_> = c["allowed_umbrella_outcome_values"]
        .as_array()
        .expect("allowed_umbrella_outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(vals, vec!["pass", "fail", "missing"]);
    assert_eq!(
        c["umbrella_passing_with_child_fail_failure_kind"].as_str(),
        Some("c3_umbrella_passing_with_child_fail")
    );
    assert_eq!(
        c["umbrella_passing_with_child_fail_exit_code"].as_integer(),
        Some(299)
    );
    assert_eq!(
        c["umbrella_passing_with_child_missing_failure_kind"].as_str(),
        Some("c3_umbrella_passing_with_child_missing")
    );
    assert_eq!(
        c["umbrella_passing_with_child_missing_exit_code"].as_integer(),
        Some(300)
    );
}

#[test]
fn per_child_blocker_tier_reporting() {
    let c = &manifest()["blocker_tier_reporting_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("per_child_blocker_tier_reporting_core_stdlib_or_3p_dep")
    );
    for key in [
        "must_report_blocker_tier_per_child",
        "must_distinguish_core_blocker_from_stdlib_blocker",
        "must_distinguish_stdlib_blocker_from_3p_dep_blocker",
        "forbid_collapsing_blocker_tier_into_overall_only",
        "forbid_omitting_blocker_tier_when_child_blocked",
        "must_distinguish_blocker_tier_missing_from_unknown_blocker_tier",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let tiers: Vec<_> = c["required_blocker_tiers"]
        .as_array()
        .expect("required_blocker_tiers")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(tiers, vec!["core", "stdlib", "3p_dep"]);
    assert_eq!(c["blocker_tier_field_name"].as_str(), Some("blocker_tier"));
    assert_eq!(
        c["blocker_tier_missing_failure_kind"].as_str(),
        Some("c3_umbrella_blocker_tier_missing")
    );
    assert_eq!(c["blocker_tier_missing_exit_code"].as_integer(), Some(301));
    assert_eq!(
        c["unknown_blocker_tier_failure_kind"].as_str(),
        Some("c3_umbrella_unknown_blocker_tier")
    );
    assert_eq!(c["unknown_blocker_tier_exit_code"].as_integer(), Some(302));
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
            "required_programs",
            "child_gate_wiring",
            "per_child_outcome",
            "umbrella_outcome",
            "child_program",
            "child_outcome",
            "child_failure_kind",
            "blocker_tier",
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
            "exactly_three_required_programs_pytest_flask_requests",
            "each_child_program_is_wired_to_its_own_fixture",
            "umbrella_is_green_only_when_all_three_children_are_green",
            "per_child_blocker_tier_reporting_core_stdlib_or_3p_dep",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_per_child_program",
        "expanding_required_programs_beyond_pytest_flask_requests",
        "performance_gates",
        "c_extension_fast_paths",
        "ws2_stdlib_layer_implementation",
        "core_99_percent_conformance_epic_implementation",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
