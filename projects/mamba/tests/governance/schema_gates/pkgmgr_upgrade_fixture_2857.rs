//! Schema gate for the package-manager upgrade fixture —
//! closes #2857.
//!
//! Acceptance (issue #2857):
//!
//!   1. Upgrade changes only expected package entries.
//!      `[upgrade_action]` flips the locked version and pins
//!      `must_only_touch_target_package = true`; the lockfile diff
//!      bounds the change to the target.
//!   2. Wrong or missing version change fails the test.
//!      `[wrong_change_guard]` pins fail flags for stale change
//!      records and missing version fields.
//!   3. Output names old and new versions.
//!      `[output_assertion]` pins must_name_{package, old_version,
//!      new_version} and the diff field keys.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("pkgmgr")
        .join("upgrade")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn pkgmgr_upgrade_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_upgrade"),
        "`fixture` must be \"pkgmgr_upgrade\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2857),
        "`issue` must record #2857"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("upgrade"),
        "`family` must be \"upgrade\""
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
fn pkgmgr_upgrade_package_block_pins_two_distinct_versions() {
    let doc = load_toml(&manifest_path());
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
fn pkgmgr_upgrade_initial_state_pins_older_version() {
    let doc = load_toml(&manifest_path());
    let init = doc
        .get("initial_state")
        .and_then(|v| v.as_table())
        .expect("missing `[initial_state]` block");

    let older = doc
        .get("package")
        .and_then(|v| v.get("older_version"))
        .and_then(|v| v.as_str())
        .expect("`[package].older_version` must be set");

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
    assert!(
        command.iter().any(|s| s.contains(older)),
        "initial pin must reference the older version {older:?}; got {command:?}"
    );

    assert_eq!(
        init.get("locked_version_must_be").and_then(|v| v.as_str()),
        Some(older),
        "`[initial_state].locked_version_must_be` must equal `[package].older_version`"
    );
    assert_eq!(
        init.get("import_probe_value_must_be")
            .and_then(|v| v.as_str()),
        Some(older),
        "`[initial_state].import_probe_value_must_be` must equal `[package].older_version`"
    );
}

#[test]
fn pkgmgr_upgrade_action_flips_to_newer_version_only() {
    let doc = load_toml(&manifest_path());
    let action = doc
        .get("upgrade_action")
        .and_then(|v| v.as_table())
        .expect("missing `[upgrade_action]` block");

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
        command.iter().any(|s| s.contains(newer)),
        "upgrade action must reference the newer version {newer:?}; got {command:?}"
    );

    assert_eq!(
        action.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[upgrade_action].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        action
            .get("expected_exit_code")
            .and_then(|v| v.as_integer()),
        Some(0),
        "`[upgrade_action].expected_exit_code` must be 0"
    );
    assert_eq!(
        action
            .get("locked_version_must_become")
            .and_then(|v| v.as_str()),
        Some(newer),
        "`[upgrade_action].locked_version_must_become` must equal `[package].newer_version`"
    );
    assert_eq!(
        action
            .get("locked_version_must_no_longer_be")
            .and_then(|v| v.as_str()),
        Some(older),
        "`[upgrade_action].locked_version_must_no_longer_be` must equal `[package].older_version`"
    );
    assert_eq!(
        action
            .get("must_only_touch_target_package")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[upgrade_action].must_only_touch_target_package` must be true"
    );
    assert_eq!(
        action
            .get("import_probe_value_must_be")
            .and_then(|v| v.as_str()),
        Some(newer),
        "`[upgrade_action].import_probe_value_must_be` must equal `[package].newer_version`"
    );
}

#[test]
fn pkgmgr_upgrade_lockfile_diff_bounds_change_to_target() {
    let doc = load_toml(&manifest_path());
    let diff = doc
        .get("lockfile_diff_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[lockfile_diff_assertion]` block \
         (acceptance: \"Upgrade changes only expected package entries.\")",
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
        Some(older),
        "`[lockfile_diff_assertion].must_record_old_version` must equal `[package].older_version`"
    );
    assert_eq!(
        diff.get("must_record_new_version").and_then(|v| v.as_str()),
        Some(newer),
        "`[lockfile_diff_assertion].must_record_new_version` must equal `[package].newer_version`"
    );
    assert_eq!(
        diff.get("must_only_change_target_package")
            .and_then(|v| v.as_str()),
        Some(pkg_name),
        "`[lockfile_diff_assertion].must_only_change_target_package` must equal `[package].name`"
    );
    assert_eq!(
        diff.get("must_not_touch_unrelated")
            .and_then(|v| v.as_str()),
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
fn pkgmgr_upgrade_output_assertion_names_old_and_new_versions() {
    let doc = load_toml(&manifest_path());
    let out = doc
        .get("output_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[output_assertion]` block \
         (acceptance: \"Output names old and new versions.\")",
        );

    for flag in &[
        "must_name_package",
        "must_name_old_version",
        "must_name_new_version",
    ] {
        assert_eq!(
            out.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[output_assertion].{flag}` must be true"
        );
    }
    assert_eq!(
        out.get("output_records_diff_direction")
            .and_then(|v| v.as_str()),
        Some("upgrade"),
        "`[output_assertion].output_records_diff_direction` must be \"upgrade\""
    );
    let old_key = out
        .get("old_version_field_key")
        .and_then(|v| v.as_str())
        .expect("`[output_assertion].old_version_field_key` must be set");
    let new_key = out
        .get("new_version_field_key")
        .and_then(|v| v.as_str())
        .expect("`[output_assertion].new_version_field_key` must be set");
    assert_ne!(old_key, new_key, "old/new version field keys must differ");
}

#[test]
fn pkgmgr_upgrade_wrong_change_guard_pins_all_failure_flags() {
    let doc = load_toml(&manifest_path());
    let guard = doc
        .get("wrong_change_guard")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[wrong_change_guard]` block \
         (acceptance: \"Wrong or missing version change fails the test.\")",
        );

    for flag in &[
        "fail_if_lockfile_records_unchanged_as_change",
        "fail_if_lockfile_records_change_for_unrelated_package",
        "fail_if_old_or_new_version_missing_from_output",
        "diagnostic_must_name_offending_package",
    ] {
        assert_eq!(
            guard.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[wrong_change_guard].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_upgrade_isolation_pins_no_global_state() {
    let doc = load_toml(&manifest_path());
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
fn pkgmgr_upgrade_runner_contract_declares_outcome_keys() {
    let doc = load_toml(&manifest_path());
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
        "from_version",
        "to_version",
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
    for required in &["initial_pin", "upgrade"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must carry `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn pkgmgr_upgrade_pins_out_of_scope_per_issue_2857() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("full_resolver_backtracking_policy")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].full_resolver_backtracking_policy` must be true \
         (issue text: \"Out of scope: full resolver backtracking policy.\")"
    );
}
