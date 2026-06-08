//! Inline migration of tests/pymalloc_equivalent_freelist_gate_fixture_1382.rs (#1382).

#![cfg(test)]

use crate::testing::{a, b, get, i, load_manifest, s};
use toml::Value;

const FIXTURE: &str = "tests/harness/cpython/config/perf/pymalloc_equivalent_freelist_gate/manifest.toml";
fn m() -> Value {
    load_manifest(FIXTURE)
}

#[test]
fn header_is_well_formed() {
    let m = m();
    assert_eq!(i(&m, "version"), 1);
    assert_eq!(s(&m, "fixture"), "pymalloc_equivalent_freelist_gate");
    assert_eq!(i(&m, "issue"), 1382);
    assert_eq!(i(&m, "parent_issue"), 1265);
    assert_eq!(s(&m, "profile"), "conformance");
    assert_eq!(s(&m, "family"), "pymalloc_equivalent_freelist_gate");
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
    assert!(b(sf, "must_cover_small_string_interning_added"));
    assert!(b(sf, "must_cover_small_object_freelist_added"));
    assert!(b(sf, "must_cover_string_concat_meets_1x_floor"));
    assert!(b(sf, "must_cover_correctness_invariants_arena_vs_global"));
    assert!(b(sf, "must_cover_fix_path_bounded"));
    assert!(b(sf, "must_be_offline_or_loopback_only"));
    assert!(b(sf, "must_be_deterministic"));
}

#[test]
fn r1_small_string_interning_added() {
    let m = m();
    let r = get(&m, "r1_small_string_interning_added_contract");
    assert_eq!(s(r, "requirement_id"), "R1");
    assert!(b(r, "must_add_intern_runtime_api"));
    assert!(b(r, "must_make_codegen_emit_intern_handle_for_literal_constants"));
    assert!(b(r, "must_keep_intern_threshold_at_64_bytes"));
    assert_eq!(s(r, "expected_intern_runtime_api"), "mb_str_intern");
    assert_eq!(i(r, "expected_intern_threshold_bytes"), 64);
    let names: Vec<&str> = a(r, "allowed_intern_handle_kind_values")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(names.contains(&"refcount_immortal_handle"));
    assert!(names.contains(&"refcounted_handle"));
    assert!(names.contains(&"per_call_alloc"));
    assert_eq!(s(r, "expected_intern_handle_kind"), "refcount_immortal_handle");
    assert_eq!(i(r, "per_call_alloc_for_intern_exit_code"), 511);
    assert_eq!(i(r, "intern_threshold_widened_exit_code"), 512);
    assert_eq!(i(r, "intern_api_missing_exit_code"), 513);
    assert!(b(
        r,
        "must_distinguish_per_call_alloc_from_widened_from_api_missing"
    ));
}

#[test]
fn r2_small_object_freelist_added() {
    let m = m();
    let r = get(&m, "r2_small_object_freelist_added_contract");
    assert_eq!(s(r, "requirement_id"), "R2");
    assert!(b(r, "must_add_per_thread_arena"));
    assert!(b(r, "must_cover_tiny_list_short_string_small_tuple_size_classes"));
    assert!(b(r, "must_recycle_frees_into_freelist"));
    assert!(b(r, "must_keep_small_object_threshold_at_512_bytes"));
    assert_eq!(i(r, "expected_small_object_threshold_bytes"), 512);
    let names: Vec<&str> = a(r, "covered_size_classes")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for c in &[
        "tiny_list_up_to_16_elements",
        "short_string_up_to_64_bytes",
        "small_tuple_up_to_8_elements",
    ] {
        assert!(names.contains(c), "missing size class: {c}");
    }
    assert_eq!(i(r, "free_returned_to_global_exit_code"), 514);
    assert_eq!(i(r, "size_class_dropped_exit_code"), 515);
    assert_eq!(i(r, "small_object_threshold_widened_exit_code"), 516);
    assert!(b(
        r,
        "must_distinguish_global_free_from_size_class_dropped_from_threshold_widened"
    ));
}

