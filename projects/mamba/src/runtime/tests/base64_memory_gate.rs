//! Inline migration of tests/base64_memory_gate_fixture_2096.rs (#2096).
//! Schema gate for base64 perf-primary memory FAIL.

#![cfg(test)]

use crate::testing::{a, b, get, i, load_manifest, s};
use toml::Value;

const FIXTURE: &str = "tests/cpython/perf/base64_memory_gate/manifest.toml";
fn m() -> Value {
    load_manifest(FIXTURE)
}

#[test]
fn header_is_well_formed() {
    let m = m();
    assert_eq!(i(&m, "version"), 1);
    assert_eq!(s(&m, "fixture"), "base64_memory_gate");
    assert_eq!(i(&m, "issue"), 2096);
    assert_eq!(i(&m, "parent_issue"), 1265);
    assert_eq!(s(&m, "profile"), "conformance");
    assert_eq!(s(&m, "family"), "base64_memory_gate");
    assert_eq!(s(&m, "network"), "offline");
}

#[test]
fn isolation_pins_no_global_state() {
    let m = m();
    let iso = get(&m, "isolation");
    assert!(b(iso, "forbid_writes_outside_project"));
    assert!(b(iso, "forbid_user_home_reads"));
    assert!(b(iso, "forbid_global_cache_reads"));
    assert!(b(iso, "forbid_global_cache_writes"));
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let m = m();
    let py = get(&m, "python_target");
    assert_eq!(i(py, "python_major"), 3);
    assert_eq!(i(py, "python_minor"), 12);
    assert!(b(py, "must_be_python_3_12"));
}

#[test]
fn surface_pins_the_five_required_axes() {
    let m = m();
    let sf = get(&m, "surface");
    assert!(b(sf, "must_cover_base64_memory_meets_1x_cpython_floor"));
    assert!(b(sf, "must_cover_speed_gate_holds_across_memory_fix"));
    assert!(b(
        sf,
        "must_cover_starting_deficit_recorded_with_measurement_shape"
    ));
    assert!(b(
        sf,
        "must_cover_fix_path_bounded_to_bytes_layout_or_clone_elim"
    ));
    assert!(b(
        sf,
        "must_cover_sibling_bytes_libs_enumerated_as_unblock_targets"
    ));
    assert!(b(sf, "must_be_offline_or_loopback_only"));
    assert!(b(sf, "must_be_deterministic"));
}

#[test]
fn r1_base64_memory_meets_1x_cpython() {
    let m = m();
    let r = get(&m, "r1_base64_memory_meets_1x_cpython_contract");
    assert_eq!(s(r, "requirement_id"), "R1");
    assert!(b(r, "must_meet_or_exceed_cpython_memory_ratio_floor"));
    assert!(b(r, "must_pin_canonical_bench_name"));
    assert!(b(r, "must_pin_baseline_workload"));
    assert!(b(r, "forbid_silently_changing_workload_to_meet_floor"));
    assert!(b(r, "forbid_silently_disabling_bench_to_meet_floor"));
    assert_eq!(s(r, "expected_bench_name"), "base64/encode_decode");
    assert_eq!(s(r, "expected_memory_ratio_floor"), "1.0x");
    let names: Vec<&str> = a(r, "allowed_workload_values")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(names.contains(&"base64_encode_short_n_1000"));
    assert!(names.contains(&"base64_decode_short_n_1000"));
    assert!(names.contains(&"base64_encode_decode_roundtrip_n_1000"));
    assert_eq!(i(r, "memory_below_floor_exit_code"), 477);
    assert_eq!(i(r, "workload_silently_changed_exit_code"), 478);
    assert_eq!(i(r, "bench_silently_disabled_exit_code"), 479);
    assert!(b(
        r,
        "must_distinguish_memory_below_from_workload_change_from_silently_disabled"
    ));
}

#[test]
fn r2_speed_gate_holds_across_memory_fix() {
    let m = m();
    let r = get(&m, "r2_speed_gate_holds_across_memory_fix_contract");
    assert_eq!(s(r, "requirement_id"), "R2");
    assert!(b(r, "must_measure_post_fix_speed_ratio"));
    assert!(b(r, "must_compare_against_pre_fix_speed_baseline"));
    assert!(b(r, "forbid_silently_trading_speed_for_memory"));
    assert!(b(r, "forbid_silently_widening_speed_regression_threshold"));
    assert_eq!(s(r, "expected_pre_fix_speed_ratio"), "14.39x");
    assert_eq!(s(r, "expected_speed_regression_threshold"), "1.0x_pre_fix");
    assert_eq!(i(r, "speed_regressed_exit_code"), 480);
    assert_eq!(i(r, "speed_threshold_widened_exit_code"), 481);
    assert!(b(
        r,
        "must_distinguish_speed_regressed_from_threshold_widening"
    ));
}

