//! Inline migration of tests/string_concat_perf_gate_fixture_2512.rs (#2512).
//!
//! Locks the shape of the string_concat perf gate fixture pinned by
//! tests/cpython/perf/string_concat_perf_gate/manifest.toml.

#![cfg(test)]

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cpython/perf/string_concat_perf_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("string_concat_perf_gate"));
    assert_eq!(m["issue"].as_integer(), Some(2512));
    assert_eq!(m["parent_issue"].as_integer(), Some(2458));
    assert_eq!(m["profile"].as_str(), Some("conformance"));
    assert_eq!(m["family"].as_str(), Some("string_concat_perf_gate"));
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
        "must_cover_string_concat_meets_1x_cpython_floor",
        "must_cover_no_other_accepted_bench_regresses",
        "must_cover_starting_deficit_is_recorded_with_measurement_shape",
        "must_cover_str_semantics_and_unicode_correctness_preserved",
        "must_cover_fix_path_bounded_to_known_specialization_set",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_string_concat_meets_1x_cpython() {
    let c = &manifest()["r1_string_concat_meets_1x_cpython_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("string_concat_reaches_at_least_1_0x_cpython_on_baseline_workload")
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
    assert_eq!(c["expected_bench_name"].as_str(), Some("string_concat"));
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
            "ascii_short_concat_n_1000",
            "unicode_short_concat_n_1000",
            "mixed_concat_plus_and_join_n_1000",
        ]
    );
    assert_eq!(
        c["ratio_below_floor_failure_kind"].as_str(),
        Some("perf_string_concat_ratio_below_1x_cpython")
    );
    assert_eq!(c["ratio_below_floor_exit_code"].as_integer(), Some(440));
    assert_eq!(
        c["workload_silently_changed_failure_kind"].as_str(),
        Some("perf_string_concat_workload_silently_changed")
    );
    assert_eq!(
        c["workload_silently_changed_exit_code"].as_integer(),
        Some(441)
    );
    assert_eq!(
        c["bench_silently_disabled_failure_kind"].as_str(),
        Some("perf_string_concat_bench_silently_disabled")
    );
    assert_eq!(
        c["bench_silently_disabled_exit_code"].as_integer(),
        Some(442)
    );
}

#[test]
fn r2_no_other_accepted_bench_regresses() {
    let c = &manifest()["r2_no_other_accepted_bench_regresses_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("improving_string_concat_does_not_regress_any_other_accepted_bench_below_its_floor")
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
        Some("perf_string_concat_caused_per_bench_regression")
    );
    assert_eq!(c["per_bench_regression_exit_code"].as_integer(), Some(443));
    assert_eq!(
        c["geomean_only_check_failure_kind"].as_str(),
        Some("perf_string_concat_used_geomean_only_check")
    );
    assert_eq!(c["geomean_only_check_exit_code"].as_integer(), Some(444));
    assert_eq!(
        c["regression_threshold_widened_failure_kind"].as_str(),
        Some("perf_string_concat_regression_threshold_silently_widened")
    );
    assert_eq!(
        c["regression_threshold_widened_exit_code"].as_integer(),
        Some(445)
    );
}

