//! Schema gate for the Mode 2 lockfile assertion fixture — closes
//! #2574.
//!
//! Acceptance (issue #2574):
//!
//!   1. Running the build twice produces equivalent lockfile
//!      content.
//!      `[double_build_equivalence_contract]` pins
//!      must_be_byte_equivalent_across_runs +
//!      must_be_path_independent + must_be_timestamp_independent +
//!      must_be_user_home_independent +
//!      must_be_machine_independent +
//!      forbid_wallclock_in_lockfile +
//!      forbid_absolute_path_in_lockfile +
//!      forbid_user_home_path_in_lockfile +
//!      forbid_temp_directory_path_in_lockfile + distinct exit
//!      codes 180 (byte inequivalence) / 181 (machine dependence) +
//!      must_distinguish_byte_inequivalence_from_machine_dependence.
//!   2. Removing the dependency changes the lockfile assertion and
//!      fails.
//!      `[dependency_removal_failure_contract]` pins
//!      must_fail_when_mode_2_dependency_removed +
//!      must_emit_dependency_removal_diff +
//!      forbid_silently_passing_with_missing_dependency +
//!      forbid_lockfile_assertion_being_ignored_when_dep_removed +
//!      distinct exit codes 182 (removal) / 183 (version drift) +
//!      must_distinguish_dependency_removal_from_version_drift.
//!   3. Lockfile format is documented in the fixture README or test
//!      comment.
//!      `[lockfile_format_documentation_contract]` pins
//!      must_document_lockfile_format + must_document_required_fields
//!      + must_document_path_normalization_rules +
//!      must_document_machine_independence_rules +
//!      allowed_documentation_locations=[readme, test_comment] +
//!      required_documented_fields (dependency name/version,
//!      resolved artifact identity/kind) +
//!      forbid_undocumented_lockfile_format + exit 184.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("mode2_lockfile_assertion")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path).unwrap();
    raw.parse().unwrap()
}

