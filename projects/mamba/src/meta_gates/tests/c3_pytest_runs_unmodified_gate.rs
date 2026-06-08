#![cfg(test)]

// Locks the shape of the C3 pytest-runs-unmodified fixture pinned by
// tests/governance/gates/third_party/c3_pytest_runs_unmodified_gate/
// manifest.toml. Closes #1234.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/third_party/c3_pytest_runs_unmodified_gate/manifest.toml")
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
        Some("c3_pytest_runs_unmodified_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(1234));
    assert_eq!(m["profile"].as_str(), Some("third_party"));
    assert_eq!(
        m["family"].as_str(),
        Some("c3_pytest_runs_unmodified_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
    let related: Vec<_> = m["related_issues"]
        .as_array()
        .expect("related_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(related, vec![1263, 1259, 1257, 1265]);
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
fn surface_pins_all_six_requirements() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_pytest_q_exit_zero",
        "must_cover_assert_rewrite_diff_output",
        "must_cover_fixture_function_and_module_scope",
        "must_cover_parametrize_distinct_ids",
        "must_cover_pytest_v_matches_cpython_baseline",
        "must_cover_no_regression_on_conformance_and_cpython_compat",
        "must_be_offline",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_pytest_q_test_hello_exits_zero_on_trivial_assert() {
    let c = &manifest()["r1_pytest_q_exit_zero_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("pytest_q_test_hello_exits_zero_on_trivial_assert")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_install_pytest_unmodified_from_pypi",
        "must_run_pytest_q_against_test_hello",
        "must_exit_zero_when_test_passes",
        "forbid_modifying_pytest_on_disk_before_run",
        "forbid_skipping_pytest_q_with_workaround_runner",
        "must_record_pytest_q_exit_code",
        "must_distinguish_nonzero_exit_from_modified_runner",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["hello_test_relative_path"].as_str(),
        Some("tests/governance/gates/third_party/c3_pytest_runs_unmodified_gate/test_hello.py")
    );
    assert_eq!(
        c["hello_test_relative_path_field_name"].as_str(),
        Some("hello_test_path")
    );
    assert_eq!(
        c["pytest_q_exit_code_field_name"].as_str(),
        Some("pytest_q_exit_code")
    );
    assert_eq!(
        c["pytest_q_nonzero_exit_failure_kind"].as_str(),
        Some("c3_pytest_q_nonzero_exit")
    );
    assert_eq!(c["pytest_q_nonzero_exit_exit_code"].as_integer(), Some(265));
    assert_eq!(
        c["pytest_q_modified_runner_failure_kind"].as_str(),
        Some("c3_pytest_q_modified_runner_used_instead_of_pypi_pytest")
    );
    assert_eq!(
        c["pytest_q_modified_runner_exit_code"].as_integer(),
        Some(266)
    );
}

#[test]
fn r2_assertion_rewriting_hook_produces_diff_output() {
    let c = &manifest()["r2_assert_rewrite_diff_output_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("assertion_rewriting_hook_produces_diff_output_not_bare_assertion_error")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_install_assertion_rewriting_hook",
        "must_emit_assertrepr_compare_diff_on_failure",
        "must_match_cpython_3_12_diff_output_modulo_paths_and_line_numbers",
        "forbid_falling_back_to_bare_assertion_error_output",
        "forbid_silently_disabling_assert_rewrite",
        "must_distinguish_hook_missing_from_bare_assertion_error_fallback",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["assert_rewrite_hook_name_field_name"].as_str(),
        Some("assert_rewrite_hook_name")
    );
    assert_eq!(
        c["expected_assert_rewrite_hook_name"].as_str(),
        Some("AssertionRewritingHook")
    );
    assert_eq!(
        c["diff_output_field_name"].as_str(),
        Some("assert_rewrite_diff_output")
    );
    assert_eq!(
        c["assert_rewrite_hook_missing_failure_kind"].as_str(),
        Some("c3_pytest_assert_rewrite_hook_missing")
    );
    assert_eq!(
        c["assert_rewrite_hook_missing_exit_code"].as_integer(),
        Some(267)
    );
    assert_eq!(
        c["assert_rewrite_bare_assertion_error_failure_kind"].as_str(),
        Some("c3_pytest_assert_rewrite_fell_back_to_bare_assertion_error")
    );
    assert_eq!(
        c["assert_rewrite_bare_assertion_error_exit_code"].as_integer(),
        Some(268)
    );
}

#[test]
fn r3_pytest_fixture_function_and_module_scope_including_params_chain() {
    let c = &manifest()["r3_fixture_scope_and_parametrize_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("pytest_fixture_function_and_module_scope_including_params_chain")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_resolve_function_scope_fixture",
        "must_resolve_module_scope_fixture",
        "must_resolve_parametrized_fixture_with_params_list",
        "must_support_module_scope_feeding_function_scope_chain",
        "forbid_silently_treating_module_scope_as_function_scope",
        "forbid_silently_dropping_params_from_parametrized_fixture",
        "must_distinguish_scope_collapse_from_params_drop",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let scopes: Vec<_> = c["required_fixture_scopes"]
        .as_array()
        .expect("required_fixture_scopes")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(scopes, vec!["function", "module"]);
    assert_eq!(
        c["fixture_scope_field_name"].as_str(),
        Some("fixture_scope")
    );
    assert_eq!(
        c["fixture_chain_path_field_name"].as_str(),
        Some("fixture_chain")
    );
    assert_eq!(
        c["fixture_scope_collapsed_failure_kind"].as_str(),
        Some("c3_pytest_fixture_scope_collapsed_to_function")
    );
    assert_eq!(
        c["fixture_scope_collapsed_exit_code"].as_integer(),
        Some(269)
    );
    assert_eq!(
        c["fixture_params_dropped_failure_kind"].as_str(),
        Some("c3_pytest_fixture_params_silently_dropped")
    );
    assert_eq!(
        c["fixture_params_dropped_exit_code"].as_integer(),
        Some(270)
    );
}

#[test]
fn r4_parametrize_emits_n_runs_with_distinct_ids() {
    let c = &manifest()["r4_parametrize_distinct_ids_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("pytest_mark_parametrize_emits_n_runs_with_distinct_ids")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_collect_n_runs_for_parametrize_n_cases",
        "must_emit_distinct_test_id_per_parametrize_case",
        "must_surface_ids_in_dash_v_output",
        "forbid_collapsing_parametrize_to_single_run",
        "forbid_silently_reusing_test_id_across_parametrize_cases",
        "must_distinguish_parametrize_collapse_from_duplicate_id",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["parametrize_case_count_field_name"].as_str(),
        Some("parametrize_case_count")
    );
    assert_eq!(
        c["test_ids_field_name"].as_str(),
        Some("parametrize_test_ids")
    );
    assert_eq!(
        c["parametrize_collapsed_failure_kind"].as_str(),
        Some("c3_pytest_parametrize_collapsed_to_single_run")
    );
    assert_eq!(
        c["parametrize_collapsed_exit_code"].as_integer(),
        Some(271)
    );
    assert_eq!(
        c["parametrize_duplicate_id_failure_kind"].as_str(),
        Some("c3_pytest_parametrize_duplicate_test_id")
    );
    assert_eq!(
        c["parametrize_duplicate_id_exit_code"].as_integer(),
        Some(272)
    );
}

#[test]
fn r5_pytest_v_matches_cpython_baseline() {
    let c = &manifest()["r5_pytest_v_matches_cpython_baseline_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("pytest_v_exit_and_pass_fail_counts_match_cpython_3_12_baseline")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_run_pytest_v",
        "must_compare_exit_code_against_cpython_3_12_baseline",
        "must_compare_pass_count_against_cpython_3_12_baseline",
        "must_compare_fail_count_against_cpython_3_12_baseline",
        "forbid_comparing_timing_as_part_of_baseline_match",
        "forbid_silently_ignoring_baseline_divergence",
        "must_distinguish_exit_divergence_from_count_divergence",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["baseline_runtime"].as_str(), Some("cpython_3_12"));
    assert_eq!(
        c["exit_code_field_name"].as_str(),
        Some("pytest_v_exit_code")
    );
    assert_eq!(
        c["pass_count_field_name"].as_str(),
        Some("pytest_v_pass_count")
    );
    assert_eq!(
        c["fail_count_field_name"].as_str(),
        Some("pytest_v_fail_count")
    );
    assert_eq!(
        c["baseline_exit_divergence_failure_kind"].as_str(),
        Some("c3_pytest_v_baseline_exit_divergence")
    );
    assert_eq!(
        c["baseline_exit_divergence_exit_code"].as_integer(),
        Some(273)
    );
    assert_eq!(
        c["baseline_count_divergence_failure_kind"].as_str(),
        Some("c3_pytest_v_baseline_pass_or_fail_count_divergence")
    );
    assert_eq!(
        c["baseline_count_divergence_exit_code"].as_integer(),
        Some(274)
    );
}

#[test]
fn r6_no_regression_on_existing_suites() {
    let c = &manifest()["r6_no_regression_on_existing_suites_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("conformance_591_591_and_cpython_compat_54_54_stay_green")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R6"));
    for key in [
        "must_keep_conformance_suite_passing",
        "must_keep_cpython_compat_suite_passing",
        "forbid_silently_relaxing_conformance_count",
        "forbid_silently_relaxing_cpython_compat_count",
        "must_distinguish_conformance_regression_from_cpython_compat_regression",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["required_conformance_pass_count"].as_integer(),
        Some(591)
    );
    assert_eq!(
        c["required_conformance_total_count"].as_integer(),
        Some(591)
    );
    assert_eq!(
        c["required_cpython_compat_pass_count"].as_integer(),
        Some(54)
    );
    assert_eq!(
        c["required_cpython_compat_total_count"].as_integer(),
        Some(54)
    );
    assert_eq!(
        c["conformance_pass_count_field_name"].as_str(),
        Some("conformance_pass_count")
    );
    assert_eq!(
        c["cpython_compat_pass_count_field_name"].as_str(),
        Some("cpython_compat_pass_count")
    );
    assert_eq!(
        c["conformance_regression_failure_kind"].as_str(),
        Some("c3_pytest_conformance_regression")
    );
    assert_eq!(
        c["conformance_regression_exit_code"].as_integer(),
        Some(275)
    );
    assert_eq!(
        c["cpython_compat_regression_failure_kind"].as_str(),
        Some("c3_pytest_cpython_compat_regression")
    );
    assert_eq!(
        c["cpython_compat_regression_exit_code"].as_integer(),
        Some(276)
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
            "hello_test_path",
            "pytest_q_exit_code",
            "assert_rewrite_hook_name",
            "assert_rewrite_diff_output",
            "fixture_scope",
            "fixture_chain",
            "parametrize_case_count",
            "parametrize_test_ids",
            "pytest_v_exit_code",
            "pytest_v_pass_count",
            "pytest_v_fail_count",
            "conformance_pass_count",
            "cpython_compat_pass_count",
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
            "pytest_q_test_hello_exits_zero_on_trivial_assert",
            "assertion_rewriting_hook_produces_diff_output_not_bare_assertion_error",
            "pytest_fixture_function_and_module_scope_including_params_chain",
            "pytest_mark_parametrize_emits_n_runs_with_distinct_ids",
            "pytest_v_exit_and_pass_fail_counts_match_cpython_3_12_baseline",
            "conformance_591_591_and_cpython_compat_54_54_stay_green",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "flask_werkzeug_wsgi",
        "requests_urllib3_tls_ssl",
        "pytest_plugins_beyond_core",
        "c_extension_fast_paths",
        "performance_gates",
        "runtime_implementation_of_assert_rewrite",
        "runtime_implementation_of_fixture_resolution",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
