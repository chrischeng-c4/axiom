// Locks the shape of the MVP smoke gate compiles and lists tests
// fixture pinned by tests/governance/gates/mvp/smoke_gate_compile_and_list/
// manifest.toml. Closes #2527. Parent: #2526.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/smoke_gate_compile_and_list/manifest.toml")
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
        Some("mvp_smoke_gate_compile_and_list")
    );
    assert_eq!(m["issue"].as_integer(), Some(2527));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(
        m["family"].as_str(),
        Some("mvp_smoke_gate_compile_and_list")
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
fn surface_pins_smoke_list_summary_and_coverage_path_coverage() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_canonical_smoke_command",
        "must_cover_canonical_list_command",
        "must_cover_smoke_summary_emission",
        "must_cover_coverage_tooling_path_repair",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
    assert_eq!(s["canonical_package_name"].as_str(), Some("mamba"));
    assert_eq!(
        s["canonical_package_relative_path"].as_str(),
        Some("projects/mamba")
    );
}

#[test]
fn atomic_queue_pins_four_children() {
    let q = &manifest()["atomic_queue"];
    for key in [
        "issue_2534_make_cargo_test_list_the_first_worker_gate",
        "issue_2535_document_canonical_mvp_test_profile_commands",
        "issue_2537_add_mvp_smoke_test_inventory_summary",
        "issue_2538_repair_mamba_test_coverage_skill_path",
    ] {
        assert_eq!(q[key].as_bool(), Some(true), "atomic_queue.{key}");
    }
}

#[test]
fn worker_can_run_the_canonical_smoke_and_list_commands() {
    let c = &manifest()["canonical_smoke_and_list_command_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("worker_can_run_the_canonical_smoke_and_list_commands")
    );
    for key in [
        "must_record_canonical_smoke_command",
        "must_record_canonical_list_command",
        "must_record_smoke_command_in_documentation",
        "must_record_list_command_in_documentation",
        "forbid_workers_inventing_ad_hoc_smoke_commands",
        "forbid_workers_inventing_ad_hoc_list_commands",
        "must_distinguish_smoke_command_missing_from_list_command_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["canonical_smoke_command"].as_str(),
        Some("cargo test --manifest-path projects/mamba/Cargo.toml")
    );
    assert_eq!(
        c["canonical_list_command"].as_str(),
        Some("cargo test --manifest-path projects/mamba/Cargo.toml -- --list")
    );
    assert_eq!(
        c["canonical_smoke_command_field_name"].as_str(),
        Some("canonical_smoke_command")
    );
    assert_eq!(
        c["canonical_list_command_field_name"].as_str(),
        Some("canonical_list_command")
    );
    assert_eq!(
        c["smoke_command_missing_failure_kind"].as_str(),
        Some("mvp_smoke_command_missing")
    );
    assert_eq!(c["smoke_command_missing_exit_code"].as_integer(), Some(212));
    assert_eq!(
        c["list_command_missing_failure_kind"].as_str(),
        Some("mvp_list_command_missing")
    );
    assert_eq!(c["list_command_missing_exit_code"].as_integer(), Some(213));
}

#[test]
fn smoke_summary_reports_total_tests_fixture_counts_and_debt_counts() {
    let c = &manifest()["smoke_summary_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("smoke_summary_reports_total_tests_fixture_counts_and_debt_counts")
    );
    for key in [
        "must_emit_total_test_count",
        "must_emit_fixture_count",
        "must_emit_debt_count",
        "must_emit_ignored_count",
        "must_emit_xfail_count",
        "must_emit_skip_count",
        "forbid_smoke_summary_being_freeform_text",
        "forbid_omitting_required_summary_counts",
        "must_distinguish_summary_freeform_from_missing_required_count",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_summary_fields"]
        .as_array()
        .expect("required_summary_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "total_test_count",
            "fixture_count",
            "debt_count",
            "ignored_count",
            "xfail_count",
            "skip_count",
        ]
    );
    assert_eq!(
        c["smoke_summary_field_name"].as_str(),
        Some("smoke_summary")
    );
    assert_eq!(
        c["summary_freeform_failure_kind"].as_str(),
        Some("mvp_smoke_summary_freeform")
    );
    assert_eq!(c["summary_freeform_exit_code"].as_integer(), Some(214));
    assert_eq!(
        c["summary_missing_required_count_failure_kind"].as_str(),
        Some("mvp_smoke_summary_missing_required_count")
    );
    assert_eq!(
        c["summary_missing_required_count_exit_code"].as_integer(),
        Some(215)
    );
}

#[test]
fn coverage_tooling_points_at_projects_mamba_and_package_mamba_not_stale_crate_paths() {
    let c = &manifest()["coverage_tooling_path_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("coverage_tooling_points_at_projects_mamba_and_package_mamba_not_stale_crate_paths")
    );
    for key in [
        "must_point_coverage_tooling_at_projects_mamba",
        "must_point_coverage_tooling_at_package_mamba",
        "forbid_pointing_coverage_tooling_at_stale_crate_paths",
        "forbid_silently_skipping_coverage_when_path_is_stale",
        "must_distinguish_stale_path_from_stale_package",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["required_coverage_target_path"].as_str(),
        Some("projects/mamba")
    );
    assert_eq!(
        c["required_coverage_target_package"].as_str(),
        Some("mamba")
    );
    assert_eq!(
        c["stale_coverage_path_field_name"].as_str(),
        Some("coverage_target_path")
    );
    assert_eq!(
        c["stale_coverage_package_field_name"].as_str(),
        Some("coverage_target_package")
    );
    assert_eq!(
        c["stale_coverage_path_failure_kind"].as_str(),
        Some("mvp_smoke_coverage_target_path_stale")
    );
    assert_eq!(c["stale_coverage_path_exit_code"].as_integer(), Some(216));
    assert_eq!(
        c["stale_coverage_package_failure_kind"].as_str(),
        Some("mvp_smoke_coverage_target_package_stale")
    );
    assert_eq!(
        c["stale_coverage_package_exit_code"].as_integer(),
        Some(217)
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
            "canonical_smoke_command",
            "canonical_list_command",
            "smoke_summary",
            "total_test_count",
            "fixture_count",
            "debt_count",
            "ignored_count",
            "xfail_count",
            "skip_count",
            "coverage_target_path",
            "coverage_target_package",
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
            "worker_can_run_the_canonical_smoke_and_list_commands",
            "smoke_summary_reports_total_tests_fixture_counts_and_debt_counts",
            "coverage_tooling_points_at_projects_mamba_and_package_mamba_not_stale_crate_paths",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_of_first_worker_gate_registration",
        "runtime_implementation_of_command_documentation",
        "runtime_implementation_of_inventory_summary_emission",
        "runtime_implementation_of_coverage_tooling_path_repair",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
