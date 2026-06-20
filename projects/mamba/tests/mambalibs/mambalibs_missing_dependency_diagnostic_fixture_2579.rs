//! Schema gate for the mambalibs missing dependency diagnostic fixture
//! — closes #2579.
//!
//! Acceptance (issue #2579):
//!
//!   1. Missing dependency does not crash the interpreter.
//!      `[crash_guard]` pins all interpreter-crash fail flags +
//!      must_exit_with_python_exception_not_crash.
//!   2. Error message includes module name and recovery command.
//!      `[error_message_contract]` pins ImportError +
//!      module_name_substring + recovery_command_substring +
//!      allowed_recovery_subcommands = [build, sync].
//!   3. Positive import fixture still passes.
//!      `[positive_fixture_isolation]` pins
//!      positive_fixture_issue=2576 +
//!      positive_fixture_must_remain_green +
//!      this_fixture_must_not_affect_positive_cases.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("missing_dependency_diagnostic")
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
        Some("mambalibs_missing_dependency_diagnostic"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2579));
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
        Some("missing_dependency_diagnostic")
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
fn binding_declares_but_does_not_build_missing_module() {
    let doc = load_toml(&manifest_path());
    let b = doc
        .get("binding")
        .and_then(|v| v.as_table())
        .expect("[binding] missing");
    assert_eq!(
        b.get("module_name").and_then(|v| v.as_str()),
        Some("mambalibs")
    );
    let missing = b
        .get("missing_fixture_module")
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(!missing.is_empty());
    let stmt = b.get("import_statement").and_then(|v| v.as_str()).unwrap();
    assert!(stmt.starts_with("from mambalibs import "));
    assert!(stmt.contains(missing));
    assert_eq!(
        b.get("mamba_toml_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2575)
    );
    assert_eq!(
        b.get("must_be_declared_in_mamba_toml")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        b.get("must_not_be_built").and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Missing dependency does not crash the interpreter."
#[test]
fn missing_dependency_does_not_crash_interpreter() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("crash_guard").and_then(|v| v.as_table()).expect(
        "[crash_guard] missing — acceptance: \
         \"Missing dependency does not crash the interpreter.\"",
    );
    for f in &[
        "fail_if_interpreter_segfaults",
        "fail_if_interpreter_aborts",
        "fail_if_rust_panic_aborts_process",
        "fail_if_exit_code_indicates_crash",
        "must_exit_with_python_exception_not_crash",
        "crash_diagnostic_must_name_crash_mode",
    ] {
        assert_eq!(
            c.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

// Acceptance: "Error message includes module name and recovery command."
#[test]
fn error_message_includes_module_name_and_recovery_command() {
    let doc = load_toml(&manifest_path());
    let e = doc
        .get("error_message_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[error_message_contract] missing — acceptance: \
         \"Error message includes module name and recovery command.\"",
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
        e.get("exception_message_must_include_module_name")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let module_substr = e
        .get("module_name_substring")
        .and_then(|v| v.as_str())
        .unwrap();
    let binding_missing = doc
        .get("binding")
        .and_then(|v| v.get("missing_fixture_module"))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(
        module_substr, binding_missing,
        "module_name_substring must match [binding].missing_fixture_module"
    );
    assert_eq!(
        e.get("exception_message_must_include_recovery_command")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let recovery = e
        .get("recovery_command_substring")
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(
        recovery.contains("mamba"),
        "recovery_command_substring must mention mamba"
    );
    let subcommands: Vec<&str> = e
        .get("allowed_recovery_subcommands")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["build", "sync"] {
        assert!(
            subcommands.contains(required),
            "allowed_recovery_subcommands must include {required}"
        );
    }
    assert_eq!(
        e.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        e.get("recovery_exit_code").and_then(|v| v.as_integer()),
        Some(6)
    );
}

// Acceptance: "Positive import fixture still passes."
#[test]
fn positive_import_fixture_still_passes() {
    let doc = load_toml(&manifest_path());
    let p = doc
        .get("positive_fixture_isolation")
        .and_then(|v| v.as_table())
        .expect(
            "[positive_fixture_isolation] missing — acceptance: \
         \"Positive import fixture still passes.\"",
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
        "missing_module",
        "exception_type",
        "exception_message",
        "recovery_command",
        "crash_mode",
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
    assert!(cases.contains(&"missing_dependency_raises_diagnostic_exception"));
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("implementing_automatic_dependency_build")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
