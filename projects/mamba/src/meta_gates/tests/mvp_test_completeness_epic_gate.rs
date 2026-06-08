#![cfg(test)]

// Locks the shape of the MVP test-completeness epic gate fixture
// pinned by tests/governance/gates/mvp/mvp_test_completeness_epic_gate/
// manifest.toml. Closes #2526.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/mvp_test_completeness_epic_gate/manifest.toml")
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
        Some("mvp_test_completeness_epic_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(
        m["family"].as_str(),
        Some("mvp_test_completeness_epic_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
    let children: Vec<_> = m["child_issues"]
        .as_array()
        .expect("child_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(children, vec![2527, 2528, 2529, 2530, 2531, 2532, 2533]);
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
fn surface_pins_seven_gates_leaves_offline_no_hiding_and_wave_range() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_seven_parent_gates_are_wired",
        "must_cover_worker_takes_one_agent_sized_leaf_at_a_time",
        "must_cover_default_offline_and_deterministic",
        "must_cover_no_required_test_hides_behind_skip_xfail_stub",
        "must_cover_first_atomic_wave_leaf_range_is_enumerated",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_seven_parent_gates_wired() {
    let c = &manifest()["r1_seven_parent_gates_wired_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("seven_parent_gates_smoke_libtest_ecosystem_perf_mambalibs_pkgmgr_debt_are_wired")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_wire_smoke_gate_2527",
        "must_wire_cpython_lib_test_gate_2528",
        "must_wire_ecosystem_real_world_gate_2529",
        "must_wire_perf_10x_gate_2530",
        "must_wire_mambalibs_gate_2531",
        "must_wire_package_manager_gate_2532",
        "must_wire_skip_xfail_debt_gate_2533",
        "forbid_collapsing_seven_parent_gates_into_fewer",
        "forbid_silently_dropping_a_parent_gate",
        "must_distinguish_parent_gate_missing_from_collapsed",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let issues: Vec<_> = c["required_parent_gate_issues"]
        .as_array()
        .expect("required_parent_gate_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(issues, vec![2527, 2528, 2529, 2530, 2531, 2532, 2533]);
    assert_eq!(
        c["parent_gate_issues_field_name"].as_str(),
        Some("parent_gate_issues")
    );
    assert_eq!(
        c["parent_gate_missing_failure_kind"].as_str(),
        Some("mvp_test_completeness_parent_gate_missing")
    );
    assert_eq!(
        c["parent_gate_missing_exit_code"].as_integer(),
        Some(353)
    );
    assert_eq!(
        c["parent_gates_collapsed_failure_kind"].as_str(),
        Some("mvp_test_completeness_parent_gates_collapsed_to_fewer")
    );
    assert_eq!(
        c["parent_gates_collapsed_exit_code"].as_integer(),
        Some(354)
    );
}

#[test]
fn r2_one_agent_sized_leaf_at_a_time() {
    let c = &manifest()["r2_one_agent_sized_leaf_at_a_time_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("worker_takes_one_agent_sized_leaf_5_to_15_min_at_a_time")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_require_worker_take_one_leaf_at_a_time",
        "must_require_leaf_be_agent_sized_5_to_15_min",
        "forbid_combining_multiple_leaves_into_one_pr",
        "forbid_taking_oversized_leaf_without_split",
        "must_distinguish_multiple_leaves_from_oversized_without_split",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_leaf_size_minimum_minutes"].as_integer(),
        Some(5)
    );
    assert_eq!(
        c["expected_leaf_size_maximum_minutes"].as_integer(),
        Some(15)
    );
    assert_eq!(c["expected_leaves_per_pr"].as_integer(), Some(1));
    assert_eq!(
        c["multiple_leaves_per_pr_failure_kind"].as_str(),
        Some("mvp_test_completeness_multiple_leaves_in_single_pr")
    );
    assert_eq!(
        c["multiple_leaves_per_pr_exit_code"].as_integer(),
        Some(355)
    );
    assert_eq!(
        c["oversized_leaf_without_split_failure_kind"].as_str(),
        Some("mvp_test_completeness_oversized_leaf_without_split")
    );
    assert_eq!(
        c["oversized_leaf_without_split_exit_code"].as_integer(),
        Some(356)
    );
}

#[test]
fn r3_default_offline_and_deterministic() {
    let c = &manifest()["r3_default_offline_and_deterministic_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("default_gates_are_offline_and_deterministic_unless_leaf_opts_in_to_live")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_default_gates_to_offline",
        "must_default_gates_to_deterministic",
        "must_require_explicit_opt_in_for_live_integration",
        "forbid_silently_enabling_live_network_in_default_gate",
        "forbid_silently_introducing_nondeterminism",
        "must_distinguish_live_network_from_silent_nondeterminism",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["expected_default_network"].as_str(), Some("offline"));
    assert_eq!(
        c["expected_default_determinism"].as_str(),
        Some("deterministic")
    );
    assert_eq!(
        c["silent_live_network_failure_kind"].as_str(),
        Some("mvp_test_completeness_live_network_in_default_gate")
    );
    assert_eq!(c["silent_live_network_exit_code"].as_integer(), Some(357));
    assert_eq!(
        c["silent_nondeterminism_failure_kind"].as_str(),
        Some("mvp_test_completeness_silent_nondeterminism")
    );
    assert_eq!(
        c["silent_nondeterminism_exit_code"].as_integer(),
        Some(358)
    );
}

#[test]
fn r4_no_required_test_hides_behind_skip_xfail_stub() {
    let c = &manifest()["r4_no_required_test_hides_behind_skip_xfail_stub_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("no_required_test_hides_behind_skip_xfail_or_stub_outcomes")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_forbid_required_test_marked_skip",
        "must_forbid_required_test_marked_xfail",
        "must_forbid_required_test_emitting_stub_outcome",
        "must_track_skip_xfail_stub_debt_under_2533",
        "forbid_silently_marking_required_test_as_skip",
        "forbid_silently_marking_required_test_as_xfail",
        "forbid_silently_emitting_stub_outcome_for_required_test",
        "must_distinguish_skip_from_xfail_from_stub_hiding",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let outcomes: Vec<_> = c["disallowed_hiding_outcomes"]
        .as_array()
        .expect("disallowed_hiding_outcomes")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(outcomes, vec!["skip", "xfail", "stub"]);
    assert_eq!(
        c["disallowed_hiding_outcomes_field_name"].as_str(),
        Some("disallowed_hiding_outcomes_for_required_test")
    );
    assert_eq!(c["expected_debt_gate_issue"].as_integer(), Some(2533));
    assert_eq!(
        c["required_test_hidden_by_skip_failure_kind"].as_str(),
        Some("mvp_test_completeness_required_test_hidden_by_skip")
    );
    assert_eq!(
        c["required_test_hidden_by_skip_exit_code"].as_integer(),
        Some(359)
    );
    assert_eq!(
        c["required_test_hidden_by_xfail_failure_kind"].as_str(),
        Some("mvp_test_completeness_required_test_hidden_by_xfail")
    );
    assert_eq!(
        c["required_test_hidden_by_xfail_exit_code"].as_integer(),
        Some(360)
    );
    assert_eq!(
        c["required_test_emitted_stub_failure_kind"].as_str(),
        Some("mvp_test_completeness_required_test_emitted_stub_outcome")
    );
    assert_eq!(
        c["required_test_emitted_stub_exit_code"].as_integer(),
        Some(361)
    );
}

#[test]
fn r5_first_atomic_wave_enumerated() {
    let c = &manifest()["r5_first_atomic_wave_enumerated_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("first_atomic_wave_leaf_range_2534_to_2603_is_enumerated_with_64_leaves")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_pin_first_wave_leaf_range_start",
        "must_pin_first_wave_leaf_range_end",
        "must_pin_first_wave_leaf_count",
        "forbid_leaving_first_wave_range_as_todo",
        "forbid_silently_resizing_first_wave_count",
        "must_distinguish_range_is_todo_from_count_resized",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_first_wave_leaf_range_start"].as_integer(),
        Some(2534)
    );
    assert_eq!(
        c["expected_first_wave_leaf_range_end"].as_integer(),
        Some(2603)
    );
    assert_eq!(c["expected_first_wave_leaf_count"].as_integer(), Some(64));
    assert_eq!(
        c["first_wave_range_is_todo_failure_kind"].as_str(),
        Some("mvp_test_completeness_first_wave_range_is_todo")
    );
    assert_eq!(
        c["first_wave_range_is_todo_exit_code"].as_integer(),
        Some(362)
    );
    assert_eq!(
        c["first_wave_count_silently_resized_failure_kind"].as_str(),
        Some("mvp_test_completeness_first_wave_count_silently_resized")
    );
    assert_eq!(
        c["first_wave_count_silently_resized_exit_code"].as_integer(),
        Some(363)
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
            "parent_gate_issues",
            "leaf_size_minimum_minutes",
            "leaf_size_maximum_minutes",
            "leaves_per_pr",
            "default_network",
            "default_determinism",
            "live_integration_opt_in",
            "disallowed_hiding_outcomes_for_required_test",
            "debt_gate_issue",
            "first_wave_leaf_range_start",
            "first_wave_leaf_range_end",
            "first_wave_leaf_count",
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
            "seven_parent_gates_smoke_libtest_ecosystem_perf_mambalibs_pkgmgr_debt_are_wired",
            "worker_takes_one_agent_sized_leaf_5_to_15_min_at_a_time",
            "default_gates_are_offline_and_deterministic_unless_leaf_opts_in_to_live",
            "no_required_test_hides_behind_skip_xfail_or_stub_outcomes",
            "first_atomic_wave_leaf_range_2534_to_2603_is_enumerated_with_64_leaves",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "implementation_of_seven_child_gates",
        "implementation_of_first_wave_64_leaves",
        "release_infra_beyond_test_gates",
        "c_extension_fast_paths",
        "runtime_implementation_of_smoke_gate",
        "runtime_implementation_of_cpython_lib_test_gate",
        "runtime_implementation_of_ecosystem_real_world_gate",
        "runtime_implementation_of_perf_10x_gate",
        "runtime_implementation_of_mambalibs_gate",
        "runtime_implementation_of_package_manager_gate",
        "runtime_implementation_of_skip_xfail_debt_gate",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
