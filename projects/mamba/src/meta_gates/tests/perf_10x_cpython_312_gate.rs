#![cfg(test)]

// Locks the shape of the MVP 10x CPython 3.12 performance gate
// pinned by tests/governance/gates/mvp/perf_10x_cpython_312_gate/manifest.toml.
// Closes #2530. Parent: #2526.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/perf_10x_cpython_312_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("perf_10x_cpython_312_gate"));
    assert_eq!(m["issue"].as_integer(), Some(2530));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(m["family"].as_str(), Some("perf_10x_cpython_312_gate"));
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
fn surface_pins_all_required_coverage() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_output_mismatch_hard_failure",
        "must_cover_one_x_per_benchmark_floor",
        "must_cover_ten_x_suite_geomean_gate",
        "must_cover_cpython_3_12_identity_recording",
        "must_cover_machine_readable_summary",
        "must_require_internal_time_markers",
        "must_parse_tier_headers_in_cross_runtime_fixtures",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn atomic_queue_pins_ten_children() {
    let q = &manifest()["atomic_queue"];
    for key in [
        "issue_2563_make_perf_comparison_output_mismatch_a_hard_failure",
        "issue_2564_make_perf_benchmark_output_mismatch_a_hard_failure",
        "issue_2565_add_one_x_per_benchmark_floor_checker",
        "issue_2566_add_tier_metadata_to_baseline_json",
        "issue_2567_add_mvp_performance_benchmark_manifest",
        "issue_2569_add_ten_x_suite_geomean_checker",
        "issue_2570_require_internal_time_markers_for_accepted_perf_fixtures",
        "issue_2571_parse_tier_headers_in_cross_runtime_bench_fixtures",
        "issue_2572_record_cpython_3_12_executable_identity_in_perf_gate",
        "issue_2573_emit_machine_readable_performance_gate_summary",
    ] {
        assert_eq!(q[key].as_bool(), Some(true), "atomic_queue.{key}");
    }
}

#[test]
fn wrong_output_fails_before_speedup_is_accepted() {
    let c = &manifest()["output_mismatch_hard_failure_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("wrong_output_fails_before_speedup_is_accepted")
    );
    for key in [
        "must_fail_on_perf_comparison_output_mismatch",
        "must_fail_on_perf_benchmark_output_mismatch",
        "must_check_output_before_recording_speedup",
        "forbid_accepting_speedup_when_output_differs_from_cpython",
        "forbid_silently_treating_output_mismatch_as_warning",
        "must_distinguish_output_mismatch_from_speedup_accepted_with_mismatch",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["output_mismatch_failure_kind"].as_str(),
        Some("mvp_perf_output_mismatch")
    );
    assert_eq!(c["output_mismatch_exit_code"].as_integer(), Some(204));
    assert_eq!(
        c["speedup_accepted_with_mismatch_failure_kind"].as_str(),
        Some("mvp_perf_speedup_accepted_with_output_mismatch")
    );
    assert_eq!(
        c["speedup_accepted_with_mismatch_exit_code"].as_integer(),
        Some(205)
    );
}

#[test]
fn required_benchmarks_have_a_one_x_floor_and_ten_x_suite_average_gate() {
    let c = &manifest()["one_x_floor_and_ten_x_suite_gate_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("required_benchmarks_have_a_one_x_floor_and_ten_x_suite_average_gate")
    );
    for key in [
        "must_enforce_one_x_per_benchmark_floor",
        "must_enforce_ten_x_suite_geomean",
        "forbid_required_benchmark_below_one_x",
        "forbid_suite_geomean_below_ten_x",
        "must_distinguish_per_benchmark_floor_from_suite_geomean",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["one_x_floor_field_name"].as_str(),
        Some("per_benchmark_floor_ratio")
    );
    assert_eq!(
        c["ten_x_geomean_field_name"].as_str(),
        Some("suite_geomean_ratio")
    );
    assert_eq!(c["one_x_floor_required_value"].as_float(), Some(1.0));
    assert_eq!(c["ten_x_geomean_required_value"].as_float(), Some(10.0));
    assert_eq!(
        c["below_one_x_floor_failure_kind"].as_str(),
        Some("mvp_perf_required_benchmark_below_one_x_floor")
    );
    assert_eq!(c["below_one_x_floor_exit_code"].as_integer(), Some(206));
    assert_eq!(
        c["below_ten_x_geomean_failure_kind"].as_str(),
        Some("mvp_perf_suite_geomean_below_ten_x")
    );
    assert_eq!(c["below_ten_x_geomean_exit_code"].as_integer(), Some(207));
}

