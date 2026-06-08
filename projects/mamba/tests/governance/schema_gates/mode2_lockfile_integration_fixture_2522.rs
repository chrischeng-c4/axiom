// Locks the shape of the Mode 2 lockfile-integration fixture
// pinned by tests/mambalibs/fixtures/mode2_lockfile_integration/
// manifest.toml. Closes #2522. Umbrella: #2459.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/mambalibs/fixtures/mode2_lockfile_integration/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("mode2_lockfile_integration"));
    assert_eq!(m["issue"].as_integer(), Some(2522));
    assert_eq!(m["umbrella_issue"].as_integer(), Some(2459));
    assert_eq!(m["profile"].as_str(), Some("mambalibs"));
    assert_eq!(m["family"].as_str(), Some("mode2_lockfile_integration"));
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
fn surface_pins_mode_2_lockfile_integration_coverage() {
    let s = &manifest()["surface"];
    assert_eq!(s["mode"].as_str(), Some("mode_2"));
    for key in [
        "must_cover_canonical_lockfile_filename",
        "must_cover_lockfile_provenance_decision",
        "must_cover_mode2_dependency_pinning",
        "must_cover_byte_identical_binary_across_machines",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn dependencies_pin_2520_and_pkgmgr_phase_1_4() {
    let d = &manifest()["dependencies"];
    assert_eq!(d["depends_on_issue_2520_fetch_step"].as_bool(), Some(true));
    assert_eq!(
        d["depends_on_pkgmgr_phase_1_4_lockfile"].as_bool(),
        Some(true)
    );
}

#[test]
fn canonical_lockfile_filename_and_provenance_are_pinned() {
    let c = &manifest()["canonical_lockfile_decision_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("canonical_lockfile_filename_and_provenance_are_pinned")
    );
    assert_eq!(
        c["canonical_lockfile_filename"].as_str(),
        Some("mamba.lock")
    );
    assert_eq!(
        c["canonical_lockfile_relative_path"].as_str(),
        Some("mamba.lock")
    );
    assert_eq!(c["canonical_lockfile_owner"].as_str(), Some("pkgmgr"));
    for key in [
        "must_share_lockfile_with_pkgmgr",
        "must_emit_single_lockfile_per_project",
        "forbid_independent_mode2_lockfile",
        "forbid_multiple_lockfiles_per_project",
        "forbid_mode2_lockfile_outside_project_root",
        "must_distinguish_filename_mismatch_from_multiple_lockfiles",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["canonical_lockfile_filename_field_name"].as_str(),
        Some("lockfile_filename")
    );
    assert_eq!(
        c["canonical_lockfile_owner_field_name"].as_str(),
        Some("lockfile_owner")
    );
    assert_eq!(
        c["lockfile_filename_mismatch_failure_kind"].as_str(),
        Some("mode2_lockfile_filename_mismatch")
    );
    assert_eq!(
        c["lockfile_filename_mismatch_exit_code"].as_integer(),
        Some(223)
    );
    assert_eq!(
        c["multiple_lockfiles_failure_kind"].as_str(),
        Some("mode2_multiple_lockfiles_per_project")
    );
    assert_eq!(c["multiple_lockfiles_exit_code"].as_integer(), Some(224));
}

#[test]
fn same_mamba_toml_and_lockfile_build_byte_identical_binary_across_machines() {
    let c = &manifest()["byte_identical_binary_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("same_mamba_toml_and_lockfile_build_byte_identical_binary_across_machines")
    );
    for key in [
        "must_be_byte_identical_across_machines",
        "must_be_byte_identical_across_runs",
        "must_be_path_independent",
        "must_be_user_home_independent",
        "must_be_wallclock_independent",
        "forbid_wallclock_in_binary",
        "forbid_absolute_path_in_binary",
        "forbid_user_home_path_in_binary",
        "forbid_machine_dependent_metadata_in_binary",
        "must_distinguish_cross_machine_inequivalence_from_cross_run_inequivalence",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["binary_artifact_field_name"].as_str(),
        Some("binary_artifact_digest")
    );
    assert_eq!(
        c["binary_artifact_digest_algorithm"].as_str(),
        Some("sha256")
    );
    assert_eq!(
        c["byte_inequivalence_across_machines_failure_kind"].as_str(),
        Some("mode2_binary_not_byte_identical_across_machines")
    );
    assert_eq!(
        c["byte_inequivalence_across_machines_exit_code"].as_integer(),
        Some(225)
    );
    assert_eq!(
        c["byte_inequivalence_across_runs_failure_kind"].as_str(),
        Some("mode2_binary_not_byte_identical_across_runs")
    );
    assert_eq!(
        c["byte_inequivalence_across_runs_exit_code"].as_integer(),
        Some(226)
    );
}

#[test]
fn mode2_lockfile_entry_records_full_identity_for_binding_crate() {
    let c = &manifest()["mode2_lockfile_entry_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("mode2_lockfile_entry_records_full_identity_for_binding_crate")
    );
    for key in [
        "must_record_dependency_name",
        "must_record_dependency_version",
        "must_record_resolved_artifact_identity",
        "must_record_resolved_artifact_kind",
        "must_record_binding_crate_name",
        "must_record_binding_crate_version",
        "must_record_rust_target_triple_when_relevant",
        "forbid_omitting_required_lockfile_entry_fields",
        "forbid_freeform_lockfile_entry",
        "must_distinguish_missing_field_from_freeform_entry",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_lockfile_entry_fields"]
        .as_array()
        .expect("required_lockfile_entry_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec![
            "dependency_name",
            "dependency_version",
            "resolved_artifact_identity",
            "resolved_artifact_kind",
            "binding_crate_name",
            "binding_crate_version",
        ]
    );
    assert_eq!(
        c["mode2_lockfile_entry_field_name"].as_str(),
        Some("mode2_lockfile_entry")
    );
    assert_eq!(
        c["missing_lockfile_entry_field_failure_kind"].as_str(),
        Some("mode2_lockfile_entry_missing_required_field")
    );
    assert_eq!(
        c["missing_lockfile_entry_field_exit_code"].as_integer(),
        Some(227)
    );
    assert_eq!(
        c["freeform_lockfile_entry_failure_kind"].as_str(),
        Some("mode2_lockfile_entry_freeform")
    );
    assert_eq!(
        c["freeform_lockfile_entry_exit_code"].as_integer(),
        Some(228)
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
            "lockfile_filename",
            "lockfile_owner",
            "binary_artifact_digest",
            "mode2_lockfile_entry",
            "dependency_name",
            "dependency_version",
            "resolved_artifact_identity",
            "resolved_artifact_kind",
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
            "canonical_lockfile_filename_and_provenance_are_pinned",
            "same_mamba_toml_and_lockfile_build_byte_identical_binary_across_machines",
            "mode2_lockfile_entry_records_full_identity_for_binding_crate",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "full_pkgmgr_lockfile_policy",
        "runtime_implementation_of_lockfile_emission",
        "runtime_implementation_of_binary_artifact_digest",
        "runtime_implementation_of_resolved_artifact_identity",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
