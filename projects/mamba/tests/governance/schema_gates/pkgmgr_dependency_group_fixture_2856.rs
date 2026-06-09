//! Schema gate for the package-manager dependency group fixture —
//! closes #2856.
//!
//! Acceptance (issue #2856):
//!
//!   1. Group dependency is included only when requested.
//!      `[default_sync_case]` pins group import as
//!      `module_not_found`; `[group_sync_case]` flips it to
//!      `import_ok` and selects `["docs"]`.
//!   2. Summary names selected groups.
//!      `[summary_assertion]` pins `must_name_selected_groups`,
//!      `must_name_available_groups`, and the field keys.
//!   3. Fixture is fully offline.
//!      Header + `[isolation]` block.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("dependency_group")
        .join("manifest.toml")
}

#[test]
fn pkgmgr_dependency_group_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_dependency_group"),
        "`fixture` must be \"pkgmgr_dependency_group\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2856),
        "`issue` must record #2856"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("dependency_group"),
        "`family` must be \"dependency_group\""
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
fn pkgmgr_dependency_group_blocks_pin_distinct_runtime_and_group_deps() {
    let doc = crate::common::load_toml(&manifest_path());

    let runtime_name = doc
        .get("runtime_dependency")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[runtime_dependency].name` must be set");
    let group_dep = doc
        .get("group")
        .and_then(|v| v.get("dependency"))
        .and_then(|v| v.as_str())
        .expect("`[group].dependency` must be set");
    assert_ne!(
        runtime_name, group_dep,
        "runtime and group dependency names must differ"
    );

    let group_name = doc
        .get("group")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[group].name` must be set");
    assert!(!group_name.is_empty(), "group name must be non-empty");
}

#[test]
fn pkgmgr_dependency_group_default_sync_excludes_group_dep() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc
        .get("default_sync_case")
        .and_then(|v| v.as_table())
        .expect("missing `[default_sync_case]` block");

    let command: Vec<&str> = case
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("sync"),
        "`[default_sync_case].command[0]` must be `sync`"
    );
    assert!(
        !command.iter().any(|s| *s == "--group"),
        "default sync must NOT include `--group`; got {command:?}"
    );

    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[default_sync_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[default_sync_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("must_install_runtime_dependency").and_then(|v| v.as_bool()),
        Some(true),
        "`[default_sync_case].must_install_runtime_dependency` must be true"
    );
    assert_eq!(
        case.get("must_not_install_group_dependency").and_then(|v| v.as_bool()),
        Some(true),
        "`[default_sync_case].must_not_install_group_dependency` must be true"
    );

    let group_dep = doc
        .get("group")
        .and_then(|v| v.get("dependency"))
        .and_then(|v| v.as_str())
        .expect("`[group].dependency` must be set");
    assert_eq!(
        case.get("group_import_probe").and_then(|v| v.as_str()),
        Some(group_dep),
        "`[default_sync_case].group_import_probe` must equal `[group].dependency`"
    );
    assert_eq!(
        case.get("group_expected_import_outcome").and_then(|v| v.as_str()),
        Some("module_not_found"),
        "`[default_sync_case].group_expected_import_outcome` must be \"module_not_found\""
    );

    let selected: Vec<&str> = case
        .get("summary_selected_groups_must_be")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        selected.is_empty(),
        "`[default_sync_case].summary_selected_groups_must_be` must be empty; got {selected:?}"
    );
}