#[test]
fn r3_string_concat_meets_1x_floor() {
    let m = m();
    let r = get(&m, "r3_string_concat_meets_1x_floor_contract");
    assert_eq!(s(r, "requirement_id"), "R3");
    assert!(b(r, "must_meet_or_exceed_string_concat_cpython_floor"));
    assert!(b(r, "must_move_floor_compliance_count"));
    assert_eq!(s(r, "expected_bench_name"), "string_concat");
    assert_eq!(
        s(r, "expected_baseline_snippet"),
        "join_hello_space_world_bang_n_iter"
    );
    assert_eq!(s(r, "expected_cpython_ratio_floor"), "1.0x");
    assert_eq!(s(r, "expected_structural_target"), "1.5x");
    assert_eq!(s(r, "expected_pre_fix_compliance"), "5_of_8");
    assert_eq!(s(r, "expected_post_fix_compliance_floor"), "6_of_8");
    assert_eq!(i(r, "floor_not_met_exit_code"), 517);
    assert_eq!(i(r, "compliance_did_not_move_exit_code"), 518);
    assert!(b(r, "must_distinguish_floor_not_met_from_compliance_did_not_move"));
}

#[test]
fn r4_correctness_invariants() {
    let m = m();
    let r = get(&m, "r4_correctness_invariants_arena_vs_global_contract");
    assert_eq!(s(r, "requirement_id"), "R4");
    assert!(b(r, "must_route_arena_release_to_freelist_not_global_free"));
    assert!(b(r, "must_keep_large_object_path_on_global_allocator"));
    assert!(b(r, "must_keep_interned_string_refcount_immortal"));
    assert!(b(r, "forbid_arena_release_calling_global_free"));
    assert!(b(r, "forbid_regressing_large_object_path"));
    assert!(b(r, "forbid_premature_free_of_interned_string"));
    let names: Vec<&str> = a(r, "allowed_allocator_counter_values")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(names.contains(&"global_free_calls_for_arena_object"));
    assert!(names.contains(&"global_free_calls_for_large_object"));
    assert!(names.contains(&"freelist_recycle_for_arena_object"));
    assert_eq!(
        s(r, "expected_arena_release_counter"),
        "freelist_recycle_for_arena_object"
    );
    assert_eq!(i(r, "arena_global_free_called_exit_code"), 519);
    assert_eq!(i(r, "large_object_regressed_exit_code"), 520);
    assert_eq!(i(r, "interned_string_premature_free_exit_code"), 521);
    assert!(b(
        r,
        "must_distinguish_arena_global_free_from_large_regress_from_premature_free"
    ));
}

#[test]
fn r5_fix_path_bounded() {
    let m = m();
    let r = get(&m, "r5_fix_path_bounded_contract");
    assert_eq!(s(r, "requirement_id"), "R5");
    assert!(b(r, "must_pin_allowed_fix_paths"));
    assert!(b(r, "must_forbid_wholesale_global_allocator_replacement"));
    assert!(b(r, "must_forbid_cpython_baseline_change"));
    let allowed: Vec<&str> = a(r, "allowed_fix_path_kind_values")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for k in &[
        "small_string_interning_path",
        "per_thread_small_object_arena",
        "mimalloc_size_class_wrap",
        "vendored_pymalloc_obmalloc_thin_layer",
    ] {
        assert!(allowed.contains(k), "missing allowed: {k}");
    }
    let disallowed: Vec<&str> = a(r, "disallowed_fix_path_kind_values")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for k in &[
        "wholesale_global_allocator_replacement",
        "cpython_baseline_change",
        "deferred_sys_intern_runtime_in_this_issue",
    ] {
        assert!(disallowed.contains(k), "missing disallowed: {k}");
    }
    assert_eq!(i(r, "disallowed_fix_path_used_exit_code"), 522);
    assert_eq!(i(r, "cpython_baseline_changed_exit_code"), 523);
    assert_eq!(i(r, "deferred_pulled_in_exit_code"), 524);
    assert!(b(
        r,
        "must_distinguish_disallowed_from_baseline_change_from_deferred_pulled_in"
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
        "intern_runtime_api",
        "intern_threshold_bytes",
        "intern_handle_kind",
        "small_object_threshold_bytes",
        "covered_size_classes",
        "bench_name",
        "baseline_snippet",
        "cpython_ratio_floor",
        "structural_target",
        "pre_fix_floor_compliance",
        "post_fix_floor_compliance_floor",
        "allocator_counter",
        "fix_path_kind",
        "disallowed_fix_path_kind_values",
        "failure_kind",
        "exit_code",
    ] {
        assert!(names.contains(k), "missing key: {k}");
    }
    assert_eq!(a(rc, "case_values").len(), 5);
}

#[test]
fn pins_out_of_scope_per_issue() {
    let m = m();
    let oos = get(&m, "out_of_scope");
    assert!(b(oos, "large_object_allocator_change"));
    assert!(b(oos, "runtime_sys_intern_for_concat_results"));
    assert!(b(oos, "rewriting_mamba_refcount"));
    assert!(b(oos, "cpython_baseline_change"));
    assert!(b(oos, "c_extension_fast_paths"));
}
