//! Schema gate for the package-manager lockfile determinism fixture
//! — closes #2586.
//!
//! Acceptance (issue #2586):
//!
//!   1. Two lock runs produce equivalent normalized content.
//!      `[double_lock_case]` pins must_run_lock_generation_twice +
//!      must_compare_normalized_content +
//!      normalization_must_be_deterministic + divergence_failure_
//!      kind=lockfile_nondeterministic + exit_code=13.
//!   2. Changing one fixture package version changes the lockfile
//!      deterministically. `[version_change_case]` pins must_change_
//!      one_fixture_package_version + assert lockfile reflects
//!      change + unchanged packages remain byte-equivalent +
//!      version_change_failure_kind=version_change_not_propagated +
//!      exit_code=14.
//!   3. Machine-specific temp paths are excluded.
//!      `[path_redaction_contract]` pins
//!      must_exclude_machine_specific_temp_paths +
//!      forbid_tmpdir_random_suffixes + forbid_user_home_paths +
//!      machine_specific_path_failure_kind + exit_code=15.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("package_manager")
        .join("lockfile_determinism")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("lockfile_determinism"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2586));
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
        Some("lockfile_determinism")
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
fn index_and_lockfile_cross_reference_sibling_fixtures() {
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
        i.get("must_be_offline").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        i.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );

    let l = doc
        .get("lockfile")
        .and_then(|v| v.as_table())
        .expect("[lockfile] missing");
    assert_eq!(
        l.get("mode2_lockfile_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2574),
        "must cross-reference Mode 2 lockfile fixture #2574",
    );
    let required: Vec<&str> = l
        .get("required_fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for f in &["name", "version", "hash", "source_url"] {
        assert!(required.contains(f), "required_fields must include {f}");
    }
    assert_eq!(
        l.get("hash_algorithm").and_then(|v| v.as_str()),
        Some("sha256")
    );
}

// Acceptance: "Two lock runs produce equivalent normalized content."
#[test]
fn two_lock_runs_produce_equivalent_normalized_content() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc
        .get("double_lock_case")
        .and_then(|v| v.as_table())
        .expect(
            "[double_lock_case] missing — acceptance: \
         \"Two lock runs produce equivalent normalized content.\"",
        );
    assert_eq!(
        c.get("must_run_lock_generation_twice")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_compare_normalized_content")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("normalization_must_be_deterministic")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("expected_outcome_when_runs_are_equivalent")
            .and_then(|v| v.as_str()),
        Some("pass")
    );
    assert_eq!(
        c.get("expected_outcome_when_runs_differ")
            .and_then(|v| v.as_str()),
        Some("fail")
    );
    assert_eq!(
        c.get("divergence_failure_kind").and_then(|v| v.as_str()),
        Some("lockfile_nondeterministic")
    );
    let exit = c
        .get("divergence_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 13);
    assert_eq!(
        c.get("divergence_diagnostic_must_show_normalized_diff")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Changing one fixture package version changes the lockfile deterministically."
#[test]
fn version_change_propagates_deterministically_to_lockfile() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc
        .get("version_change_case")
        .and_then(|v| v.as_table())
        .expect(
            "[version_change_case] missing — acceptance: \
         \"Changing one fixture package version changes the lockfile deterministically.\"",
        );
    assert_eq!(
        c.get("must_change_one_fixture_package_version")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_assert_lockfile_changes_for_that_package")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_assert_unchanged_packages_remain_byte_equivalent")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("expected_outcome_when_lockfile_reflects_change")
            .and_then(|v| v.as_str()),
        Some("pass")
    );
    assert_eq!(
        c.get("expected_outcome_when_lockfile_does_not_reflect_change")
            .and_then(|v| v.as_str()),
        Some("fail")
    );
    assert_eq!(
        c.get("version_change_failure_kind")
            .and_then(|v| v.as_str()),
        Some("version_change_not_propagated")
    );
    let exit = c
        .get("version_change_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 14);
    assert_eq!(
        c.get("must_be_deterministic_for_change_diff")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Machine-specific temp paths are excluded."
#[test]
fn machine_specific_temp_paths_are_excluded() {
    let doc = crate::common::load_toml(&manifest_path());
    let p = doc
        .get("path_redaction_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[path_redaction_contract] missing — acceptance: \
         \"Machine-specific temp paths are excluded.\"",
        );
    assert_eq!(
        p.get("must_exclude_machine_specific_temp_paths_from_lockfile")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("forbid_tmpdir_random_suffixes_in_lockfile")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("forbid_user_home_paths_in_lockfile")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let allowed: Vec<&str> = p
        .get("allowed_path_kinds_in_lockfile")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for k in &["relative_to_project", "frozen_local_index_url"] {
        assert!(
            allowed.contains(k),
            "allowed_path_kinds_in_lockfile must include {k}"
        );
    }
    assert_eq!(
        p.get("machine_specific_path_failure_kind")
            .and_then(|v| v.as_str()),
        Some("machine_specific_path_in_lockfile")
    );
    let exit = p
        .get("machine_specific_path_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 15);
    assert_eq!(
        p.get("normalization_must_strip_random_temp_suffixes")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn failure_kinds_have_distinct_exit_codes() {
    let doc = crate::common::load_toml(&manifest_path());
    let a = doc
        .get("double_lock_case")
        .and_then(|v| v.get("divergence_exit_code"))
        .and_then(|v| v.as_integer())
        .unwrap();
    let b = doc
        .get("version_change_case")
        .and_then(|v| v.get("version_change_exit_code"))
        .and_then(|v| v.as_integer())
        .unwrap();
    let c = doc
        .get("path_redaction_contract")
        .and_then(|v| v.get("machine_specific_path_exit_code"))
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
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
        "lockfile_path",
        "run_index",
        "normalized_hash",
        "changed_package",
        "old_version",
        "new_version",
        "machine_specific_path",
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
        "two_lock_runs_produce_equivalent_normalized_content",
        "version_change_propagates_deterministically_to_lockfile",
        "machine_specific_paths_are_excluded_from_lockfile",
    ] {
        assert!(
            cases.contains(required),
            "runner_contract.case_values must include {required}"
        );
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("final_lockfile_format_design_beyond_required_fields")
            .and_then(|v| v.as_bool()),
        Some(true),
    );
}
