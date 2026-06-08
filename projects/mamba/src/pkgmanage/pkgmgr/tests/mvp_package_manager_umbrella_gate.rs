//! Inline migration of tests/mvp_package_manager_umbrella_gate_fixture_751.rs (#751).
//!
//! Locks the shape of the MVP package manager umbrella fixture pinned by
//! tests/governance/gates/package_manager/mvp_package_manager_umbrella_gate/manifest.toml.

#![cfg(test)]

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "tests/governance/gates/package_manager/mvp_package_manager_umbrella_gate/manifest.toml",
    )
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
        Some("mvp_package_manager_umbrella_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(751));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("package_manager"));
    assert_eq!(
        m["family"].as_str(),
        Some("mvp_package_manager_umbrella_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
    let children: Vec<_> = m["child_issues"]
        .as_array()
        .expect("child_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(children, vec![1262]);
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
fn surface_pins_index_resolver_install_lockfile_and_venv() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_pypi_compatible_index_client",
        "must_cover_dependency_resolver",
        "must_cover_wheel_install_path",
        "must_cover_deterministic_lockfile_is_source_of_truth",
        "must_cover_pep_405_venv_and_verb_workflow",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_pypi_compatible_index_client() {
    let c = &manifest()["r1_pypi_compatible_index_client_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("pypi_compatible_index_client_fetches_sdist_and_wheels")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_speak_pep_503_simple_index",
        "must_speak_pep_691_json_index",
        "must_fetch_wheel_artifact",
        "must_fetch_sdist_artifact",
        "forbid_relying_on_non_pep_index_protocol",
        "forbid_silently_skipping_index_protocol_negotiation",
        "must_distinguish_non_pep_protocol_from_skipped_negotiation",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let protocols: Vec<_> = c["allowed_index_protocol_values"]
        .as_array()
        .expect("allowed_index_protocol_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(protocols, vec!["pep_503_simple", "pep_691_json"]);
    let kinds: Vec<_> = c["allowed_artifact_kind_values"]
        .as_array()
        .expect("allowed_artifact_kind_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(kinds, vec!["wheel", "sdist"]);
    assert_eq!(
        c["non_pep_index_failure_kind"].as_str(),
        Some("mvp_package_manager_non_pep_index_protocol")
    );
    assert_eq!(c["non_pep_index_exit_code"].as_integer(), Some(377));
    assert_eq!(
        c["index_protocol_skipped_failure_kind"].as_str(),
        Some("mvp_package_manager_index_protocol_negotiation_skipped")
    );
    assert_eq!(
        c["index_protocol_skipped_exit_code"].as_integer(),
        Some(378)
    );
}

#[test]
fn r2_dependency_resolver() {
    let c = &manifest()["r2_dependency_resolver_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("dependency_resolver_produces_deterministic_resolution_graph")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_resolve_from_declared_specs",
        "must_produce_deterministic_resolution_graph",
        "must_report_unsatisfiable_with_reason",
        "forbid_silently_dropping_unsatisfiable_constraint",
        "forbid_nondeterministic_resolver_output",
        "must_distinguish_unsatisfiable_drop_from_nondeterminism",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let outcomes: Vec<_> = c["allowed_resolver_outcome_values"]
        .as_array()
        .expect("allowed_resolver_outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(outcomes, vec!["resolved", "unsatisfiable", "ambiguous"]);
    assert_eq!(
        c["unsatisfiable_silently_dropped_failure_kind"].as_str(),
        Some("mvp_package_manager_unsatisfiable_constraint_silently_dropped")
    );
    assert_eq!(
        c["unsatisfiable_silently_dropped_exit_code"].as_integer(),
        Some(379)
    );
    assert_eq!(
        c["nondeterministic_resolution_failure_kind"].as_str(),
        Some("mvp_package_manager_nondeterministic_resolution")
    );
    assert_eq!(
        c["nondeterministic_resolution_exit_code"].as_integer(),
        Some(380)
    );
}

#[test]
fn r3_wheel_install_path() {
    let c = &manifest()["r3_wheel_install_path_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("wheel_install_lands_pep_427_wheel_in_project_env_without_shell_activation")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_install_pep_427_wheel",
        "must_install_into_project_env",
        "must_make_installed_package_importable_without_shell_activation",
        "forbid_requiring_shell_activation_for_import",
        "forbid_installing_outside_project_env",
        "must_distinguish_shell_required_from_outside_env_from_non_pep_427",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let outcomes: Vec<_> = c["allowed_wheel_install_outcome_values"]
        .as_array()
        .expect("allowed_wheel_install_outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        outcomes,
        vec!["installed", "rejected_non_pep_427", "rejected_outside_env"]
    );
    assert_eq!(
        c["shell_activation_required_failure_kind"].as_str(),
        Some("mvp_package_manager_shell_activation_required")
    );
    assert_eq!(
        c["shell_activation_required_exit_code"].as_integer(),
        Some(381)
    );
    assert_eq!(
        c["install_outside_env_failure_kind"].as_str(),
        Some("mvp_package_manager_install_outside_project_env")
    );
    assert_eq!(c["install_outside_env_exit_code"].as_integer(), Some(382));
    assert_eq!(
        c["non_pep_427_wheel_failure_kind"].as_str(),
        Some("mvp_package_manager_non_pep_427_wheel")
    );
    assert_eq!(c["non_pep_427_wheel_exit_code"].as_integer(), Some(383));
}

#[test]
fn r4_deterministic_lockfile_is_source_of_truth() {
    let c = &manifest()["r4_deterministic_lockfile_is_source_of_truth_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("deterministic_lockfile_is_source_of_truth_for_reproducible_installs")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_emit_lockfile_on_first_resolve",
        "must_pin_resolved_versions_in_lockfile",
        "must_resolve_subsequent_installs_from_lockfile",
        "must_reject_unlocked_install_in_ci",
        "forbid_silently_regenerating_lockfile_in_ci",
        "forbid_silently_resolving_outside_lockfile",
        "must_distinguish_unlocked_from_silent_resolve_from_regenerate",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_project_lockfile_filename"].as_str(),
        Some("mamba.lock")
    );
    let statuses: Vec<_> = c["allowed_lockfile_status_values"]
        .as_array()
        .expect("allowed_lockfile_status_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(statuses, vec!["present", "missing", "stale"]);
    assert_eq!(
        c["unlocked_in_ci_failure_kind"].as_str(),
        Some("mvp_package_manager_unlocked_install_in_ci")
    );
    assert_eq!(c["unlocked_in_ci_exit_code"].as_integer(), Some(384));
    assert_eq!(
        c["silent_resolve_outside_lockfile_failure_kind"].as_str(),
        Some("mvp_package_manager_silent_resolve_outside_lockfile")
    );
    assert_eq!(
        c["silent_resolve_outside_lockfile_exit_code"].as_integer(),
        Some(385)
    );
    assert_eq!(
        c["silent_regenerate_lockfile_failure_kind"].as_str(),
        Some("mvp_package_manager_silent_regenerate_lockfile_in_ci")
    );
    assert_eq!(
        c["silent_regenerate_lockfile_exit_code"].as_integer(),
        Some(386)
    );
}

#[test]
fn r5_pep_405_venv_and_verb_workflow() {
    let c = &manifest()["r5_pep_405_venv_and_verb_workflow_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("pep_405_venv_and_init_add_remove_install_sync_run_verbs_are_wired")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_create_pep_405_compatible_venv",
        "must_wire_mamba_init_verb",
        "must_wire_mamba_add_verb",
        "must_wire_mamba_remove_verb",
        "must_wire_mamba_install_verb",
        "must_wire_mamba_sync_verb",
        "must_wire_mamba_run_verb",
        "forbid_silently_dropping_a_verb",
        "forbid_skipping_pep_405_venv_creation",
        "must_distinguish_verb_missing_from_non_pep_405_from_creation_skipped",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let layouts: Vec<_> = c["allowed_venv_layout_values"]
        .as_array()
        .expect("allowed_venv_layout_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(layouts, vec!["pep_405"]);
    let verbs: Vec<_> = c["required_verbs"]
        .as_array()
        .expect("required_verbs")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        verbs,
        vec!["init", "add", "remove", "install", "sync", "run"]
    );
    assert_eq!(
        c["verb_missing_failure_kind"].as_str(),
        Some("mvp_package_manager_verb_missing")
    );
    assert_eq!(c["verb_missing_exit_code"].as_integer(), Some(387));
    assert_eq!(
        c["non_pep_405_venv_failure_kind"].as_str(),
        Some("mvp_package_manager_non_pep_405_venv")
    );
    assert_eq!(c["non_pep_405_venv_exit_code"].as_integer(), Some(388));
    assert_eq!(
        c["venv_creation_skipped_failure_kind"].as_str(),
        Some("mvp_package_manager_venv_creation_skipped")
    );
    assert_eq!(c["venv_creation_skipped_exit_code"].as_integer(), Some(389));
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
            "index_protocol",
            "artifact_kind",
            "resolver_outcome",
            "wheel_install_outcome",
            "installed_importable_without_activation",
            "project_lockfile_relative_path",
            "lockfile_status",
            "venv_layout",
            "required_verbs",
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
            "pypi_compatible_index_client_fetches_sdist_and_wheels",
            "dependency_resolver_produces_deterministic_resolution_graph",
            "wheel_install_lands_pep_427_wheel_in_project_env_without_shell_activation",
            "deterministic_lockfile_is_source_of_truth_for_reproducible_installs",
            "pep_405_venv_and_init_add_remove_install_sync_run_verbs_are_wired",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_per_verb",
        "global_cache_implementation",
        "parallel_download_install_implementation",
        "wheel_build_from_sdist_implementation",
        "resolver_backtracking_heuristics",
        "mambalibs_binding_crate_dependency_surface",
        "c_extension_fast_paths",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
