//! Schema gate for the mambalibs ABI mismatch failure fixture —
//! closes #2582.
//!
//! Acceptance (issue #2582):
//!
//!   1. ABI mismatch does not panic or produce a vague import
//!      error. `[crash_guard]` pins all interpreter-crash fail flags
//!      + must_exit_with_python_exception_not_crash +
//!      forbid_vague_import_error.
//!   2. Failure message names expected and found ABI or schema
//!      version. `[error_message_contract]` pins ImportError +
//!      expected_version_substring + found_version_substring +
//!      must_distinguish_expected_from_found + failure_kind=
//!      abi_mismatch + exit_code=9.
//!   3. Positive Mode 2 fixture remains unaffected.
//!      `[positive_fixture_isolation]` pins positive_fixture_issue=
//!      2576 + must_not_register_globally.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("abi_mismatch_failure")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path).unwrap();
    raw.parse().unwrap()
}

#[test]
fn header_is_well_formed() {
    let doc = load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_abi_mismatch_failure"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2582));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2531)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("mambalibs")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("abi_mismatch_failure")
    );
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = load_toml(&manifest_path());
    let i = doc.get("isolation").and_then(|v| v.as_table()).unwrap();
    for f in &[
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(i.get(*f).and_then(|v| v.as_bool()), Some(true));
    }
}

#[test]
fn binding_supplies_small_non_platform_specific_fake_artifact() {
    let doc = load_toml(&manifest_path());
    let b = doc
        .get("binding")
        .and_then(|v| v.as_table())
        .expect("[binding] missing");
    assert_eq!(
        b.get("module_name").and_then(|v| v.as_str()),
        Some("mambalibs")
    );
    let module = b.get("fixture_module").and_then(|v| v.as_str()).unwrap();
    assert!(!module.is_empty());
    let stmt = b.get("import_statement").and_then(|v| v.as_str()).unwrap();
    assert!(stmt.starts_with("from mambalibs import "));
    assert!(stmt.contains(module));
    assert_eq!(
        b.get("local_binding_crate_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2577)
    );
    assert_eq!(
        b.get("must_supply_fake_artifact").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        b.get("fake_artifact_must_be_small")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        b.get("fake_artifact_must_not_be_platform_specific")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn abi_versions_must_differ() {
    let doc = load_toml(&manifest_path());
    let a = doc
        .get("abi")
        .and_then(|v| v.as_table())
        .expect("[abi] missing");
    let exp = a
        .get("expected_abi_version")
        .and_then(|v| v.as_str())
        .unwrap();
    let found = a.get("found_abi_version").and_then(|v| v.as_str()).unwrap();
    assert_ne!(
        exp, found,
        "expected_abi_version must differ from found_abi_version"
    );
    let exp_schema = a
        .get("expected_schema_version")
        .and_then(|v| v.as_integer())
        .unwrap();
    let found_schema = a
        .get("found_schema_version")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(
        exp_schema, found_schema,
        "expected_schema_version must differ from found_schema_version"
    );
    assert_eq!(
        a.get("versions_must_differ").and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "ABI mismatch does not panic or produce a vague import
// error."
#[test]
fn abi_mismatch_does_not_panic_or_produce_vague_import_error() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("crash_guard").and_then(|v| v.as_table()).expect(
        "[crash_guard] missing — acceptance: \
         \"ABI mismatch does not panic or produce a vague import error.\"",
    );
    for f in &[
        "fail_if_interpreter_segfaults",
        "fail_if_interpreter_aborts",
        "fail_if_rust_panic_aborts_process",
        "fail_if_exit_code_indicates_crash",
        "must_exit_with_python_exception_not_crash",
        "forbid_vague_import_error",
        "vague_diagnostic_must_be_treated_as_test_failure",
    ] {
        assert_eq!(
            c.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

// Acceptance: "Failure message names expected and found ABI or
// schema version."
#[test]
fn failure_message_names_expected_and_found_abi_version() {
    let doc = load_toml(&manifest_path());
    let e = doc
        .get("error_message_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[error_message_contract] missing — acceptance: \
         \"Failure message names expected and found ABI or schema version.\"",
        );
    assert_eq!(
        e.get("must_raise_python_exception")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        e.get("expected_exception_type").and_then(|v| v.as_str()),
        Some("ImportError")
    );
    assert_eq!(
        e.get("exception_message_must_name_expected_version")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        e.get("exception_message_must_name_found_version")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let exp_substr = e
        .get("expected_version_substring")
        .and_then(|v| v.as_str())
        .unwrap();
    let found_substr = e
        .get("found_version_substring")
        .and_then(|v| v.as_str())
        .unwrap();
    let abi_exp = doc
        .get("abi")
        .and_then(|v| v.get("expected_abi_version"))
        .and_then(|v| v.as_str())
        .unwrap();
    let abi_found = doc
        .get("abi")
        .and_then(|v| v.get("found_abi_version"))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(
        exp_substr, abi_exp,
        "expected_version_substring must match [abi].expected_abi_version"
    );
    assert_eq!(
        found_substr, abi_found,
        "found_version_substring must match [abi].found_abi_version"
    );
    assert_ne!(
        exp_substr, found_substr,
        "expected and found substrings must differ"
    );
    assert_eq!(
        e.get("exception_message_must_distinguish_expected_from_found")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        e.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        e.get("abi_mismatch_failure_kind").and_then(|v| v.as_str()),
        Some("abi_mismatch")
    );
    let exit = e
        .get("abi_mismatch_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 9);
}

// Acceptance: "Positive Mode 2 fixture remains unaffected."
#[test]
fn positive_mode_2_fixture_remains_unaffected() {
    let doc = load_toml(&manifest_path());
    let p = doc
        .get("positive_fixture_isolation")
        .and_then(|v| v.as_table())
        .expect(
            "[positive_fixture_isolation] missing — acceptance: \
         \"Positive Mode 2 fixture remains unaffected.\"",
        );
    assert_eq!(
        p.get("positive_fixture_issue").and_then(|v| v.as_integer()),
        Some(2576)
    );
    assert_eq!(
        p.get("positive_fixture_must_remain_green")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("this_fixture_must_not_affect_positive_cases")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("this_fixture_must_not_register_globally")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .unwrap();
    let keys: Vec<&str> = c
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "outcome",
        "case",
        "module",
        "expected_abi_version",
        "found_abi_version",
        "exception_type",
        "exception_message",
        "failure_kind",
        "exit_code",
    ] {
        assert!(
            keys.contains(required),
            "runner_contract.keys must include {required}"
        );
    }
    let cases: Vec<&str> = c
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(cases.contains(&"abi_mismatch_raises_diagnostic_exception"));
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("designing_final_abi_compatibility_policy")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
