//! Schema gate for the mambalibs clean command fixture — closes
//! #2674.
//!
//! Acceptance (issue #2674):
//!
//!   1. Clean removes expected artifact paths.
//!      `[clean_case].must_remove_relative_paths` is non-empty and
//!      is a subset of `[artifact_paths].relative_paths`;
//!      `must_leave_artifact_root_empty_or_absent = true`.
//!   2. Clean does not delete source fixture files.
//!      `[source_preservation]` lists at least one path and pins
//!      `must_not_delete_outside_artifact_root = true`;
//!      `[lockfile_policy].must_be_preserved_by_clean = true` and
//!      the lockfile is NOT in `[clean_case].must_remove_relative_paths`.
//!   3. Rebuild after clean still succeeds.
//!      `[rebuild_after_clean_case].must_recreate_relative_paths`
//!      equals `[artifact_paths].relative_paths` as a SET.
//!
//! Cheap test — single TOML read + field walk. Runs in well under
//! a second; stays in the default `cargo test -p mamba` set.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("clean_command")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

fn string_array(doc: &toml::Value, table: &str, key: &str) -> Vec<String> {
    doc.get(table)
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn mambalibs_clean_command_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_clean_command"),
        "`fixture` must be \"mambalibs_clean_command\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2674),
        "`issue` must record #2674"
    );
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2531),
        "`parent_issue` must record #2531"
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`profile` must be \"mambalibs\""
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("clean_command"),
        "`family` must be \"clean_command\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn mambalibs_clean_binding_pins_artifact_and_source_roots() {
    let doc = load_toml(&manifest_path());
    let bind = doc
        .get("binding")
        .and_then(|v| v.as_table())
        .expect("missing `[binding]` block");

    assert_eq!(
        bind.get("module_name").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`[binding].module_name` must be \"mambalibs\""
    );

    let artifact_root = bind
        .get("artifact_root")
        .and_then(|v| v.as_str())
        .expect("`[binding].artifact_root` must be set");
    assert!(
        artifact_root.starts_with("build/"),
        "`[binding].artifact_root` must live under build/; got {artifact_root:?}"
    );

    let source_root = bind
        .get("source_root")
        .and_then(|v| v.as_str())
        .expect("`[binding].source_root` must be set");
    assert!(
        !source_root.is_empty(),
        "`[binding].source_root` must be non-empty"
    );
    assert_ne!(
        source_root, artifact_root,
        "`[binding].source_root` MUST differ from `[binding].artifact_root` — clean must \
         not be allowed to target the source tree"
    );
    assert!(
        !source_root.starts_with("build/"),
        "`[binding].source_root` must NOT live under build/; got {source_root:?}"
    );
}

#[test]
fn mambalibs_clean_artifact_paths_cover_metadata_lock_module_and_native() {
    let doc = load_toml(&manifest_path());
    let paths = string_array(&doc, "artifact_paths", "relative_paths");

    for required in &[
        "metadata.json",
        "mamba.lock",
        "module/__init__.py",
        "module/_mambalibs_native{shared_lib_ext}",
    ] {
        assert!(
            paths.contains(&required.to_string()),
            "`[artifact_paths].relative_paths` must include `{required}`; got {paths:?}"
        );
    }

    assert_eq!(
        doc.get("artifact_paths")
            .and_then(|v| v.get("shared_lib_extension_placeholder"))
            .and_then(|v| v.as_str()),
        Some("{shared_lib_ext}"),
        "`[artifact_paths].shared_lib_extension_placeholder` must be `{{shared_lib_ext}}`"
    );
}

