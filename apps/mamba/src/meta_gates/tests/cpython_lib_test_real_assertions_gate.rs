#![cfg(test)]

// Locks the shape of the CPython Lib/test executes-assertions-not-
// stub-pass parent gate pinned by tests/harness/cpython/config/seeds/
// executes_assertions_not_stub_pass/manifest.toml. Closes #2528.
// Parent: #2526.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/harness/cpython/config/seeds/executes_assertions_not_stub_pass/manifest.toml")
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
        Some("cpython_lib_test_executes_assertions_not_stub_pass")
    );
    assert_eq!(m["issue"].as_integer(), Some(2528));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("cpython_lib_test"));
    assert_eq!(
        m["family"].as_str(),
        Some("cpython_lib_test_executes_assertions_not_stub_pass")
    );
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
fn surface_pins_all_outcomes_and_exclusion_rules() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_assertion_pass_outcome",
        "must_cover_import_pass_outcome",
        "must_cover_stub_outcome",
        "must_cover_fail_outcome",
        "must_cover_timeout_outcome",
        "must_cover_parser_only_outcome",
        "must_exclude_stub_from_mvp_pass_count",
        "must_exclude_parser_only_from_mvp_pass_count",
        "must_fail_on_missing_seed_baseline_entries",
        "must_fail_on_unbaselined_seed_fixtures",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn atomic_queue_pins_ten_children() {
    let q = &manifest()["atomic_queue"];
    for key in [
        "issue_2536_cpython_lib_test_seed_baseline_inventory_check",
        "issue_2539_add_unittest_assertion_sentinel_seed",
        "issue_2540_add_assertion_pass_outcome_to_cpython_lib_test_runner",
        "issue_2541_add_missing_cpython_lib_test_seed_baseline_entries",
        "issue_2542_rename_cpython_lib_test_pass_to_import_pass",
        "issue_2543_emit_cpython_lib_test_machine_readable_summary",
        "issue_2544_make_stub_never_count_as_mvp_pass",
        "issue_2545_wire_one_minimal_unittest_dispatch_path",
        "issue_2546_split_cpython_parser_only_checks_from_runtime_checks",
        "issue_2547_add_cpython_lib_test_allowlist_regression_gate",
    ] {
        assert_eq!(q[key].as_bool(), Some(true), "atomic_queue.{key}");
    }
}

#[test]
fn runtime_assertion_pass_import_pass_stub_fail_timeout_and_parser_only_are_separate_categories() {
    let c = &manifest()["separate_outcome_categories_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some(
            "runtime_assertion_pass_import_pass_stub_fail_timeout_and_parser_only_are_separate_categories"
        )
    );
    for key in [
        "must_define_assertion_pass_category",
        "must_define_import_pass_category",
        "must_define_stub_category",
        "must_define_fail_category",
        "must_define_timeout_category",
        "must_define_parser_only_category",
        "forbid_collapsing_outcome_categories",
        "forbid_aliasing_stub_to_assertion_pass",
        "forbid_aliasing_import_pass_to_assertion_pass",
        "forbid_aliasing_parser_only_to_assertion_pass",
        "must_distinguish_each_outcome_category",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let cats: Vec<_> = c["required_outcome_categories"]
        .as_array()
        .expect("required_outcome_categories")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        cats,
        vec![
            "assertion_pass",
            "import_pass",
            "stub",
            "fail",
            "timeout",
            "parser_only",
        ]
    );
    assert_eq!(
        c["outcome_category_field_name"].as_str(),
        Some("outcome_category")
    );
    assert_eq!(
        c["collapsed_outcome_failure_kind"].as_str(),
        Some("cpython_lib_test_outcome_collapsed")
    );
    assert_eq!(c["collapsed_outcome_exit_code"].as_integer(), Some(218));
}

#[test]
fn mvp_pass_count_excludes_stub_and_parser_only_checks() {
    let c = &manifest()["mvp_pass_count_exclusion_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("mvp_pass_count_excludes_stub_and_parser_only_checks")
    );
    for key in [
        "must_exclude_stub_from_mvp_pass_count",
        "must_exclude_parser_only_from_mvp_pass_count",
        "forbid_stub_being_treated_as_mvp_pass",
        "forbid_parser_only_being_treated_as_mvp_pass",
        "forbid_silently_inflating_mvp_pass_count",
        "must_distinguish_stub_inclusion_from_parser_only_inclusion",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["mvp_pass_count_field_name"].as_str(),
        Some("mvp_pass_count")
    );
    assert_eq!(
        c["excluded_outcome_categories_field_name"].as_str(),
        Some("excluded_from_mvp_pass_count")
    );
    let excluded: Vec<_> = c["required_excluded_outcome_categories"]
        .as_array()
        .expect("required_excluded_outcome_categories")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(excluded, vec!["stub", "parser_only"]);
    assert_eq!(
        c["stub_in_mvp_pass_count_failure_kind"].as_str(),
        Some("cpython_lib_test_stub_counted_as_mvp_pass")
    );
    assert_eq!(
        c["stub_in_mvp_pass_count_exit_code"].as_integer(),
        Some(219)
    );
    assert_eq!(
        c["parser_only_in_mvp_pass_count_failure_kind"].as_str(),
        Some("cpython_lib_test_parser_only_counted_as_mvp_pass")
    );
    assert_eq!(
        c["parser_only_in_mvp_pass_count_exit_code"].as_integer(),
        Some(220)
    );
}

#[test]
fn missing_or_unbaselined_seed_fixtures_fail_the_gate() {
    let c = &manifest()["missing_or_unbaselined_seed_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("missing_or_unbaselined_seed_fixtures_fail_the_gate")
    );
    for key in [
        "must_fail_on_missing_seed_baseline_entries",
        "must_fail_on_unbaselined_seed_fixtures",
        "forbid_silently_passing_with_missing_baseline",
        "forbid_silently_passing_with_unbaselined_seed",
        "must_distinguish_missing_baseline_entry_from_unbaselined_seed",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["seed_baseline_inventory_field_name"].as_str(),
        Some("seed_baseline_inventory")
    );
    assert_eq!(
        c["missing_seed_baseline_failure_kind"].as_str(),
        Some("cpython_lib_test_seed_baseline_entry_missing")
    );
    assert_eq!(c["missing_seed_baseline_exit_code"].as_integer(), Some(221));
    assert_eq!(
        c["unbaselined_seed_failure_kind"].as_str(),
        Some("cpython_lib_test_seed_unbaselined")
    );
    assert_eq!(c["unbaselined_seed_exit_code"].as_integer(), Some(222));
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
            "outcome_category",
            "mvp_pass_count",
            "excluded_from_mvp_pass_count",
            "seed_baseline_inventory",
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
            "runtime_assertion_pass_import_pass_stub_fail_timeout_and_parser_only_are_separate_categories",
            "mvp_pass_count_excludes_stub_and_parser_only_checks",
            "missing_or_unbaselined_seed_fixtures_fail_the_gate",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_of_seed_baseline_inventory_check",
        "runtime_implementation_of_unittest_assertion_sentinel_seed",
        "runtime_implementation_of_assertion_pass_outcome",
        "runtime_implementation_of_seed_baseline_entries",
        "runtime_implementation_of_import_pass_rename",
        "runtime_implementation_of_machine_readable_summary",
        "runtime_implementation_of_stub_never_counts_as_mvp_pass",
        "runtime_implementation_of_minimal_unittest_dispatch_path",
        "runtime_implementation_of_parser_only_runtime_split",
        "runtime_implementation_of_allowlist_regression_gate",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
