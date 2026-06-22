//! Schema gate for the package-manager workspace member fixture —
//! closes #2854.
//!
//! Acceptance (issue #2854):
//!
//!   1. Workspace status appears in package-manager summary.
//!      `[summary_assertion]` pins workspace_kind, member_count, the
//!      per-member name flag, and status.
//!   2. Passing path proves one member can depend on another.
//!      `[cross_member_dependency_case]` pins direct + indirect
//!      imports with the indirect resolving via workspace, not index.
//!   3. Unsupported path is linked and not counted as pass.
//!      `[unsupported_behavior_contract]` pins blocked outcome and
//!      forbids silent pass.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("workspace_member")
        .join("manifest.toml")
}

#[test]
fn pkgmgr_workspace_member_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_workspace_member"),
        "`fixture` must be \"pkgmgr_workspace_member\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2854),
        "`issue` must record #2854"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("workspace_member"),
        "`family` must be \"workspace_member\""
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
        Some("workspace_local"),
        "`index_source` must be \"workspace_local\""
    );
}

#[test]
fn pkgmgr_workspace_member_blocks_pin_two_distinct_members() {
    let doc = crate::common::load_toml(&manifest_path());

    let workspace = doc
        .get("workspace")
        .and_then(|v| v.as_table())
        .expect("missing `[workspace]` block");
    let ws_path = workspace
        .get("relative_path")
        .and_then(|v| v.as_str())
        .expect("`[workspace].relative_path` must be set");
    assert!(
        !Path::new(ws_path).is_absolute(),
        "`[workspace].relative_path` must be project-relative; got {ws_path:?}"
    );
    let glob = workspace
        .get("members_glob")
        .and_then(|v| v.as_str())
        .expect("`[workspace].members_glob` must be set");
    assert!(
        glob.contains('*'),
        "`[workspace].members_glob` must be a glob pattern; got {glob:?}"
    );

    let a = doc
        .get("member_a")
        .and_then(|v| v.as_table())
        .expect("missing `[member_a]` block");
    let b = doc
        .get("member_b")
        .and_then(|v| v.as_table())
        .expect("missing `[member_b]` block");

    let a_name = a
        .get("name")
        .and_then(|v| v.as_str())
        .expect("`[member_a].name` must be set");
    let b_name = b
        .get("name")
        .and_then(|v| v.as_str())
        .expect("`[member_b].name` must be set");
    assert_ne!(a_name, b_name, "member names must differ");

    let a_path = a
        .get("relative_path")
        .and_then(|v| v.as_str())
        .expect("`[member_a].relative_path` must be set");
    let b_path = b
        .get("relative_path")
        .and_then(|v| v.as_str())
        .expect("`[member_b].relative_path` must be set");
    assert!(
        a_path.starts_with(ws_path) && b_path.starts_with(ws_path),
        "member paths must be inside workspace root {ws_path:?}; got {a_path:?} and {b_path:?}"
    );
    assert_ne!(a_path, b_path, "member paths must differ");
}

#[test]
fn pkgmgr_workspace_member_dependency_block_pins_local_resolution() {
    let doc = crate::common::load_toml(&manifest_path());
    let dep = doc
        .get("member_dependency")
        .and_then(|v| v.as_table())
        .expect("missing `[member_dependency]` block");

    let declarant = dep
        .get("declarant")
        .and_then(|v| v.as_str())
        .expect("`[member_dependency].declarant` must be set");
    let target = dep
        .get("target")
        .and_then(|v| v.as_str())
        .expect("`[member_dependency].target` must be set");
    assert_ne!(declarant, target, "declarant and target must differ");

    let a_name = doc
        .get("member_a")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[member_a].name` must be set");
    let b_name = doc
        .get("member_b")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[member_b].name` must be set");
    assert_eq!(
        declarant, a_name,
        "`[member_dependency].declarant` must equal `[member_a].name`"
    );
    assert_eq!(
        target, b_name,
        "`[member_dependency].target` must equal `[member_b].name`"
    );

    let symbol = dep
        .get("imported_symbol")
        .and_then(|v| v.as_str())
        .expect("`[member_dependency].imported_symbol` must be set");
    let b_symbol = doc
        .get("member_b")
        .and_then(|v| v.get("exposes_symbol"))
        .and_then(|v| v.as_str())
        .expect("`[member_b].exposes_symbol` must be set");
    assert_eq!(
        symbol, b_symbol,
        "`[member_dependency].imported_symbol` must equal `[member_b].exposes_symbol`"
    );

    let expected = dep
        .get("expected_imported_value")
        .and_then(|v| v.as_str())
        .expect("`[member_dependency].expected_imported_value` must be set");
    let b_value = doc
        .get("member_b")
        .and_then(|v| v.get("exposed_value"))
        .and_then(|v| v.as_str())
        .expect("`[member_b].exposed_value` must be set");
    assert_eq!(
        expected, b_value,
        "`[member_dependency].expected_imported_value` must equal `[member_b].exposed_value`"
    );
}

#[test]
fn pkgmgr_workspace_member_summary_assertion_names_workspace_status() {
    let doc = crate::common::load_toml(&manifest_path());
    let summary = doc
        .get("summary_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[summary_assertion]` block \
         (acceptance: \"Workspace status appears in package-manager summary.\")",
        );

    for flag in &[
        "must_name_workspace_kind",
        "must_name_member_count",
        "must_name_each_member",
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
            .get("expected_workspace_kind")
            .and_then(|v| v.as_str()),
        Some("local"),
        "`[summary_assertion].expected_workspace_kind` must be \"local\""
    );
    assert_eq!(
        summary
            .get("expected_member_count")
            .and_then(|v| v.as_integer()),
        Some(2),
        "`[summary_assertion].expected_member_count` must be 2"
    );
}