#[test]
fn mambalibs_clean_case_removes_artifacts_but_not_lockfile() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("clean_case")
        .and_then(|v| v.as_table())
        .expect("missing `[clean_case]` block");

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("clean_removes_artifacts"),
        "`[clean_case].case` must be \"clean_removes_artifacts\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[clean_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[clean_case].expected_exit_code` must be 0"
    );

    let command: Vec<&str> = case
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command,
        vec!["clean"],
        "`[clean_case].command` must be [\"clean\"]; got {command:?}"
    );

    // Removed paths MUST be a subset of declared artifact paths.
    let removed = string_array(&doc, "clean_case", "must_remove_relative_paths");
    let artifact_set: HashSet<_> = string_array(&doc, "artifact_paths", "relative_paths")
        .into_iter()
        .collect();
    assert!(
        !removed.is_empty(),
        "`[clean_case].must_remove_relative_paths` must be non-empty"
    );
    for path in &removed {
        assert!(
            artifact_set.contains(path),
            "`[clean_case].must_remove_relative_paths` entry `{path}` must appear in \
             `[artifact_paths].relative_paths`; got artifact set {artifact_set:?}"
        );
    }

    // The lockfile MUST NOT be in the removed set — acceptance:
    // "Clean does not delete source fixture files." We treat the
    // lockfile as source-like.
    let lockfile = doc
        .get("lockfile_policy")
        .and_then(|v| v.get("lockfile_relative_path"))
        .and_then(|v| v.as_str())
        .expect("`[lockfile_policy].lockfile_relative_path` must be set");
    assert!(
        !removed.iter().any(|p| p == lockfile),
        "`[clean_case].must_remove_relative_paths` must NOT include the lockfile `{lockfile}`; \
         got {removed:?}"
    );

    for flag in &[
        "must_leave_artifact_root_empty_or_absent",
        "must_not_delete_lockfile",
        "must_not_touch_source_root",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[clean_case].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_clean_source_preservation_pins_source_tree() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("source_preservation")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[source_preservation]` block \
         (acceptance: \"Clean does not delete source fixture files.\")",
        );

    let preserved = string_array(&doc, "source_preservation", "must_preserve_relative_paths");
    assert!(
        !preserved.is_empty(),
        "`[source_preservation].must_preserve_relative_paths` must be non-empty"
    );
    for path in &preserved {
        assert!(
            !path.starts_with("build/"),
            "preserved source path `{path}` must NOT live under build/"
        );
    }

    for flag in &[
        "must_preserve_source_root",
        "must_not_delete_outside_artifact_root",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[source_preservation].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_clean_lockfile_policy_preserves_lockfile() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("lockfile_policy")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[lockfile_policy]` block \
         (acceptance: \"Clean does not delete source fixture files.\")",
        );

    let lockfile = block
        .get("lockfile_relative_path")
        .and_then(|v| v.as_str())
        .expect("`[lockfile_policy].lockfile_relative_path` must be set");
    assert_eq!(
        lockfile, "mamba.lock",
        "`[lockfile_policy].lockfile_relative_path` must be \"mamba.lock\""
    );

    assert_eq!(
        block
            .get("must_be_preserved_by_clean")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_policy].must_be_preserved_by_clean` must be true"
    );

    let rationale = block
        .get("rationale")
        .and_then(|v| v.as_str())
        .expect("`[lockfile_policy].rationale` must be set");
    assert!(
        !rationale.is_empty(),
        "`[lockfile_policy].rationale` must be non-empty"
    );

    // Cross-check: the lockfile MUST appear in the artifact tree
    // (so that "clean leaves it behind" is a meaningful claim).
    let artifact_set: HashSet<_> = string_array(&doc, "artifact_paths", "relative_paths")
        .into_iter()
        .collect();
    assert!(
        artifact_set.contains(lockfile),
        "lockfile `{lockfile}` must appear in `[artifact_paths].relative_paths` so the \
         preservation claim is meaningful; got {artifact_set:?}"
    );
}

#[test]
fn mambalibs_clean_rebuild_after_clean_recreates_full_artifact_set() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("rebuild_after_clean_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[rebuild_after_clean_case]` block \
         (acceptance: \"Rebuild after clean still succeeds.\")",
        );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("rebuild_after_clean"),
        "`[rebuild_after_clean_case].case` must be \"rebuild_after_clean\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[rebuild_after_clean_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[rebuild_after_clean_case].expected_exit_code` must be 0"
    );

    let command: Vec<&str> = case
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command,
        vec!["build"],
        "`[rebuild_after_clean_case].command` must be [\"build\"]; got {command:?}"
    );

    for flag in &[
        "must_recreate_all_expected_files",
        "must_match_artifact_paths_set",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[rebuild_after_clean_case].{flag}` must be true"
        );
    }

    // Rebuild set MUST equal the full artifact set.
    let recreated: HashSet<_> = string_array(
        &doc,
        "rebuild_after_clean_case",
        "must_recreate_relative_paths",
    )
    .into_iter()
    .collect();
    let artifact_set: HashSet<_> = string_array(&doc, "artifact_paths", "relative_paths")
        .into_iter()
        .collect();
    assert_eq!(
        recreated, artifact_set,
        "`[rebuild_after_clean_case].must_recreate_relative_paths` must equal \
         `[artifact_paths].relative_paths` as a set"
    );
}

#[test]
fn mambalibs_clean_diagnostic_contract_pins_field_keys() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("diagnostic_contract")
        .and_then(|v| v.as_table())
        .expect("missing `[diagnostic_contract]` block");

    for flag in &[
        "diagnostic_must_name_artifact_root",
        "diagnostic_must_be_deterministic",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[diagnostic_contract].{flag}` must be true"
        );
    }

    let removed_key = block
        .get("diagnostic_must_name_removed_paths_field_key")
        .and_then(|v| v.as_str())
        .expect("`[diagnostic_contract].diagnostic_must_name_removed_paths_field_key` must be set");
    let preserved_lockfile_key = block
        .get("diagnostic_must_name_preserved_lockfile_field_key")
        .and_then(|v| v.as_str())
        .expect(
            "`[diagnostic_contract].diagnostic_must_name_preserved_lockfile_field_key` must be set",
        );

    let contract_keys: Vec<String> = string_array(&doc, "runner_contract", "keys");
    assert!(
        contract_keys.iter().any(|k| k == removed_key),
        "`[runner_contract].keys` must include the removed-paths key `{removed_key}`; got \
         {contract_keys:?}"
    );
    assert!(
        contract_keys.iter().any(|k| k == preserved_lockfile_key),
        "`[runner_contract].keys` must include the preserved-lockfile key \
         `{preserved_lockfile_key}`; got {contract_keys:?}"
    );
}

#[test]
fn mambalibs_clean_isolation_pins_no_global_state() {
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
fn mambalibs_clean_runner_contract_declares_keys_and_cases() {
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
        "artifact_root",
        "source_root",
        "removed_paths",
        "preserved_paths",
        "preserved_lockfile",
        "files_present_after_clean",
        "files_present_after_rebuild",
        "diagnostic_message",
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
    for required in &["clean_removes_artifacts", "rebuild_after_clean"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_clean_pins_out_of_scope_per_issue_2674() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("global_cache_cleanup").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].global_cache_cleanup` must be true \
         (issue text: \"Out of scope: global cache cleanup.\")"
    );
}
