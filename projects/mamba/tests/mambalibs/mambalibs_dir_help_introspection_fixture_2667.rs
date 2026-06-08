//! Schema gate for the mambalibs dir/help introspection fixture —
//! closes #2667.
//!
//! Acceptance (issue #2667):
//!
//!   1. Fixture fails if exported symbols are hidden from dir or
//!      getattr.
//!      `[hidden_symbol_guard]` pins fail flags for missing-from-dir,
//!      AttributeError-from-getattr, and absent doc/name attributes;
//!      the dir and getattr cases require visibility for a `pass`.
//!   2. Failure distinguishes import failure from introspection
//!      failure.
//!      `[outcome_distinction]` declares two disjoint failure modes
//!      and the runner_contract surfaces both values.
//!   3. No production cclab library behavior is changed.
//!      `[no_production_change].cclab_library_behavior_unchanged ==
//!      true`; the fixture is sandboxed to its own directory.
//!
//! Cheap test — single TOML read + field walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("dir_help_introspection")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_dir_help_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_dir_help_introspection"),
        "`fixture` must be \"mambalibs_dir_help_introspection\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2667),
        "`issue` must record #2667"
    );
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2531),
        "`parent_issue` must record the Mode 2 mambalibs MVP cohort (#2531)"
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`profile` must be \"mambalibs\""
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("introspection"),
        "`family` must be \"introspection\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\" (fixture is local-only)"
    );
}

#[test]
fn mambalibs_dir_help_binding_block_pins_module_and_exported_function() {
    let doc = load_toml(&manifest_path());
    let bind = doc
        .get("binding")
        .and_then(|v| v.as_table())
        .expect("missing `[binding]` block");

    let module = bind
        .get("module_name")
        .and_then(|v| v.as_str())
        .expect("`[binding].module_name` must be set");
    assert_eq!(
        module, "mambalibs",
        "module name must be \"mambalibs\" — fixture imports via `from mambalibs import ...`"
    );

    let exported = bind
        .get("exported_function")
        .and_then(|v| v.as_str())
        .expect("`[binding].exported_function` must be set");
    assert!(
        !exported.is_empty(),
        "`[binding].exported_function` must be non-empty"
    );

    let import_stmt = bind
        .get("import_statement")
        .and_then(|v| v.as_str())
        .expect("`[binding].import_statement` must be set");
    assert!(
        import_stmt.contains("from mambalibs import"),
        "import statement must use `from mambalibs import ...` shape; got {import_stmt:?}"
    );
    assert!(
        import_stmt.contains(exported),
        "import statement {import_stmt:?} must reference {exported:?}"
    );

    assert_eq!(
        bind.get("must_be_introspectable").and_then(|v| v.as_bool()),
        Some(true),
        "`[binding].must_be_introspectable` must be true"
    );
}

