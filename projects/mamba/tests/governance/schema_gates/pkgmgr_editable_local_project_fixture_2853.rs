//! Schema gate for the package-manager editable local project
//! fixture — closes #2853.
//!
//! Acceptance (issue #2853):
//!
//!   1. Editable behavior status is visible in the package-manager
//!      summary.
//!      `[summary_assertion]` pins install_kind, editable flag,
//!      local path, and status.
//!   2. Passing path proves import reflects a source change without
//!      reinstall.
//!      `[live_reload_case]` pins distinct sentinel values across a
//!      single edit, with `no_reinstall_between_imports = true`.
//!   3. Unsupported behavior is linked to a blocker and not counted
//!      as pass.
//!      `[unsupported_behavior_contract]` pins blocked outcome,
//!      blocker diagnostic, linked tracking issue, no silent pass.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("pkgmgr")
        .join("editable_local_project")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn pkgmgr_editable_local_project_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_editable_local_project"),
        "`fixture` must be \"pkgmgr_editable_local_project\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2853),
        "`issue` must record #2853"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("editable_local_project"),
        "`family` must be \"editable_local_project\""
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
    assert_eq!(
        doc.get("index_source").and_then(|v| v.as_str()),
        Some("direct_local_project"),
        "`index_source` must be \"direct_local_project\""
    );
}

#[test]
fn pkgmgr_editable_local_project_block_pins_distinct_sentinel_values() {
    let doc = load_toml(&manifest_path());
    let proj = doc
        .get("local_project")
        .and_then(|v| v.as_table())
        .expect("missing `[local_project]` block");

    let name = proj
        .get("name")
        .and_then(|v| v.as_str())
        .expect("`[local_project].name` must be set");
    assert!(!name.is_empty(), "local project name must be non-empty");
    let module = proj
        .get("module_name")
        .and_then(|v| v.as_str())
        .expect("`[local_project].module_name` must be set");
    assert_eq!(
        module, name,
        "MVP keeps `module_name` aligned with `name` — single-module project"
    );

    let initial = proj
        .get("initial_sentinel_value")
        .and_then(|v| v.as_str())
        .expect("`[local_project].initial_sentinel_value` must be set");
    let mutated = proj
        .get("mutated_sentinel_value")
        .and_then(|v| v.as_str())
        .expect("`[local_project].mutated_sentinel_value` must be set");
    assert_ne!(
        initial, mutated,
        "initial and mutated sentinel values must differ — live-reload proof"
    );

    let relative_path = proj
        .get("relative_path")
        .and_then(|v| v.as_str())
        .expect("`[local_project].relative_path` must be set");
    assert!(
        !Path::new(relative_path).is_absolute(),
        "`[local_project].relative_path` must be project-relative; got {relative_path:?}"
    );
}

