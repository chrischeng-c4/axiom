// Locks the shape of the MVP uv-like package manager offline E2E
// gate pinned by tests/governance/gates/mvp/
// uv_like_package_manager_offline_e2e/manifest.toml. Closes #2532.
// Parent: #2526.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/uv_like_package_manager_offline_e2e/manifest.toml")
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
        Some("uv_like_package_manager_offline_e2e")
    );
    assert_eq!(m["issue"].as_integer(), Some(2532));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(
        m["family"].as_str(),
        Some("uv_like_package_manager_offline_e2e")
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
fn surface_pins_offline_default_and_all_paths() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_offline_default_gate",
        "must_cover_resolver_conflict_path",
        "must_cover_yanked_version_path",
        "must_cover_transitive_dependency_path",
        "must_cover_pure_python_wheel_install_path",
        "must_cover_lockfile_determinism_path",
        "must_cover_venv_site_packages_activation_path",
        "must_cover_console_script_entrypoint_path",
        "must_keep_live_pypi_opt_in",
        "must_use_frozen_local_simple_index",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn atomic_queue_pins_ten_children() {
    let q = &manifest()["atomic_queue"];
    for key in [
        "issue_2584_unignore_resolver_yanked_version_test_using_frozen_index",
        "issue_2585_add_frozen_local_simple_index_fixture",
        "issue_2586_add_package_manager_lockfile_determinism_test",
        "issue_2587_unignore_resolver_conflict_test_using_frozen_index",
        "issue_2588_add_offline_init_add_install_sync_run_e2e_skeleton",
        "issue_2589_add_venv_site_packages_import_activation_test",
        "issue_2590_keep_live_pypi_checks_outside_default_mvp_gate",
        "issue_2591_add_pure_python_wheel_install_fixture",
        "issue_2592_add_transitive_dependency_resolution_fixture",
        "issue_2593_add_console_script_entrypoint_fixture",
    ] {
        assert_eq!(q[key].as_bool(), Some(true), "atomic_queue.{key}");
    }
}

#[test]
fn default_package_manager_gate_is_offline() {
    let c = &manifest()["offline_default_gate_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("default_package_manager_gate_is_offline")
    );
    for key in [
        "must_default_to_offline_mode",
        "must_fail_when_network_access_attempted_in_default_gate",
        "must_use_frozen_local_simple_index",
        "forbid_resolving_against_live_pypi_in_default_gate",
        "forbid_downloading_artifacts_from_network_in_default_gate",
        "must_distinguish_network_attempt_from_index_misconfiguration",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["network_access_attempted_failure_kind"].as_str(),
        Some("mvp_pkgmgr_network_access_attempted_in_offline_gate")
    );
    assert_eq!(
        c["network_access_attempted_exit_code"].as_integer(),
        Some(190)
    );
    assert_eq!(
        c["index_misconfiguration_failure_kind"].as_str(),
        Some("mvp_pkgmgr_frozen_index_misconfigured")
    );
    assert_eq!(
        c["index_misconfiguration_exit_code"].as_integer(),
        Some(191)
    );
}

#[test]
fn resolver_conflict_yanked_transitive_wheel_lockfile_activation_console_script_paths_are_covered()
{
    let c = &manifest()["covered_paths_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some(
            "resolver_conflict_yanked_version_transitive_wheel_lockfile_activation_console_script_paths_are_covered"
        )
    );
    for key in [
        "must_cover_resolver_conflict_path",
        "must_cover_yanked_version_path",
        "must_cover_transitive_dependency_path",
        "must_cover_pure_python_wheel_install_path",
        "must_cover_lockfile_determinism_path",
        "must_cover_venv_site_packages_activation_path",
        "must_cover_console_script_entrypoint_path",
        "forbid_silently_dropping_any_required_path",
        "must_distinguish_missing_path_from_uncovered_path",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let paths: Vec<_> = c["required_covered_paths"]
        .as_array()
        .expect("required_covered_paths")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        paths,
        vec![
            "resolver_conflict",
            "yanked_version",
            "transitive_dependency",
            "pure_python_wheel_install",
            "lockfile_determinism",
            "venv_site_packages_activation",
            "console_script_entrypoint",
        ]
    );
    assert_eq!(c["covered_paths_field_name"].as_str(), Some("covered_paths"));
    assert_eq!(
        c["missing_covered_path_failure_kind"].as_str(),
        Some("mvp_pkgmgr_required_path_uncovered")
    );
    assert_eq!(c["missing_covered_path_exit_code"].as_integer(), Some(192));
    assert_eq!(
        c["uncovered_path_failure_kind"].as_str(),
        Some("mvp_pkgmgr_required_path_present_but_not_executed")
    );
    assert_eq!(c["uncovered_path_exit_code"].as_integer(), Some(193));
}

#[test]
fn live_pypi_remains_opt_in() {
    let c = &manifest()["live_pypi_opt_in_contract"];
    assert_eq!(c["case"].as_str(), Some("live_pypi_remains_opt_in"));
    for key in [
        "must_keep_live_pypi_opt_in",
        "must_not_run_live_pypi_in_default_gate",
        "forbid_live_pypi_being_required_for_default_gate_to_pass",
        "forbid_silently_enabling_live_pypi_in_default_gate",
        "must_distinguish_live_pypi_opt_in_from_offline_gate_breach",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["live_pypi_opt_in_field_name"].as_str(),
        Some("live_pypi_opt_in")
    );
    assert_eq!(
        c["default_live_pypi_opt_in_value"].as_bool(),
        Some(false)
    );
    assert_eq!(
        c["live_pypi_enabled_in_default_gate_failure_kind"].as_str(),
        Some("mvp_pkgmgr_live_pypi_enabled_in_default_gate")
    );
    assert_eq!(
        c["live_pypi_enabled_in_default_gate_exit_code"].as_integer(),
        Some(194)
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
            "covered_paths",
            "live_pypi_opt_in",
            "frozen_index_path",
            "lockfile_determinism_status",
            "venv_activation_status",
            "console_script_entrypoint_status",
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
            "default_package_manager_gate_is_offline",
            "resolver_conflict_yanked_version_transitive_wheel_lockfile_activation_console_script_paths_are_covered",
            "live_pypi_remains_opt_in",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_of_resolver_conflict_detection",
        "runtime_implementation_of_yanked_version_handling",
        "runtime_implementation_of_transitive_dependency_resolution",
        "runtime_implementation_of_pure_python_wheel_install",
        "runtime_implementation_of_lockfile_determinism",
        "runtime_implementation_of_venv_site_packages_activation",
        "runtime_implementation_of_console_script_entrypoint",
        "runtime_implementation_of_live_pypi_opt_in_path",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
