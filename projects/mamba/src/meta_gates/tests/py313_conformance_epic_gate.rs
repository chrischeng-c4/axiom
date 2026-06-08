#![cfg(test)]

// Locks the shape of the Py3.13 conformance epic gate fixture pinned
// by tests/governance/gates/mvp/py313_conformance_epic_gate/manifest.toml.
// Closes #1266.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/py313_conformance_epic_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("py313_conformance_epic_gate"));
    assert_eq!(m["issue"].as_integer(), Some(1266));
    assert_eq!(m["parent_issue"].as_integer(), Some(1265));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(m["family"].as_str(), Some("py313_conformance_epic_gate"));
    assert_eq!(m["network"].as_str(), Some("offline"));
    let related: Vec<_> = m["related_issues"]
        .as_array()
        .expect("related_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(related, vec![1267, 1260, 751]);
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
fn python_target_is_pinned_to_3_13() {
    let py = &manifest()["python_target"];
    assert_eq!(py["python_major"].as_integer(), Some(3));
    assert_eq!(py["python_minor"].as_integer(), Some(13));
    assert_eq!(py["must_be_python_3_13"].as_bool(), Some(true));
}

#[test]
fn surface_pins_target_gating_criteria_deferrals_and_suite_fork() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_py313_is_conformance_target",
        "must_cover_hard_gated_on_py312_master_epic",
        "must_cover_eight_conformance_criteria_c1_through_c8",
        "must_cover_explicit_deferrals_are_out_of_scope",
        "must_cover_conformance_suite_forks_into_py313_subdir",
        "must_be_offline",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_py313_is_conformance_target() {
    let c = &manifest()["r1_py313_is_conformance_target_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("py313_is_the_conformance_target_for_this_epic")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_pin_python_target_to_3_13",
        "forbid_silently_targeting_3_12",
        "forbid_silently_targeting_3_14",
        "must_distinguish_target_3_12_from_target_3_14",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["expected_python_target_major"].as_integer(), Some(3));
    assert_eq!(c["expected_python_target_minor"].as_integer(), Some(13));
    assert_eq!(
        c["silently_targeting_3_12_failure_kind"].as_str(),
        Some("py313_epic_silently_targets_3_12")
    );
    assert_eq!(
        c["silently_targeting_3_12_exit_code"].as_integer(),
        Some(333)
    );
    assert_eq!(
        c["silently_targeting_3_14_failure_kind"].as_str(),
        Some("py313_epic_silently_targets_3_14")
    );
    assert_eq!(
        c["silently_targeting_3_14_exit_code"].as_integer(),
        Some(334)
    );
}

#[test]
fn r2_hard_gated_on_py312_epic() {
    let c = &manifest()["r2_hard_gated_on_py312_epic_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("py313_epic_is_hard_gated_on_py312_master_epic_completing")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_block_until_py312_master_epic_complete",
        "must_link_py312_master_epic_as_hard_dependency",
        "forbid_starting_workstream_before_py312_complete",
        "forbid_silently_lifting_the_hard_gate",
        "must_distinguish_workstream_started_from_gate_silently_lifted",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["hard_dependency_epic_issue_field_name"].as_str(),
        Some("hard_dependency_epic_issue")
    );
    assert_eq!(
        c["expected_hard_dependency_epic_issue"].as_integer(),
        Some(1265)
    );
    let statuses: Vec<_> = c["allowed_py312_master_epic_status_values"]
        .as_array()
        .expect("allowed_py312_master_epic_status_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(statuses, vec!["incomplete", "complete"]);
    assert_eq!(
        c["workstream_started_before_py312_complete_failure_kind"].as_str(),
        Some("py313_workstream_started_before_py312_epic_complete")
    );
    assert_eq!(
        c["workstream_started_before_py312_complete_exit_code"].as_integer(),
        Some(335)
    );
    assert_eq!(
        c["hard_gate_silently_lifted_failure_kind"].as_str(),
        Some("py313_hard_gate_silently_lifted")
    );
    assert_eq!(
        c["hard_gate_silently_lifted_exit_code"].as_integer(),
        Some(336)
    );
}

#[test]
fn r3_eight_conformance_criteria() {
    let c = &manifest()["r3_eight_conformance_criteria_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("eight_py313_conformance_criteria_c1_through_c8_are_pinned")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_pin_c1_cpython_lib_test_at_or_above_95_percent",
        "must_pin_c2_pep_667_locals_mutable_dict",
        "must_pin_c3_pep_669_sys_monitoring_events",
        "must_pin_c4_pep_742_typing_type_is",
        "must_pin_c5_pep_705_typing_read_only",
        "must_pin_c6_pep_696_type_parameter_defaults",
        "must_pin_c7_pep_594_removed_stdlib_module_not_found",
        "must_pin_c8_repl_features_degrade_gracefully",
        "forbid_silently_dropping_a_conformance_criterion",
        "forbid_collapsing_multiple_criteria_into_one_observable",
        "must_distinguish_criterion_dropped_from_collapsed",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let crits: Vec<_> = c["required_conformance_criteria"]
        .as_array()
        .expect("required_conformance_criteria")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        crits,
        vec![
            "c1_cpython_lib_test_at_or_above_95_percent",
            "c2_pep_667_locals_mutable_dict",
            "c3_pep_669_sys_monitoring_events",
            "c4_pep_742_typing_type_is",
            "c5_pep_705_typing_read_only",
            "c6_pep_696_type_parameter_defaults",
            "c7_pep_594_removed_stdlib_module_not_found",
            "c8_repl_features_degrade_gracefully",
        ]
    );
    assert_eq!(
        c["conformance_criteria_field_name"].as_str(),
        Some("conformance_criteria")
    );
    assert_eq!(
        c["c1_lib_test_minimum_ratio_field_name"].as_str(),
        Some("c1_lib_test_minimum_ratio")
    );
    assert_eq!(
        c["expected_c1_lib_test_minimum_ratio"].as_float(),
        Some(0.95)
    );
    assert_eq!(
        c["criterion_dropped_failure_kind"].as_str(),
        Some("py313_conformance_criterion_dropped")
    );
    assert_eq!(c["criterion_dropped_exit_code"].as_integer(), Some(337));
    assert_eq!(
        c["criteria_collapsed_failure_kind"].as_str(),
        Some("py313_conformance_criteria_collapsed_into_one_observable")
    );
    assert_eq!(c["criteria_collapsed_exit_code"].as_integer(), Some(338));
}

#[test]
fn r4_explicit_deferrals_are_out_of_scope() {
    let c = &manifest()["r4_explicit_deferrals_are_out_of_scope_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("pep_703_pep_744_mobile_tiers_and_perf_are_explicit_deferrals")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_exclude_pep_703_free_threaded_no_gil",
        "must_exclude_pep_744_copy_and_patch_jit",
        "must_exclude_ios_tier_support",
        "must_exclude_android_tier_support",
        "must_exclude_py313_performance_target",
        "forbid_folding_deferral_into_epic_to_pass",
        "forbid_silently_dropping_deferral_marker",
        "must_distinguish_deferral_folded_in_from_marker_dropped",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let defs: Vec<_> = c["explicit_deferrals"]
        .as_array()
        .expect("explicit_deferrals")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        defs,
        vec![
            "pep_703_free_threaded_no_gil",
            "pep_744_copy_and_patch_jit",
            "ios_tier_support",
            "android_tier_support",
            "py313_performance_target",
        ]
    );
    assert_eq!(
        c["deferral_folded_into_epic_failure_kind"].as_str(),
        Some("py313_deferral_folded_into_epic_to_pass")
    );
    assert_eq!(
        c["deferral_folded_into_epic_exit_code"].as_integer(),
        Some(339)
    );
    assert_eq!(
        c["deferral_marker_dropped_failure_kind"].as_str(),
        Some("py313_deferral_marker_silently_dropped")
    );
    assert_eq!(
        c["deferral_marker_dropped_exit_code"].as_integer(),
        Some(340)
    );
}

#[test]
fn r5_conformance_suite_forks_into_py313_subdir() {
    let c = &manifest()["r5_conformance_suite_forks_into_py313_subdir_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("conformance_suite_forks_into_py313_subdir_with_pinned_cpython_tag")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_fork_conformance_suite_into_py313_subdir",
        "must_keep_py312_and_py313_suites_separated",
        "must_pin_cpython_3_13_release_tag",
        "forbid_running_py313_tests_in_py312_subdir",
        "forbid_silently_overwriting_py312_baseline_with_py313_results",
        "must_distinguish_wrong_subdir_from_baseline_overwritten",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["py313_subdir_relative_path"].as_str(),
        Some("projects/mamba/tests/cpython/cpython_ported/py313")
    );
    assert_eq!(
        c["py313_subdir_relative_path_field_name"].as_str(),
        Some("py313_subdir_relative_path")
    );
    assert_eq!(
        c["cpython_3_13_release_tag_field_name"].as_str(),
        Some("cpython_3_13_release_tag")
    );
    let tags: Vec<_> = c["allowed_cpython_3_13_release_tag_formats"]
        .as_array()
        .expect("allowed_cpython_3_13_release_tag_formats")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(tags, vec!["v3.13.X", "v3.13.X-rcY"]);
    assert_eq!(
        c["py313_tests_in_py312_subdir_failure_kind"].as_str(),
        Some("py313_tests_landed_in_py312_subdir")
    );
    assert_eq!(
        c["py313_tests_in_py312_subdir_exit_code"].as_integer(),
        Some(341)
    );
    assert_eq!(
        c["py312_baseline_overwritten_failure_kind"].as_str(),
        Some("py313_results_overwrote_py312_baseline")
    );
    assert_eq!(
        c["py312_baseline_overwritten_exit_code"].as_integer(),
        Some(342)
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
            "hard_dependency_epic_issue",
            "py312_master_epic_status",
            "conformance_criteria",
            "c1_lib_test_minimum_ratio",
            "explicit_deferrals",
            "py313_subdir_relative_path",
            "cpython_3_13_release_tag",
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
    assert_eq!(
        outcomes,
        vec!["pass", "fail", "missing", "skip", "blocked"]
    );
    let cases: Vec<_> = r["case_values"]
        .as_array()
        .expect("case_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        cases,
        vec![
            "py313_is_the_conformance_target_for_this_epic",
            "py313_epic_is_hard_gated_on_py312_master_epic_completing",
            "eight_py313_conformance_criteria_c1_through_c8_are_pinned",
            "pep_703_pep_744_mobile_tiers_and_perf_are_explicit_deferrals",
            "conformance_suite_forks_into_py313_subdir_with_pinned_cpython_tag",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "pep_703_free_threaded_no_gil",
        "pep_744_copy_and_patch_jit",
        "ios_tier_support",
        "android_tier_support",
        "py313_performance_target",
        "uv_like_package_management",
        "cpython_full_regression_suite",
        "implementation_of_pep_667_frame_locals_rework",
        "implementation_of_pep_669_sys_monitoring_hooks",
        "implementation_of_pep_696_generic_substitution",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