#[test]
fn cpython_comparison_runtime_is_proven_to_be_cpython_3_12() {
    let c = &manifest()["cpython_3_12_identity_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("cpython_comparison_runtime_is_proven_to_be_cpython_3_12")
    );
    for key in [
        "must_record_cpython_executable_path",
        "must_record_cpython_sys_version",
        "must_record_cpython_major_minor",
        "must_assert_cpython_major_is_3",
        "must_assert_cpython_minor_is_12",
        "forbid_using_non_cpython_runtime_as_comparison",
        "forbid_silently_proceeding_when_cpython_version_unknown",
        "must_distinguish_version_mismatch_from_identity_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_cpython_identity_fields"]
        .as_array()
        .expect("required_cpython_identity_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "cpython_executable_path",
            "cpython_sys_version",
            "cpython_major",
            "cpython_minor",
        ]
    );
    assert_eq!(
        c["cpython_identity_field_name"].as_str(),
        Some("cpython_identity")
    );
    assert_eq!(
        c["cpython_version_mismatch_failure_kind"].as_str(),
        Some("mvp_perf_cpython_version_mismatch")
    );
    assert_eq!(
        c["cpython_version_mismatch_exit_code"].as_integer(),
        Some(208)
    );
    assert_eq!(
        c["cpython_identity_missing_failure_kind"].as_str(),
        Some("mvp_perf_cpython_identity_missing")
    );
    assert_eq!(
        c["cpython_identity_missing_exit_code"].as_integer(),
        Some(209)
    );
}

#[test]
fn summary_is_machine_readable_for_ci_and_worker_triage() {
    let c = &manifest()["machine_readable_summary_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("summary_is_machine_readable_for_ci_and_worker_triage")
    );
    for key in [
        "must_emit_machine_readable_summary",
        "must_emit_summary_as_json",
        "forbid_summary_being_freeform_text_only",
        "forbid_summary_being_collapsed_to_human_text",
        "must_distinguish_not_machine_readable_from_missing_required_field",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["summary_field_name"].as_str(), Some("perf_gate_summary"));
    assert_eq!(c["summary_format"].as_str(), Some("json"));
    let fields: Vec<_> = c["required_summary_fields"]
        .as_array()
        .expect("required_summary_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "outcome",
            "per_benchmark_floor_ratio",
            "suite_geomean_ratio",
            "cpython_identity",
            "benchmark_results",
            "internal_time_markers",
            "tier_headers",
        ]
    );
    assert_eq!(
        c["summary_not_machine_readable_failure_kind"].as_str(),
        Some("mvp_perf_summary_not_machine_readable")
    );
    assert_eq!(
        c["summary_not_machine_readable_exit_code"].as_integer(),
        Some(210)
    );
    assert_eq!(
        c["summary_missing_required_field_failure_kind"].as_str(),
        Some("mvp_perf_summary_missing_required_field")
    );
    assert_eq!(
        c["summary_missing_required_field_exit_code"].as_integer(),
        Some(211)
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
            "per_benchmark_floor_ratio",
            "suite_geomean_ratio",
            "cpython_identity",
            "cpython_executable_path",
            "cpython_sys_version",
            "cpython_major",
            "cpython_minor",
            "benchmark_results",
            "internal_time_markers",
            "tier_headers",
            "perf_gate_summary",
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
            "wrong_output_fails_before_speedup_is_accepted",
            "required_benchmarks_have_a_one_x_floor_and_ten_x_suite_average_gate",
            "cpython_comparison_runtime_is_proven_to_be_cpython_3_12",
            "summary_is_machine_readable_for_ci_and_worker_triage",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_of_output_mismatch_enforcement",
        "runtime_implementation_of_one_x_floor_checker",
        "runtime_implementation_of_ten_x_geomean_checker",
        "runtime_implementation_of_tier_metadata_in_baseline_json",
        "runtime_implementation_of_performance_benchmark_manifest",
        "runtime_implementation_of_internal_time_markers",
        "runtime_implementation_of_tier_header_parsing",
        "runtime_implementation_of_cpython_3_12_executable_identity_capture",
        "runtime_implementation_of_machine_readable_summary_emission",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