#[test]
fn pkgmgr_editable_local_project_summary_assertion_names_install_kind() {
    let doc = load_toml(&manifest_path());
    let summary = doc
        .get("summary_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[summary_assertion]` block \
         (acceptance: \"Editable behavior status is visible in the package-manager summary.\")",
        );

    for flag in &[
        "must_name_install_kind",
        "must_name_editable_flag",
        "must_name_local_project_path",
        "must_name_status",
    ] {
        assert_eq!(
            summary.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[summary_assertion].{flag}` must be true"
        );
    }

    assert_eq!(
        summary
            .get("expected_install_kind")
            .and_then(|v| v.as_str()),
        Some("editable"),
        "`[summary_assertion].expected_install_kind` must be \"editable\""
    );
}

#[test]
fn pkgmgr_editable_install_action_uses_editable_flag() {
    let doc = load_toml(&manifest_path());
    let action = doc
        .get("editable_install_action")
        .and_then(|v| v.as_table())
        .expect("missing `[editable_install_action]` block");

    let command: Vec<&str> = action
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("add"),
        "`[editable_install_action].command[0]` must be `add`"
    );
    assert!(
        command.iter().any(|s| *s == "--editable" || *s == "-e"),
        "editable action must include `--editable` or `-e`; got {command:?}"
    );

    let relative_path = doc
        .get("local_project")
        .and_then(|v| v.get("relative_path"))
        .and_then(|v| v.as_str())
        .expect("`[local_project].relative_path` must be set");
    assert!(
        command.iter().any(|s| s.contains(relative_path)),
        "`[editable_install_action].command` must reference `[local_project].relative_path`; got {command:?}"
    );

    assert_eq!(
        action.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[editable_install_action].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        action
            .get("expected_exit_code")
            .and_then(|v| v.as_integer()),
        Some(0),
        "`[editable_install_action].expected_exit_code` must be 0"
    );
    for flag in &["must_install_package", "must_record_editable_source"] {
        assert_eq!(
            action.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[editable_install_action].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_editable_live_reload_case_pins_distinct_imports_without_reinstall() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("live_reload_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[live_reload_case]` block \
         (acceptance: \"Passing path proves import reflects a source change without reinstall.\")",
        );

    let attr = case
        .get("import_probe_attribute")
        .and_then(|v| v.as_str())
        .expect("`[live_reload_case].import_probe_attribute` must be set");
    let proj_attr = doc
        .get("local_project")
        .and_then(|v| v.get("sentinel_attribute"))
        .and_then(|v| v.as_str())
        .expect("`[local_project].sentinel_attribute` must be set");
    assert_eq!(
        attr, proj_attr,
        "`[live_reload_case].import_probe_attribute` must equal `[local_project].sentinel_attribute`"
    );

    let first = case
        .get("first_import_expected_value")
        .and_then(|v| v.as_str())
        .expect("`[live_reload_case].first_import_expected_value` must be set");
    let second = case
        .get("second_import_expected_value")
        .and_then(|v| v.as_str())
        .expect("`[live_reload_case].second_import_expected_value` must be set");
    let proj_initial = doc
        .get("local_project")
        .and_then(|v| v.get("initial_sentinel_value"))
        .and_then(|v| v.as_str())
        .expect("`[local_project].initial_sentinel_value` must be set");
    let proj_mutated = doc
        .get("local_project")
        .and_then(|v| v.get("mutated_sentinel_value"))
        .and_then(|v| v.as_str())
        .expect("`[local_project].mutated_sentinel_value` must be set");
    assert_eq!(
        first, proj_initial,
        "`[live_reload_case].first_import_expected_value` must equal `[local_project].initial_sentinel_value`"
    );
    assert_eq!(
        second, proj_mutated,
        "`[live_reload_case].second_import_expected_value` must equal `[local_project].mutated_sentinel_value`"
    );
    assert_ne!(
        first, second,
        "first and second import values must differ — the test proves live reload"
    );

    for flag in &[
        "mutate_between_imports",
        "no_reinstall_between_imports",
        "must_drop_import_cache_between_reads",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[live_reload_case].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_editable_lockfile_records_editable_source_no_wheel_hash() {
    let doc = load_toml(&manifest_path());
    let lock = doc
        .get("lockfile_assertion")
        .and_then(|v| v.as_table())
        .expect("missing `[lockfile_assertion]` block");

    let proj_name = doc
        .get("local_project")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[local_project].name` must be set");
    let proj_path = doc
        .get("local_project")
        .and_then(|v| v.get("relative_path"))
        .and_then(|v| v.as_str())
        .expect("`[local_project].relative_path` must be set");

    assert_eq!(
        lock.get("must_contain_dependency").and_then(|v| v.as_str()),
        Some(proj_name),
        "`[lockfile_assertion].must_contain_dependency` must equal `[local_project].name`"
    );
    assert_eq!(
        lock.get("must_record_install_kind")
            .and_then(|v| v.as_str()),
        Some("editable"),
        "`[lockfile_assertion].must_record_install_kind` must be \"editable\""
    );
    assert_eq!(
        lock.get("must_record_relative_path").and_then(|v| v.as_str()),
        Some(proj_path),
        "`[lockfile_assertion].must_record_relative_path` must equal `[local_project].relative_path`"
    );
    assert_eq!(
        lock.get("must_not_record_wheel_hash").and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].must_not_record_wheel_hash` must be true — editable installs have no wheel"
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
fn pkgmgr_editable_unsupported_behavior_forbids_silent_pass() {
    let doc = load_toml(&manifest_path());
    let con = doc
        .get("unsupported_behavior_contract")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[unsupported_behavior_contract]` block \
         (acceptance: \"Unsupported behavior is linked to a blocker and not counted as pass.\")",
        );

    assert_eq!(
        con.get("when_editable_unsupported_outcome")
            .and_then(|v| v.as_str()),
        Some("blocked"),
        "`[unsupported_behavior_contract].when_editable_unsupported_outcome` must be \"blocked\""
    );
    for flag in &[
        "must_emit_blocker_diagnostic",
        "must_link_tracking_issue",
        "forbid_silent_pass",
    ] {
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
fn pkgmgr_editable_isolation_pins_no_global_state() {
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
fn pkgmgr_editable_runner_contract_includes_blocked_outcome() {
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
        "project_name",
        "project_path",
        "install_kind",
        "first_sentinel",
        "second_sentinel",
        "reinstall_count",
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
        "`[runner_contract].outcome_values` must include `blocked`; got {outcomes:?}"
    );

    let cases: Vec<&str> = contract
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["editable_install", "live_reload"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must carry `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn pkgmgr_editable_pins_out_of_scope_per_issue_2853() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("publishing_editable_artifacts")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].publishing_editable_artifacts` must be true \
         (issue text: \"Out of scope: publishing editable artifacts.\")"
    );
}
