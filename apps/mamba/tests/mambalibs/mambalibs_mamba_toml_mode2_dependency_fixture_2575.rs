//! Schema gate for the mambalibs mamba.toml Mode 2 dependency fixture —
//! closes #2575.
//!
//! Acceptance (issue #2575):
//!
//!   1. Fixture parse test fails if the dependency stanza is removed.
//!      `[dependency_stanza_removal_case]` pins
//!      `parse_must_fail_if_stanza_removed=true` plus the expected
//!      pass/fail outcomes when the stanza is present vs. removed.
//!   2. Fixture is usable by the mambalibs E2E harness.
//!      `[e2e_harness_compatibility]` pins
//!      `must_be_consumable_by_e2e_harness_verbatim=true`,
//!      `must_not_require_post_processing=true`, and cross-references
//!      the E2E harness fixture (#2578).
//!   3. The syntax matches the documented target workflow.
//!      `[documented_workflow_match]` pins
//!      `must_match_documented_section_name/mode_field_name/
//!      path_field_name/mode_value=true` and the documented values.
//!
//! Cheap test — single TOML read + field walk. Stays in the default
//! `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("mamba_toml_mode2_dependency")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn header_is_well_formed() {
    let doc = load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_mamba_toml_mode2_dependency"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2575));
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
        Some("mamba_toml_mode2_dependency"),
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
fn mamba_toml_declares_local_mode2_dependency() {
    let doc = load_toml(&manifest_path());
    let m = doc
        .get("mamba_toml")
        .and_then(|v| v.as_table())
        .expect("missing `[mamba_toml]` block");

    assert_eq!(
        m.get("section_name").and_then(|v| v.as_str()),
        Some("dependencies")
    );
    let dep_name = m
        .get("dependency_name")
        .and_then(|v| v.as_str())
        .expect("dependency_name must be set");
    assert!(!dep_name.is_empty(), "dependency_name must be non-empty");
    assert_eq!(
        m.get("dependency_source_kind").and_then(|v| v.as_str()),
        Some("local_path"),
        "Mode 2 fixture must declare a local path source",
    );
    assert!(
        m.get("dependency_path").and_then(|v| v.as_str()).is_some(),
        "dependency_path must be set"
    );
    assert_eq!(
        m.get("dependency_mode").and_then(|v| v.as_integer()),
        Some(2),
        "dependency_mode must be 2 (Mode 2)",
    );
    assert_eq!(
        m.get("local_binding_crate_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2577),
        "must cross-reference the #2577 local binding crate fixture",
    );
}

// Acceptance: "Fixture parse test fails if the dependency stanza is
// removed."
#[test]
fn parse_test_fails_when_dependency_stanza_removed() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("dependency_stanza_removal_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[dependency_stanza_removal_case]` block — acceptance: \
         \"Fixture parse test fails if the dependency stanza is removed.\"",
        );
    assert_eq!(
        c.get("must_observe_section").and_then(|v| v.as_str()),
        Some("dependencies")
    );
    let dep_in_case = c
        .get("must_observe_dependency_name")
        .and_then(|v| v.as_str())
        .expect("must_observe_dependency_name must be set");
    let dep_in_mamba_toml = doc
        .get("mamba_toml")
        .and_then(|v| v.get("dependency_name"))
        .and_then(|v| v.as_str())
        .expect("mamba_toml.dependency_name must be set");
    assert_eq!(
        dep_in_case, dep_in_mamba_toml,
        "must_observe_dependency_name must match [mamba_toml].dependency_name",
    );
    assert_eq!(
        c.get("parse_must_fail_if_stanza_removed")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("parse_must_fail_diagnostic_names_dependency_name")
            .and_then(|v| v.as_bool()),
        Some(true),
    );
    assert_eq!(
        c.get("parse_failure_exit_code_when_missing")
            .and_then(|v| v.as_integer()),
        Some(1),
    );
    assert_eq!(
        c.get("expected_outcome_when_present")
            .and_then(|v| v.as_str()),
        Some("pass")
    );
    assert_eq!(
        c.get("expected_outcome_when_removed")
            .and_then(|v| v.as_str()),
        Some("fail")
    );
}

// Acceptance: "Fixture is usable by the mambalibs E2E harness."
#[test]
fn fixture_is_usable_by_e2e_harness() {
    let doc = load_toml(&manifest_path());
    let e = doc
        .get("e2e_harness_compatibility")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[e2e_harness_compatibility]` block — acceptance: \
         \"Fixture is usable by the mambalibs E2E harness.\"",
        );
    assert_eq!(
        e.get("mamba_build_e2e_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2578),
        "must cross-reference the #2578 mamba-build E2E harness fixture",
    );
    for f in &[
        "must_be_consumable_by_e2e_harness_verbatim",
        "must_not_require_post_processing",
        "must_resolve_local_binding_crate_via_relative_path",
    ] {
        assert_eq!(
            e.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

// Acceptance: "The syntax matches the documented target workflow."
#[test]
fn syntax_matches_documented_target_workflow() {
    let doc = load_toml(&manifest_path());
    let d = doc
        .get("documented_workflow_match")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[documented_workflow_match]` block — acceptance: \
         \"The syntax matches the documented target workflow.\"",
        );
    assert_eq!(
        d.get("documented_section_name").and_then(|v| v.as_str()),
        Some("dependencies")
    );
    assert_eq!(
        d.get("documented_mode_field_name").and_then(|v| v.as_str()),
        Some("mode")
    );
    assert_eq!(
        d.get("documented_path_field_name").and_then(|v| v.as_str()),
        Some("path")
    );
    assert_eq!(
        d.get("documented_mode_value_for_mode2")
            .and_then(|v| v.as_integer()),
        Some(2),
    );
    for f in &[
        "must_match_documented_section_name",
        "must_match_documented_mode_field_name",
        "must_match_documented_path_field_name",
        "must_match_documented_mode_value",
    ] {
        assert_eq!(
            d.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }

    // Cross-check: the documented values must match the values actually
    // declared in [mamba_toml]. If anyone changes the fixture's
    // mamba.toml shape, this test forces them to also update the
    // documented baseline (and vice versa).
    let m = doc.get("mamba_toml").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        m.get("section_name").and_then(|v| v.as_str()),
        d.get("documented_section_name").and_then(|v| v.as_str()),
    );
    assert_eq!(
        m.get("mode_field_name").and_then(|v| v.as_str()),
        d.get("documented_mode_field_name").and_then(|v| v.as_str()),
    );
    assert_eq!(
        m.get("path_field_name").and_then(|v| v.as_str()),
        d.get("documented_path_field_name").and_then(|v| v.as_str()),
    );
    assert_eq!(
        m.get("dependency_mode").and_then(|v| v.as_integer()),
        d.get("documented_mode_value_for_mode2")
            .and_then(|v| v.as_integer()),
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
        "section_name",
        "dependency_name",
        "dependency_source_kind",
        "dependency_path",
        "dependency_mode",
        "parse_failure_diagnostic",
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
    for required in &["dependency_stanza_present", "dependency_stanza_removed"] {
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
        o.get("resolver_implementation_changes")
            .and_then(|v| v.as_bool()),
        Some(true),
    );
}