#[test]
fn mambalibs_dir_help_dir_case_requires_exported_symbol() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("dir_case")
        .and_then(|v| v.as_table())
        .expect("missing `[dir_case]` block");

    assert_eq!(
        block.get("case").and_then(|v| v.as_str()),
        Some("dir_lists_exported_function"),
        "`[dir_case].case` must be \"dir_lists_exported_function\""
    );
    assert_eq!(
        block.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[dir_case].expected_outcome` must be \"pass\""
    );

    let probe = block
        .get("probe")
        .and_then(|v| v.as_str())
        .expect("`[dir_case].probe` must be set");
    assert!(
        probe.contains("dir("),
        "`[dir_case].probe` must call `dir(...)`; got {probe:?}"
    );

    let exported = doc
        .get("binding")
        .and_then(|v| v.get("exported_function"))
        .and_then(|v| v.as_str())
        .expect("`[binding].exported_function` must be set");
    let probe_symbol = block
        .get("exported_symbol")
        .and_then(|v| v.as_str())
        .expect("`[dir_case].exported_symbol` must be set");
    assert_eq!(
        probe_symbol, exported,
        "`[dir_case].exported_symbol` must equal `[binding].exported_function`"
    );

    for flag in &[
        "must_contain_exported_symbol",
        "diagnostic_must_name_symbol",
        "diagnostic_must_name_module",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[dir_case].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_dir_help_getattr_case_requires_callable_return() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("getattr_case")
        .and_then(|v| v.as_table())
        .expect("missing `[getattr_case]` block");

    assert_eq!(
        block.get("case").and_then(|v| v.as_str()),
        Some("getattr_returns_exported_function"),
        "`[getattr_case].case` must be \"getattr_returns_exported_function\""
    );
    assert_eq!(
        block.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[getattr_case].expected_outcome` must be \"pass\""
    );

    let probe = block
        .get("probe")
        .and_then(|v| v.as_str())
        .expect("`[getattr_case].probe` must be set");
    assert!(
        probe.contains("getattr("),
        "`[getattr_case].probe` must call `getattr(...)`; got {probe:?}"
    );

    let exported = doc
        .get("binding")
        .and_then(|v| v.get("exported_function"))
        .and_then(|v| v.as_str())
        .expect("`[binding].exported_function` must be set");
    assert!(
        probe.contains(exported),
        "`[getattr_case].probe` must name the exported function {exported:?}; got {probe:?}"
    );

    for flag in &[
        "must_return_callable",
        "must_not_raise_attribute_error",
        "diagnostic_must_name_symbol",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[getattr_case].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_dir_help_doc_or_name_case_accepts_either() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("doc_or_name_case")
        .and_then(|v| v.as_table())
        .expect("missing `[doc_or_name_case]` block");

    assert_eq!(
        block.get("case").and_then(|v| v.as_str()),
        Some("doc_or_name_attribute_present"),
        "`[doc_or_name_case].case` must be \"doc_or_name_attribute_present\""
    );
    assert_eq!(
        block.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[doc_or_name_case].expected_outcome` must be \"pass\""
    );

    let probes: Vec<&str> = block
        .get("probes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        probes.iter().any(|p| p.contains("__name__")),
        "`[doc_or_name_case].probes` must include a __name__ probe; got {probes:?}"
    );
    assert!(
        probes.iter().any(|p| p.contains("__doc__")),
        "`[doc_or_name_case].probes` must include a __doc__ probe; got {probes:?}"
    );

    assert_eq!(
        block.get("at_least_one_must_be_non_none").and_then(|v| v.as_bool()),
        Some(true),
        "`[doc_or_name_case].at_least_one_must_be_non_none` must be true \
         (issue: \"__doc__ or __name__\" — either is sufficient)"
    );
    assert_eq!(
        block.get("expected_name_value").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`[doc_or_name_case].expected_name_value` must be \"mambalibs\""
    );
    assert_eq!(
        block.get("diagnostic_must_name_attribute").and_then(|v| v.as_bool()),
        Some(true),
        "`[doc_or_name_case].diagnostic_must_name_attribute` must be true"
    );
}

#[test]
fn mambalibs_dir_help_hidden_symbol_guard_pins_all_fail_flags() {
    let doc = load_toml(&manifest_path());
    let guard = doc.get("hidden_symbol_guard").and_then(|v| v.as_table()).expect(
        "missing `[hidden_symbol_guard]` block \
         (acceptance: \"Fixture fails if exported symbols are hidden \
         from dir or getattr.\")",
    );

    for flag in &[
        "fail_if_symbol_missing_from_dir",
        "fail_if_getattr_raises_attribute_error",
        "fail_if_doc_and_name_both_none",
        "diagnostic_must_name_offending_symbol",
        "diagnostic_must_name_offending_probe",
    ] {
        assert_eq!(
            guard.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[hidden_symbol_guard].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_dir_help_outcome_distinction_disambiguates_failure_modes() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("outcome_distinction").and_then(|v| v.as_table()).expect(
        "missing `[outcome_distinction]` block \
         (acceptance: \"Failure distinguishes import failure from \
         introspection failure.\")",
    );

    let import_fail = block
        .get("import_failure_outcome")
        .and_then(|v| v.as_str())
        .expect("`[outcome_distinction].import_failure_outcome` must be set");
    let intro_fail = block
        .get("introspection_failure_outcome")
        .and_then(|v| v.as_str())
        .expect("`[outcome_distinction].introspection_failure_outcome` must be set");
    assert_ne!(
        import_fail, intro_fail,
        "import_failure and introspection_failure outcomes MUST be distinct"
    );

    assert_eq!(
        block.get("must_be_disjoint").and_then(|v| v.as_bool()),
        Some(true),
        "`[outcome_distinction].must_be_disjoint` must be true"
    );
    assert_eq!(
        block.get("diagnostic_must_name_failure_mode").and_then(|v| v.as_bool()),
        Some(true),
        "`[outcome_distinction].diagnostic_must_name_failure_mode` must be true"
    );

    // Both outcome values MUST be declared in the runner contract so
    // they can land in the JSON report.
    let outcomes: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("outcome_values"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[import_fail, intro_fail] {
        assert!(
            outcomes.contains(required),
            "`[runner_contract].outcome_values` must include `{required}`; got {outcomes:?}"
        );
    }
}

#[test]
fn mambalibs_dir_help_no_production_change_keeps_scope_local() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("no_production_change").and_then(|v| v.as_table()).expect(
        "missing `[no_production_change]` block \
         (acceptance: \"No production cclab library behavior is changed.\")",
    );

    assert_eq!(
        block.get("cclab_library_behavior_unchanged").and_then(|v| v.as_bool()),
        Some(true),
        "`[no_production_change].cclab_library_behavior_unchanged` must be true"
    );
    assert_eq!(
        block.get("forbid_touching_production_crates").and_then(|v| v.as_bool()),
        Some(true),
        "`[no_production_change].forbid_touching_production_crates` must be true"
    );
    let scope_path = block
        .get("fixture_scope_path")
        .and_then(|v| v.as_str())
        .expect("`[no_production_change].fixture_scope_path` must be set");
    assert!(
        scope_path.ends_with("dir_help_introspection"),
        "scope path must stay under the fixture directory; got {scope_path:?}"
    );
}

#[test]
fn mambalibs_dir_help_isolation_pins_no_global_state() {
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
fn mambalibs_dir_help_runner_contract_declares_all_keys_and_cases() {
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
        "exported_symbol",
        "probe",
        "probe_result",
        "failure_mode",
        "diagnostic_message",
        "exit_code",
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
    for required in &["pass", "fail", "import_failure", "introspection_failure"] {
        assert!(
            outcomes.contains(required),
            "`[runner_contract].outcome_values` must include `{required}`; got {outcomes:?}"
        );
    }

    let cases: Vec<&str> = contract
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "dir_lists_exported_function",
        "getattr_returns_exported_function",
        "doc_or_name_attribute_present",
    ] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_dir_help_pins_out_of_scope_per_issue_2667() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("rich_inspect_signature_support").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].rich_inspect_signature_support` must be true \
         (issue text: \"Out of scope: rich inspect.signature support.\")"
    );
}
