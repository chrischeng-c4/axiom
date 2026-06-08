//! Inline migration of tests/list_sort_builtin_perf_gate_fixture_2513.rs (#2513).
//!
//! Locks the shape of the list_sort_builtin perf gate fixture pinned by
//! tests/harness/cpython/config/perf/list_sort_builtin_perf_gate/manifest.toml.

#![cfg(test)]

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/harness/cpython/config/perf/list_sort_builtin_perf_gate/manifest.toml")
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
        Some("list_sort_builtin_perf_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(2513));
    assert_eq!(m["parent_issue"].as_integer(), Some(2458));
    assert_eq!(m["profile"].as_str(), Some("conformance"));
    assert_eq!(
        m["family"].as_str(),
        Some("list_sort_builtin_perf_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
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
fn surface_pins_floor_regression_deficit_correctness_and_bounded_fix() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_list_sort_builtin_meets_1x_cpython_floor",
        "must_cover_no_other_accepted_bench_regresses",
        "must_cover_starting_deficit_is_recorded_with_measurement_shape",
        "must_cover_timsort_stability_in_place_and_kwargs_preserved",
        "must_cover_fix_path_bounded_to_list_dispatch_or_sort_specialization",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_list_sort_builtin_meets_1x_cpython() {
    let c = &manifest()["r1_list_sort_builtin_meets_1x_cpython_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("list_sort_builtin_reaches_at_least_1_0x_cpython_on_baseline_workload")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_meet_or_exceed_cpython_ratio_floor",
        "must_pin_canonical_bench_name",
        "must_pin_baseline_workload",
        "forbid_silently_changing_workload_to_meet_floor",
        "forbid_silently_disabling_bench_to_meet_floor",
        "must_distinguish_ratio_below_from_workload_change_from_silently_disabled",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["expected_bench_name"].as_str(), Some("list_sort_builtin"));
    assert_eq!(c["expected_cpython_ratio_floor"].as_str(), Some("1.0x"));
    let workloads: Vec<_> = c["allowed_workload_values"]
        .as_array()
        .expect("allowed_workload_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        workloads,
        vec![
            "random_int_list_n_1000",
            "sorted_int_list_n_1000",
            "reverse_int_list_n_1000",
        ]
    );
    assert_eq!(
        c["ratio_below_floor_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_ratio_below_1x_cpython")
    );
    assert_eq!(c["ratio_below_floor_exit_code"].as_integer(), Some(427));
    assert_eq!(
        c["workload_silently_changed_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_workload_silently_changed")
    );
    assert_eq!(
        c["workload_silently_changed_exit_code"].as_integer(),
        Some(428)
    );
    assert_eq!(
        c["bench_silently_disabled_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_bench_silently_disabled")
    );
    assert_eq!(
        c["bench_silently_disabled_exit_code"].as_integer(),
        Some(429)
    );
}

#[test]
fn r2_no_other_accepted_bench_regresses() {
    let c = &manifest()["r2_no_other_accepted_bench_regresses_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("improving_list_sort_builtin_does_not_regress_any_other_accepted_bench_below_its_floor")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_measure_each_accepted_bench_post_change",
        "must_compare_against_per_bench_pre_change_floor",
        "must_apply_per_bench_regression_threshold",
        "forbid_geomean_only_regression_check",
        "forbid_silently_widening_regression_threshold",
        "must_distinguish_per_bench_regression_from_geomean_from_threshold_widening",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let modes: Vec<_> = c["allowed_regression_check_mode_values"]
        .as_array()
        .expect("allowed_regression_check_mode_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(modes, vec!["per_bench", "geomean_only"]);
    assert_eq!(
        c["expected_regression_check_mode"].as_str(),
        Some("per_bench")
    );
    assert_eq!(
        c["expected_regression_threshold"].as_str(),
        Some("1.0x_pre_change")
    );
    assert_eq!(
        c["per_bench_regression_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_caused_per_bench_regression")
    );
    assert_eq!(
        c["per_bench_regression_exit_code"].as_integer(),
        Some(430)
    );
    assert_eq!(
        c["geomean_only_check_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_used_geomean_only_check")
    );
    assert_eq!(
        c["geomean_only_check_exit_code"].as_integer(),
        Some(431)
    );
    assert_eq!(
        c["regression_threshold_widened_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_regression_threshold_silently_widened")
    );
    assert_eq!(
        c["regression_threshold_widened_exit_code"].as_integer(),
        Some(432)
    );
}

#[test]
fn r3_starting_deficit_recorded() {
    let c = &manifest()["r3_starting_deficit_recorded_with_measurement_shape_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("starting_deficit_1000x_recorded_with_mamba_ns_cpython_ns_ratio_samples_warmup")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_record_mamba_ns",
        "must_record_cpython_ns",
        "must_record_ratio",
        "must_record_sample_count",
        "must_record_warmup_iters",
        "forbid_recording_ratio_without_underlying_ns",
        "must_distinguish_ratio_without_ns_from_field_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_measurement_fields"]
        .as_array()
        .expect("required_measurement_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "mamba_ns",
            "cpython_ns",
            "ratio",
            "sample_count",
            "warmup_iters",
        ]
    );
    assert_eq!(
        c["expected_starting_mamba_ns"].as_integer(),
        Some(400050)
    );
    assert_eq!(c["expected_starting_cpython_ns"].as_integer(), Some(411));
    assert_eq!(c["expected_starting_ratio"].as_str(), Some("0.001x"));
    assert_eq!(
        c["ratio_without_ns_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_ratio_without_underlying_ns")
    );
    assert_eq!(c["ratio_without_ns_exit_code"].as_integer(), Some(433));
    assert_eq!(
        c["measurement_field_missing_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_measurement_field_missing")
    );
    assert_eq!(
        c["measurement_field_missing_exit_code"].as_integer(),
        Some(434)
    );
}

#[test]
fn r4_timsort_stability_in_place_kwargs_preserved() {
    let c = &manifest()["r4_timsort_stability_in_place_and_kwargs_preserved_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("list_sort_under_mamba_preserves_timsort_stability_in_place_and_key_reverse_kwargs")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_preserve_timsort_stability",
        "must_preserve_in_place_contract",
        "must_preserve_key_kwarg",
        "must_preserve_reverse_kwarg",
        "must_reject_cmp_kwarg_per_python_3",
        "forbid_replacing_sort_with_non_stable_algorithm",
        "forbid_sorting_outside_in_place_contract",
        "must_distinguish_non_stable_from_not_in_place_from_kwarg_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let algos: Vec<_> = c["allowed_sort_algorithm_kind_values"]
        .as_array()
        .expect("allowed_sort_algorithm_kind_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(algos, vec!["timsort"]);
    let contracts: Vec<_> = c["allowed_in_place_contract_values"]
        .as_array()
        .expect("allowed_in_place_contract_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(contracts, vec!["in_place", "returns_new_list"]);
    assert_eq!(c["expected_in_place_contract"].as_str(), Some("in_place"));
    let kwargs: Vec<_> = c["required_kwargs"]
        .as_array()
        .expect("required_kwargs")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(kwargs, vec!["key", "reverse"]);
    assert_eq!(
        c["non_stable_algorithm_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_non_stable_algorithm_substituted")
    );
    assert_eq!(
        c["non_stable_algorithm_exit_code"].as_integer(),
        Some(435)
    );
    assert_eq!(
        c["not_in_place_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_not_in_place")
    );
    assert_eq!(c["not_in_place_exit_code"].as_integer(), Some(436));
    assert_eq!(
        c["required_kwarg_missing_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_required_kwarg_missing")
    );
    assert_eq!(
        c["required_kwarg_missing_exit_code"].as_integer(),
        Some(437)
    );
}

#[test]
fn r5_fix_path_bounded() {
    let c = &manifest()["r5_fix_path_bounded_to_dispatch_or_specialization_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("fix_path_bounded_to_list_dispatch_or_sort_specialization_or_comparator_inlining")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_pin_allowed_fix_paths",
        "must_forbid_non_stable_substitution",
        "must_forbid_out_of_place_sort_path",
        "must_forbid_changing_cpython_baseline_harness",
        "must_distinguish_disallowed_fix_path_from_cpython_harness_change",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let allowed: Vec<_> = c["allowed_fix_path_kind_values"]
        .as_array()
        .expect("allowed_fix_path_kind_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        allowed,
        vec![
            "list_dispatch_specialization",
            "sort_specialization_for_typed_keys",
            "comparator_inlining",
        ]
    );
    let disallowed: Vec<_> = c["disallowed_fix_path_kind_values"]
        .as_array()
        .expect("disallowed_fix_path_kind_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        disallowed,
        vec![
            "non_stable_algorithm_substitution",
            "out_of_place_sort",
            "cpython_baseline_harness_change",
        ]
    );
    assert_eq!(
        c["disallowed_fix_path_used_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_disallowed_fix_path_used")
    );
    assert_eq!(
        c["disallowed_fix_path_used_exit_code"].as_integer(),
        Some(438)
    );
    assert_eq!(
        c["cpython_harness_changed_failure_kind"].as_str(),
        Some("perf_list_sort_builtin_cpython_harness_changed")
    );
    assert_eq!(
        c["cpython_harness_changed_exit_code"].as_integer(),
        Some(439)
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
            "bench_name",
            "cpython_ratio_floor",
            "workload",
            "regression_check_mode",
            "regression_threshold",
            "required_measurement_fields",
            "starting_mamba_ns",
            "starting_cpython_ns",
            "starting_ratio",
            "sort_algorithm_kind",
            "in_place_contract",
            "required_kwargs",
            "fix_path_kind",
            "disallowed_fix_path_kind_values",
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
            "list_sort_builtin_reaches_at_least_1_0x_cpython_on_baseline_workload",
            "improving_list_sort_builtin_does_not_regress_any_other_accepted_bench_below_its_floor",
            "starting_deficit_1000x_recorded_with_mamba_ns_cpython_ns_ratio_samples_warmup",
            "list_sort_under_mamba_preserves_timsort_stability_in_place_and_key_reverse_kwargs",
            "fix_path_bounded_to_list_dispatch_or_sort_specialization_or_comparator_inlining",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "implementation_of_specific_optimization",
        "sort_perf_for_non_builtin_types_numpy_pandas",
        "cpython_baseline_harness_change",
        "non_list_sort_benches",
        "c_extension_fast_paths",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
