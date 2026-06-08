//! Schema gate for the mambalibs schema import gate fixture —
//! closes #2839.
//!
//! Acceptance (issue #2839):
//!
//!   1. Import success is asserted or unsupported status is
//!      linked to a blocker.
//!      `[support_status].current_status ∈ ["pass","xfail","blocker"]`;
//!      `[supported_case]` and `[blocked_case]` carry
//!      `status_under_which_applicable` matching one of those
//!      values; `[blocked_case].linked_blocker_issue` is set.
//!   2. Failure output names schema and mambalibs.
//!      `[diagnostic_contract]` pins
//!      `diagnostic_must_name_library = "schema"` and
//!      `diagnostic_must_name_surface = "mambalibs"`; the
//!      corresponding field keys appear in
//!      `[runner_contract].keys`.
//!   3. Test uses a tiny deterministic schema sample only if
//!      needed.
//!      `[sample_policy]` pins `must_be_tiny`,
//!      `must_be_deterministic`, and `max_sample_bytes`.
//!
//! Cheap test — single TOML read + field walk. Runs in well under
//! a second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("schema_import_gate")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_schema_import_gate_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_schema_import_gate"),
        "`fixture` must be \"mambalibs_schema_import_gate\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2839),
        "`issue` must record #2839"
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
        Some("schema_import_gate"),
        "`family` must be \"schema_import_gate\""
    );
    assert_eq!(
        doc.get("library").and_then(|v| v.as_str()),
        Some("schema"),
        "`library` must be \"schema\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn mambalibs_schema_binding_uses_documented_surface_and_minimal_symbol() {
    let doc = load_toml(&manifest_path());
    let bind = doc
        .get("binding")
        .and_then(|v| v.as_table())
        .expect("missing `[binding]` block");

    assert_eq!(
        bind.get("surface").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`[binding].surface` must be \"mambalibs\""
    );
    assert_eq!(
        bind.get("library").and_then(|v| v.as_str()),
        Some("schema"),
        "`[binding].library` must be \"schema\""
    );

    let import_stmt = bind
        .get("import_statement")
        .and_then(|v| v.as_str())
        .expect("`[binding].import_statement` must be set");
    assert!(
        import_stmt.contains("from mambalibs import"),
        "import statement must use `from mambalibs import ...`; got {import_stmt:?}"
    );
    assert!(
        import_stmt.contains("schema"),
        "import statement must reference `schema`; got {import_stmt:?}"
    );

    let symbol = bind
        .get("minimal_exported_symbol")
        .and_then(|v| v.as_str())
        .expect("`[binding].minimal_exported_symbol` must be set");
    assert!(
        !symbol.is_empty(),
        "`[binding].minimal_exported_symbol` must be non-empty"
    );
}

#[test]
fn mambalibs_schema_support_status_enum_is_well_formed() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("support_status")
        .and_then(|v| v.as_table())
        .expect("missing `[support_status]` block");

    let allowed: Vec<&str> = block
        .get("allowed_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["pass", "xfail", "blocker"] {
        assert!(
            allowed.contains(required),
            "`[support_status].allowed_values` must include `{required}`; got {allowed:?}"
        );
    }
    let default = block
        .get("default_status")
        .and_then(|v| v.as_str())
        .expect("`[support_status].default_status` must be set");
    let current = block
        .get("current_status")
        .and_then(|v| v.as_str())
        .expect("`[support_status].current_status` must be set");
    assert!(
        allowed.contains(&default) && allowed.contains(&current),
        "default and current statuses must be members of allowed_values"
    );
}

#[test]
fn mambalibs_schema_supported_case_asserts_minimal_symbol() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("supported_case")
        .and_then(|v| v.as_table())
        .expect("missing `[supported_case]` block");

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("schema_import_supported"),
        "`[supported_case].case` must be \"schema_import_supported\""
    );
    assert_eq!(
        case.get("status_under_which_applicable")
            .and_then(|v| v.as_str()),
        Some("pass"),
        "`[supported_case].status_under_which_applicable` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[supported_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[supported_case].expected_exit_code` must be 0"
    );
    for flag in &["must_import_module", "must_assert_minimal_symbol"] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[supported_case].{flag}` must be true"
        );
    }

    // The asserted symbol MUST equal the binding's minimal symbol.
    let bind_symbol = doc
        .get("binding")
        .and_then(|v| v.get("minimal_exported_symbol"))
        .and_then(|v| v.as_str())
        .expect("`[binding].minimal_exported_symbol` must be set");
    assert_eq!(
        case.get("asserted_symbol").and_then(|v| v.as_str()),
        Some(bind_symbol),
        "`[supported_case].asserted_symbol` must equal \
         `[binding].minimal_exported_symbol`"
    );
}

#[test]
fn mambalibs_schema_blocked_case_links_to_tracker() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("blocked_case").and_then(|v| v.as_table()).expect(
        "missing `[blocked_case]` block \
         (acceptance: \"Import success is asserted or unsupported status is linked to a blocker.\")",
    );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("schema_import_blocked"),
        "`[blocked_case].case` must be \"schema_import_blocked\""
    );
    assert_eq!(
        case.get("status_under_which_applicable")
            .and_then(|v| v.as_str()),
        Some("blocker"),
        "`[blocked_case].status_under_which_applicable` must be \"blocker\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("blocked"),
        "`[blocked_case].expected_outcome` must be \"blocked\""
    );
    assert_eq!(
        case.get("linked_blocker_issue")
            .and_then(|v| v.as_integer()),
        Some(2839),
        "`[blocked_case].linked_blocker_issue` must record #2839"
    );
    for flag in &["must_emit_structured_blocker", "must_not_attempt_import"] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[blocked_case].{flag}` must be true"
        );
    }
    assert_eq!(
        case.get("must_name_offending_library")
            .and_then(|v| v.as_str()),
        Some("schema"),
        "`[blocked_case].must_name_offending_library` must be \"schema\""
    );
    assert_eq!(
        case.get("must_name_surface").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`[blocked_case].must_name_surface` must be \"mambalibs\""
    );

    // "blocked" MUST appear in runner_contract.outcome_values.
    let outcomes: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("outcome_values"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        outcomes.contains(&"blocked"),
        "`[runner_contract].outcome_values` MUST include `blocked`; got {outcomes:?}"
    );
}

