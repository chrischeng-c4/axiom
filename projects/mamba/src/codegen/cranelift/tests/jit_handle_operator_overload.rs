//! Inline migration of tests/jit_handle_operator_overload_gate_fixture_2129.rs (#2129).
//! Schema gate for JIT handle+handle native-i64 fold operator-overload gap.

#![cfg(test)]

use crate::testing::{a, b, get, i, load_manifest, s};
use toml::Value;

const FIXTURE: &str =
    "tests/governance/gates/runtime/jit_handle_operator_overload_gate/manifest.toml";
fn m() -> Value {
    load_manifest(FIXTURE)
}

#[test]
fn header_is_well_formed() {
    let m = m();
    assert_eq!(i(&m, "version"), 1);
    assert_eq!(s(&m, "fixture"), "jit_handle_operator_overload_gate");
    assert_eq!(i(&m, "issue"), 2129);
    assert_eq!(s(&m, "profile"), "conformance");
    assert_eq!(s(&m, "family"), "jit_handle_operator_overload_gate");
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
    assert!(b(sf, "must_cover_jit_must_not_fold_handle_arith_to_native"));
    assert!(b(
        sf,
        "must_cover_module_level_dispatcher_workaround_recorded"
    ));
    assert!(b(sf, "must_cover_hasattr_conditional_binding_refused"));
    assert!(b(sf, "must_cover_phase_3_fix_paths_enumerated"));
    assert!(b(
        sf,
        "must_cover_method_call_only_libs_explicitly_unaffected"
    ));
    assert!(b(sf, "must_be_offline_or_loopback_only"));
    assert!(b(sf, "must_be_deterministic"));
}

#[test]
fn r1_jit_must_not_fold_handle_arith_to_native() {
    let m = m();
    let r = get(&m, "r1_jit_must_not_fold_handle_arith_to_native_contract");
    assert_eq!(
        s(r, "case"),
        "jit_routes_arithmetic_operators_for_typed_handle_operands_through_dunder_dispatch_not_native_i64"
    );
    assert_eq!(s(r, "requirement_id"), "R1");
    assert!(b(r, "must_route_through_dunder_for_typed_handle_operands"));
    assert!(b(r, "must_cover_add_sub_mul_truediv_floordiv_mod_pow_neg"));
    assert!(b(
        r,
        "forbid_folding_to_native_i64_add_for_typed_handle_operands"
    ));
    assert!(b(
        r,
        "forbid_silently_succeeding_with_wrong_arithmetic_result"
    ));
    let names: Vec<&str> = a(r, "affected_operators")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for op in &[
        "__add__",
        "__sub__",
        "__mul__",
        "__truediv__",
        "__floordiv__",
        "__mod__",
        "__pow__",
        "__neg__",
    ] {
        assert!(names.contains(op), "missing op: {op}");
    }
    let pnames: Vec<&str> = a(r, "allowed_dispatch_path_values")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(pnames.contains(&"dunder_via_class_rs_mb_call_method"));
    assert!(pnames.contains(&"native_i64_fold"));
    assert!(pnames.contains(&"silently_wrong"));
    assert_eq!(
        s(r, "expected_dispatch_path"),
        "dunder_via_class_rs_mb_call_method"
    );
    assert_eq!(i(r, "native_fold_used_exit_code"), 466);
    assert_eq!(i(r, "dunder_dispatch_missing_exit_code"), 467);
    assert_eq!(i(r, "silently_wrong_exit_code"), 468);
    assert!(b(
        r,
        "must_distinguish_native_fold_from_dunder_missing_from_silently_wrong"
    ));
}

#[test]
fn r2_module_level_dispatcher_workaround_recorded() {
    let m = m();
    let r = get(
        &m,
        "r2_module_level_dispatcher_workaround_recorded_contract",
    );
    assert_eq!(s(r, "requirement_id"), "R2");
    assert!(b(r, "must_pin_dispatcher_prefix_per_lib"));
    assert!(b(r, "must_record_workaround_per_affected_lib"));
    let lib_names: Vec<&str> = a(r, "affected_libs")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(lib_names.contains(&"fractions"));
    assert!(lib_names.contains(&"decimal"));
    assert_eq!(s(r, "expected_fractions_prefix"), "fraction_");
    assert_eq!(s(r, "expected_decimal_prefix"), "decimal_");
    assert_eq!(
        s(r, "expected_worked_example_call"),
        "fractions.fraction_add"
    );
    assert_eq!(i(r, "workaround_dropped_exit_code"), 469);
    assert_eq!(i(r, "prefix_mismatch_exit_code"), 470);
    assert!(b(
        r,
        "must_distinguish_workaround_dropped_from_prefix_mismatch"
    ));
}