#[test]
fn header_is_well_formed() {
    let doc = load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_mode2_lockfile_assertion")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2574));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2531)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("mambalibs")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("mambalibs_mode2_lockfile_assertion")
    );
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = load_toml(&manifest_path());
    let i = doc.get("isolation").and_then(|v| v.as_table()).unwrap();
    for f in &[
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(i.get(*f).and_then(|v| v.as_bool()), Some(true));
    }
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let doc = load_toml(&manifest_path());
    let p = doc
        .get("python_target")
        .and_then(|v| v.as_table())
        .expect("[python_target] missing");
    assert_eq!(p.get("python_major").and_then(|v| v.as_integer()), Some(3));
    assert_eq!(p.get("python_minor").and_then(|v| v.as_integer()), Some(12));
    assert_eq!(
        p.get("must_be_python_3_12").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn surface_pins_mode_2_lockfile_coverage() {
    let doc = load_toml(&manifest_path());
    let s = doc
        .get("surface")
        .and_then(|v| v.as_table())
        .expect("[surface] missing");
    assert_eq!(s.get("mode").and_then(|v| v.as_str()), Some("mode_2"));
    for f in &[
        "must_cover_lockfile_emission",
        "must_cover_lockfile_assertion_in_test",
        "must_cover_dependency_removal_diff",
        "must_cover_double_build_equivalence",
        "must_document_lockfile_format_in_readme_or_comment",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

#[test]
fn fixture_project_definition_pins_canonical_project() {
    let doc = load_toml(&manifest_path());
    let p = doc
        .get("fixture_project_definition")
        .and_then(|v| v.as_table())
        .expect("[fixture_project_definition] missing");
    assert_eq!(
        p.get("project_name").and_then(|v| v.as_str()),
        Some("mamba2574")
    );
    assert_eq!(
        p.get("manifest_relative_path").and_then(|v| v.as_str()),
        Some("mamba.toml")
    );
    assert_eq!(
        p.get("lockfile_relative_path").and_then(|v| v.as_str()),
        Some("mamba.lock")
    );
    assert_eq!(
        p.get("readme_relative_path").and_then(|v| v.as_str()),
        Some("README.md")
    );
    assert_eq!(
        p.get("mode_2_dependency_name").and_then(|v| v.as_str()),
        Some("mamba_demo_dep")
    );
    assert_eq!(
        p.get("mode_2_dependency_version").and_then(|v| v.as_str()),
        Some("0.1.0")
    );
}

// Acceptance: "Running the build twice produces equivalent lockfile
// content."
#[test]
fn running_the_build_twice_produces_equivalent_lockfile_content() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("double_build_equivalence_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[double_build_equivalence_contract] missing — acceptance: \
         \"Running the build twice produces equivalent lockfile content.\"",
        );
    for k in &[
        "must_be_byte_equivalent_across_runs",
        "must_be_path_independent",
        "must_be_timestamp_independent",
        "must_be_user_home_independent",
        "must_be_machine_independent",
        "forbid_wallclock_in_lockfile",
        "forbid_absolute_path_in_lockfile",
        "forbid_user_home_path_in_lockfile",
        "forbid_temp_directory_path_in_lockfile",
        "must_distinguish_byte_inequivalence_from_machine_dependence",
    ] {
        assert_eq!(
            c.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let byte = c
        .get("non_equivalent_lockfile_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let machine = c
        .get("machine_dependent_lockfile_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(byte, 180);
    assert_eq!(machine, 181);
    assert_ne!(
        byte, machine,
        "byte-inequivalence and machine-dependence exit codes must differ"
    );
    assert_eq!(
        c.get("non_equivalent_lockfile_failure_kind")
            .and_then(|v| v.as_str()),
        Some("mode2_lockfile_not_byte_equivalent_across_runs"),
    );
    assert_eq!(
        c.get("machine_dependent_lockfile_failure_kind")
            .and_then(|v| v.as_str()),
        Some("mode2_lockfile_machine_dependent_content"),
    );
}

// Acceptance: "Removing the dependency changes the lockfile
// assertion and fails."
#[test]
fn removing_the_dependency_changes_the_lockfile_assertion_and_fails() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("dependency_removal_failure_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[dependency_removal_failure_contract] missing — acceptance: \
         \"Removing the dependency changes the lockfile assertion and fails.\"",
        );
    for k in &[
        "must_fail_when_mode_2_dependency_removed",
        "must_emit_dependency_removal_diff",
        "forbid_silently_passing_with_missing_dependency",
        "forbid_lockfile_assertion_being_ignored_when_dep_removed",
        "must_distinguish_dependency_removal_from_version_drift",
    ] {
        assert_eq!(
            c.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let removal = c
        .get("dependency_removal_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let drift = c
        .get("version_drift_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(removal, 182);
    assert_eq!(drift, 183);
    assert_ne!(
        removal, drift,
        "dependency-removal and version-drift exit codes must differ"
    );
    assert_eq!(
        c.get("dependency_removal_failure_kind")
            .and_then(|v| v.as_str()),
        Some("mode2_lockfile_dependency_removal_undetected"),
    );
    assert_eq!(
        c.get("version_drift_failure_kind").and_then(|v| v.as_str()),
        Some("mode2_lockfile_dependency_version_drift"),
    );
}

// Acceptance: "Lockfile format is documented in the fixture README
// or test comment."
#[test]
fn lockfile_format_is_documented_in_the_fixture_readme_or_test_comment() {
    let doc = load_toml(&manifest_path());
    let d = doc
        .get("lockfile_format_documentation_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[lockfile_format_documentation_contract] missing — acceptance: \
         \"Lockfile format is documented in the fixture README or test comment.\"",
        );
    for k in &[
        "must_document_lockfile_format",
        "must_document_required_fields",
        "must_document_path_normalization_rules",
        "must_document_machine_independence_rules",
        "forbid_undocumented_lockfile_format",
    ] {
        assert_eq!(
            d.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        d.get("documentation_field_name").and_then(|v| v.as_str()),
        Some("lockfile_format_documentation_location"),
    );
    let allowed: Vec<&str> = d
        .get("allowed_documentation_locations")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &["readme", "test_comment"] {
        assert!(
            allowed.contains(v),
            "allowed_documentation_locations must include {v}"
        );
    }
    let fields: Vec<&str> = d
        .get("required_documented_fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &[
        "dependency_name",
        "dependency_version",
        "resolved_artifact_identity",
        "resolved_artifact_kind",
    ] {
        assert!(
            fields.contains(v),
            "required_documented_fields must include {v}"
        );
    }
    let exit = d
        .get("missing_documentation_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 184);
    assert_eq!(
        d.get("missing_documentation_failure_kind")
            .and_then(|v| v.as_str()),
        Some("mode2_lockfile_format_documentation_missing"),
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .unwrap();
    let keys: Vec<&str> = c
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "outcome",
        "case",
        "project_name",
        "manifest_relative_path",
        "lockfile_relative_path",
        "lockfile_format_documentation_location",
        "dependency_name",
        "dependency_version",
        "resolved_artifact_identity",
        "resolved_artifact_kind",
        "failure_kind",
        "exit_code",
    ] {
        assert!(
            keys.contains(required),
            "runner_contract.keys must include {required}"
        );
    }
    let cases: Vec<&str> = c
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "running_the_build_twice_produces_equivalent_lockfile_content",
        "removing_the_dependency_changes_the_lockfile_assertion_and_fails",
        "lockfile_format_is_documented_in_the_fixture_readme_or_test_comment",
    ] {
        assert!(
            cases.contains(required),
            "runner_contract.case_values must include {required}"
        );
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("full_resolver_lockfile_policy")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
