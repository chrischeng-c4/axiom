//! Schema gate for the mambalibs local path override fixture —
//! closes #2671.
//!
//! Acceptance (issue #2671):
//!
//!   1. Missing or wrong override path fails clearly.
//!      `[missing_override_path_case]` and
//!      `[wrong_override_path_case]` each pin `fail` outcome with
//!      named diagnostics; both forbid silent registry fall-back.
//!   2. Lockfile records the local path dependency identity
//!      deterministically.
//!      `[lockfile_assertion]` pins source_kind = "local_path",
//!      records the override path, `must_be_deterministic` and
//!      `must_be_byte_identical_on_replay`.
//!   3. Import result proves the override was used.
//!      `[override_apply_case]` pins `import_probe_value_must_be ==
//!      local_override_sentinel_value` and
//!      `import_probe_value_must_not_be ==
//!      registry_default_sentinel_value`.
//!
//! Cheap test — single TOML read + field walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("local_path_override")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_local_path_override_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_local_path_override"),
        "`fixture` must be \"mambalibs_local_path_override\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2671),
        "`issue` must record #2671"
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
        Some("local_path_override"),
        "`family` must be \"local_path_override\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn mambalibs_local_path_override_binding_pins_distinct_sentinels() {
    let doc = load_toml(&manifest_path());
    let bind = doc
        .get("binding")
        .and_then(|v| v.as_table())
        .expect("missing `[binding]` block");

    let registry = bind
        .get("registry_default_sentinel_value")
        .and_then(|v| v.as_str())
        .expect("`[binding].registry_default_sentinel_value` must be set");
    let local = bind
        .get("local_override_sentinel_value")
        .and_then(|v| v.as_str())
        .expect("`[binding].local_override_sentinel_value` must be set");
    assert_ne!(
        registry, local,
        "registry-default and local-override sentinel values MUST differ \
         — the import probe distinguishes which source was used"
    );

    let module = bind
        .get("module_name")
        .and_then(|v| v.as_str())
        .expect("`[binding].module_name` must be set");
    assert_eq!(module, "mambalibs", "module must be \"mambalibs\"");
}

#[test]
fn mambalibs_local_path_override_block_uses_local_path_kind() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("override")
        .and_then(|v| v.as_table())
        .expect("missing `[override]` block");

    assert_eq!(
        block.get("override_kind").and_then(|v| v.as_str()),
        Some("local_path"),
        "`[override].override_kind` must be \"local_path\""
    );

    let rel = block
        .get("relative_path")
        .and_then(|v| v.as_str())
        .expect("`[override].relative_path` must be set");
    assert!(
        rel.starts_with("./") || rel.starts_with("../"),
        "relative_path must be a relative file path (start with ./ or ../); got {rel:?}"
    );
    assert_eq!(
        block
            .get("must_not_escape_project_tree")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[override].must_not_escape_project_tree` must be true"
    );

    let abs = block
        .get("absolute_path_template")
        .and_then(|v| v.as_str())
        .expect("`[override].absolute_path_template` must be set");
    assert!(
        abs.contains("${FIXTURE_TMPDIR}"),
        "absolute_path_template must live under ${{FIXTURE_TMPDIR}}; got {abs:?}"
    );
}

#[test]
fn mambalibs_local_path_override_apply_case_imports_override_sentinel() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("override_apply_case")
        .and_then(|v| v.as_table())
        .expect("missing `[override_apply_case]` block");

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("override_applied"),
        "`[override_apply_case].case` must be \"override_applied\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[override_apply_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[override_apply_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("must_resolve_via_override")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[override_apply_case].must_resolve_via_override` must be true"
    );

    let local = doc
        .get("binding")
        .and_then(|v| v.get("local_override_sentinel_value"))
        .and_then(|v| v.as_str())
        .expect("`[binding].local_override_sentinel_value` must be set");
    let registry = doc
        .get("binding")
        .and_then(|v| v.get("registry_default_sentinel_value"))
        .and_then(|v| v.as_str())
        .expect("`[binding].registry_default_sentinel_value` must be set");

    assert_eq!(
        case.get("import_probe_value_must_be")
            .and_then(|v| v.as_str()),
        Some(local),
        "`[override_apply_case].import_probe_value_must_be` must equal \
         `[binding].local_override_sentinel_value`"
    );
    assert_eq!(
        case.get("import_probe_value_must_not_be")
            .and_then(|v| v.as_str()),
        Some(registry),
        "`[override_apply_case].import_probe_value_must_not_be` must equal \
         `[binding].registry_default_sentinel_value`"
    );
    assert_eq!(
        case.get("must_record_source_kind").and_then(|v| v.as_str()),
        Some("local_path"),
        "`[override_apply_case].must_record_source_kind` must be \"local_path\""
    );
}

