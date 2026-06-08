//! Schema gate for the live-PyPI opt-in fixture — closes #2590.
//!
//! Acceptance (issue #2590):
//!
//!   1. Default package-manager gate runs without network.
//!      `[default_profile_network_assertion]` pins
//!      must_assert_no_network_in_default_profile +
//!      must_be_part_of_default_package_manager_gate +
//!      forbid_dns_lookups + forbid_outbound_sockets +
//!      must_fail_if_network_access_observed +
//!      default_profile_network_violation_exit_code=22.
//!   2. Live PyPI tests still have an explicit command path.
//!      `[live_command_path_contract]` pins
//!      must_document_live_command_separately +
//!      documented_command_template +
//!      documented_command_must_include_feature_flag +
//!      documented_command_must_include_ignore_filter +
//!      live_command_must_be_excluded_from_default_test_run.
//!   3. CI or worker summary shows live checks as opt-in.
//!      `[summary_visibility_contract]` pins
//!      must_emit_summary_section + summary_must_record_live_checks_
//!      as_opt_in + json summary_record_format +
//!      summary_required_fields including live_pypi_opt_in and
//!      live_pypi_executed.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("package_manager")
        .join("live_pypi_opt_in")
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
        Some("live_pypi_opt_in"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2590));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2532)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("live_pypi_opt_in")
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
fn default_index_cross_references_frozen_local_simple_index() {
    let doc = load_toml(&manifest_path());
    let i = doc
        .get("default_index")
        .and_then(|v| v.as_table())
        .expect("[default_index] missing");
    assert_eq!(
        i.get("kind").and_then(|v| v.as_str()),
        Some("frozen_local_simple_index")
    );
    assert_eq!(
        i.get("local_simple_index_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2585),
        "must cross-reference frozen local simple-index fixture #2585",
    );
    assert_eq!(
        i.get("must_be_offline").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        i.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn opt_in_gate_pins_both_feature_and_env_var_names() {
    let doc = load_toml(&manifest_path());
    let g = doc
        .get("opt_in_gate")
        .and_then(|v| v.as_table())
        .expect("[opt_in_gate] missing");
    assert_eq!(
        g.get("must_be_opt_in").and_then(|v| v.as_bool()),
        Some(true)
    );
    let feature = g
        .get("opt_in_cargo_feature_name")
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(!feature.is_empty());
    assert!(
        feature.contains("pypi") || feature.contains("live"),
        "feature name must mention live or pypi: {feature}"
    );
    let env_var = g
        .get("opt_in_env_var_name")
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(
        env_var.contains("PYPI") || env_var.contains("LIVE"),
        "env var must mention LIVE or PYPI: {env_var}"
    );
    assert!(
        env_var
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()),
        "env var name must be SCREAMING_SNAKE_CASE: {env_var}"
    );
    assert_eq!(
        g.get("forbid_running_under_default_profile")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        g.get("must_attach_ignore_marker_when_not_opted_in")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Default package-manager gate runs without network."
#[test]
fn default_package_manager_gate_runs_without_network() {
    let doc = load_toml(&manifest_path());
    let d = doc
        .get("default_profile_network_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "[default_profile_network_assertion] missing — acceptance: \
         \"Default package-manager gate runs without network.\"",
        );
    for f in &[
        "must_assert_no_network_in_default_profile",
        "must_be_part_of_default_package_manager_gate",
        "forbid_dns_lookups_in_default_profile",
        "forbid_outbound_sockets_in_default_profile",
        "must_fail_if_network_access_observed_in_default_profile",
    ] {
        assert_eq!(
            d.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    assert_eq!(
        d.get("default_profile_network_violation_failure_kind")
            .and_then(|v| v.as_str()),
        Some("network_in_default_profile")
    );
    let exit = d
        .get("default_profile_network_violation_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 22);
}

// Acceptance: "Live PyPI tests still have an explicit command path."
#[test]
fn live_pypi_tests_still_have_explicit_command_path() {
    let doc = load_toml(&manifest_path());
    let l = doc
        .get("live_command_path_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[live_command_path_contract] missing — acceptance: \
         \"Live PyPI tests still have an explicit command path.\"",
        );
    assert_eq!(
        l.get("must_document_live_command_separately")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let template = l
        .get("documented_command_template")
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(
        template.contains("cargo test"),
        "documented_command_template must use cargo test"
    );
    let feature = doc
        .get("opt_in_gate")
        .and_then(|v| v.get("opt_in_cargo_feature_name"))
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(
        template.contains(feature),
        "documented_command_template must mention opt-in feature {feature}"
    );
    assert!(
        template.contains("--ignored"),
        "documented_command_template must include --ignored filter"
    );
    assert_eq!(
        l.get("documented_command_must_include_feature_flag")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("documented_command_must_include_ignore_filter")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("live_command_must_be_excluded_from_default_test_run")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "CI or worker summary shows live checks as opt-in."
#[test]
fn ci_summary_shows_live_checks_as_opt_in() {
    let doc = load_toml(&manifest_path());
    let s = doc
        .get("summary_visibility_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[summary_visibility_contract] missing — acceptance: \
         \"CI or worker summary shows live checks as opt-in.\"",
        );
    assert_eq!(
        s.get("must_emit_summary_section").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("summary_must_record_live_checks_as_opt_in")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("summary_must_record_default_profile_as_offline")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("summary_must_be_machine_readable")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("summary_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    let required: Vec<&str> = s
        .get("summary_required_fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for f in &[
        "default_profile",
        "default_profile_network_mode",
        "live_pypi_opt_in",
        "live_pypi_executed",
        "live_pypi_skip_reason",
    ] {
        assert!(
            required.contains(f),
            "summary_required_fields must include {f}"
        );
    }
    let allowed: Vec<&str> = s
        .get("allowed_default_profile_network_mode_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(allowed.contains(&"offline"));
    assert!(
        !allowed.contains(&"online"),
        "default profile must not allow online network mode"
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
        "profile",
        "live_pypi_opt_in",
        "live_pypi_executed",
        "network_access_observed",
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
        "default_profile_runs_offline",
        "live_pypi_tests_are_explicit",
        "summary_records_live_checks_as_opt_in",
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
        o.get("removing_live_integration_coverage")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
