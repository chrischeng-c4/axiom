//! Schema gate for the mambalibs `from mambalibs import` fixture —
//! closes #2576.
//!
//! Acceptance (issue #2576):
//!
//!   1. Test fails if import path is unavailable.
//!      `[import_case]` pins the user-facing import statement,
//!      expected outcomes (pass/fail), and the import_failure exit
//!      code.
//!   2. Test fails if the exported function cannot be called.
//!      `[call_case]` pins must_call_exported_function,
//!      must_assert_exact_returned_value, and the runtime_call_failure
//!      exit code.
//!   3. Output distinguishes import failure from runtime call
//!      failure. `[failure_kind_distinction]` pins
//!      allowed_failure_kinds = [import_failure, runtime_call_failure,
//!      none] AND must_emit_distinct_exit_codes = true with
//!      import_failure_exit_code != runtime_call_failure_exit_code.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("from_mambalibs_import")
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
        Some("mambalibs_from_mambalibs_import"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2576));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2531));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("mambalibs"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("from_mambalibs_import"));
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
fn binding_pins_user_facing_import_shape() {
    let doc = load_toml(&manifest_path());
    let b = doc.get("binding").and_then(|v| v.as_table()).expect("[binding] missing");
    assert_eq!(b.get("module_name").and_then(|v| v.as_str()), Some("mambalibs"));
    let fixture_module = b.get("fixture_module").and_then(|v| v.as_str()).unwrap();
    let exported = b.get("exported_function").and_then(|v| v.as_str()).unwrap();
    assert!(!fixture_module.is_empty());
    assert!(!exported.is_empty());
    let stmt = b.get("import_statement").and_then(|v| v.as_str()).unwrap();
    assert!(stmt.starts_with("from mambalibs import "), "must use the user-facing import path; got {stmt:?}");
    assert!(stmt.contains(fixture_module));
    assert_eq!(
        b.get("local_binding_crate_fixture_issue").and_then(|v| v.as_integer()),
        Some(2577),
        "must cross-reference the #2577 local binding crate fixture",
    );
}

// Acceptance: "Test fails if import path is unavailable."
#[test]
fn fails_when_import_path_is_unavailable() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("import_case").and_then(|v| v.as_table()).expect(
        "[import_case] missing — acceptance: \
         \"Test fails if import path is unavailable.\"",
    );
    let stmt = c.get("must_use_user_facing_import_statement").and_then(|v| v.as_str()).unwrap();
    assert!(stmt.starts_with("from mambalibs import "));
    assert_eq!(c.get("expected_outcome_when_import_succeeds").and_then(|v| v.as_str()), Some("pass"));
    assert_eq!(c.get("expected_outcome_when_import_unavailable").and_then(|v| v.as_str()), Some("fail"));
    assert_eq!(c.get("import_unavailable_failure_kind").and_then(|v| v.as_str()), Some("import_failure"));
    assert_eq!(c.get("import_unavailable_exit_code").and_then(|v| v.as_integer()), Some(2));
    assert_eq!(c.get("import_unavailable_diagnostic_must_name_module").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_not_mask_runtime_call_failures").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Test fails if the exported function cannot be called."
#[test]
fn fails_when_exported_function_cannot_be_called() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("call_case").and_then(|v| v.as_table()).expect(
        "[call_case] missing — acceptance: \
         \"Test fails if the exported function cannot be called.\"",
    );
    let must_call = c.get("must_call_exported_function").and_then(|v| v.as_str()).unwrap();
    let exported = doc.get("binding").and_then(|v| v.get("exported_function")).and_then(|v| v.as_str()).unwrap();
    assert_eq!(must_call, exported, "must_call_exported_function must match [binding].exported_function");
    assert_eq!(c.get("must_assert_exact_returned_value").and_then(|v| v.as_bool()), Some(true));
    let asserted = c.get("asserted_value").and_then(|v| v.as_integer()).unwrap();
    let expected = doc.get("expected_return_value").and_then(|v| v.get("value")).and_then(|v| v.as_integer()).unwrap();
    assert_eq!(asserted, expected, "asserted_value must match [expected_return_value].value");
    assert_eq!(c.get("expected_outcome_when_call_returns_expected").and_then(|v| v.as_str()), Some("pass"));
    assert_eq!(c.get("expected_outcome_when_call_fails").and_then(|v| v.as_str()), Some("fail"));
    assert_eq!(c.get("call_failure_kind").and_then(|v| v.as_str()), Some("runtime_call_failure"));
    assert_eq!(c.get("call_failure_exit_code").and_then(|v| v.as_integer()), Some(3));
    assert_eq!(c.get("call_failure_diagnostic_must_name_function").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_not_mask_import_failures").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Output distinguishes import failure from runtime call
// failure."
#[test]
fn output_distinguishes_import_from_runtime_call_failure() {
    let doc = load_toml(&manifest_path());
    let d = doc.get("failure_kind_distinction").and_then(|v| v.as_table()).expect(
        "[failure_kind_distinction] missing — acceptance: \
         \"Output distinguishes import failure from runtime call failure.\"",
    );
    assert_eq!(d.get("must_distinguish_import_vs_runtime_call_failure").and_then(|v| v.as_bool()), Some(true));
    let kinds: Vec<&str> = d.get("allowed_failure_kinds").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for k in &["import_failure", "runtime_call_failure", "none"] {
        assert!(kinds.contains(k), "allowed_failure_kinds must include {k}");
    }
    assert_eq!(d.get("must_emit_failure_kind_in_summary").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("must_emit_distinct_exit_codes").and_then(|v| v.as_bool()), Some(true));
    let imp_code = d.get("import_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    let rt_code = d.get("runtime_call_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    let clean = d.get("clean_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_ne!(imp_code, rt_code, "import and runtime call exit codes must differ");
    assert_ne!(imp_code, clean);
    assert_ne!(rt_code, clean);

    // Cross-check with per-case exit codes.
    let imp_case_code = doc.get("import_case").and_then(|v| v.get("import_unavailable_exit_code")).and_then(|v| v.as_integer()).unwrap();
    let call_case_code = doc.get("call_case").and_then(|v| v.get("call_failure_exit_code")).and_then(|v| v.as_integer()).unwrap();
    assert_eq!(imp_case_code, imp_code, "[import_case].import_unavailable_exit_code must match the cross-cutting code");
    assert_eq!(call_case_code, rt_code, "[call_case].call_failure_exit_code must match the cross-cutting code");
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module", "fixture_module", "exported_function",
        "returned_value", "expected_returned_value",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &["user_facing_import_succeeds", "exported_function_returns_expected_value"] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("importing_every_cclab_library").and_then(|v| v.as_bool()), Some(true));
}
