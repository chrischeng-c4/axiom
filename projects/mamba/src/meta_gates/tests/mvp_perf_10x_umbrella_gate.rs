#![cfg(test)]

// Locks the shape of the MVP performance 10× umbrella fixture
// pinned by tests/governance/gates/mvp/mvp_perf_10x_umbrella_gate/
// manifest.toml. Closes #1260.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/mvp_perf_10x_umbrella_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("mvp_perf_10x_umbrella_gate"));
    assert_eq!(m["issue"].as_integer(), Some(1260));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(m["family"].as_str(), Some("mvp_perf_10x_umbrella_gate"));
    assert_eq!(m["network"].as_str(), Some("offline"));
    let children: Vec<_> = m["child_issues"]
        .as_array()
        .expect("child_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(children, vec![2096, 2110, 2512, 2513, 2516, 2517, 2518]);
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
fn surface_pins_baseline_geomean_floor_tiers_and_ci_gate() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_cpython_baseline_recorded",
        "must_cover_geomean_at_least_10x_cpython",
        "must_cover_no_accepted_bench_below_1x_floor",
        "must_cover_tiered_benchmarks_compute_app_dynamic",
        "must_cover_mamba_bench_check_is_canonical_ci_gate",
        "must_be_offline",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_cpython_baseline_recorded() {
    let c = &manifest()["r1_cpython_baseline_recorded_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("cpython_3_12_baseline_recorded_for_every_accepted_benchmark")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_record_cpython_baseline_per_accepted_bench",
        "must_use_cpython_3_12_as_baseline_runtime",
        "must_record_ratio_field_for_each_accepted_bench",
        "forbid_skipping_baseline_for_accepted_bench",
        "forbid_using_non_3_12_cpython_as_baseline",
        "must_distinguish_baseline_missing_from_non_3_12_baseline",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["baseline_runtime_field_name"].as_str(),
        Some("baseline_runtime")
    );
    assert_eq!(
        c["expected_baseline_runtime"].as_str(),
        Some("cpython_3_12")
    );
    assert_eq!(c["ratio_field_name"].as_str(), Some("ratio_vs_cpython"));
    assert_eq!(
        c["accepted_bench_count_field_name"].as_str(),
        Some("accepted_bench_count")
    );
    assert_eq!(
        c["baseline_missing_failure_kind"].as_str(),
        Some("mvp_perf_baseline_missing_for_accepted_bench")
    );
    assert_eq!(c["baseline_missing_exit_code"].as_integer(), Some(313));
    assert_eq!(
        c["non_3_12_baseline_failure_kind"].as_str(),
        Some("mvp_perf_baseline_runtime_is_not_cpython_3_12")
    );
    assert_eq!(c["non_3_12_baseline_exit_code"].as_integer(), Some(314));
}

#[test]
fn r2_geomean_at_least_10x() {
    let c = &manifest()["r2_geomean_at_least_10x_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("geomean_across_accepted_suite_is_at_least_10x_cpython")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_compute_geomean_across_accepted_suite",
        "must_fail_when_geomean_below_10x_cpython",
        "forbid_silently_passing_when_geomean_below_target",
        "forbid_replacing_geomean_with_arithmetic_mean",
        "must_distinguish_below_target_from_replaced_with_arithmetic_mean",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["geomean_target_field_name"].as_str(),
        Some("geomean_target")
    );
    assert_eq!(c["expected_geomean_target"].as_float(), Some(10.0));
    assert_eq!(
        c["geomean_observed_field_name"].as_str(),
        Some("geomean_observed")
    );
    assert_eq!(
        c["geomean_below_target_failure_kind"].as_str(),
        Some("mvp_perf_geomean_below_10x_target")
    );
    assert_eq!(c["geomean_below_target_exit_code"].as_integer(), Some(315));
    assert_eq!(
        c["geomean_replaced_with_arithmetic_mean_failure_kind"].as_str(),
        Some("mvp_perf_geomean_replaced_with_arithmetic_mean")
    );
    assert_eq!(
        c["geomean_replaced_with_arithmetic_mean_exit_code"].as_integer(),
        Some(316)
    );
}

#[test]
fn r3_no_accepted_bench_below_1x_floor() {
    let c = &manifest()["r3_no_accepted_bench_below_1x_floor_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("no_accepted_benchmark_is_below_1x_cpython_3_12_floor")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_enforce_floor_per_accepted_bench",
        "must_report_violating_bench_names_when_floor_broken",
        "forbid_allowing_below_1x_for_accepted_bench",
        "forbid_silently_excluding_below_1x_bench_from_geomean",
        "must_distinguish_floor_violation_from_silent_exclusion",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["floor_ratio_field_name"].as_str(), Some("floor_ratio"));
    assert_eq!(c["expected_floor_ratio"].as_float(), Some(1.0));
    assert_eq!(
        c["floor_violations_field_name"].as_str(),
        Some("floor_violations")
    );
    assert_eq!(
        c["floor_violation_failure_kind"].as_str(),
        Some("mvp_perf_accepted_bench_below_1x_floor")
    );
    assert_eq!(c["floor_violation_exit_code"].as_integer(), Some(317));
    assert_eq!(
        c["silent_exclusion_failure_kind"].as_str(),
        Some("mvp_perf_accepted_bench_silently_excluded_from_geomean")
    );
    assert_eq!(c["silent_exclusion_exit_code"].as_integer(), Some(318));
}

#[test]
fn r4_tiered_benchmarks_compute_app_dynamic() {
    let c = &manifest()["r4_tiered_benchmarks_compute_app_dynamic_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("benchmarks_are_tiered_compute_app_and_dynamic_not_collapsed")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_tier_benchmarks_into_compute_app_dynamic",
        "must_record_tier_per_accepted_bench",
        "must_require_nonzero_app_bench_count",
        "must_require_nonzero_dynamic_bench_count",
        "forbid_collapsing_suite_into_compute_only",
        "forbid_omitting_tier_field_per_bench",
        "must_distinguish_suite_collapsed_from_tier_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let tiers: Vec<_> = c["required_tiers"]
        .as_array()
        .expect("required_tiers")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(tiers, vec!["compute", "app", "dynamic"]);
    assert_eq!(c["tier_field_name"].as_str(), Some("tier"));
    assert_eq!(
        c["compute_count_field_name"].as_str(),
        Some("compute_bench_count")
    );
    assert_eq!(c["app_count_field_name"].as_str(), Some("app_bench_count"));
    assert_eq!(
        c["dynamic_count_field_name"].as_str(),
        Some("dynamic_bench_count")
    );
    assert_eq!(
        c["suite_collapsed_compute_only_failure_kind"].as_str(),
        Some("mvp_perf_suite_collapsed_to_compute_only")
    );
    assert_eq!(
        c["suite_collapsed_compute_only_exit_code"].as_integer(),
        Some(319)
    );
    assert_eq!(
        c["tier_missing_failure_kind"].as_str(),
        Some("mvp_perf_tier_missing_for_accepted_bench")
    );
    assert_eq!(c["tier_missing_exit_code"].as_integer(), Some(320));
}

#[test]
fn r5_mamba_bench_check_is_canonical_gate() {
    let c = &manifest()["r5_mamba_bench_check_is_canonical_gate_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("mamba_bench_check_is_canonical_ci_gate_for_mvp_perf")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_wire_mamba_bench_check_as_ci_gate",
        "must_fail_check_when_geomean_below_target",
        "must_fail_check_when_any_accepted_bench_below_floor",
        "forbid_treating_missing_check_as_pass",
        "forbid_silently_skipping_check_in_ci",
        "must_distinguish_check_missing_from_skipped_in_ci",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["ci_gate_command_field_name"].as_str(),
        Some("ci_gate_command")
    );
    assert_eq!(
        c["expected_ci_gate_command"].as_str(),
        Some("mamba bench --check")
    );
    assert_eq!(
        c["ci_gate_status_field_name"].as_str(),
        Some("ci_gate_status")
    );
    let statuses: Vec<_> = c["allowed_ci_gate_status_values"]
        .as_array()
        .expect("allowed_ci_gate_status_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(statuses, vec!["pass", "fail", "missing"]);
    assert_eq!(
        c["check_missing_treated_as_pass_failure_kind"].as_str(),
        Some("mvp_perf_check_missing_treated_as_pass")
    );
    assert_eq!(
        c["check_missing_treated_as_pass_exit_code"].as_integer(),
        Some(321)
    );
    assert_eq!(
        c["check_skipped_in_ci_failure_kind"].as_str(),
        Some("mvp_perf_check_silently_skipped_in_ci")
    );
    assert_eq!(c["check_skipped_in_ci_exit_code"].as_integer(), Some(322));
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
            "baseline_runtime",
            "ratio_vs_cpython",
            "accepted_bench_count",
            "geomean_target",
            "geomean_observed",
            "floor_ratio",
            "floor_violations",
            "tier",
            "compute_bench_count",
            "app_bench_count",
            "dynamic_bench_count",
            "ci_gate_command",
            "ci_gate_status",
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
            "cpython_3_12_baseline_recorded_for_every_accepted_benchmark",
            "geomean_across_accepted_suite_is_at_least_10x_cpython",
            "no_accepted_benchmark_is_below_1x_cpython_3_12_floor",
            "benchmarks_are_tiered_compute_app_and_dynamic_not_collapsed",
            "mamba_bench_check_is_canonical_ci_gate_for_mvp_perf",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_per_child_issue",
        "ecosystem_conformance_owned_by_1265",
        "c_extension_fast_paths",
        "package_manager_and_venv",
        "runtime_implementation_of_mir_escape_analysis",
        "runtime_implementation_of_mb_object_layout",
        "runtime_implementation_of_rwlock_fast_path",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