#[test]
fn pkgmgr_dependency_group_sync_case_includes_group_dep() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc
        .get("group_sync_case")
        .and_then(|v| v.as_table())
        .expect("missing `[group_sync_case]` block");

    let command: Vec<&str> = case
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("sync"),
        "`[group_sync_case].command[0]` must be `sync`"
    );
    assert!(
        command.iter().any(|s| *s == "--group"),
        "group sync must include `--group`; got {command:?}"
    );
    let group_name = doc
        .get("group")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[group].name` must be set");
    assert!(
        command.iter().any(|s| *s == group_name),
        "group sync must pass the group name {group_name:?}; got {command:?}"
    );

    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[group_sync_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[group_sync_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("must_install_runtime_dependency").and_then(|v| v.as_bool()),
        Some(true),
        "`[group_sync_case].must_install_runtime_dependency` must be true"
    );
    assert_eq!(
        case.get("must_install_group_dependency").and_then(|v| v.as_bool()),
        Some(true),
        "`[group_sync_case].must_install_group_dependency` must be true"
    );
    assert_eq!(
        case.get("group_expected_import_outcome").and_then(|v| v.as_str()),
        Some("import_ok"),
        "`[group_sync_case].group_expected_import_outcome` must be \"import_ok\""
    );

    let selected: Vec<&str> = case
        .get("summary_selected_groups_must_be")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        selected, vec![group_name],
        "`[group_sync_case].summary_selected_groups_must_be` must equal [{group_name:?}]; got {selected:?}"
    );
}

#[test]
fn pkgmgr_dependency_group_lockfile_separates_group_section() {
    let doc = crate::common::load_toml(&manifest_path());
    let lock = doc
        .get("lockfile_assertion")
        .and_then(|v| v.as_table())
        .expect("missing `[lockfile_assertion]` block");

    let runtime_name = doc
        .get("runtime_dependency")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[runtime_dependency].name` must be set");
    let group_dep = doc
        .get("group")
        .and_then(|v| v.get("dependency"))
        .and_then(|v| v.as_str())
        .expect("`[group].dependency` must be set");
    let group_name = doc
        .get("group")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[group].name` must be set");

    assert_eq!(
        lock.get("must_contain_runtime_dependency").and_then(|v| v.as_str()),
        Some(runtime_name),
        "`[lockfile_assertion].must_contain_runtime_dependency` must equal `[runtime_dependency].name`"
    );
    assert_eq!(
        lock.get("must_contain_group_dependency").and_then(|v| v.as_str()),
        Some(group_dep),
        "`[lockfile_assertion].must_contain_group_dependency` must equal `[group].dependency`"
    );
    assert_eq!(
        lock.get("group_section_records_group_name").and_then(|v| v.as_str()),
        Some(group_name),
        "`[lockfile_assertion].group_section_records_group_name` must equal `[group].name`"
    );
    assert_eq!(
        lock.get("runtime_section_must_not_contain").and_then(|v| v.as_str()),
        Some(group_dep),
        "`[lockfile_assertion].runtime_section_must_not_contain` must equal `[group].dependency`"
    );

    let group_section_key = lock
        .get("must_contain_group_section")
        .and_then(|v| v.as_str())
        .expect("`[lockfile_assertion].must_contain_group_section` must be set");
    assert!(
        !group_section_key.is_empty(),
        "group section key must be non-empty; got {group_section_key:?}"
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
fn pkgmgr_dependency_group_summary_names_selected_and_available_groups() {
    let doc = crate::common::load_toml(&manifest_path());
    let summary = doc.get("summary_assertion").and_then(|v| v.as_table()).expect(
        "missing `[summary_assertion]` block \
         (acceptance: \"Summary names selected groups.\")",
    );

    for flag in &[
        "must_name_selected_groups",
        "must_name_available_groups",
        "must_name_runtime_dependency_count",
        "must_name_group_dependency_count",
    ] {
        assert_eq!(
            summary.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[summary_assertion].{flag}` must be true"
        );
    }

    let selected_key = summary
        .get("selected_groups_field_key")
        .and_then(|v| v.as_str())
        .expect("`[summary_assertion].selected_groups_field_key` must be set");
    let available_key = summary
        .get("available_groups_field_key")
        .and_then(|v| v.as_str())
        .expect("`[summary_assertion].available_groups_field_key` must be set");
    assert_ne!(
        selected_key, available_key,
        "selected and available group field keys must differ"
    );
}

#[test]
fn pkgmgr_dependency_group_isolation_pins_no_global_state() {
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
fn pkgmgr_dependency_group_runner_contract_declares_outcome_keys() {
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
        "selected_groups",
        "available_groups",
        "runtime_dependency",
        "group_dependency",
        "runtime_import_outcome",
        "group_import_outcome",
        "exit_code",
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
    for required in &["default_sync", "group_sync"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must carry `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn pkgmgr_dependency_group_pins_out_of_scope_per_issue_2856() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("every_group_selection_cli_variant").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].every_group_selection_cli_variant` must be true \
         (issue text: \"Out of scope: every group-selection CLI variant.\")"
    );
}
