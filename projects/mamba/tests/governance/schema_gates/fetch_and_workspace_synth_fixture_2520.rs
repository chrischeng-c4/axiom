// Locks the shape of the Mode 2 fetch + cargo workspace synth
// fixture pinned by tests/mambalibs/fixtures/
// fetch_and_workspace_synth/manifest.toml. Closes #2520.
// Umbrella: #2459. Depends: #2519.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/mambalibs/fixtures/fetch_and_workspace_synth/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("fetch_and_workspace_synth"));
    assert_eq!(m["issue"].as_integer(), Some(2520));
    assert_eq!(m["umbrella_issue"].as_integer(), Some(2459));
    assert_eq!(m["depends_on_issue"].as_integer(), Some(2519));
    assert_eq!(m["profile"].as_str(), Some("mambalibs"));
    assert_eq!(m["family"].as_str(), Some("fetch_and_workspace_synth"));
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
fn surface_pins_mode_2_fetch_synth_relink_and_import() {
    let s = &manifest()["surface"];
    assert_eq!(s["mode"].as_str(), Some("mode_2"));
    for key in [
        "must_cover_fetch_from_mamba_toml_dependencies",
        "must_cover_cargo_workspace_synthesis",
        "must_cover_relink_mamba_binary",
        "must_cover_external_crate_importable_after_build",
        "must_use_offline_frozen_source_index",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn fetch_sources_cover_git_and_registry() {
    let c = &manifest()["fetch_sources_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("fetch_sources_cover_git_and_registry")
    );
    for key in [
        "must_cover_git_source",
        "must_cover_registry_source",
        "must_cover_offline_frozen_source_index",
        "forbid_resolving_to_live_git_remote_in_default_gate",
        "forbid_resolving_to_live_registry_remote_in_default_gate",
        "forbid_unknown_fetch_source_kind",
        "must_distinguish_unknown_source_kind_from_network_access_in_default_gate",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["fetch_source_field_name"].as_str(), Some("fetch_source"));
    let kinds: Vec<_> = c["allowed_fetch_source_kinds"]
        .as_array()
        .expect("allowed_fetch_source_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(kinds, vec!["git", "registry", "frozen_local_source_index"]);
    assert_eq!(
        c["unknown_fetch_source_failure_kind"].as_str(),
        Some("mambalibs_fetch_source_unknown_kind")
    );
    assert_eq!(c["unknown_fetch_source_exit_code"].as_integer(), Some(234));
    assert_eq!(
        c["network_access_in_default_gate_failure_kind"].as_str(),
        Some("mambalibs_fetch_network_access_in_default_gate")
    );
    assert_eq!(
        c["network_access_in_default_gate_exit_code"].as_integer(),
        Some(235)
    );
}

#[test]
fn cargo_workspace_synthesis_includes_host_and_each_binding_crate() {
    let c = &manifest()["cargo_workspace_synthesis_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("cargo_workspace_synthesis_includes_host_and_each_binding_crate")
    );
    for key in [
        "must_synthesize_cargo_workspace",
        "must_include_host_crate_in_workspace",
        "must_include_each_binding_crate_in_workspace",
        "must_emit_synthesized_workspace_manifest",
        "forbid_synthesizing_workspace_outside_project",
        "forbid_overwriting_user_provided_cargo_toml",
        "forbid_partial_workspace_synthesis",
        "must_distinguish_missing_workspace_from_workspace_outside_project",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["synthesized_workspace_relative_path"].as_str(),
        Some("target/mamba_build/workspace")
    );
    assert_eq!(
        c["workspace_manifest_relative_path"].as_str(),
        Some("target/mamba_build/workspace/Cargo.toml")
    );
    assert_eq!(
        c["synthesized_workspace_relative_path_field_name"].as_str(),
        Some("synthesized_workspace_path")
    );
    assert_eq!(
        c["workspace_manifest_relative_path_field_name"].as_str(),
        Some("workspace_manifest_path")
    );
    assert_eq!(
        c["workspace_missing_failure_kind"].as_str(),
        Some("mambalibs_synth_workspace_missing")
    );
    assert_eq!(c["workspace_missing_exit_code"].as_integer(), Some(236));
    assert_eq!(
        c["workspace_outside_project_failure_kind"].as_str(),
        Some("mambalibs_synth_workspace_outside_project")
    );
    assert_eq!(
        c["workspace_outside_project_exit_code"].as_integer(),
        Some(237)
    );
}

#[test]
fn example_external_crate_importable_from_mamba_script_after_mamba_build() {
    let c = &manifest()["importable_after_build_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("example_external_crate_importable_from_mamba_script_after_mamba_build")
    );
    for key in [
        "must_relink_mamba_binary_after_synth",
        "must_register_binding_module_in_mambalibs_namespace",
        "must_resolve_from_mambalibs_import_after_build",
        "forbid_silent_skip_of_relink_phase",
        "forbid_silent_skip_of_module_registration",
        "forbid_importable_check_being_skipped",
        "must_distinguish_relink_skip_from_module_registration_skip",
        "must_distinguish_module_registration_skip_from_importable_check_skip",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["example_external_crate_name"].as_str(),
        Some("mamba_external_demo_dep")
    );
    assert_eq!(c["example_external_crate_version"].as_str(), Some("0.1.0"));
    assert_eq!(
        c["example_external_crate_python_import"].as_str(),
        Some("from mambalibs import mamba_external_demo_dep")
    );
    assert_eq!(
        c["example_external_crate_python_import_field_name"].as_str(),
        Some("example_python_import_statement")
    );
    assert_eq!(
        c["relink_phase_skipped_failure_kind"].as_str(),
        Some("mambalibs_relink_phase_skipped")
    );
    assert_eq!(c["relink_phase_skipped_exit_code"].as_integer(), Some(238));
    assert_eq!(
        c["module_registration_skipped_failure_kind"].as_str(),
        Some("mambalibs_module_registration_skipped")
    );
    assert_eq!(
        c["module_registration_skipped_exit_code"].as_integer(),
        Some(239)
    );
    assert_eq!(
        c["importable_check_skipped_failure_kind"].as_str(),
        Some("mambalibs_importable_check_skipped")
    );
    assert_eq!(
        c["importable_check_skipped_exit_code"].as_integer(),
        Some(240)
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
            "fetch_source",
            "synthesized_workspace_path",
            "workspace_manifest_path",
            "example_python_import_statement",
            "binding_crate_name",
            "binding_crate_version",
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
            "fetch_sources_cover_git_and_registry",
            "cargo_workspace_synthesis_includes_host_and_each_binding_crate",
            "example_external_crate_importable_from_mamba_script_after_mamba_build",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "mamba_toml_dependencies_schema_definition",
        "binary_equivalence_reproducibility_decision",
        "runtime_implementation_of_fetch",
        "runtime_implementation_of_workspace_synth",
        "runtime_implementation_of_relink_phase",
        "runtime_implementation_of_mambalibs_module_registration",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
