// Locks the shape of the MVP mambalibs umbrella fixture pinned by
// tests/governance/gates/mvp/mvp_mambalibs_umbrella_gate/manifest.toml.
// Closes #2459.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/mvp_mambalibs_umbrella_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("mvp_mambalibs_umbrella_gate"));
    assert_eq!(m["issue"].as_integer(), Some(2459));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(m["family"].as_str(), Some("mvp_mambalibs_umbrella_gate"));
    assert_eq!(m["network"].as_str(), Some("offline"));
    let children: Vec<_> = m["child_issues"]
        .as_array()
        .expect("child_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(children, vec![2519, 2520, 2521, 2522]);
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
fn surface_pins_schema_fetch_abi_lockfile_and_import() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_mamba_toml_declares_external_binding_crate_deps",
        "must_cover_mamba_build_fetches_and_synthesizes_workspace",
        "must_cover_cclab_mamba_registry_abi_semver_gate",
        "must_cover_mode_2_lockfile_is_source_of_truth",
        "must_cover_from_mambalibs_import_resolves_to_rust_crate_module",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_mamba_toml_external_binding_dep_schema() {
    let c = &manifest()["r1_mamba_toml_external_binding_dep_schema_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("mamba_toml_declares_external_binding_crate_dependency_surface")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_define_external_binding_crate_dependency_section",
        "must_pin_dependency_kind_field",
        "must_pin_dependency_source_field",
        "must_pin_dependency_version_field",
        "forbid_using_cargo_toml_for_mambalibs_deps",
        "forbid_implicit_dependency_declaration",
        "must_distinguish_section_missing_from_implicit_dependency",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_external_binding_dep_section"].as_str(),
        Some("[mambalibs.external]")
    );
    let kinds: Vec<_> = c["allowed_dependency_kind_values"]
        .as_array()
        .expect("allowed_dependency_kind_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(kinds, vec!["git", "registry"]);
    assert_eq!(
        c["expected_mamba_toml_filename"].as_str(),
        Some("mamba.toml")
    );
    assert_eq!(
        c["external_binding_section_missing_failure_kind"].as_str(),
        Some("mvp_mambalibs_external_binding_section_missing")
    );
    assert_eq!(
        c["external_binding_section_missing_exit_code"].as_integer(),
        Some(364)
    );
    assert_eq!(
        c["implicit_dependency_failure_kind"].as_str(),
        Some("mvp_mambalibs_dependency_declared_implicitly")
    );
    assert_eq!(c["implicit_dependency_exit_code"].as_integer(), Some(365));
}

#[test]
fn r2_mamba_build_fetch_and_workspace_synth() {
    let c = &manifest()["r2_mamba_build_fetch_and_workspace_synth_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("mamba_build_fetches_external_binding_crates_and_synthesizes_workspace")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_fetch_external_binding_crate_during_mamba_build",
        "must_support_git_source_fetch",
        "must_support_registry_source_fetch",
        "must_synthesize_cargo_workspace_linking_binding_crates",
        "must_link_synthesized_crates_into_mamba_binary",
        "forbid_skipping_fetch_step_when_external_binding_declared",
        "forbid_linking_without_workspace_synth_step",
        "must_distinguish_fetch_skipped_from_synth_skipped",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let steps: Vec<_> = c["allowed_fetch_step_values"]
        .as_array()
        .expect("allowed_fetch_step_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(steps, vec!["git", "registry"]);
    assert_eq!(
        c["fetch_step_skipped_failure_kind"].as_str(),
        Some("mvp_mambalibs_fetch_step_skipped")
    );
    assert_eq!(c["fetch_step_skipped_exit_code"].as_integer(), Some(366));
    assert_eq!(
        c["workspace_synth_skipped_failure_kind"].as_str(),
        Some("mvp_mambalibs_workspace_synth_skipped")
    );
    assert_eq!(
        c["workspace_synth_skipped_exit_code"].as_integer(),
        Some(367)
    );
}

#[test]
fn r3_cclab_mamba_registry_abi_semver_gate() {
    let c = &manifest()["r3_cclab_mamba_registry_abi_semver_gate_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("cclab_mamba_registry_refuses_incompatible_binding_crate_versions")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_run_abi_semver_check_on_each_binding_crate",
        "must_refuse_incompatible_version",
        "must_report_incompatible_version_with_reason",
        "forbid_silently_loading_incompatible_binding_crate",
        "forbid_treating_missing_abi_metadata_as_compatible",
        "must_distinguish_violation_from_missing_metadata_from_silent_load",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let kinds: Vec<_> = c["allowed_abi_semver_check_kind_values"]
        .as_array()
        .expect("allowed_abi_semver_check_kind_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        kinds,
        vec!["compatible", "incompatible", "missing_metadata"]
    );
    assert_eq!(
        c["abi_semver_violation_failure_kind"].as_str(),
        Some("mvp_mambalibs_abi_semver_violation")
    );
    assert_eq!(c["abi_semver_violation_exit_code"].as_integer(), Some(368));
    assert_eq!(
        c["missing_abi_metadata_failure_kind"].as_str(),
        Some("mvp_mambalibs_abi_metadata_missing")
    );
    assert_eq!(c["missing_abi_metadata_exit_code"].as_integer(), Some(369));
    assert_eq!(
        c["silent_load_incompatible_failure_kind"].as_str(),
        Some("mvp_mambalibs_silent_load_incompatible_crate")
    );
    assert_eq!(
        c["silent_load_incompatible_exit_code"].as_integer(),
        Some(370)
    );
}

#[test]
fn r4_mode_2_lockfile_is_source_of_truth() {
    let c = &manifest()["r4_mode_2_lockfile_is_source_of_truth_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("mode_2_lockfile_is_source_of_truth_for_reproducible_builds")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_emit_mode_2_lockfile_after_first_build",
        "must_lock_external_binding_crate_versions_in_lockfile",
        "must_resolve_subsequent_builds_from_lockfile",
        "must_reject_unlocked_external_binding_crate_in_ci",
        "forbid_silently_resolving_outside_lockfile",
        "forbid_silently_regenerating_lockfile_in_ci",
        "must_distinguish_unlocked_from_silent_resolve_from_regenerate",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_mode_2_lockfile_filename"].as_str(),
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
        Some("mvp_mambalibs_unlocked_external_crate_in_ci")
    );
    assert_eq!(c["unlocked_in_ci_exit_code"].as_integer(), Some(371));
    assert_eq!(
        c["silent_resolve_outside_lockfile_failure_kind"].as_str(),
        Some("mvp_mambalibs_silent_resolve_outside_lockfile")
    );
    assert_eq!(
        c["silent_resolve_outside_lockfile_exit_code"].as_integer(),
        Some(372)
    );
    assert_eq!(
        c["silent_regenerate_lockfile_failure_kind"].as_str(),
        Some("mvp_mambalibs_silent_regenerate_lockfile_in_ci")
    );
    assert_eq!(
        c["silent_regenerate_lockfile_exit_code"].as_integer(),
        Some(373)
    );
}

#[test]
fn r5_from_mambalibs_import_resolves_to_rust_crate_module() {
    let c = &manifest()["r5_from_mambalibs_import_resolves_to_rust_crate_module_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("from_mambalibs_import_xxxx_resolves_to_cclab_rust_crate_module")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_make_from_mambalibs_import_resolve_to_rust_crate_module",
        "must_expose_rust_crate_exported_surface_through_python_module",
        "forbid_resolving_mambalibs_import_to_stub_module",
        "forbid_substituting_mambalibs_with_cpython_shim",
        "must_distinguish_stub_from_cpython_shim_from_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["canonical_import_statement"].as_str(),
        Some("from mambalibs import XXXX")
    );
    let allowed: Vec<_> = c["allowed_resolved_module_kinds"]
        .as_array()
        .expect("allowed_resolved_module_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(allowed, vec!["rust_crate_module"]);
    let disallowed: Vec<_> = c["disallowed_resolved_module_kinds"]
        .as_array()
        .expect("disallowed_resolved_module_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        disallowed,
        vec!["stub_module", "cpython_shim_module", "missing_module"]
    );
    assert_eq!(
        c["resolved_as_stub_failure_kind"].as_str(),
        Some("mvp_mambalibs_import_resolved_as_stub_module")
    );
    assert_eq!(c["resolved_as_stub_exit_code"].as_integer(), Some(374));
    assert_eq!(
        c["resolved_as_cpython_shim_failure_kind"].as_str(),
        Some("mvp_mambalibs_import_resolved_as_cpython_shim")
    );
    assert_eq!(
        c["resolved_as_cpython_shim_exit_code"].as_integer(),
        Some(375)
    );
    assert_eq!(
        c["import_missing_failure_kind"].as_str(),
        Some("mvp_mambalibs_import_module_missing")
    );
    assert_eq!(c["import_missing_exit_code"].as_integer(), Some(376));
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
            "external_binding_dep_section",
            "dependency_kind",
            "mamba_toml_relative_path",
            "fetch_step",
            "workspace_synth_step",
            "linked_into_mamba_binary",
            "abi_semver_check_kind",
            "mode_2_lockfile_relative_path",
            "lockfile_status",
            "canonical_import_statement",
            "resolved_module_kind",
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
            "mamba_toml_declares_external_binding_crate_dependency_surface",
            "mamba_build_fetches_external_binding_crates_and_synthesizes_workspace",
            "cclab_mamba_registry_refuses_incompatible_binding_crate_versions",
            "mode_2_lockfile_is_source_of_truth_for_reproducible_builds",
            "from_mambalibs_import_xxxx_resolves_to_cclab_rust_crate_module",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_per_child_issue",
        "mode_1_in_tree_cclab_crates_linking",
        "binary_packaging_and_distribution",
        "uv_like_package_workflow",
        "c_extension_fast_paths",
        "runtime_implementation_of_mamba_build_fetcher",
        "runtime_implementation_of_cclab_mamba_registry_server",
        "runtime_implementation_of_lockfile_resolver",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
