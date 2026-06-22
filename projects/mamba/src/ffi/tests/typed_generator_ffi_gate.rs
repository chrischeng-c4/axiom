//! Inline migration of tests/typed_generator_ffi_gate_fixture_1381.rs (#1381).
//!
//! Schema gate for typed generator FFI. Locks the manifest shape only.

#![cfg(test)]

use std::fs;
use std::path::PathBuf;
use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/harness/cpython/config/perf/typed_generator_ffi_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

fn get<'a>(v: &'a Value, key: &str) -> &'a Value {
    v.get(key).unwrap_or_else(|| panic!("missing key: {key}"))
}
fn b(v: &Value, key: &str) -> bool {
    get(v, key)
        .as_bool()
        .unwrap_or_else(|| panic!("{key} not bool"))
}
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    get(v, key)
        .as_str()
        .unwrap_or_else(|| panic!("{key} not str"))
}
fn i(v: &Value, key: &str) -> i64 {
    get(v, key)
        .as_integer()
        .unwrap_or_else(|| panic!("{key} not int"))
}
fn a<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    get(v, key)
        .as_array()
        .unwrap_or_else(|| panic!("{key} not array"))
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(i(&m, "version"), 1);
    assert_eq!(s(&m, "fixture"), "typed_generator_ffi_gate");
    assert_eq!(i(&m, "issue"), 1381);
    assert_eq!(i(&m, "parent_issue"), 1265);
    assert_eq!(s(&m, "profile"), "conformance");
    assert_eq!(s(&m, "family"), "typed_generator_ffi_gate");
    assert_eq!(s(&m, "network"), "offline");
}

#[test]
fn isolation_pins_no_global_state() {
    let m = manifest();
    let iso = get(&m, "isolation");
    assert!(b(iso, "forbid_writes_outside_project"));
    assert!(b(iso, "forbid_user_home_reads"));
    assert!(b(iso, "forbid_global_cache_reads"));
    assert!(b(iso, "forbid_global_cache_writes"));
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let m = manifest();
    let py = get(&m, "python_target");
    assert_eq!(i(py, "python_major"), 3);
    assert_eq!(i(py, "python_minor"), 12);
    assert!(b(py, "must_be_python_3_12"));
}

#[test]
fn surface_pins_the_five_required_axes() {
    let m = manifest();
    let sf = get(&m, "surface");
    assert!(b(sf, "must_cover_typed_ffi_entrypoints_added"));
    assert!(b(
        sf,
        "must_cover_codegen_emits_typed_variant_when_statically_known"
    ));
    assert!(b(sf, "must_cover_generator_sum_meets_3x_floor"));
    assert!(b(sf, "must_cover_untyped_fallback_no_regression"));
    assert!(b(sf, "must_cover_fix_path_bounded"));
    assert!(b(sf, "must_be_offline_or_loopback_only"));
    assert!(b(sf, "must_be_deterministic"));
}

#[test]
fn r1_typed_ffi_entrypoints_added() {
    let m = manifest();
    let r = get(&m, "r1_typed_ffi_entrypoints_added_contract");
    assert_eq!(s(r, "requirement_id"), "R1");
    assert!(b(r, "must_add_mb_next_i64_entrypoint"));
    assert!(b(r, "must_add_mb_next_f64_entrypoint"));
    assert!(b(r, "must_keep_mb_next_value_as_untyped_fallback"));
    assert!(b(r, "forbid_per_yield_primitive_boxing_when_both_typed"));
    assert!(b(r, "forbid_removing_untyped_fallback_in_this_issue"));
    let eps = a(r, "typed_ffi_entrypoints");
    let names: Vec<&str> = eps.iter().filter_map(|v| v.as_str()).collect();
    for ep in &["mb_next_i64", "mb_next_f64", "mb_next_value"] {
        assert!(names.contains(ep), "missing entrypoint: {ep}");
    }
    let kinds = a(r, "allowed_typed_return_record_kind_values");
    let knames: Vec<&str> = kinds.iter().filter_map(|v| v.as_str()).collect();
    assert!(knames.contains(&"value_and_done_struct"));
    assert!(knames.contains(&"tagged_union"));
    assert!(knames.contains(&"out_param_with_bool_return"));
    assert_eq!(
        s(r, "expected_typed_return_record_kind"),
        "value_and_done_struct"
    );
    assert_eq!(i(r, "typed_entrypoint_missing_exit_code"), 525);
    assert_eq!(i(r, "per_yield_boxing_when_both_typed_exit_code"), 526);
    assert_eq!(i(r, "untyped_fallback_removed_exit_code"), 527);
    assert!(b(
        r,
        "must_distinguish_typed_missing_from_boxing_from_fallback_removed"
    ));
}