#[test]
fn mambalibs_local_path_override_lockfile_assertion_records_deterministic_identity() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("lockfile_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[lockfile_assertion]` block \
         (acceptance: \"Lockfile records the local path dependency identity \
         deterministically.\")",
        );

    assert_eq!(
        block.get("file").and_then(|v| v.as_str()),
        Some("mamba.lock"),
        "`[lockfile_assertion].file` must be \"mamba.lock\""
    );
    assert_eq!(
        block
            .get("must_record_source_kind")
            .and_then(|v| v.as_str()),
        Some("local_path"),
        "lockfile must record source_kind == \"local_path\""
    );
    assert_eq!(
        block.get("source_kind_field_key").and_then(|v| v.as_str()),
        Some("source"),
        "source_kind_field_key must be `source`"
    );
    assert_eq!(
        block
            .get("override_path_field_key")
            .and_then(|v| v.as_str()),
        Some("path"),
        "override_path_field_key must be `path`"
    );

    // Lockfile must record the same relative path as the override
    // block declares.
    let rel = doc
        .get("override")
        .and_then(|v| v.get("relative_path"))
        .and_then(|v| v.as_str())
        .expect("`[override].relative_path` must be set");
    assert_eq!(
        block
            .get("must_record_relative_path")
            .and_then(|v| v.as_str()),
        Some(rel),
        "`[lockfile_assertion].must_record_relative_path` must equal `[override].relative_path`"
    );

    for flag in &[
        "must_record_override_path",
        "must_not_record_index_url",
        "must_be_deterministic",
        "must_be_byte_identical_on_replay",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[lockfile_assertion].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_local_path_override_missing_path_case_fails_with_named_diagnostic() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("missing_override_path_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[missing_override_path_case]` block \
         (acceptance: \"Missing or wrong override path fails clearly.\")",
        );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("missing_override_path"),
        "`[missing_override_path_case].case` must be \"missing_override_path\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[missing_override_path_case].expected_outcome` must be \"fail\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(1),
        "`[missing_override_path_case].expected_exit_code` must be 1"
    );
    assert_eq!(
        case.get("override_path_exists").and_then(|v| v.as_bool()),
        Some(false),
        "`[missing_override_path_case].override_path_exists` must be false"
    );

    for flag in &[
        "diagnostic_must_name_offending_path",
        "diagnostic_must_name_override_field",
        "must_not_silently_fall_back_to_registry",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[missing_override_path_case].{flag}` must be true"
        );
    }

    let msg = case
        .get("diagnostic_message_substring")
        .and_then(|v| v.as_str())
        .expect("`[missing_override_path_case].diagnostic_message_substring` must be set");
    assert!(
        !msg.is_empty(),
        "diagnostic_message_substring must be non-empty"
    );
}

#[test]
fn mambalibs_local_path_override_wrong_path_case_fails_with_named_diagnostic() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("wrong_override_path_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[wrong_override_path_case]` block \
         (acceptance: \"Missing or wrong override path fails clearly.\")",
        );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("wrong_override_path"),
        "`[wrong_override_path_case].case` must be \"wrong_override_path\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[wrong_override_path_case].expected_outcome` must be \"fail\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(1),
        "`[wrong_override_path_case].expected_exit_code` must be 1"
    );

    let target = case
        .get("override_path_target")
        .and_then(|v| v.as_str())
        .expect("`[wrong_override_path_case].override_path_target` must be set");
    let real = doc
        .get("override")
        .and_then(|v| v.get("relative_path"))
        .and_then(|v| v.as_str())
        .expect("`[override].relative_path` must be set");
    assert_ne!(
        target, real,
        "wrong-path target must differ from the real override path"
    );

    for flag in &[
        "diagnostic_must_name_offending_path",
        "diagnostic_must_name_offending_dependency",
        "diagnostic_must_be_actionable",
        "must_not_silently_fall_back_to_registry",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[wrong_override_path_case].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_local_path_override_import_proof_contract_pins_summary_naming() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("import_proof_contract")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[import_proof_contract]` block \
         (acceptance: \"Import result proves the override was used.\")",
        );

    for flag in &[
        "must_distinguish_override_from_default",
        "must_name_resolved_source_in_summary",
        "must_name_import_probe_value_in_summary",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[import_proof_contract].{flag}` must be true"
        );
    }

    // Both `resolved_source` and `import_probe_value` MUST appear
    // in the runner contract so the summary can name them.
    let keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["resolved_source", "import_probe_value"] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }
}

#[test]
fn mambalibs_local_path_override_isolation_pins_no_global_state() {
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
fn mambalibs_local_path_override_runner_contract_declares_keys_and_cases() {
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
        "module",
        "exported_function",
        "resolved_source",
        "override_path",
        "lockfile_source_kind",
        "import_probe_value",
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
    for required in &[
        "override_applied",
        "missing_override_path",
        "wrong_override_path",
    ] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_local_path_override_pins_out_of_scope_per_issue_2671() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("remote_registry_override_policy")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].remote_registry_override_policy` must be true \
         (issue text: \"Out of scope: remote registry override policy.\")"
    );
}