#[test]
fn mambalibs_schema_diagnostic_contract_names_library_and_surface() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("diagnostic_contract")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[diagnostic_contract]` block \
         (acceptance: \"Failure output names schema and mambalibs.\")",
        );

    for flag in &[
        "diagnostic_must_be_deterministic",
        "diagnostic_must_be_actionable",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[diagnostic_contract].{flag}` must be true"
        );
    }
    assert_eq!(
        block
            .get("diagnostic_must_name_library")
            .and_then(|v| v.as_str()),
        Some("schema"),
        "`[diagnostic_contract].diagnostic_must_name_library` must be \"schema\""
    );
    assert_eq!(
        block
            .get("diagnostic_must_name_surface")
            .and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`[diagnostic_contract].diagnostic_must_name_surface` must be \"mambalibs\""
    );

    let contract_keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for key_field in &[
        "diagnostic_must_name_library_field_key",
        "diagnostic_must_name_surface_field_key",
        "diagnostic_must_name_linked_blocker_field_key",
    ] {
        let key = block
            .get(*key_field)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("`[diagnostic_contract].{key_field}` must be set"));
        assert!(
            contract_keys.contains(&key),
            "`[runner_contract].keys` must include `{key}` (from `{key_field}`); got \
             {contract_keys:?}"
        );
    }
}

#[test]
fn mambalibs_schema_sample_policy_pins_tiny_deterministic_input() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("sample_policy").and_then(|v| v.as_table()).expect(
        "missing `[sample_policy]` block \
         (acceptance: \"Test uses a tiny deterministic schema sample only if needed.\")",
    );

    for flag in &[
        "must_be_tiny",
        "must_be_deterministic",
        "forbid_network_io",
        "forbid_disk_io_outside_artifact_root",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[sample_policy].{flag}` must be true"
        );
    }
    let max_bytes = block
        .get("max_sample_bytes")
        .and_then(|v| v.as_integer())
        .expect("`[sample_policy].max_sample_bytes` must be set");
    assert!(
        max_bytes > 0 && max_bytes <= 1024,
        "`[sample_policy].max_sample_bytes` must be small (>0 and ≤1024); got {max_bytes}"
    );
}

#[test]
fn mambalibs_schema_gate_summary_field_lives_in_runner_contract() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("gate_summary_contract")
        .and_then(|v| v.as_table())
        .expect("missing `[gate_summary_contract]` block");

    let field_name = block
        .get("field_name")
        .and_then(|v| v.as_str())
        .expect("`[gate_summary_contract].field_name` must be set");
    assert_eq!(
        field_name, "schema_import_status",
        "`[gate_summary_contract].field_name` must be \"schema_import_status\""
    );

    let contract_keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        contract_keys.contains(&field_name),
        "`[runner_contract].keys` must include `{field_name}`; got {contract_keys:?}"
    );

    // Field-level allowed values must match support_status.allowed_values.
    let field_allowed: Vec<&str> = block
        .get("allowed_field_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let support_allowed: Vec<&str> = doc
        .get("support_status")
        .and_then(|v| v.get("allowed_values"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &support_allowed {
        assert!(
            field_allowed.contains(v),
            "`[gate_summary_contract].allowed_field_values` must include `{v}` from \
             `[support_status].allowed_values`; got {field_allowed:?}"
        );
    }
}

#[test]
fn mambalibs_schema_isolation_pins_no_global_state() {
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
fn mambalibs_schema_runner_contract_declares_keys_and_cases() {
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
        "surface",
        "library",
        "import_statement",
        "asserted_symbol",
        "schema_import_status",
        "linked_blocker_issue",
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
    for required in &["schema_import_supported", "schema_import_blocked"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_schema_pins_out_of_scope_per_issue_2839() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("full_schema_validation_behavior")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].full_schema_validation_behavior` must be true \
         (issue text: \"Out of scope: full schema validation behavior.\")"
    );
}
