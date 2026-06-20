//! Inline migration of tests/generator_runtime_type_gate_fixture_2182.rs (#2182).
//!
//! Schema gate for Generator runtime type cross-cutting. Locks the shape of the
//! acceptance manifest.

#![cfg(test)]

use std::fs;
use std::path::PathBuf;
use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/runtime/generator_runtime_type_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

fn get_table<'a>(v: &'a Value, key: &str) -> &'a Value {
    v.get(key)
        .unwrap_or_else(|| panic!("manifest missing key: {key}"))
}

fn get_bool(v: &Value, key: &str) -> bool {
    get_table(v, key)
        .as_bool()
        .unwrap_or_else(|| panic!("key {key} is not a bool"))
}

fn get_str<'a>(v: &'a Value, key: &str) -> &'a str {
    get_table(v, key)
        .as_str()
        .unwrap_or_else(|| panic!("key {key} is not a string"))
}

fn get_int(v: &Value, key: &str) -> i64 {
    get_table(v, key)
        .as_integer()
        .unwrap_or_else(|| panic!("key {key} is not an integer"))
}

fn get_array<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    get_table(v, key)
        .as_array()
        .unwrap_or_else(|| panic!("key {key} is not an array"))
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(get_int(&m, "version"), 1);
    assert_eq!(get_str(&m, "fixture"), "generator_runtime_type_gate");
    assert_eq!(get_int(&m, "issue"), 2182);
    assert_eq!(get_str(&m, "profile"), "conformance");
    assert_eq!(get_str(&m, "family"), "generator_runtime_type_gate");
    assert_eq!(get_str(&m, "network"), "offline");
}

#[test]
fn isolation_pins_no_global_state() {
    let m = manifest();
    let iso = get_table(&m, "isolation");
    assert!(get_bool(iso, "forbid_writes_outside_project"));
    assert!(get_bool(iso, "forbid_user_home_reads"));
    assert!(get_bool(iso, "forbid_global_cache_reads"));
    assert!(get_bool(iso, "forbid_global_cache_writes"));
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let m = manifest();
    let py = get_table(&m, "python_target");
    assert_eq!(get_int(py, "python_major"), 3);
    assert_eq!(get_int(py, "python_minor"), 12);
    assert!(get_bool(py, "must_be_python_3_12"));
}

#[test]
fn surface_pins_the_five_required_acceptance_axes() {
    let m = manifest();
    let s = get_table(&m, "surface");
    assert!(get_bool(s, "must_cover_generator_mbobject_variant_added"));
    assert!(get_bool(
        s,
        "must_cover_class_rs_iter_next_dispatches_on_generator"
    ));
    assert!(get_bool(
        s,
        "must_cover_csv_reader_retrofit_as_worked_example"
    ));
    assert!(get_bool(s, "must_cover_phase_2_followons_enumerated"));
    assert!(get_bool(
        s,
        "must_cover_design_alternatives_b_and_c_refused"
    ));
    assert!(get_bool(s, "must_be_offline_or_loopback_only"));
    assert!(get_bool(s, "must_be_deterministic"));
}

#[test]
fn r1_generator_mbobject_variant_added_contract_is_shaped() {
    let m = manifest();
    let r1 = get_table(&m, "r1_generator_mbobject_variant_added_contract");
    assert_eq!(
        get_str(r1, "case"),
        "mbobject_generator_variant_added_with_closure_based_next_fn_and_box_dyn_any_state"
    );
    assert_eq!(get_str(r1, "requirement_id"), "R1");
    assert!(get_bool(r1, "must_add_mbobject_generator_variant"));
    assert!(get_bool(r1, "must_pin_closure_based_shape"));
    assert!(get_bool(r1, "must_carry_next_fn_field"));
    assert!(get_bool(r1, "must_carry_boxed_state_field"));
    assert!(get_bool(
        r1,
        "forbid_replacing_list_aliased_with_other_runtime_shape"
    ));
    assert!(get_bool(
        r1,
        "forbid_adding_generator_variant_without_next_fn_field"
    ));
    assert_eq!(get_str(r1, "expected_mbobject_variant"), "Generator");
    let shapes = get_array(r1, "allowed_generator_shape_values");
    assert!(shapes.iter().any(|v| v.as_str() == Some("closure_based")));
    assert!(shapes.iter().any(|v| v.as_str() == Some("coroutine_based")));
    assert!(shapes
        .iter()
        .any(|v| v.as_str() == Some("iterator_trait_object")));
    assert_eq!(get_str(r1, "expected_generator_shape"), "closure_based");
    assert_eq!(get_str(r1, "expected_next_fn_field"), "next_fn");
    assert_eq!(get_str(r1, "expected_state_field"), "state");
    assert_eq!(get_int(r1, "variant_missing_exit_code"), 453);
    assert_eq!(get_int(r1, "wrong_shape_exit_code"), 454);
    assert_eq!(get_int(r1, "next_fn_field_missing_exit_code"), 455);
    assert!(get_bool(
        r1,
        "must_distinguish_variant_missing_from_wrong_shape_from_next_fn_missing"
    ));
}

