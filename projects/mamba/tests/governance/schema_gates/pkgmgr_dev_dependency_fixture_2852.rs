//! Schema gate for the package-manager dev dependency fixture —
//! closes #2852.
//!
//! Acceptance (issue #2852):
//!
//!   1. Dev dependency is recorded distinctly from runtime
//!      dependency.
//!      `[lockfile_assertion]` pins separate runtime + dev sections
//!      and asserts no cross-contamination.
//!   2. Runtime-only install does not expose dev dependency unless
//!      policy says otherwise.
//!      `[runtime_only_sync_case]` asserts dev import resolves to
//!      `module_not_found`.
//!   3. Unsupported dev dependency behavior is a linked blocker,
//!      not silent pass.
//!      `[unsupported_behavior_contract]` pins `outcome = "blocked"`,
//!      `must_emit_blocker_diagnostic`, `must_link_tracking_issue`,
//!      and `forbid_silent_pass`.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("dev_dependency")
        .join("manifest.toml")
}

#[test]
fn pkgmgr_dev_dependency_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_dev_dependency"),
        "`fixture` must be \"pkgmgr_dev_dependency\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2852),
        "`issue` must record #2852"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("dev_dependency"),
        "`family` must be \"dev_dependency\""
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
fn pkgmgr_dev_dependency_dependencies_block_pins_distinct_runtime_and_dev() {
    let doc = crate::common::load_toml(&manifest_path());
    let deps = doc
        .get("dependencies")
        .and_then(|v| v.as_table())
        .expect("missing `[dependencies]` block");

    let runtime_name = deps
        .get("runtime_name")
        .and_then(|v| v.as_str())
        .expect("`[dependencies].runtime_name` must be set");
    let dev_name = deps
        .get("dev_name")
        .and_then(|v| v.as_str())
        .expect("`[dependencies].dev_name` must be set");
    assert_ne!(
        runtime_name, dev_name,
        "runtime and dev names must differ — gate compares both ways"
    );

    let runtime_version = deps
        .get("runtime_version")
        .and_then(|v| v.as_str())
        .expect("`[dependencies].runtime_version` must be set");
    let dev_version = deps
        .get("dev_version")
        .and_then(|v| v.as_str())
        .expect("`[dependencies].dev_version` must be set");
    assert!(!runtime_version.is_empty(), "runtime version must be non-empty");
    assert!(!dev_version.is_empty(), "dev version must be non-empty");
}

