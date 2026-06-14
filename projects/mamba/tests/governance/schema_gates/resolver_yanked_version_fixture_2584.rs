//! Schema gate for the resolver yanked-version fixture — closes
//! #2584.
//!
//! Acceptance (issue #2584):
//!
//!   1. Test runs without network. `[network_gate]` pins
//!      forbid_network_access + forbid_dns_lookups +
//!      forbid_outbound_sockets +
//!      must_run_against_local_index_only +
//!      test_must_fail_if_network_access_observed.
//!   2. Yanked version behavior is deterministic.
//!      `[yanked_behavior_contract]` pins
//!      must_avoid_yanked_versions_by_default +
//!      must_select_yanked_version_only_when_explicitly_allowed +
//!      allow_yanked_flag_name + deterministic_resolution_across_runs.
//!   3. Failure output names the selected or rejected version.
//!      `[diagnostic_contract]` pins
//!      failure_message_must_name_rejected_version + selected +
//!      yanked_marker + distinct failure_kind/exit_code pairs.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("package_manager")
        .join("resolver_yanked_version")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("resolver_yanked_version"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2584));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2532));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("package_manager"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("resolver_yanked_version"));
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = crate::common::load_toml(&manifest_path());
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
fn index_is_frozen_local_simple_index() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("index").and_then(|v| v.as_table()).expect("[index] missing");
    assert_eq!(i.get("kind").and_then(|v| v.as_str()), Some("frozen_local_simple_index"));
    assert_eq!(
        i.get("local_simple_index_fixture_issue").and_then(|v| v.as_integer()),
        Some(2585),
        "must cross-reference frozen local simple-index fixture #2585",
    );
    assert_eq!(i.get("must_be_frozen").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(i.get("must_be_offline").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(i.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn test_unignore_is_scoped_to_this_focused_test_only() {
    let doc = crate::common::load_toml(&manifest_path());
    let u = doc.get("test_unignore").and_then(|v| v.as_table()).expect("[test_unignore] missing");
    assert_eq!(u.get("must_remove_ignore_marker").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(u.get("must_only_remove_for_this_focused_test").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(u.get("must_not_remove_unrelated_ignore_markers").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Test runs without network."
#[test]
fn test_runs_without_network() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("network_gate").and_then(|v| v.as_table()).expect(
        "[network_gate] missing — acceptance: \"Test runs without network.\"",
    );
    for f in &[
        "forbid_network_access",
        "forbid_dns_lookups",
        "forbid_outbound_sockets",
        "must_run_against_local_index_only",
        "test_must_fail_if_network_access_observed",
    ] {
        assert_eq!(n.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

// Acceptance: "Yanked version behavior is deterministic."
#[test]
fn yanked_version_behavior_is_deterministic() {
    let doc = crate::common::load_toml(&manifest_path());
    let y = doc.get("yanked_behavior_contract").and_then(|v| v.as_table()).expect(
        "[yanked_behavior_contract] missing — acceptance: \
         \"Yanked version behavior is deterministic.\"",
    );
    assert_eq!(y.get("must_avoid_yanked_versions_by_default").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(y.get("must_select_yanked_version_only_when_explicitly_allowed").and_then(|v| v.as_bool()), Some(true));
    let flag = y.get("allow_yanked_flag_name").and_then(|v| v.as_str()).unwrap();
    assert!(flag.starts_with("--"), "allow_yanked_flag_name must be a long flag");
    assert!(flag.contains("yanked"), "allow_yanked_flag_name must mention yanked");
    assert_eq!(y.get("deterministic_resolution_across_runs").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(y.get("must_record_yanked_decision_in_resolution_log").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(y.get("yanked_decision_log_format").and_then(|v| v.as_str()), Some("json"));
}

// Acceptance: "Failure output names the selected or rejected version."
#[test]
fn failure_output_names_selected_or_rejected_version() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("diagnostic_contract").and_then(|v| v.as_table()).expect(
        "[diagnostic_contract] missing — acceptance: \
         \"Failure output names the selected or rejected version.\"",
    );
    assert_eq!(d.get("failure_message_must_name_rejected_version").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("failure_message_must_name_selected_version_when_available").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("failure_message_must_include_yanked_marker").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("failure_message_must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("yanked_rejection_failure_kind").and_then(|v| v.as_str()), Some("yanked_version_rejected"));
    let yanked_exit = d.get("yanked_rejection_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_ne!(yanked_exit, 0);
    assert_eq!(d.get("no_candidate_failure_kind").and_then(|v| v.as_str()), Some("no_acceptable_version"));
    let no_cand_exit = d.get("no_candidate_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_ne!(no_cand_exit, 0);
    assert_ne!(yanked_exit, no_cand_exit, "yanked-rejection and no-candidate exit codes must differ");
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "package",
        "selected_version", "rejected_version", "yanked_versions_seen",
        "allow_yanked", "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "resolver_avoids_yanked_version_by_default",
        "resolver_selects_yanked_version_when_explicitly_allowed",
        "resolver_fails_when_no_acceptable_version",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("every_pypi_metadata_edge_case").and_then(|v| v.as_bool()), Some(true));
}