#[test]
fn r2_class_rs_iter_next_dispatches_on_generator_contract_is_shaped() {
    let m = manifest();
    let r2 = get_table(&m, "r2_class_rs_iter_next_dispatches_on_generator_contract");
    assert_eq!(
        get_str(r2, "case"),
        "class_rs_mb_iter_next_dispatches_on_generator_variant_without_materializing_a_list"
    );
    assert_eq!(get_str(r2, "requirement_id"), "R2");
    assert!(get_bool(r2, "must_dispatch_mb_iter_next_on_generator"));
    assert!(get_bool(r2, "must_invoke_next_fn_per_step"));
    assert!(get_bool(
        r2,
        "forbid_materializing_full_list_in_for_loop_lowering"
    ));
    assert!(get_bool(
        r2,
        "forbid_silently_falling_back_to_list_iter_dispatch"
    ));
    assert_eq!(get_str(r2, "expected_dispatch_entrypoint"), "mb_iter_next");
    let kinds = get_array(r2, "allowed_dispatch_kind_values");
    assert!(kinds.iter().any(|v| v.as_str() == Some("per_step_next_fn")));
    assert!(kinds
        .iter()
        .any(|v| v.as_str() == Some("materialize_full_list")));
    assert!(kinds
        .iter()
        .any(|v| v.as_str() == Some("silent_fallback_to_list_iter")));
    assert_eq!(get_str(r2, "expected_dispatch_kind"), "per_step_next_fn");
    assert_eq!(get_int(r2, "materialization_in_lowering_exit_code"), 456);
    assert_eq!(get_int(r2, "dispatch_missing_exit_code"), 457);
    assert_eq!(get_int(r2, "silent_fallback_exit_code"), 458);
    assert!(get_bool(
        r2,
        "must_distinguish_materialization_from_dispatch_missing_from_silent_fallback"
    ));
}

#[test]
fn r3_csv_reader_retrofit_as_worked_example_contract_is_shaped() {
    let m = manifest();
    let r3 = get_table(&m, "r3_csv_reader_retrofit_as_worked_example_contract");
    assert_eq!(
        get_str(r3, "case"),
        "csv_reader_converted_to_generator_and_mem_ratio_moves_vs_pre_conversion_baseline"
    );
    assert_eq!(get_str(r3, "requirement_id"), "R3");
    assert!(get_bool(r3, "must_convert_csv_reader_to_generator"));
    assert!(get_bool(r3, "must_convert_csv_dictreader_to_generator"));
    assert!(get_bool(r3, "must_record_pre_conversion_mem_ratio"));
    assert!(get_bool(r3, "must_record_post_conversion_mem_ratio"));
    assert!(get_bool(r3, "must_assert_post_better_than_pre"));
    assert!(get_bool(r3, "forbid_keeping_list_aliased_csv_reader"));
    assert_eq!(get_str(r3, "expected_worked_example_lib"), "csv");
    let apis = get_array(r3, "allowed_worked_example_api_values");
    assert!(apis.iter().any(|v| v.as_str() == Some("csv.reader")));
    assert!(apis.iter().any(|v| v.as_str() == Some("csv.DictReader")));
    assert_eq!(get_str(r3, "expected_pre_conversion_mem_ratio"), "0.13x");
    assert!(get_bool(r3, "expected_mem_ratio_must_improve"));
    assert_eq!(get_int(r3, "still_list_aliased_exit_code"), 459);
    assert_eq!(get_int(r3, "mem_did_not_move_exit_code"), 460);
    assert!(get_bool(
        r3,
        "must_distinguish_still_list_aliased_from_mem_did_not_move"
    ));
}