#[test]
fn r3_starting_deficit_recorded() {
    let c = &manifest()["r3_starting_deficit_recorded_with_measurement_shape_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("starting_deficit_3333x_recorded_with_mamba_ns_cpython_ns_ratio_samples_warmup")
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
    assert_eq!(c["expected_starting_mamba_ns"].as_integer(), Some(420776));
    assert_eq!(c["expected_starting_cpython_ns"].as_integer(), Some(143));
    assert_eq!(c["expected_starting_ratio"].as_str(), Some("0.0003x"));
    assert_eq!(
        c["ratio_without_ns_failure_kind"].as_str(),
        Some("perf_string_concat_ratio_without_underlying_ns")
    );
    assert_eq!(c["ratio_without_ns_exit_code"].as_integer(), Some(446));
    assert_eq!(
        c["measurement_field_missing_failure_kind"].as_str(),
        Some("perf_string_concat_measurement_field_missing")
    );
    assert_eq!(
        c["measurement_field_missing_exit_code"].as_integer(),
        Some(447)
    );
}

#[test]
fn r4_str_semantics_unicode_correctness_preserved() {
    let c = &manifest()["r4_str_semantics_and_unicode_correctness_preserved_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("str_concat_under_mamba_preserves_plus_iadd_join_fstring_and_unicode_semantics")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_preserve_plus_operator_semantics",
        "must_preserve_iadd_operator_semantics",
        "must_preserve_join_method_semantics",
        "must_preserve_fstring_semantics",
        "must_preserve_unicode_correctness_for_multibyte_input",
        "must_preserve_str_immutability",
        "forbid_in_place_mutation_of_str_object",
        "forbid_latin_1_shortcircuit_on_multibyte_input",
        "must_distinguish_immutability_from_unicode_from_op_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let ops: Vec<_> = c["required_str_ops"]
        .as_array()
        .expect("required_str_ops")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(ops, vec!["plus", "iadd", "join", "fstring"]);
    assert_eq!(
        c["immutability_violation_failure_kind"].as_str(),
        Some("perf_string_concat_str_immutability_violation")
    );
    assert_eq!(
        c["immutability_violation_exit_code"].as_integer(),
        Some(448)
    );
    assert_eq!(
        c["unicode_correctness_violation_failure_kind"].as_str(),
        Some("perf_string_concat_unicode_correctness_violation")
    );
    assert_eq!(
        c["unicode_correctness_violation_exit_code"].as_integer(),
        Some(449)
    );
    assert_eq!(
        c["required_str_op_missing_failure_kind"].as_str(),
        Some("perf_string_concat_required_str_op_missing")
    );
    assert_eq!(
        c["required_str_op_missing_exit_code"].as_integer(),
        Some(450)
    );
}

#[test]
fn r5_fix_path_bounded() {
    let c = &manifest()["r5_fix_path_bounded_to_known_specialization_set_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("fix_path_bounded_to_bytes_buffer_or_interning_or_ropes_or_jit_specialization")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_pin_allowed_fix_paths",
        "must_forbid_in_place_str_mutation_path",
        "must_forbid_latin_1_shortcircuit_path",
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
            "bytes_concat_buffer",
            "small_string_interning",
            "ropes_or_chunked_repr",
            "jit_string_dispatch_specialization",
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
            "in_place_str_mutation",
            "latin_1_shortcircuit_for_multibyte_input",
            "cpython_baseline_harness_change",
        ]
    );
    assert_eq!(
        c["disallowed_fix_path_used_failure_kind"].as_str(),
        Some("perf_string_concat_disallowed_fix_path_used")
    );
    assert_eq!(
        c["disallowed_fix_path_used_exit_code"].as_integer(),
        Some(451)
    );
    assert_eq!(
        c["cpython_harness_changed_failure_kind"].as_str(),
        Some("perf_string_concat_cpython_harness_changed")
    );
    assert_eq!(
        c["cpython_harness_changed_exit_code"].as_integer(),
        Some(452)
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
            "required_str_ops",
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
            "string_concat_reaches_at_least_1_0x_cpython_on_baseline_workload",
            "improving_string_concat_does_not_regress_any_other_accepted_bench_below_its_floor",
            "starting_deficit_3333x_recorded_with_mamba_ns_cpython_ns_ratio_samples_warmup",
            "str_concat_under_mamba_preserves_plus_iadd_join_fstring_and_unicode_semantics",
            "fix_path_bounded_to_bytes_buffer_or_interning_or_ropes_or_jit_specialization",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "implementation_of_specific_optimization",
        "bytes_or_bytearray_concat_perf",
        "cpython_baseline_harness_change",
        "non_string_concat_benches",
        "c_extension_fast_paths",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