#[test]
fn pkgmgr_dev_dependency_add_dev_action_uses_dev_flag() {
    let doc = crate::common::load_toml(&manifest_path());
    let action = doc
        .get("add_dev_action")
        .and_then(|v| v.as_table())
        .expect("missing `[add_dev_action]` block");

    let command: Vec<&str> = action
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("add"),
        "`[add_dev_action].command[0]` must be `add`"
    );
    assert!(
        command.iter().any(|s| *s == "--dev"),
        "`[add_dev_action].command` must include `--dev` flag; got {command:?}"
    );

    let dev_name = doc
        .get("dependencies")
        .and_then(|v| v.get("dev_name"))
        .and_then(|v| v.as_str())
        .expect("`[dependencies].dev_name` must be set");
    assert!(
        command.iter().any(|s| s.contains(dev_name)),
        "`[add_dev_action].command` must reference the dev dep name; got {command:?}"
    );

    assert_eq!(
        action.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[add_dev_action].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        action.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[add_dev_action].expected_exit_code` must be 0"
    );
    assert_eq!(
        action.get("must_add_dev_section").and_then(|v| v.as_bool()),
        Some(true),
        "`[add_dev_action].must_add_dev_section` must be true"
    );
}

#[test]
fn pkgmgr_dev_dependency_lockfile_separates_runtime_from_dev() {
    let doc = crate::common::load_toml(&manifest_path());
    let lock = doc.get("lockfile_assertion").and_then(|v| v.as_table()).expect(
        "missing `[lockfile_assertion]` block \
         (acceptance: \"Dev dependency is recorded distinctly from runtime dependency.\")",
    );

    let runtime_name = doc
        .get("dependencies")
        .and_then(|v| v.get("runtime_name"))
        .and_then(|v| v.as_str())
        .expect("`[dependencies].runtime_name` must be set");
    let dev_name = doc
        .get("dependencies")
        .and_then(|v| v.get("dev_name"))
        .and_then(|v| v.as_str())
        .expect("`[dependencies].dev_name` must be set");

    assert_eq!(
        lock.get("must_contain_runtime_dependency").and_then(|v| v.as_str()),
        Some(runtime_name),
        "`[lockfile_assertion].must_contain_runtime_dependency` must equal `[dependencies].runtime_name`"
    );
    assert_eq!(
        lock.get("must_contain_dev_dependency").and_then(|v| v.as_str()),
        Some(dev_name),
        "`[lockfile_assertion].must_contain_dev_dependency` must equal `[dependencies].dev_name`"
    );
    assert_eq!(
        lock.get("runtime_section_must_not_contain").and_then(|v| v.as_str()),
        Some(dev_name),
        "`[lockfile_assertion].runtime_section_must_not_contain` must equal `[dependencies].dev_name`"
    );
    assert_eq!(
        lock.get("dev_section_must_not_contain").and_then(|v| v.as_str()),
        Some(runtime_name),
        "`[lockfile_assertion].dev_section_must_not_contain` must equal `[dependencies].runtime_name`"
    );

    let runtime_key = lock
        .get("runtime_section_key")
        .and_then(|v| v.as_str())
        .expect("`[lockfile_assertion].runtime_section_key` must be set");
    let dev_key = lock
        .get("dev_section_key")
        .and_then(|v| v.as_str())
        .expect("`[lockfile_assertion].dev_section_key` must be set");
    assert_ne!(
        runtime_key, dev_key,
        "runtime and dev section keys must differ — distinct lockfile categories"
    );

    for flag in &["deterministic", "byte_identical_on_replay"] {
        assert_eq!(
            lock.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[lockfile_assertion].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_dev_dependency_runtime_only_sync_excludes_dev_dep() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc.get("runtime_only_sync_case").and_then(|v| v.as_table()).expect(
        "missing `[runtime_only_sync_case]` block \
         (acceptance: \"Runtime-only install does not expose dev dependency unless policy says otherwise.\")",
    );

    let command: Vec<&str> = case
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("sync"),
        "`[runtime_only_sync_case].command[0]` must be `sync`"
    );
    assert!(
        !command.iter().any(|s| *s == "--dev"),
        "runtime-only sync must NOT include `--dev`; got {command:?}"
    );

    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[runtime_only_sync_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[runtime_only_sync_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("must_install_runtime_dependency").and_then(|v| v.as_bool()),
        Some(true),
        "`[runtime_only_sync_case].must_install_runtime_dependency` must be true"
    );
    assert_eq!(
        case.get("must_not_install_dev_dependency").and_then(|v| v.as_bool()),
        Some(true),
        "`[runtime_only_sync_case].must_not_install_dev_dependency` must be true"
    );
    assert_eq!(
        case.get("runtime_expected_import_outcome").and_then(|v| v.as_str()),
        Some("import_ok"),
        "`[runtime_only_sync_case].runtime_expected_import_outcome` must be \"import_ok\""
    );
    assert_eq!(
        case.get("dev_expected_import_outcome").and_then(|v| v.as_str()),
        Some("module_not_found"),
        "`[runtime_only_sync_case].dev_expected_import_outcome` must be \"module_not_found\""
    );
}

#[test]
fn pkgmgr_dev_dependency_dev_sync_case_uses_dev_flag() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc
        .get("dev_sync_case")
        .and_then(|v| v.as_table())
        .expect("missing `[dev_sync_case]` block");

    let command: Vec<&str> = case
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("sync"),
        "`[dev_sync_case].command[0]` must be `sync`"
    );
    assert!(
        command.iter().any(|s| *s == "--dev"),
        "dev sync must include `--dev`; got {command:?}"
    );

    assert_eq!(
        case.get("supported").and_then(|v| v.as_bool()),
        Some(true),
        "`[dev_sync_case].supported` must be true — opt-in dev install is part of MVP"
    );
    assert_eq!(
        case.get("must_install_runtime_dependency").and_then(|v| v.as_bool()),
        Some(true),
        "`[dev_sync_case].must_install_runtime_dependency` must be true"
    );
    assert_eq!(
        case.get("must_install_dev_dependency").and_then(|v| v.as_bool()),
        Some(true),
        "`[dev_sync_case].must_install_dev_dependency` must be true"
    );
    assert_eq!(
        case.get("dev_expected_import_outcome").and_then(|v| v.as_str()),
        Some("import_ok"),
        "`[dev_sync_case].dev_expected_import_outcome` must be \"import_ok\""
    );
}

#[test]
fn pkgmgr_dev_dependency_unsupported_behavior_forbids_silent_pass() {
    let doc = crate::common::load_toml(&manifest_path());
    let con = doc.get("unsupported_behavior_contract").and_then(|v| v.as_table()).expect(
        "missing `[unsupported_behavior_contract]` block \
         (acceptance: \"Unsupported dev dependency behavior is a linked blocker, not silent pass.\")",
    );

    assert_eq!(
        con.get("when_dev_sync_unsupported_outcome").and_then(|v| v.as_str()),
        Some("blocked"),
        "`[unsupported_behavior_contract].when_dev_sync_unsupported_outcome` must be \"blocked\""
    );
    for flag in &["must_emit_blocker_diagnostic", "must_link_tracking_issue", "forbid_silent_pass"] {
        assert_eq!(
            con.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[unsupported_behavior_contract].{flag}` must be true"
        );
    }

    let linked = con
        .get("linked_blocker_issue")
        .and_then(|v| v.as_integer())
        .expect("`[unsupported_behavior_contract].linked_blocker_issue` must be set");
    assert!(
        linked > 0,
        "`[unsupported_behavior_contract].linked_blocker_issue` must be a positive issue number; got {linked}"
    );
}

#[test]
fn pkgmgr_dev_dependency_isolation_pins_no_global_state() {
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
fn pkgmgr_dev_dependency_runner_contract_includes_blocked_outcome() {
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
        "runtime_dependency",
        "dev_dependency",
        "lockfile_runtime_section_contains_dev",
        "lockfile_dev_section_contains_dev",
        "runtime_import_outcome",
        "dev_import_outcome",
        "exit_code",
        "blocker_issue",
    ] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }

    let outcomes: Vec<&str> = contract
        .get("outcome_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        outcomes.contains(&"blocked"),
        "`[runner_contract].outcome_values` must include `blocked` — required by unsupported-behavior contract; got {outcomes:?}"
    );

    let cases: Vec<&str> = contract
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["add_dev", "runtime_only_sync", "dev_sync"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must carry `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn pkgmgr_dev_dependency_pins_out_of_scope_per_issue_2852() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("full_dependency_group_ux").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].full_dependency_group_ux` must be true \
         (issue text: \"Out of scope: full dependency group UX.\")"
    );
}
