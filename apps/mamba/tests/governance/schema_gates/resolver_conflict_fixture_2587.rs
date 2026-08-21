//! Schema gate for the resolver conflict fixture — closes #2587.
//!
//! Acceptance (issue #2587):
//!
//!   1. Test runs in the default package-manager profile.
//!      `[default_profile_case]` pins must_be_default_profile +
//!      profile_name="package_manager" +
//!      must_run_in_default_profile_without_extra_flags +
//!      must_not_require_opt_in_marker.
//!   2. Conflict result is deterministic and offline.
//!      `[determinism_and_offline_contract]` pins
//!      forbid_network_access + forbid_dns_lookups +
//!      forbid_outbound_sockets +
//!      must_be_deterministic_across_runs +
//!      must_produce_identical_diagnostic_across_runs.
//!   3. Diagnostic is stable enough for regression checks.
//!      `[diagnostic_stability_contract]` pins
//!      must_name_incompatible_packages +
//!      must_name_incompatible_constraints +
//!      diagnostic_record_format="json" + conflict_exit_code=16
//!      distinct from #2584's exit codes 10 and 11.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("package_manager")
        .join("resolver_conflict")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("resolver_conflict"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2587));
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
        Some("resolver_conflict")
    );
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
fn index_cross_references_frozen_local_simple_index() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc
        .get("index")
        .and_then(|v| v.as_table())
        .expect("[index] missing");
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
        i.get("must_be_frozen").and_then(|v| v.as_bool()),
        Some(true)
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
fn test_unignore_is_scoped_to_this_focused_test_only() {
    let doc = crate::common::load_toml(&manifest_path());
    let u = doc
        .get("test_unignore")
        .and_then(|v| v.as_table())
        .expect("[test_unignore] missing");
    assert_eq!(
        u.get("must_remove_ignore_marker").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        u.get("must_only_remove_for_this_focused_test")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        u.get("must_not_remove_unrelated_ignore_markers")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn requirements_and_conflict_pair_are_consistent() {
    let doc = crate::common::load_toml(&manifest_path());
    let reqs = doc
        .get("requirements")
        .and_then(|v| v.as_array())
        .expect("[[requirements]] missing");
    assert_eq!(
        reqs.len(),
        2,
        "must declare exactly two conflicting requirements"
    );
    let names: Vec<&str> = reqs
        .iter()
        .filter_map(|r| {
            r.as_table()
                .and_then(|t| t.get("name"))
                .and_then(|v| v.as_str())
        })
        .collect();
    let constraints: Vec<&str> = reqs
        .iter()
        .filter_map(|r| {
            r.as_table()
                .and_then(|t| t.get("version_constraint"))
                .and_then(|v| v.as_str())
        })
        .collect();
    assert_eq!(names.len(), 2);
    assert_eq!(constraints.len(), 2);
    assert_ne!(names[0], names[1]);
    assert_ne!(
        constraints[0], constraints[1],
        "version constraints must disagree"
    );

    let c = doc
        .get("conflict")
        .and_then(|v| v.as_table())
        .expect("[conflict] missing");
    assert_eq!(
        c.get("must_be_unsatisfiable").and_then(|v| v.as_bool()),
        Some(true)
    );
    let pair: Vec<&str> = c
        .get("incompatible_pair")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(pair.len(), 2);
    for n in &names {
        assert!(
            pair.contains(n),
            "incompatible_pair must include requirement name {n}"
        );
    }
    let ca = c
        .get("incompatible_constraint_a")
        .and_then(|v| v.as_str())
        .unwrap();
    let cb = c
        .get("incompatible_constraint_b")
        .and_then(|v| v.as_str())
        .unwrap();
    assert_ne!(ca, cb);
    assert!(constraints.contains(&ca));
    assert!(constraints.contains(&cb));
    assert_eq!(
        c.get("constraints_must_disagree").and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Test runs in the default package-manager profile."
#[test]
fn test_runs_in_default_package_manager_profile() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc
        .get("default_profile_case")
        .and_then(|v| v.as_table())
        .expect(
            "[default_profile_case] missing — acceptance: \
         \"Test runs in the default package-manager profile.\"",
        );
    assert_eq!(
        d.get("must_be_default_profile").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("profile_name").and_then(|v| v.as_str()),
        Some("package_manager")
    );
    assert_eq!(
        d.get("must_run_in_default_profile_without_extra_flags")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("must_not_require_opt_in_marker")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Conflict result is deterministic and offline."
#[test]
fn conflict_result_is_deterministic_and_offline() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc
        .get("determinism_and_offline_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[determinism_and_offline_contract] missing — acceptance: \
         \"Conflict result is deterministic and offline.\"",
        );
    for f in &[
        "must_run_offline",
        "forbid_network_access",
        "forbid_dns_lookups",
        "forbid_outbound_sockets",
        "must_be_deterministic_across_runs",
        "must_produce_identical_diagnostic_across_runs",
    ] {
        assert_eq!(
            d.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

// Acceptance: "Diagnostic is stable enough for regression checks."
#[test]
fn diagnostic_is_stable_enough_for_regression_checks() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc
        .get("diagnostic_stability_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[diagnostic_stability_contract] missing — acceptance: \
         \"Diagnostic is stable enough for regression checks.\"",
        );
    assert_eq!(
        d.get("must_name_incompatible_packages")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("must_name_incompatible_constraints")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("must_be_machine_readable").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("diagnostic_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    assert_eq!(
        d.get("conflict_failure_kind").and_then(|v| v.as_str()),
        Some("resolver_conflict")
    );
    let exit = d
        .get("conflict_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 16);
    let yanked_exit = d
        .get("yanked_rejection_exit_code_from_2584")
        .and_then(|v| v.as_integer())
        .unwrap();
    let no_cand_exit = d
        .get("no_candidate_exit_code_from_2584")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(yanked_exit, 10);
    assert_eq!(no_cand_exit, 11);
    assert_ne!(
        exit, yanked_exit,
        "conflict exit must differ from yanked-rejection exit (#2584)"
    );
    assert_ne!(
        exit, no_cand_exit,
        "conflict exit must differ from no-candidate exit (#2584)"
    );
    assert_eq!(
        d.get("must_distinguish_from_yanked_rejection_exit_code")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("must_distinguish_from_no_candidate_exit_code")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
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
        "incompatible_package_a",
        "incompatible_package_b",
        "incompatible_constraint_a",
        "incompatible_constraint_b",
        "diagnostic_message",
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
    assert!(cases.contains(&"resolver_conflict_produces_stable_diagnostic"));
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("implementing_full_sat_resolver_redesign")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
