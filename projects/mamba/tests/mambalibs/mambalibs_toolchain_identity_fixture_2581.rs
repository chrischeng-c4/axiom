//! Schema gate for the mambalibs toolchain-identity fixture — closes
//! #2581.
//!
//! Acceptance (issue #2581):
//!
//!   1. Build summary includes Rust toolchain identity on success
//!      and failure. `[summary_contract]` pins
//!      must_emit_on_build_success + must_emit_on_build_failure +
//!      must_include_toolchain_identity_on_success +
//!      must_include_toolchain_identity_on_failure +
//!      required_toolchain_fields = [rustc_version, cargo_version,
//!      target_triple, build_profile].
//!   2. Missing fields fail the test. `[missing_field_case]` pins
//!      missing_toolchain_field_fails_test + failure_kind=
//!      toolchain_identity_incomplete + exit_code=8 +
//!      diagnostic_must_name_missing_field.
//!   3. Output avoids machine-specific absolute paths unless needed
//!      for diagnostics. `[path_redaction_contract]` pins
//!      must_avoid_machine_specific_absolute_paths +
//!      forbid_user_home_paths_in_summary_unless_diagnostic +
//!      diagnostic_paths_must_be_explicitly_marked.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("toolchain_identity")
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
        Some("mambalibs_toolchain_identity"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2581));
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
        Some("toolchain_identity")
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
fn binding_cross_references_build_harness_and_binding_crate() {
    let doc = load_toml(&manifest_path());
    let b = doc
        .get("binding")
        .and_then(|v| v.as_table())
        .expect("[binding] missing");
    assert_eq!(
        b.get("module_name").and_then(|v| v.as_str()),
        Some("mambalibs")
    );
    assert_eq!(
        b.get("mamba_build_e2e_harness_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2578)
    );
    assert_eq!(
        b.get("local_binding_crate_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2577)
    );
    assert_eq!(
        b.get("must_emit_structured_summary")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Build summary includes Rust toolchain identity on
// success and failure."
#[test]
fn build_summary_includes_toolchain_identity_on_success_and_failure() {
    let doc = load_toml(&manifest_path());
    let s = doc
        .get("summary_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[summary_contract] missing — acceptance: \
         \"Build summary includes Rust toolchain identity on success and failure.\"",
        );
    assert_eq!(
        s.get("must_be_machine_readable").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("summary_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    assert_eq!(
        s.get("must_emit_on_build_success")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_emit_on_build_failure")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_include_toolchain_identity_on_success")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_include_toolchain_identity_on_failure")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let required: Vec<&str> = s
        .get("required_toolchain_fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for f in &[
        "rustc_version",
        "cargo_version",
        "target_triple",
        "build_profile",
    ] {
        assert!(
            required.contains(f),
            "required_toolchain_fields must include {f}"
        );
    }
    let profiles: Vec<&str> = s
        .get("allowed_build_profile_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for p in &["debug", "release"] {
        assert!(
            profiles.contains(p),
            "allowed_build_profile_values must include {p}"
        );
    }
    assert_eq!(
        s.get("toolchain_identity_must_be_recorded_before_artifact_check")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Missing fields fail the test."
#[test]
fn missing_field_fails_the_test() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("missing_field_case")
        .and_then(|v| v.as_table())
        .expect("[missing_field_case] missing — acceptance: \"Missing fields fail the test.\"");
    assert_eq!(
        c.get("must_assert_each_required_field_is_present")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("expected_outcome_when_all_fields_present")
            .and_then(|v| v.as_str()),
        Some("pass")
    );
    assert_eq!(
        c.get("expected_outcome_when_any_field_missing")
            .and_then(|v| v.as_str()),
        Some("fail")
    );
    assert_eq!(
        c.get("missing_field_failure_kind").and_then(|v| v.as_str()),
        Some("toolchain_identity_incomplete")
    );
    let exit = c
        .get("missing_field_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0, "missing-field exit code must be non-zero");
    assert_eq!(exit, 8);
    assert_eq!(
        c.get("missing_field_diagnostic_must_name_missing_field")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_not_emit_speedup_when_summary_incomplete")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Output avoids machine-specific absolute paths unless
// needed for diagnostics."
#[test]
fn output_avoids_machine_specific_absolute_paths() {
    let doc = load_toml(&manifest_path());
    let p = doc
        .get("path_redaction_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[path_redaction_contract] missing — acceptance: \
         \"Output avoids machine-specific absolute paths unless needed for diagnostics.\"",
        );
    assert_eq!(
        p.get("must_avoid_machine_specific_absolute_paths")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("forbid_user_home_paths_in_summary_unless_diagnostic")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("forbid_tmpdir_random_suffixes_in_summary_unless_diagnostic")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let allowed: Vec<&str> = p
        .get("allowed_path_kinds_in_summary")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for k in &["relative_to_workspace", "diagnostic_only_absolute"] {
        assert!(
            allowed.contains(k),
            "allowed_path_kinds_in_summary must include {k}"
        );
    }
    assert_eq!(
        p.get("diagnostic_paths_must_be_explicitly_marked")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("must_be_deterministic_across_machines")
            .and_then(|v| v.as_bool()),
        Some(true)
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
        "rustc_version",
        "cargo_version",
        "target_triple",
        "build_profile",
        "missing_field",
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
        "toolchain_identity_recorded_on_build_success",
        "toolchain_identity_recorded_on_build_failure",
        "missing_toolchain_field_fails_test",
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
        o.get("installing_or_selecting_rust_toolchains")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
