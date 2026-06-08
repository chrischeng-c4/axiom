//! Schema gate for the mambalibs registry metadata validation fixture —
//! closes #2670.
//!
//! Acceptance (issue #2670):
//!
//!   1. Valid fixture metadata passes validation.
//!      `[valid_metadata_case]` carries every required field;
//!      pinned to `pass` outcome with exit 0.
//!   2. Missing ABI or export metadata fails validation.
//!      `[missing_abi_case]` and `[missing_exports_case]` pin the
//!      `fail` outcome with the offending field name.
//!   3. Diagnostic names the missing field.
//!      Every failing case pins
//!      `diagnostic_must_name_missing_field = true`; the runner
//!      contract carries `missing_field` as a first-class key.
//!
//! Cheap test — single TOML read + field walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("registry_metadata_validation")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_registry_metadata_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_registry_metadata_validation"),
        "`fixture` must be \"mambalibs_registry_metadata_validation\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2670),
        "`issue` must record #2670"
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
        Some("registry_metadata"),
        "`family` must be \"registry_metadata\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn mambalibs_registry_metadata_required_fields_cover_acceptance_set() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("required_fields")
        .and_then(|v| v.as_table())
        .expect("missing `[required_fields]` block");

    let fields: BTreeSet<&str> = block
        .get("fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["name", "version", "abi", "artifact", "exports"] {
        assert!(
            fields.contains(required),
            "`[required_fields].fields` must include `{required}`; got {fields:?}"
        );
    }
    assert_eq!(
        block.get("must_all_be_non_empty").and_then(|v| v.as_bool()),
        Some(true),
        "`[required_fields].must_all_be_non_empty` must be true"
    );
}

#[test]
fn mambalibs_registry_metadata_valid_case_passes_with_every_field() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("valid_metadata_case").and_then(|v| v.as_table()).expect(
        "missing `[valid_metadata_case]` block \
         (acceptance: \"Valid fixture metadata passes validation.\")",
    );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("valid_metadata"),
        "`[valid_metadata_case].case` must be \"valid_metadata\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[valid_metadata_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[valid_metadata_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("diagnostic_must_be_empty").and_then(|v| v.as_bool()),
        Some(true),
        "`[valid_metadata_case].diagnostic_must_be_empty` must be true"
    );

    let metadata_file = case
        .get("metadata_file")
        .and_then(|v| v.as_str())
        .expect("`[valid_metadata_case].metadata_file` must be set");
    assert!(
        metadata_file.ends_with(".json"),
        "metadata file must be a .json document; got {metadata_file:?}"
    );

    let metadata = case
        .get("metadata")
        .and_then(|v| v.as_table())
        .expect("`[valid_metadata_case.metadata]` must be present");

    let required: Vec<&str> = doc
        .get("required_fields")
        .and_then(|v| v.get("fields"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for field in &required {
        assert!(
            metadata.contains_key(*field),
            "`[valid_metadata_case.metadata]` must populate every required field; missing `{field}`"
        );
    }

    let exports: Vec<&str> = metadata
        .get("exports")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let exported_fn = doc
        .get("binding")
        .and_then(|v| v.get("exported_function"))
        .and_then(|v| v.as_str())
        .expect("`[binding].exported_function` must be set");
    assert!(
        exports.contains(&exported_fn),
        "valid metadata's `exports` must include the binding's exported function {exported_fn:?}; \
         got {exports:?}"
    );
}

#[test]
fn mambalibs_registry_metadata_missing_abi_case_fails_and_names_field() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("missing_abi_case").and_then(|v| v.as_table()).expect(
        "missing `[missing_abi_case]` block \
         (acceptance: \"Missing ABI or export metadata fails validation.\")",
    );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("missing_abi"),
        "`[missing_abi_case].case` must be \"missing_abi\""
    );
    assert_eq!(
        case.get("omitted_field").and_then(|v| v.as_str()),
        Some("abi"),
        "`[missing_abi_case].omitted_field` must be \"abi\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[missing_abi_case].expected_outcome` must be \"fail\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(1),
        "`[missing_abi_case].expected_exit_code` must be 1"
    );
    assert_eq!(
        case.get("diagnostic_must_name_missing_field").and_then(|v| v.as_bool()),
        Some(true),
        "`[missing_abi_case].diagnostic_must_name_missing_field` must be true"
    );
    assert_eq!(
        case.get("diagnostic_must_name_field_value").and_then(|v| v.as_str()),
        Some("abi"),
        "`[missing_abi_case].diagnostic_must_name_field_value` must be \"abi\""
    );
    assert_eq!(
        case.get("must_not_silently_accept").and_then(|v| v.as_bool()),
        Some(true),
        "`[missing_abi_case].must_not_silently_accept` must be true"
    );

    // The omitted field MUST be one of the required fields,
    // otherwise the failure case is structurally incoherent.
    let required: Vec<&str> = doc
        .get("required_fields")
        .and_then(|v| v.get("fields"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        required.contains(&"abi"),
        "`abi` must be listed in `[required_fields].fields`; got {required:?}"
    );
}

