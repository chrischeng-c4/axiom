#![cfg(test)]

// Locks the shape of the Py3.14 conformance epic gate fixture pinned
// by tests/governance/gates/mvp/py314_conformance_epic_gate/manifest.toml.
// Closes #1267.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/py314_conformance_epic_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("py314_conformance_epic_gate"));
    assert_eq!(m["issue"].as_integer(), Some(1267));
    assert_eq!(m["parent_issue"].as_integer(), Some(1266));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(m["family"].as_str(), Some("py314_conformance_epic_gate"));
    assert_eq!(m["network"].as_str(), Some("offline"));
    let related: Vec<_> = m["related_issues"]
        .as_array()
        .expect("related_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(related, vec![1265, 1260, 751]);
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
fn python_target_is_pinned_to_3_14() {
    let py = &manifest()["python_target"];
    assert_eq!(py["python_major"].as_integer(), Some(3));
    assert_eq!(py["python_minor"].as_integer(), Some(14));
    assert_eq!(py["must_be_python_3_14"].as_bool(), Some(true));
}

#[test]
fn surface_pins_target_double_gating_criteria_deferrals_and_suite_fork() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_py314_is_conformance_target",
        "must_cover_doubly_hard_gated_on_py313_and_py312_epics",
        "must_cover_eight_conformance_criteria_c1_through_c8",
        "must_cover_explicit_deferrals_are_out_of_scope",
        "must_cover_conformance_suite_forks_into_py314_subdir",
        "must_be_offline",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_py314_is_conformance_target() {
    let c = &manifest()["r1_py314_is_conformance_target_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("py314_is_the_conformance_target_for_this_epic")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_pin_python_target_to_3_14",
        "forbid_silently_targeting_3_12",
        "forbid_silently_targeting_3_13",
        "must_distinguish_target_3_12_from_target_3_13",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["expected_python_target_major"].as_integer(), Some(3));
    assert_eq!(c["expected_python_target_minor"].as_integer(), Some(14));
    assert_eq!(
        c["silently_targeting_3_12_failure_kind"].as_str(),
        Some("py314_epic_silently_targets_3_12")
    );
    assert_eq!(
        c["silently_targeting_3_12_exit_code"].as_integer(),
        Some(343)
    );
    assert_eq!(
        c["silently_targeting_3_13_failure_kind"].as_str(),
        Some("py314_epic_silently_targets_3_13")
    );
    assert_eq!(
        c["silently_targeting_3_13_exit_code"].as_integer(),
        Some(344)
    );
}

#[test]
fn r2_doubly_hard_gated() {
    let c = &manifest()["r2_doubly_hard_gated_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("py314_epic_is_doubly_hard_gated_on_py313_and_py312_epics")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_block_until_py313_epic_complete",
        "must_block_until_py312_epic_complete_transitively",
        "must_link_py313_epic_as_hard_dependency",
        "must_link_py312_epic_as_transitive_dependency",
        "forbid_starting_workstream_before_py313_complete",
        "forbid_silently_lifting_either_gate",
        "must_distinguish_workstream_started_from_gate_lifted",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_hard_dependency_epic_issue"].as_integer(),
        Some(1266)
    );
    assert_eq!(
        c["expected_transitive_dependency_epic_issue"].as_integer(),
        Some(1265)
    );
    let statuses: Vec<_> = c["allowed_epic_status_values"]
        .as_array()
        .expect("allowed_epic_status_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(statuses, vec!["incomplete", "complete"]);
    assert_eq!(
        c["workstream_started_before_py313_complete_failure_kind"].as_str(),
        Some("py314_workstream_started_before_py313_epic_complete")
    );
    assert_eq!(
        c["workstream_started_before_py313_complete_exit_code"].as_integer(),
        Some(345)
    );
    assert_eq!(
        c["gate_silently_lifted_failure_kind"].as_str(),
        Some("py314_doubly_hard_gate_silently_lifted")
    );
    assert_eq!(
        c["gate_silently_lifted_exit_code"].as_integer(),
        Some(346)
    );
}

#[test]
fn r3_eight_conformance_criteria() {
    let c = &manifest()["r3_eight_conformance_criteria_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("eight_py314_conformance_criteria_c1_through_c8_are_pinned")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_pin_c1_cpython_lib_test_at_or_above_95_percent",
        "must_pin_c2_pep_649_deferred_annotations",
        "must_pin_c3_pep_750_template_strings",
        "must_pin_c4_pep_758_parenthesis_free_except",
        "must_pin_c5_pep_765_finally_control_flow_syntax_warning",
        "must_pin_c6_pep_734_interpreters_stdlib",
        "must_pin_c7_pep_784_compression_zstd",
        "must_pin_c8_uuid_v6_v7_v8",
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
            "c2_pep_649_deferred_annotations",
            "c3_pep_750_template_strings",
            "c4_pep_758_parenthesis_free_except",
            "c5_pep_765_finally_control_flow_syntax_warning",
            "c6_pep_734_interpreters_stdlib",
            "c7_pep_784_compression_zstd",
            "c8_uuid_v6_v7_v8",
        ]
    );
    assert_eq!(
        c["expected_c1_lib_test_minimum_ratio"].as_float(),
        Some(0.95)
    );
    assert_eq!(
        c["criterion_dropped_failure_kind"].as_str(),
        Some("py314_conformance_criterion_dropped")
    );
    assert_eq!(c["criterion_dropped_exit_code"].as_integer(), Some(347));
    assert_eq!(
        c["criteria_collapsed_failure_kind"].as_str(),
        Some("py314_conformance_criteria_collapsed_into_one_observable")
    );
    assert_eq!(c["criteria_collapsed_exit_code"].as_integer(), Some(348));
}

#[test]
fn r4_explicit_deferrals_are_out_of_scope() {
    let c = &manifest()["r4_explicit_deferrals_are_out_of_scope_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("pep_779_pep_768_pep_793_pep_761_and_perf_are_explicit_deferrals")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_exclude_pep_779_free_threaded_ga",
        "must_exclude_pep_768_remote_debugging",
        "must_exclude_pep_793_new_c_api_for_type_construction",
        "must_exclude_pep_761_sigstore_signing",
        "must_exclude_py314_performance_target",
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
            "pep_779_free_threaded_ga",
            "pep_768_remote_debugging",
            "pep_793_new_c_api_for_type_construction",
            "pep_761_sigstore_signing",
            "py314_performance_target",
        ]
    );
    assert_eq!(
        c["deferral_folded_into_epic_failure_kind"].as_str(),
        Some("py314_deferral_folded_into_epic_to_pass")
    );
    assert_eq!(
        c["deferral_folded_into_epic_exit_code"].as_integer(),
        Some(349)
    );
    assert_eq!(
        c["deferral_marker_dropped_failure_kind"].as_str(),
        Some("py314_deferral_marker_silently_dropped")
    );
    assert_eq!(
        c["deferral_marker_dropped_exit_code"].as_integer(),
        Some(350)
    );
}