#[test]
fn r4_phase_2_followons_enumerated_contract_is_shaped() {
    let m = manifest();
    let r4 = get_table(&m, "r4_phase_2_followons_enumerated_contract");
    assert_eq!(
        get_str(r4, "case"),
        "phase_2_followon_libs_pinned_in_manifest_and_explicitly_marked_out_of_scope_for_this_issue"
    );
    assert_eq!(get_str(r4, "requirement_id"), "R4");
    assert!(get_bool(r4, "must_pin_phase_2_followon_set"));
    assert!(get_bool(
        r4,
        "must_mark_phase_2_followons_out_of_scope_for_this_issue"
    ));
    assert!(get_bool(r4, "forbid_silently_dropping_followon_from_set"));
    let followons = get_array(r4, "phase_2_followons");
    let names: Vec<&str> = followons.iter().filter_map(|v| v.as_str()).collect();
    assert!(names.contains(&"xml.etree.ElementTree.iter"));
    assert!(names.contains(&"glob.iglob"));
    assert!(names.contains(&"re.finditer"));
    assert!(names.contains(&"itertools.chain.from_iterable"));
    assert!(names.contains(&"xml.etree.iterparse"));
    assert!(names.contains(&"os.walk"));
    assert!(names.contains(&"mmap.find_iterator_mode"));
    assert_eq!(get_int(r4, "followon_dropped_exit_code"), 461);
    assert_eq!(
        get_int(r4, "followon_implemented_in_phase_1_exit_code"),
        462
    );
    assert!(get_bool(
        r4,
        "must_distinguish_followon_dropped_from_followon_pulled_into_phase_1"
    ));
}

#[test]
fn r5_design_alternatives_b_and_c_refused_contract_is_shaped() {
    let m = manifest();
    let r5 = get_table(&m, "r5_design_alternatives_b_and_c_refused_contract");
    assert_eq!(
        get_str(r5, "case"),
        "coroutine_based_and_iterator_trait_object_designs_refused_and_recorded_with_reason"
    );
    assert_eq!(get_str(r5, "requirement_id"), "R5");
    assert!(get_bool(r5, "must_refuse_coroutine_based_design"));
    assert!(get_bool(r5, "must_refuse_iterator_trait_object_design"));
    assert!(get_bool(
        r5,
        "must_record_refusal_reason_for_each_alternative"
    ));
    assert!(get_bool(
        r5,
        "forbid_silently_switching_to_alternative_b_or_c"
    ));
    let refused = get_array(r5, "refused_designs");
    let names: Vec<&str> = refused.iter().filter_map(|v| v.as_str()).collect();
    assert!(names.contains(&"coroutine_based"));
    assert!(names.contains(&"iterator_trait_object"));
    assert_eq!(
        get_str(r5, "expected_coroutine_refusal_reason"),
        "heavy_runtime_not_aligned_with_force_typed_compilation"
    );
    assert_eq!(
        get_str(r5, "expected_iterator_trait_refusal_reason"),
        "box_dyn_allocation_per_generator_costs_dominate_short_iter_streams"
    );
    assert_eq!(get_int(r5, "silently_switched_to_coroutine_exit_code"), 463);
    assert_eq!(
        get_int(r5, "silently_switched_to_iterator_trait_exit_code"),
        464
    );
    assert_eq!(get_int(r5, "refusal_reason_missing_exit_code"), 465);
    assert!(get_bool(
        r5,
        "must_distinguish_switch_to_coroutine_from_switch_to_iterator_trait_from_refusal_reason_missing"
    ));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let m = manifest();
    let rc = get_table(&m, "runner_contract");
    let keys = get_array(rc, "keys");
    let key_names: Vec<&str> = keys.iter().filter_map(|v| v.as_str()).collect();
    for expected in &[
        "outcome",
        "case",
        "requirement_id",
        "mbobject_variant",
        "generator_shape",
        "next_fn_field_name",
        "state_field_name",
        "dispatch_entrypoint",
        "dispatch_kind",
        "worked_example_lib",
        "worked_example_api",
        "pre_conversion_mem_ratio",
        "mem_ratio_must_improve",
        "phase_2_followons",
        "refused_designs",
        "coroutine_refusal_reason",
        "iterator_trait_refusal_reason",
        "failure_kind",
        "exit_code",
    ] {
        assert!(
            key_names.contains(expected),
            "missing runner_contract key: {expected}"
        );
    }
    let outcomes = get_array(rc, "outcome_values");
    let outcome_names: Vec<&str> = outcomes.iter().filter_map(|v| v.as_str()).collect();
    for expected in &["pass", "fail", "missing", "skip"] {
        assert!(outcome_names.contains(expected));
    }
    let cases = get_array(rc, "case_values");
    assert_eq!(cases.len(), 5);
}

#[test]
fn pins_out_of_scope_per_issue() {
    let m = manifest();
    let oos = get_table(&m, "out_of_scope");
    assert!(get_bool(oos, "phase_2_retrofits_of_other_stdlib_iterators"));
    assert!(get_bool(oos, "elimination_of_mbobject_list"));
    assert!(get_bool(oos, "cpython_baseline_change"));
    assert!(get_bool(oos, "non_iterator_stdlib_perf_work"));
    assert!(get_bool(oos, "c_extension_fast_paths"));
}