#[test]
fn r2_codegen_emits_typed_variant() {
    let m = manifest();
    let r = get(
        &m,
        "r2_codegen_emits_typed_variant_when_statically_known_contract",
    );
    assert_eq!(s(r, "requirement_id"), "R2");
    assert!(b(r, "must_emit_typed_variant_when_element_type_known"));
    assert!(b(r, "must_skip_boxing_on_yield_to_consume_path"));
    assert!(b(r, "must_verify_via_cranelift_ir"));
    assert!(b(r, "forbid_emitting_box_int_call_on_typed_yield_path"));
    assert!(b(r, "forbid_emitting_unbox_int_call_on_typed_yield_path"));
    assert_eq!(i(r, "expected_box_int_call_count_on_typed_path"), 0);
    assert_eq!(i(r, "expected_unbox_int_call_count_on_typed_path"), 0);
    let kinds = a(r, "allowed_ir_verification_kind_values");
    let names: Vec<&str> = kinds.iter().filter_map(|v| v.as_str()).collect();
    assert!(names.contains(&"cranelift_clif_inspection"));
    assert!(names.contains(&"post_compile_disassembly"));
    assert!(names.contains(&"no_ir_check"));
    assert_eq!(
        s(r, "expected_ir_verification_kind"),
        "cranelift_clif_inspection"
    );
    assert_eq!(i(r, "box_int_emitted_on_typed_path_exit_code"), 528);
    assert_eq!(i(r, "unbox_int_emitted_on_typed_path_exit_code"), 529);
    assert_eq!(i(r, "ir_verification_skipped_exit_code"), 530);
    assert!(b(r, "must_distinguish_box_from_unbox_from_ir_skip"));
}

#[test]
fn r3_generator_sum_meets_3x_floor() {
    let m = manifest();
    let r = get(&m, "r3_generator_sum_meets_3x_floor_contract");
    assert_eq!(s(r, "requirement_id"), "R3");
    assert!(b(r, "must_meet_or_exceed_generator_sum_3x_floor"));
    assert!(b(r, "must_move_floor_compliance_count"));
    assert_eq!(s(r, "expected_bench_name"), "generator_sum_typed");
    assert_eq!(
        s(r, "expected_baseline_workload"),
        "yield_i_over_10000_ints_annotated_iterator_int"
    );
    assert_eq!(s(r, "expected_cpython_ratio_floor"), "3.0x");
    assert_eq!(s(r, "expected_pre_fix_starting_ratio"), "0.66x");
    assert_eq!(s(r, "expected_pre_fix_compliance"), "5_of_8");
    assert_eq!(s(r, "expected_post_fix_compliance_floor"), "6_of_8");
    assert_eq!(i(r, "floor_not_met_exit_code"), 531);
    assert_eq!(i(r, "compliance_did_not_move_exit_code"), 532);
    assert!(b(
        r,
        "must_distinguish_floor_not_met_from_compliance_did_not_move"
    ));
}