#[test]
fn mambalibs_registry_metadata_missing_exports_case_fails_and_names_field() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("missing_exports_case").and_then(|v| v.as_table()).expect(
        "missing `[missing_exports_case]` block \
         (acceptance: \"Missing ABI or export metadata fails validation.\")",
    );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("missing_exports"),
        "`[missing_exports_case].case` must be \"missing_exports\""
    );
    assert_eq!(
        case.get("omitted_field").and_then(|v| v.as_str()),
        Some("exports"),
        "`[missing_exports_case].omitted_field` must be \"exports\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[missing_exports_case].expected_outcome` must be \"fail\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(1),
        "`[missing_exports_case].expected_exit_code` must be 1"
    );
    assert_eq!(
        case.get("diagnostic_must_name_missing_field").and_then(|v| v.as_bool()),
        Some(true),
        "`[missing_exports_case].diagnostic_must_name_missing_field` must be true"
    );
    assert_eq!(
        case.get("diagnostic_must_name_field_value").and_then(|v| v.as_str()),
        Some("exports"),
        "`[missing_exports_case].diagnostic_must_name_field_value` must be \"exports\""
    );
    assert_eq!(
        case.get("must_not_silently_accept").and_then(|v| v.as_bool()),
        Some(true),
        "`[missing_exports_case].must_not_silently_accept` must be true"
    );

    let required: Vec<&str> = doc
        .get("required_fields")
        .and_then(|v| v.get("fields"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        required.contains(&"exports"),
        "`exports` must be listed in `[required_fields].fields`; got {required:?}"
    );
}

#[test]
fn mambalibs_registry_metadata_diagnostic_contract_pins_all_flags() {
    let doc = load_toml(&manifest_path());
    let contract = doc.get("diagnostic_contract").and_then(|v| v.as_table()).expect(
        "missing `[diagnostic_contract]` block \
         (acceptance: \"Diagnostic names the missing field.\")",
    );

    for flag in &[
        "diagnostic_must_name_offending_metadata_file",
        "diagnostic_must_be_deterministic",
        "diagnostic_must_be_actionable",
    ] {
        assert_eq!(
            contract.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[diagnostic_contract].{flag}` must be true"
        );
    }

    let field_key = contract
        .get("diagnostic_must_name_missing_field_key")
        .and_then(|v| v.as_str())
        .expect("`[diagnostic_contract].diagnostic_must_name_missing_field_key` must be set");
    assert_eq!(
        field_key, "missing_field",
        "diagnostic key for the missing field name must be `missing_field`; got {field_key:?}"
    );

    // That key MUST also appear in the runner contract so the
    // JSON report surfaces it.
    let keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        keys.contains(&field_key),
        "`[runner_contract].keys` must include `{field_key}`; got {keys:?}"
    );
}

#[test]
fn mambalibs_registry_metadata_isolation_pins_no_global_state() {
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
fn mambalibs_registry_metadata_runner_contract_declares_keys_and_cases() {
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
        "metadata_file",
        "missing_field",
        "validator_decision",
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
    for required in &["valid_metadata", "missing_abi", "missing_exports"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_registry_metadata_pins_out_of_scope_per_issue_2670() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("real_remote_registry_implementation").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].real_remote_registry_implementation` must be true \
         (issue text: \"Out of scope: real remote registry implementation.\")"
    );
}