#[test]
fn pkgmgr_workspace_sync_action_discovers_both_members() {
    let doc = crate::common::load_toml(&manifest_path());
    let action = doc
        .get("workspace_sync_action")
        .and_then(|v| v.as_table())
        .expect("missing `[workspace_sync_action]` block");

    let command: Vec<&str> = action
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("sync"),
        "`[workspace_sync_action].command[0]` must be `sync`"
    );

    for flag in &["must_discover_member_a", "must_discover_member_b"] {
        assert_eq!(
            action.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[workspace_sync_action].{flag}` must be true"
        );
    }
    assert_eq!(
        action
            .get("must_record_member_count")
            .and_then(|v| v.as_integer()),
        Some(2),
        "`[workspace_sync_action].must_record_member_count` must be 2"
    );
}

#[test]
fn pkgmgr_workspace_cross_member_dependency_resolves_via_workspace() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc
        .get("cross_member_dependency_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[cross_member_dependency_case]` block \
         (acceptance: \"Passing path proves one member can depend on another.\")",
        );

    let probe_mod = case
        .get("import_probe_module")
        .and_then(|v| v.as_str())
        .expect("`[cross_member_dependency_case].import_probe_module` must be set");
    let a_name = doc
        .get("member_a")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[member_a].name` must be set");
    assert_eq!(
        probe_mod, a_name,
        "`[cross_member_dependency_case].import_probe_module` must equal `[member_a].name`"
    );

    let indirect_mod = case
        .get("indirect_import_module")
        .and_then(|v| v.as_str())
        .expect("`[cross_member_dependency_case].indirect_import_module` must be set");
    let b_name = doc
        .get("member_b")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[member_b].name` must be set");
    assert_eq!(
        indirect_mod, b_name,
        "`[cross_member_dependency_case].indirect_import_module` must equal `[member_b].name`"
    );

    let indirect_val = case
        .get("indirect_expected_value")
        .and_then(|v| v.as_str())
        .expect("`[cross_member_dependency_case].indirect_expected_value` must be set");
    let b_val = doc
        .get("member_b")
        .and_then(|v| v.get("exposed_value"))
        .and_then(|v| v.as_str())
        .expect("`[member_b].exposed_value` must be set");
    assert_eq!(
        indirect_val, b_val,
        "`[cross_member_dependency_case].indirect_expected_value` must equal `[member_b].exposed_value`"
    );

    assert_eq!(
        case.get("resolves_via_workspace_not_index")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[cross_member_dependency_case].resolves_via_workspace_not_index` must be true"
    );
}

#[test]
fn pkgmgr_workspace_lockfile_records_workspace_kind_and_members() {
    let doc = crate::common::load_toml(&manifest_path());
    let lock = doc
        .get("lockfile_assertion")
        .and_then(|v| v.as_table())
        .expect("missing `[lockfile_assertion]` block");

    let a_name = doc
        .get("member_a")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[member_a].name` must be set");
    let b_name = doc
        .get("member_b")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[member_b].name` must be set");

    assert_eq!(
        lock.get("must_record_workspace_kind")
            .and_then(|v| v.as_str()),
        Some("local"),
        "`[lockfile_assertion].must_record_workspace_kind` must be \"local\""
    );
    assert_eq!(
        lock.get("must_record_member_count")
            .and_then(|v| v.as_integer()),
        Some(2),
        "`[lockfile_assertion].must_record_member_count` must be 2"
    );
    assert_eq!(
        lock.get("must_contain_dependency_a")
            .and_then(|v| v.as_str()),
        Some(a_name),
        "`[lockfile_assertion].must_contain_dependency_a` must equal `[member_a].name`"
    );
    assert_eq!(
        lock.get("must_contain_dependency_b")
            .and_then(|v| v.as_str()),
        Some(b_name),
        "`[lockfile_assertion].must_contain_dependency_b` must equal `[member_b].name`"
    );
    for flag in &[
        "deterministic",
        "byte_identical_on_replay",
        "must_record_member_relative_paths",
        "must_not_resolve_members_via_index",
    ] {
        assert_eq!(
            lock.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[lockfile_assertion].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_workspace_unsupported_behavior_forbids_silent_pass() {
    let doc = crate::common::load_toml(&manifest_path());
    let con = doc
        .get("unsupported_behavior_contract")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[unsupported_behavior_contract]` block \
         (acceptance: \"Unsupported path is linked and not counted as pass.\")",
        );

    assert_eq!(
        con.get("when_workspace_unsupported_outcome")
            .and_then(|v| v.as_str()),
        Some("blocked"),
        "`[unsupported_behavior_contract].when_workspace_unsupported_outcome` must be \"blocked\""
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
fn pkgmgr_workspace_isolation_pins_no_global_state() {
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
fn pkgmgr_workspace_runner_contract_includes_blocked_outcome() {
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
        "workspace_kind",
        "member_count",
        "members",
        "import_outcome",
        "indirect_import_outcome",
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
    for required in &["workspace_sync", "cross_member_dependency"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must carry `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn pkgmgr_workspace_pins_out_of_scope_per_issue_2854() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("monorepo_publishing_behavior")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].monorepo_publishing_behavior` must be true \
         (issue text: \"Out of scope: monorepo publishing behavior.\")"
    );
}
