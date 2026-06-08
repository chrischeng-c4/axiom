//! Inline migration of tests/mir_escape_analysis_skip_gc_track_gate_fixture_2516.rs (#2516).

#![cfg(test)]

use crate::testing::{b, get, i, load_manifest, s, strs};
use toml::Value;

const FIXTURE: &str =
    "tests/harness/cpython/config/perf/mir_escape_analysis_skip_gc_track_gate/manifest.toml";
fn m() -> Value {
    load_manifest(FIXTURE)
}

#[test]
fn header_is_well_formed() {
    let m = m();
    assert_eq!(i(&m, "version"), 1);
    assert_eq!(s(&m, "fixture"), "mir_escape_analysis_skip_gc_track_gate");
    assert_eq!(i(&m, "issue"), 2516);
    assert_eq!(i(&m, "parent_issue"), 2458);
    assert_eq!(s(&m, "profile"), "conformance");
    assert_eq!(s(&m, "family"), "mir_escape_analysis_skip_gc_track_gate");
    assert_eq!(s(&m, "network"), "offline");
}

#[test]
fn isolation_pins_no_global_state() {
    let m = m();
    let iso = get(&m, "isolation");
    for key in [
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert!(b(iso, key), "isolation.{key}");
    }
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
fn surface_pins_pass_codegen_conservatism_bench_and_regression_guard() {
    let m = m();
    let sf = get(&m, "surface");
    for key in [
        "must_cover_mir_pass_detects_non_escaping_literals",
        "must_cover_codegen_skips_gc_track_for_non_escaping",
        "must_cover_conservative_must_track_under_uncertainty",
        "must_cover_bench_acceptance_string_concat_and_list_sort_builtin",
        "must_cover_escaping_value_must_remain_gc_tracked",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert!(b(sf, key), "surface.{key}");
    }
}

#[test]
fn r1_mir_pass_detects_non_escaping_literals() {
    let m = m();
    let c = get(&m, "r1_mir_pass_detects_non_escaping_literals_contract");
    assert_eq!(
        s(c, "case"),
        "mir_escape_analysis_pass_detects_list_and_dict_literals_that_never_escape_scope"
    );
    assert_eq!(s(c, "requirement_id"), "R1");
    for key in [
        "must_run_pass_before_codegen",
        "must_classify_each_literal_as_escaping_or_non_escaping",
        "must_propagate_through_intra_scope_aliasing",
        "must_treat_return_as_escape",
        "must_treat_store_to_arg_as_escape",
        "must_treat_closure_capture_as_escape",
        "must_treat_exception_path_as_escape",
        "forbid_classifying_escaping_value_as_non_escaping",
        "must_distinguish_misclassification_from_pass_not_run",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(strs(c, "allowed_literal_kind_values"), vec!["list_literal", "dict_literal"]);
    assert_eq!(
        strs(c, "allowed_escape_classification_values"),
        vec!["escaping", "non_escaping"]
    );
    assert_eq!(
        s(c, "escaping_misclassified_failure_kind"),
        "perf_mir_escape_escaping_value_misclassified_as_non_escaping"
    );
    assert_eq!(i(c, "escaping_misclassified_exit_code"), 417);
    assert_eq!(
        s(c, "pass_not_run_before_codegen_failure_kind"),
        "perf_mir_escape_pass_not_run_before_codegen"
    );
    assert_eq!(i(c, "pass_not_run_before_codegen_exit_code"), 418);
}

#[test]
fn r2_codegen_skips_gc_track() {
    let m = m();
    let c = get(&m, "r2_codegen_skips_gc_track_for_non_escaping_contract");
    assert_eq!(
        s(c, "case"),
        "codegen_emits_non_escaping_literal_without_gc_track_or_gc_untrack"
    );
    assert_eq!(s(c, "requirement_id"), "R2");
    for key in [
        "must_skip_gc_track_call_for_non_escaping",
        "must_skip_gc_untrack_call_for_non_escaping",
        "must_emit_value_into_stack_or_arena_slot",
        "forbid_silently_emitting_gc_track_for_non_escaping",
        "forbid_emitting_gc_untrack_without_matching_gc_track",
        "must_distinguish_track_emitted_from_unbalanced_untrack",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(
        strs(c, "allowed_codegen_slot_kind_values"),
        vec!["stack", "arena", "heap_tracked"]
    );
    assert_eq!(
        strs(c, "allowed_gc_track_emission_values"),
        vec!["emitted", "skipped"]
    );
    assert_eq!(
        s(c, "gc_track_emitted_for_non_escaping_failure_kind"),
        "perf_mir_escape_gc_track_emitted_for_non_escaping"
    );
    assert_eq!(i(c, "gc_track_emitted_for_non_escaping_exit_code"), 419);
    assert_eq!(
        s(c, "unbalanced_gc_untrack_failure_kind"),
        "perf_mir_escape_unbalanced_gc_untrack"
    );
    assert_eq!(i(c, "unbalanced_gc_untrack_exit_code"), 420);
}

#[test]
fn r3_conservative_must_track_under_uncertainty() {
    let m = m();
    let c = get(&m, "r3_conservative_must_track_under_uncertainty_contract");
    assert_eq!(
        s(c, "case"),
        "uncertain_reachability_must_classify_literal_as_escaping_must_track"
    );
    assert_eq!(s(c, "requirement_id"), "R3");
    for key in [
        "must_default_to_escaping_when_reachability_unknown",
        "must_default_to_escaping_when_pass_lacks_information",
        "forbid_unsound_non_escaping_classification_on_uncertainty",
        "must_distinguish_unsoundness_from_normal_escape_classification",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(
        strs(c, "allowed_uncertainty_outcome_values"),
        vec!["defaulted_to_escaping", "defaulted_to_non_escaping"]
    );
    assert_eq!(
        s(c, "unsound_non_escaping_failure_kind"),
        "perf_mir_escape_unsound_non_escaping_under_uncertainty"
    );
    assert_eq!(i(c, "unsound_non_escaping_exit_code"), 421);
}

#[test]
fn r4_bench_acceptance_per_bench_1_5x() {
    let m = m();
    let c = get(&m, "r4_bench_acceptance_string_concat_and_list_sort_builtin_contract");
    assert_eq!(
        s(c, "case"),
        "string_concat_and_list_sort_builtin_each_improve_at_least_1_5x_vs_pre_pass_mamba_baseline"
    );
    assert_eq!(s(c, "requirement_id"), "R4");
    for key in [
        "must_compare_against_pre_pass_mamba_baseline",
        "must_preserve_cpython_baseline_measurement",
        "must_apply_per_bench_minimum_improvement",
        "forbid_geomean_only_acceptance",
        "forbid_silently_changing_cpython_baseline",
        "must_distinguish_below_minimum_from_baseline_change_from_geomean_only",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(strs(c, "required_benches"), vec!["string_concat", "list_sort_builtin"]);
    assert_eq!(s(c, "expected_minimum_per_bench_improvement"), "1.5x");
    assert_eq!(
        strs(c, "allowed_baseline_kind_values"),
        vec!["pre_pass_mamba", "cpython"]
    );
    assert_eq!(
        s(c, "per_bench_below_minimum_failure_kind"),
        "perf_mir_escape_per_bench_below_1_5x"
    );
    assert_eq!(i(c, "per_bench_below_minimum_exit_code"), 422);
    assert_eq!(
        s(c, "cpython_baseline_silently_changed_failure_kind"),
        "perf_mir_escape_cpython_baseline_silently_changed"
    );
    assert_eq!(i(c, "cpython_baseline_silently_changed_exit_code"), 423);
    assert_eq!(
        s(c, "geomean_only_acceptance_failure_kind"),
        "perf_mir_escape_geomean_only_acceptance"
    );
    assert_eq!(i(c, "geomean_only_acceptance_exit_code"), 424);
}

#[test]
fn r5_escaping_value_must_remain_gc_tracked() {
    let m = m();
    let c = get(&m, "r5_escaping_value_must_remain_gc_tracked_contract");
    assert_eq!(
        s(c, "case"),
        "escaping_literal_must_remain_gc_tracked_and_gc_untracked_under_pass"
    );
    assert_eq!(s(c, "requirement_id"), "R5");
    for key in [
        "must_keep_gc_track_for_escaping_literal",
        "must_keep_gc_untrack_for_escaping_literal",
        "forbid_dropping_gc_track_for_escaping_literal",
        "forbid_dropping_gc_untrack_for_escaping_literal",
        "must_distinguish_track_dropped_from_untrack_dropped",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(
        strs(c, "allowed_escape_path_values"),
        vec!["return", "store_to_arg", "closure_capture", "exception_path"]
    );
    assert_eq!(
        s(c, "escaping_track_dropped_failure_kind"),
        "perf_mir_escape_escaping_gc_track_dropped"
    );
    assert_eq!(i(c, "escaping_track_dropped_exit_code"), 425);
    assert_eq!(
        s(c, "escaping_untrack_dropped_failure_kind"),
        "perf_mir_escape_escaping_gc_untrack_dropped"
    );
    assert_eq!(i(c, "escaping_untrack_dropped_exit_code"), 426);
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let m = m();
    let r = get(&m, "runner_contract");
    assert_eq!(
        strs(r, "keys"),
        vec![
            "outcome",
            "case",
            "requirement_id",
            "literal_kind",
            "escape_classification",
            "codegen_slot_kind",
            "gc_track_emission",
            "uncertainty_outcome",
            "required_benches",
            "minimum_per_bench_improvement",
            "baseline_kind",
            "escape_path",
            "failure_kind",
            "exit_code",
        ]
    );
    assert_eq!(strs(r, "outcome_values"), vec!["pass", "fail", "missing", "skip"]);
    assert_eq!(
        strs(r, "case_values"),
        vec![
            "mir_escape_analysis_pass_detects_list_and_dict_literals_that_never_escape_scope",
            "codegen_emits_non_escaping_literal_without_gc_track_or_gc_untrack",
            "uncertain_reachability_must_classify_literal_as_escaping_must_track",
            "string_concat_and_list_sort_builtin_each_improve_at_least_1_5x_vs_pre_pass_mamba_baseline",
            "escaping_literal_must_remain_gc_tracked_and_gc_untracked_under_pass",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let m = m();
    let o = get(&m, "out_of_scope");
    for key in [
        "per_frame_stack_or_arena_allocator_implementation",
        "pymalloc_equivalent_freelist_1382",
        "cpython_baseline_measurement_method",
        "non_literal_value_escape_analysis",
        "c_extension_fast_paths",
    ] {
        assert!(b(o, key), "out_of_scope.{key}");
    }
}
