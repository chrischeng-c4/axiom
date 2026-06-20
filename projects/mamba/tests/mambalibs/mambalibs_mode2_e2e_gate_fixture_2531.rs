// Locks the shape of the mambalibs Mode 2 end-to-end build and
// import gate pinned by tests/mambalibs/fixtures/
// mode2_end_to_end_build_and_import/manifest.toml. Closes #2531.
// Parent: #2526.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/mambalibs/fixtures/mode2_end_to_end_build_and_import/manifest.toml")
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
        Some("mambalibs_mode2_end_to_end_build_and_import")
    );
    assert_eq!(m["issue"].as_integer(), Some(2531));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mambalibs"));
    assert_eq!(
        m["family"].as_str(),
        Some("mambalibs_mode2_end_to_end_build_and_import")
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
fn surface_pins_mode_2_and_all_lifecycle_coverage() {
    let s = &manifest()["surface"];
    assert_eq!(s["mode"].as_str(), Some("mode_2"));
    for key in [
        "must_cover_local_binding_crate_declaration",
        "must_cover_local_binding_crate_build",
        "must_cover_local_binding_crate_lock",
        "must_cover_local_binding_crate_import",
        "must_cover_positive_from_mambalibs_import",
        "must_cover_diagnostic_from_mambalibs_import",
        "must_cover_build_summary_toolchain_metadata",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn atomic_queue_pins_nine_children() {
    let q = &manifest()["atomic_queue"];
    for key in [
        "issue_2574_add_mode_2_lockfile_assertion",
        "issue_2575_add_mamba_toml_mode_2_dependency_fixture",
        "issue_2576_add_from_mambalibs_import_fixture",
        "issue_2577_add_local_mode_2_binding_crate_fixture",
        "issue_2578_add_mamba_build_e2e_harness_for_local_binding_crate",
        "issue_2579_add_mambalibs_missing_dependency_diagnostic_fixture",
        "issue_2580_add_multiple_mambalibs_import_fixture",
        "issue_2581_record_rust_toolchain_identity_in_mambalibs_build_summary",
        "issue_2582_add_mambalibs_abi_mismatch_failure_fixture",
    ] {
        assert_eq!(q[key].as_bool(), Some(true), "atomic_queue.{key}");
    }
}

#[test]
fn local_binding_crate_can_be_declared_built_locked_and_imported() {
    let c = &manifest()["local_binding_crate_lifecycle_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("local_binding_crate_can_be_declared_built_locked_and_imported")
    );
    for key in [
        "must_cover_declared_phase",
        "must_cover_built_phase",
        "must_cover_locked_phase",
        "must_cover_imported_phase",
        "forbid_silently_skipping_any_lifecycle_phase",
        "forbid_collapsed_or_implicit_lifecycle_phase",
        "must_distinguish_each_phase_failure",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let phases: Vec<_> = c["required_lifecycle_phases"]
        .as_array()
        .expect("required_lifecycle_phases")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(phases, vec!["declared", "built", "locked", "imported"]);
    assert_eq!(
        c["lifecycle_phase_field_name"].as_str(),
        Some("lifecycle_phase")
    );
    assert_eq!(
        c["declared_phase_failure_kind"].as_str(),
        Some("mambalibs_mode2_declared_phase_failed")
    );
    assert_eq!(c["declared_phase_exit_code"].as_integer(), Some(195));
    assert_eq!(
        c["built_phase_failure_kind"].as_str(),
        Some("mambalibs_mode2_built_phase_failed")
    );
    assert_eq!(c["built_phase_exit_code"].as_integer(), Some(196));
    assert_eq!(
        c["locked_phase_failure_kind"].as_str(),
        Some("mambalibs_mode2_locked_phase_failed")
    );
    assert_eq!(c["locked_phase_exit_code"].as_integer(), Some(197));
    assert_eq!(
        c["imported_phase_failure_kind"].as_str(),
        Some("mambalibs_mode2_imported_phase_failed")
    );
    assert_eq!(c["imported_phase_exit_code"].as_integer(), Some(198));
}

#[test]
fn from_mambalibs_import_has_positive_and_diagnostic_fixtures() {
    let c = &manifest()["positive_and_diagnostic_import_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("from_mambalibs_import_has_positive_and_diagnostic_fixtures")
    );
    for key in [
        "must_cover_positive_import_fixture",
        "must_cover_diagnostic_missing_dependency_fixture",
        "must_cover_diagnostic_abi_mismatch_fixture",
        "forbid_silently_passing_when_dependency_missing",
        "forbid_silently_passing_when_abi_mismatched",
        "must_distinguish_missing_dependency_from_abi_mismatch",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["positive_import_fixture_name"].as_str(),
        Some("mambalibs_positive_import")
    );
    assert_eq!(
        c["missing_dependency_fixture_name"].as_str(),
        Some("mambalibs_missing_dependency_diagnostic")
    );
    assert_eq!(
        c["abi_mismatch_fixture_name"].as_str(),
        Some("mambalibs_abi_mismatch_failure")
    );
    assert_eq!(
        c["positive_import_failure_kind"].as_str(),
        Some("mambalibs_positive_import_failed")
    );
    assert_eq!(c["positive_import_exit_code"].as_integer(), Some(199));
    assert_eq!(
        c["missing_dependency_failure_kind"].as_str(),
        Some("mambalibs_missing_dependency")
    );
    assert_eq!(c["missing_dependency_exit_code"].as_integer(), Some(200));
    assert_eq!(
        c["abi_mismatch_failure_kind"].as_str(),
        Some("mambalibs_abi_mismatch")
    );
    assert_eq!(c["abi_mismatch_exit_code"].as_integer(), Some(201));
}

#[test]
fn build_summaries_expose_enough_toolchain_metadata_to_debug_failures() {
    let c = &manifest()["build_summary_toolchain_metadata_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("build_summaries_expose_enough_toolchain_metadata_to_debug_failures")
    );
    for key in [
        "must_record_rust_toolchain_version",
        "must_record_rust_target_triple",
        "must_record_rust_profile",
        "must_record_mamba_version",
        "must_record_python_version",
        "must_record_binding_crate_name",
        "must_record_binding_crate_version",
        "forbid_omitting_required_toolchain_fields",
        "forbid_collapsing_toolchain_metadata_into_freeform_text",
        "must_distinguish_missing_metadata_from_freeform_metadata",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_toolchain_metadata_fields"]
        .as_array()
        .expect("required_toolchain_metadata_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "rust_toolchain_version",
            "rust_target_triple",
            "rust_profile",
            "mamba_version",
            "python_version",
            "binding_crate_name",
            "binding_crate_version",
        ]
    );
    assert_eq!(
        c["toolchain_metadata_field_name"].as_str(),
        Some("toolchain_metadata")
    );
    assert_eq!(
        c["missing_toolchain_metadata_failure_kind"].as_str(),
        Some("mambalibs_build_summary_toolchain_metadata_missing")
    );
    assert_eq!(
        c["missing_toolchain_metadata_exit_code"].as_integer(),
        Some(202)
    );
    assert_eq!(
        c["toolchain_metadata_freeform_failure_kind"].as_str(),
        Some("mambalibs_build_summary_toolchain_metadata_freeform")
    );
    assert_eq!(
        c["toolchain_metadata_freeform_exit_code"].as_integer(),
        Some(203)
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
            "lifecycle_phase",
            "positive_import_fixture",
            "missing_dependency_fixture",
            "abi_mismatch_fixture",
            "toolchain_metadata",
            "rust_toolchain_version",
            "rust_target_triple",
            "rust_profile",
            "mamba_version",
            "python_version",
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
            "local_binding_crate_can_be_declared_built_locked_and_imported",
            "from_mambalibs_import_has_positive_and_diagnostic_fixtures",
            "build_summaries_expose_enough_toolchain_metadata_to_debug_failures",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "runtime_implementation_of_declare_build_lock_import",
        "runtime_implementation_of_positive_import_fixture",
        "runtime_implementation_of_missing_dependency_diagnostic_fixture",
        "runtime_implementation_of_abi_mismatch_failure_fixture",
        "runtime_implementation_of_toolchain_metadata_emission",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