#[test]
fn r5_conformance_suite_forks_into_py314_subdir() {
    let c = &manifest()["r5_conformance_suite_forks_into_py314_subdir_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("conformance_suite_forks_into_py314_subdir_with_pinned_cpython_tag")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_fork_conformance_suite_into_py314_subdir",
        "must_keep_py312_py313_py314_suites_separated",
        "must_pin_cpython_3_14_release_tag",
        "forbid_running_py314_tests_in_py313_subdir",
        "forbid_silently_overwriting_py313_baseline_with_py314_results",
        "must_distinguish_wrong_subdir_from_baseline_overwritten",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["py314_subdir_relative_path"].as_str(),
        Some("projects/mamba/tests/cpython/cpython_ported/py314")
    );
    let tags: Vec<_> = c["allowed_cpython_3_14_release_tag_formats"]
        .as_array()
        .expect("allowed_cpython_3_14_release_tag_formats")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(tags, vec!["v3.14.X", "v3.14.X-rcY"]);
    assert_eq!(
        c["py314_tests_in_py313_subdir_failure_kind"].as_str(),
        Some("py314_tests_landed_in_py313_subdir")
    );
    assert_eq!(
        c["py314_tests_in_py313_subdir_exit_code"].as_integer(),
        Some(351)
    );
    assert_eq!(
        c["py313_baseline_overwritten_failure_kind"].as_str(),
        Some("py314_results_overwrote_py313_baseline")
    );
    assert_eq!(
        c["py313_baseline_overwritten_exit_code"].as_integer(),
        Some(352)
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
            "transitive_dependency_epic_issue",
            "py313_master_epic_status",
            "py312_master_epic_status",
            "conformance_criteria",
            "c1_lib_test_minimum_ratio",
            "explicit_deferrals",
            "py314_subdir_relative_path",
            "cpython_3_14_release_tag",
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
            "py314_is_the_conformance_target_for_this_epic",
            "py314_epic_is_doubly_hard_gated_on_py313_and_py312_epics",
            "eight_py314_conformance_criteria_c1_through_c8_are_pinned",
            "pep_779_pep_768_pep_793_pep_761_and_perf_are_explicit_deferrals",
            "conformance_suite_forks_into_py314_subdir_with_pinned_cpython_tag",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "pep_779_free_threaded_ga",
        "pep_768_remote_debugging",
        "pep_793_new_c_api_for_type_construction",
        "pep_761_sigstore_signing",
        "py314_performance_target",
        "uv_like_package_management",
        "implementation_of_pep_649_deferred_annotation_lazy_resolution",
        "implementation_of_pep_750_template_object",
        "implementation_of_pep_734_subinterpreter_runtime",
        "implementation_of_pep_784_zstd_compression_backend",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