#[test]
fn r3_hasattr_conditional_binding_refused() {
    let m = m();
    let r = get(&m, "r3_hasattr_conditional_binding_refused_contract");
    assert_eq!(s(r, "requirement_id"), "R3");
    assert!(b(r, "must_refuse_hasattr_conditional_binding_pattern"));
    assert!(b(
        r,
        "must_pin_sys_implementation_name_branch_as_correct_pattern"
    ));
    assert!(b(
        r,
        "forbid_using_hasattr_conditional_binding_in_cross_runtime_fixtures"
    ));
    assert!(b(
        r,
        "forbid_silently_dropping_hasattr_branch_due_to_jit_type_confusion"
    ));
    let pnames: Vec<&str> = a(r, "allowed_runtime_detection_pattern_values")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(pnames.contains(&"sys_implementation_name_branch"));
    assert!(pnames.contains(&"hasattr_conditional_binding"));
    assert_eq!(
        s(r, "expected_runtime_detection_pattern"),
        "sys_implementation_name_branch"
    );
    assert_eq!(
        s(r, "expected_sibling_fingerprint"),
        "issue_2099_jit_branch_drop_after_stdlib_call"
    );
    assert_eq!(i(r, "hasattr_pattern_used_exit_code"), 471);
    assert_eq!(i(r, "jit_silently_dropped_branch_exit_code"), 472);
    assert!(b(r, "must_distinguish_pattern_used_from_jit_drop"));
}

#[test]
fn r4_phase_3_fix_paths_enumerated() {
    let m = m();
    let r = get(&m, "r4_phase_3_fix_paths_enumerated_contract");
    assert_eq!(s(r, "requirement_id"), "R4");
    assert!(b(r, "must_enumerate_three_phase_3_fix_paths"));
    assert!(b(r, "must_defer_implementation_to_1265_phase_3"));
    assert!(b(r, "forbid_implementing_phase_3_fix_in_this_issue"));
    let names: Vec<&str> = a(r, "phase_3_fix_paths")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(names.contains(&"compile_time_type_inference"));
    assert!(names.contains(&"mbvalue_typed_handle_tag"));
    assert!(names.contains(&"wrapper_instance_approach"));
    assert_eq!(i(r, "expected_phase_3_owner_issue"), 1265);
    assert_eq!(i(r, "phase_3_implemented_in_this_issue_exit_code"), 473);
    assert_eq!(i(r, "fix_path_dropped_exit_code"), 474);
    assert!(b(
        r,
        "must_distinguish_phase_3_implemented_from_fix_path_dropped"
    ));
}

#[test]
fn r5_method_call_only_libs_explicitly_unaffected() {
    let m = m();
    let r = get(
        &m,
        "r5_method_call_only_libs_explicitly_unaffected_contract",
    );
    assert_eq!(s(r, "requirement_id"), "R5");
    assert!(b(r, "must_pin_unaffected_libs_set"));
    assert!(b(r, "must_mark_unaffected_libs_out_of_scope"));
    assert!(b(r, "forbid_widening_gate_to_method_call_only_libs"));
    let names: Vec<&str> = a(r, "unaffected_libs")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for lib in &[
        "hashlib",
        "hmac",
        "struct.Struct",
        "queue.Queue",
        "io.BytesIO",
        "re.Pattern",
    ] {
        assert!(names.contains(lib), "missing unaffected lib: {lib}");
    }
    assert_eq!(i(r, "unaffected_lib_widened_into_gate_exit_code"), 475);
    assert_eq!(i(r, "unaffected_lib_dropped_from_record_exit_code"), 476);
    assert!(b(
        r,
        "must_distinguish_widened_into_gate_from_dropped_from_record"
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
        "affected_operators",
        "dispatch_path",
        "affected_libs",
        "fractions_dispatcher_prefix",
        "decimal_dispatcher_prefix",
        "worked_example_call",
        "runtime_detection_pattern",
        "sibling_jit_fingerprint",
        "phase_3_fix_paths",
        "phase_3_owner_issue",
        "unaffected_libs",
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
    assert!(b(
        oos,
        "phase_3_implementation_of_typed_handle_tag_or_wrapper_instance"
    ));
    assert!(b(oos, "fix_of_issue_2099_jit_branch_drop"));
    assert!(b(oos, "cpython_baseline_change"));
    assert!(b(oos, "method_call_only_handle_libs"));
    assert!(b(oos, "c_extension_fast_paths"));
}