#[test]
fn r3_starting_deficit_recorded_with_measurement_shape() {
    let m = m();
    let r = get(
        &m,
        "r3_starting_deficit_recorded_with_measurement_shape_contract",
    );
    assert_eq!(s(r, "requirement_id"), "R3");
    assert!(b(r, "must_record_mamba_mb"));
    assert!(b(r, "must_record_cpython_mb"));
    assert!(b(r, "must_record_memory_ratio"));
    assert!(b(r, "must_record_speed_ratio"));
    assert!(b(r, "must_record_sample_count"));
    assert!(b(r, "must_record_warmup_iters"));
    assert!(b(r, "forbid_recording_ratio_without_underlying_mb"));
    let names: Vec<&str> = a(r, "required_measurement_fields")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for f in &[
        "mamba_mb",
        "cpython_mb",
        "memory_ratio",
        "speed_ratio",
        "sample_count",
        "warmup_iters",
    ] {
        assert!(names.contains(f), "missing required measurement field: {f}");
    }
    assert_eq!(s(r, "expected_starting_mamba_mb"), "22.25");
    assert_eq!(s(r, "expected_starting_cpython_mb"), "11.61");
    assert_eq!(s(r, "expected_starting_memory_ratio"), "0.50x");
    assert_eq!(s(r, "expected_starting_speed_ratio"), "14.39x");
    assert_eq!(i(r, "ratio_without_mb_exit_code"), 482);
    assert_eq!(i(r, "measurement_field_missing_exit_code"), 483);
    assert!(b(r, "must_distinguish_ratio_without_mb_from_field_missing"));
}

#[test]
fn r4_fix_path_bounded() {
    let m = m();
    let r = get(
        &m,
        "r4_fix_path_bounded_to_bytes_layout_or_clone_elim_contract",
    );
    assert_eq!(s(r, "requirement_id"), "R4");
    assert!(b(r, "must_pin_allowed_fix_paths"));
    assert!(b(r, "must_forbid_in_place_bytes_mutation_path"));
    assert!(b(r, "must_forbid_changing_cpython_baseline_harness"));
    let allowed: Vec<&str> = a(r, "allowed_fix_path_kind_values")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for k in &[
        "subset_a_input_clone_elimination_via_shim_borrow",
        "subset_b_mbobject_bytes_header_shrink",
        "subset_b_bytes_capacity_field_removal",
        "subset_b_copy_on_write_for_short_lived_bytes",
    ] {
        assert!(allowed.contains(k), "missing allowed fix path: {k}");
    }
    let disallowed: Vec<&str> = a(r, "disallowed_fix_path_kind_values")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for k in &[
        "in_place_bytes_mutation_outside_cpython_contract",
        "cpython_baseline_harness_change",
        "skip_bench_to_pass_gate",
    ] {
        assert!(disallowed.contains(k), "missing disallowed fix path: {k}");
    }
    assert_eq!(i(r, "disallowed_fix_path_used_exit_code"), 484);
    assert_eq!(i(r, "cpython_harness_changed_exit_code"), 485);
    assert!(b(
        r,
        "must_distinguish_disallowed_fix_path_from_cpython_harness_change"
    ));
}

#[test]
fn r5_sibling_bytes_libs_enumerated() {
    let m = m();
    let r = get(
        &m,
        "r5_sibling_bytes_libs_enumerated_as_unblock_targets_contract",
    );
    assert_eq!(s(r, "requirement_id"), "R5");
    assert!(b(r, "must_pin_sibling_unblock_target_set"));
    assert!(b(
        r,
        "must_mark_sibling_retrofits_out_of_scope_for_this_issue"
    ));
    let names: Vec<&str> = a(r, "sibling_unblock_targets")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for lib in &["hashlib", "hmac", "zlib", "gzip", "lzma", "bz2", "codecs"] {
        assert!(names.contains(lib), "missing sibling lib: {lib}");
    }
    assert_eq!(i(r, "sibling_dropped_exit_code"), 486);
    assert_eq!(i(r, "sibling_implemented_in_this_issue_exit_code"), 487);
    assert!(b(
        r,
        "must_distinguish_sibling_dropped_from_sibling_pulled_in"
    ));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let m = m();
    let rc = get(&m, "runner_contract");
    let names: Vec<&str> = a(rc, "keys").iter().filter_map(|v| v.as_str()).collect();
    for k in &[
        "outcome",
        "case",
        "requirement_id",
        "bench_name",
        "memory_ratio_floor",
        "workload",
        "pre_fix_speed_ratio",
        "speed_regression_threshold",
        "required_measurement_fields",
        "starting_mamba_mb",
        "starting_cpython_mb",
        "starting_memory_ratio",
        "starting_speed_ratio",
        "fix_path_kind",
        "disallowed_fix_path_kind_values",
        "sibling_unblock_targets",
        "failure_kind",
        "exit_code",
    ] {
        assert!(names.contains(k), "missing runner key: {k}");
    }
    assert_eq!(a(rc, "case_values").len(), 5);
}

#[test]
fn pins_out_of_scope_per_issue() {
    let m = m();
    let oos = get(&m, "out_of_scope");
    assert!(b(oos, "implementation_of_bytes_layout_shrink"));
    assert!(b(oos, "sibling_lib_retrofits"));
    assert!(b(oos, "cpython_baseline_harness_change"));
    assert!(b(oos, "non_bytes_lib_memory_gate"));
    assert!(b(oos, "c_extension_fast_paths"));
}
