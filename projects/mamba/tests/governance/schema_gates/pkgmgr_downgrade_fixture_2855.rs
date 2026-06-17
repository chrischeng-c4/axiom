//! Schema gate for the package-manager downgrade fixture —
//! closes #2855.
//!
//! Acceptance (issue #2855):
//!
//!   1. Downgrade changes only expected package entries.
//!      `[downgrade_action]` flips the locked version and asserts
//!      `must_only_touch_target_package = true`; the lockfile diff
//!      bounds the change to the target.
//!   2. Constraint conflict reports deterministic diagnostic.
//!      `[constraint_conflict_case]` pins fail exit, no lockfile
//!      mutation, deterministic diagnostic that names package +
//!      constraint + available versions.
//!   3. Fixture is offline and isolated.
//!      Header + `[isolation]` block.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("downgrade")
        .join("manifest.toml")
}

#[test]
fn pkgmgr_downgrade_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_downgrade"),
        "`fixture` must be \"pkgmgr_downgrade\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2855),
        "`issue` must record #2855"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("downgrade"),
        "`family` must be \"downgrade\""
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager"),
        "`profile` must be \"package_manager\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn pkgmgr_downgrade_package_block_pins_two_distinct_versions() {
    let doc = crate::common::load_toml(&manifest_path());
    let pkg = doc
        .get("package")
        .and_then(|v| v.as_table())
        .expect("missing `[package]` block");

    let older = pkg
        .get("older_version")
        .and_then(|v| v.as_str())
        .expect("`[package].older_version` must be set");
    let newer = pkg
        .get("newer_version")
        .and_then(|v| v.as_str())
        .expect("`[package].newer_version` must be set");
    assert_ne!(older, newer, "older and newer versions must differ");

    let older_sent = pkg
        .get("older_sentinel_value")
        .and_then(|v| v.as_str())
        .expect("`[package].older_sentinel_value` must be set");
    let newer_sent = pkg
        .get("newer_sentinel_value")
        .and_then(|v| v.as_str())
        .expect("`[package].newer_sentinel_value` must be set");
    assert_ne!(
        older_sent, newer_sent,
        "sentinel values must differ — import probe distinguishes versions"
    );
    assert_eq!(
        older_sent, older,
        "older sentinel value should equal `[package].older_version` — fixture convention"
    );
    assert_eq!(
        newer_sent, newer,
        "newer sentinel value should equal `[package].newer_version` — fixture convention"
    );
}

#[test]
fn pkgmgr_downgrade_initial_state_pins_newer_version() {
    let doc = crate::common::load_toml(&manifest_path());
    let init = doc
        .get("initial_state")
        .and_then(|v| v.as_table())
        .expect("missing `[initial_state]` block");

    let command: Vec<&str> = init
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("add"),
        "`[initial_state].command[0]` must be `add`"
    );
    let newer = doc
        .get("package")
        .and_then(|v| v.get("newer_version"))
        .and_then(|v| v.as_str())
        .expect("`[package].newer_version` must be set");
    assert!(
        command.iter().any(|s| s.contains(newer)),
        "initial pin must reference the newer version {newer:?}; got {command:?}"
    );

    assert_eq!(
        init.get("locked_version_must_be").and_then(|v| v.as_str()),
        Some(newer),
        "`[initial_state].locked_version_must_be` must equal `[package].newer_version`"
    );
    assert_eq!(
        init.get("import_probe_value_must_be").and_then(|v| v.as_str()),
        Some(newer),
        "`[initial_state].import_probe_value_must_be` must equal `[package].newer_version`"
    );
}