#[test]
fn r4_untyped_fallback_no_regression() {
    let m = manifest();
    let r = get(&m, "r4_untyped_fallback_no_regression_contract");
    assert_eq!(s(r, "requirement_id"), "R4");
    assert!(b(r, "must_keep_iterator_any_on_untyped_fallback"));
    assert!(b(r, "must_preserve_yield_order"));
    assert!(b(r, "must_preserve_terminal_done_semantics"));
    assert!(b(
        r,
        "must_preserve_stopiteration_propagation_on_exception_path"
    ));
    assert!(b(r, "forbid_regressing_untyped_generator_speed"));
    assert!(b(
        r,
        "forbid_changing_observable_yield_semantics_via_typed_path"
    ));
    assert_eq!(
        s(r, "expected_untyped_fallback_entrypoint"),
        "mb_next_value"
    );
    let axes = a(r, "preserved_semantic_axes");
    let names: Vec<&str> = axes.iter().filter_map(|v| v.as_str()).collect();
    for axis in &[
        "yield_order",
        "terminal_done_true",
        "stopiteration_propagation_on_exception_path",
    ] {
        assert!(names.contains(axis), "missing axis: {axis}");
    }
    assert_eq!(i(r, "untyped_regressed_exit_code"), 533);
    assert_eq!(i(r, "yield_order_changed_exit_code"), 534);
    assert_eq!(i(r, "stopiteration_propagation_broken_exit_code"), 535);
    assert!(b(
        r,
        "must_distinguish_untyped_regress_from_order_change_from_stopiter_break"
    ));
}

#[test]
fn r5_fix_path_bounded() {
    let m = manifest();
    let r = get(&m, "r5_fix_path_bounded_contract");
    assert_eq!(s(r, "requirement_id"), "R5");
    assert!(b(r, "must_pin_allowed_fix_paths"));
    assert!(b(r, "must_forbid_removing_untyped_fallback"));
    assert!(b(r, "must_forbid_cpython_baseline_change"));
    let allowed = a(r, "allowed_fix_path_kind_values");
    let anames: Vec<&str> = allowed.iter().filter_map(|v| v.as_str()).collect();
    for k in &[
        "monomorphized_typed_ffi_entry",
        "codegen_static_type_inference",
        "for_loop_lowering_skip_boxing_when_typed",
    ] {
        assert!(anames.contains(k), "missing allowed: {k}");
    }
    let disallowed = a(r, "disallowed_fix_path_kind_values");
    let dnames: Vec<&str> = disallowed.iter().filter_map(|v| v.as_str()).collect();
    for k in &[
        "removing_mb_next_value_untyped_fallback",
        "cpython_baseline_change",
        "rewriting_generator_state_machine_again_already_closed_under_1187",
    ] {
        assert!(dnames.contains(k), "missing disallowed: {k}");
    }
    assert_eq!(i(r, "disallowed_fix_path_used_exit_code"), 536);
    assert_eq!(i(r, "cpython_baseline_changed_exit_code"), 537);
    assert_eq!(i(r, "state_machine_rewritten_again_exit_code"), 538);
    assert!(b(
        r,
        "must_distinguish_disallowed_from_baseline_change_from_state_machine_rewrite"
    ));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let m = manifest();
    let rc = get(&m, "runner_contract");
    let keys = a(rc, "keys");
    let names: Vec<&str> = keys.iter().filter_map(|v| v.as_str()).collect();
    for k in &[
        "outcome",
        "case",
        "requirement_id",
        "typed_ffi_entrypoints",
        "typed_return_record_kind",
        "expected_box_int_call_count_on_typed_path",
        "expected_unbox_int_call_count_on_typed_path",
        "ir_verification_kind",
        "bench_name",
        "baseline_workload",
        "cpython_ratio_floor",
        "pre_fix_starting_ratio",
        "pre_fix_floor_compliance",
        "post_fix_floor_compliance_floor",
        "untyped_fallback_entrypoint",
        "preserved_semantic_axes",
        "fix_path_kind",
        "disallowed_fix_path_kind_values",
        "failure_kind",
        "exit_code",
    ] {
        assert!(names.contains(k), "missing key: {k}");
    }
    let cases = a(rc, "case_values");
    assert_eq!(cases.len(), 5);
}

#[test]
fn pins_out_of_scope_per_issue() {
    let m = manifest();
    let oos = get(&m, "out_of_scope");
    assert!(b(oos, "implementation_of_typed_ffi"));
    assert!(b(oos, "rewriting_generator_state_machine_again"));
    assert!(b(oos, "large_object_allocator_change"));
    assert!(b(oos, "cpython_baseline_change"));
    assert!(b(oos, "c_extension_fast_paths"));
}