#[test]
fn pkgmgr_downgrade_action_flips_to_older_version_only() {
    let doc = crate::common::load_toml(&manifest_path());
    let action = doc
        .get("downgrade_action")
        .and_then(|v| v.as_table())
        .expect("missing `[downgrade_action]` block");

    let older = doc
        .get("package")
        .and_then(|v| v.get("older_version"))
        .and_then(|v| v.as_str())
        .expect("`[package].older_version` must be set");
    let newer = doc
        .get("package")
        .and_then(|v| v.get("newer_version"))
        .and_then(|v| v.as_str())
        .expect("`[package].newer_version` must be set");

    let command: Vec<&str> = action
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        command.iter().any(|s| s.contains(older)),
        "downgrade action must reference the older version {older:?}; got {command:?}"
    );

    assert_eq!(
        action.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[downgrade_action].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        action.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[downgrade_action].expected_exit_code` must be 0"
    );
    assert_eq!(
        action.get("locked_version_must_become").and_then(|v| v.as_str()),
        Some(older),
        "`[downgrade_action].locked_version_must_become` must equal `[package].older_version`"
    );
    assert_eq!(
        action.get("locked_version_must_no_longer_be").and_then(|v| v.as_str()),
        Some(newer),
        "`[downgrade_action].locked_version_must_no_longer_be` must equal `[package].newer_version`"
    );
    assert_eq!(
        action.get("must_only_touch_target_package").and_then(|v| v.as_bool()),
        Some(true),
        "`[downgrade_action].must_only_touch_target_package` must be true"
    );
    assert_eq!(
        action.get("import_probe_value_must_be").and_then(|v| v.as_str()),
        Some(older),
        "`[downgrade_action].import_probe_value_must_be` must equal `[package].older_version`"
    );
}

#[test]
fn pkgmgr_downgrade_lockfile_diff_bounds_change_to_target() {
    let doc = crate::common::load_toml(&manifest_path());
    let diff = doc.get("lockfile_diff_assertion").and_then(|v| v.as_table()).expect(
        "missing `[lockfile_diff_assertion]` block \
         (acceptance: \"Downgrade changes only expected package entries.\")",
    );

    let pkg_name = doc
        .get("package")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[package].name` must be set");
    let older = doc
        .get("package")
        .and_then(|v| v.get("older_version"))
        .and_then(|v| v.as_str())
        .expect("`[package].older_version` must be set");
    let newer = doc
        .get("package")
        .and_then(|v| v.get("newer_version"))
        .and_then(|v| v.as_str())
        .expect("`[package].newer_version` must be set");
    let other_name = doc
        .get("other_dependency")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[other_dependency].name` must be set");

    assert_eq!(
        diff.get("must_record_old_version").and_then(|v| v.as_str()),
        Some(newer),
        "`[lockfile_diff_assertion].must_record_old_version` must equal `[package].newer_version`"
    );
    assert_eq!(
        diff.get("must_record_new_version").and_then(|v| v.as_str()),
        Some(older),
        "`[lockfile_diff_assertion].must_record_new_version` must equal `[package].older_version`"
    );
    assert_eq!(
        diff.get("must_only_change_target_package").and_then(|v| v.as_str()),
        Some(pkg_name),
        "`[lockfile_diff_assertion].must_only_change_target_package` must equal `[package].name`"
    );
    assert_eq!(
        diff.get("must_not_touch_unrelated").and_then(|v| v.as_str()),
        Some(other_name),
        "`[lockfile_diff_assertion].must_not_touch_unrelated` must equal `[other_dependency].name`"
    );
    for flag in &[
        "deterministic",
        "byte_identical_on_replay",
        "other_packages_byte_identical",
    ] {
        assert_eq!(
            diff.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[lockfile_diff_assertion].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_downgrade_constraint_conflict_case_is_deterministic_and_loud() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc.get("constraint_conflict_case").and_then(|v| v.as_table()).expect(
        "missing `[constraint_conflict_case]` block \
         (acceptance: \"Constraint conflict reports deterministic diagnostic.\")",
    );

    let pkg_name = doc
        .get("package")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[package].name` must be set");

    let command: Vec<&str> = case
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        command.iter().any(|s| s.contains(pkg_name)),
        "`[constraint_conflict_case].command` must reference the target package; got {command:?}"
    );

    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[constraint_conflict_case].expected_outcome` must be \"fail\""
    );
    let exit = case
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[constraint_conflict_case].expected_exit_code` must be set");
    assert_ne!(exit, 0, "constraint conflict must not exit 0; got {exit}");

    for flag in &[
        "must_not_mutate_lockfile",
        "diagnostic_is_deterministic",
        "must_name_package",
        "must_name_requested_constraint",
        "must_name_available_versions",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[constraint_conflict_case].{flag}` must be true"
        );
    }

    let substring = case
        .get("diagnostic_message_substring")
        .and_then(|v| v.as_str())
        .expect("`[constraint_conflict_case].diagnostic_message_substring` must be set");
    assert_eq!(
        substring, pkg_name,
        "diagnostic substring must match the package name; got {substring:?}"
    );
}

#[test]
fn pkgmgr_downgrade_isolation_pins_no_global_state() {
    let doc = crate::common::load_toml(&manifest_path());
    let isolation = doc
        .get("isolation")
        .and_then(|v| v.as_table())
        .expect("missing `[isolation]` block");

    for flag in &[
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(
            isolation.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[isolation].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_downgrade_runner_contract_declares_outcome_keys() {
    let doc = crate::common::load_toml(&manifest_path());
    let contract = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .expect("missing `[runner_contract]` block");

    let keys: Vec<&str> = contract
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "outcome",
        "case",
        "package",
        "previous_version",
        "next_version",
        "import_outcome",
        "lockfile_change_count",
        "exit_code",
        "diagnostic_message",
    ] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }

    let cases: Vec<&str> = contract
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["initial_pin", "downgrade", "constraint_conflict"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must carry `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn pkgmgr_downgrade_pins_out_of_scope_per_issue_2855() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("full_version_selection_ux").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].full_version_selection_ux` must be true \
         (issue text: \"Out of scope: full version-selection UX.\")"
    );
}
