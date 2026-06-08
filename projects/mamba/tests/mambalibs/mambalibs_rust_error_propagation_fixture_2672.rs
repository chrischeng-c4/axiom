//! Schema gate for the mambalibs Rust error propagation fixture —
//! closes #2672.
//!
//! Acceptance (issue #2672):
//!
//!   1. Fixture fails if the error crashes the interpreter.
//!      `[crash_guard]` pins fail flags for SIGSEGV, abort,
//!      panic-aborts-process, and crash exit codes.
//!   2. Exception message includes the fixture error code or text.
//!      `[error_call_case].must_include_error_code` and
//!      `must_include_error_text` cross-reference
//!      `[error_payload]`; the runner contract carries
//!      `exception_message` as a first-class key.
//!   3. Positive import and call fixtures remain separate.
//!      `[isolation_from_positive_cases]` pins cross-fixture
//!      invariants against #2666 and #2667.
//!
//! Cheap test — single TOML read + field walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("rust_error_propagation")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_rust_error_propagation_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_rust_error_propagation"),
        "`fixture` must be \"mambalibs_rust_error_propagation\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2672),
        "`issue` must record #2672"
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
        Some("rust_error_propagation"),
        "`family` must be \"rust_error_propagation\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn mambalibs_rust_error_binding_pins_exported_error_function() {
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
    let exported = bind
        .get("exported_error_function")
        .and_then(|v| v.as_str())
        .expect("`[binding].exported_error_function` must be set");
    assert!(
        !exported.is_empty(),
        "`[binding].exported_error_function` must be non-empty"
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
        "import statement {import_stmt:?} must reference the error function {exported:?}"
    );
}

#[test]
fn mambalibs_rust_error_payload_is_deterministic_and_small() {
    let doc = load_toml(&manifest_path());
    let payload = doc
        .get("error_payload")
        .and_then(|v| v.as_table())
        .expect("missing `[error_payload]` block");

    let code = payload
        .get("error_code")
        .and_then(|v| v.as_str())
        .expect("`[error_payload].error_code` must be set");
    assert!(
        !code.is_empty(),
        "`[error_payload].error_code` must be non-empty"
    );

    let text = payload
        .get("error_text")
        .and_then(|v| v.as_str())
        .expect("`[error_payload].error_text` must be set");
    assert!(
        !text.is_empty(),
        "`[error_payload].error_text` must be non-empty"
    );

    let exception_type = payload
        .get("exception_type")
        .and_then(|v| v.as_str())
        .expect("`[error_payload].exception_type` must be set");
    assert!(
        !exception_type.is_empty(),
        "`[error_payload].exception_type` must be non-empty"
    );

    for flag in &["must_be_deterministic", "must_be_small_and_local"] {
        assert_eq!(
            payload.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[error_payload].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_rust_error_call_case_includes_code_and_text() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("error_call_case").and_then(|v| v.as_table()).expect(
        "missing `[error_call_case]` block \
         (acceptance: \"Exception message includes the fixture error \
         code or text.\")",
    );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("rust_error_propagated"),
        "`[error_call_case].case` must be \"rust_error_propagated\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[error_call_case].expected_outcome` must be \"pass\" \
         (the runner's job is to prove the exception was raised; \
         exception raised → case passes)"
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[error_call_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("must_raise_exception").and_then(|v| v.as_bool()),
        Some(true),
        "`[error_call_case].must_raise_exception` must be true"
    );

    // Code and text in the case MUST match the payload — the
    // exception message is the contract surface.
    let payload_code = doc
        .get("error_payload")
        .and_then(|v| v.get("error_code"))
        .and_then(|v| v.as_str())
        .expect("`[error_payload].error_code` must be set");
    let payload_text = doc
        .get("error_payload")
        .and_then(|v| v.get("error_text"))
        .and_then(|v| v.as_str())
        .expect("`[error_payload].error_text` must be set");
    let payload_type = doc
        .get("error_payload")
        .and_then(|v| v.get("exception_type"))
        .and_then(|v| v.as_str())
        .expect("`[error_payload].exception_type` must be set");

    assert_eq!(
        case.get("must_include_error_code").and_then(|v| v.as_str()),
        Some(payload_code),
        "`[error_call_case].must_include_error_code` must equal `[error_payload].error_code`"
    );
    assert_eq!(
        case.get("must_include_error_text").and_then(|v| v.as_str()),
        Some(payload_text),
        "`[error_call_case].must_include_error_text` must equal `[error_payload].error_text`"
    );
    assert_eq!(
        case.get("expected_exception_type").and_then(|v| v.as_str()),
        Some(payload_type),
        "`[error_call_case].expected_exception_type` must equal `[error_payload].exception_type`"
    );

    // And the case must name the offending function — matching
    // the binding.
    let exported = doc
        .get("binding")
        .and_then(|v| v.get("exported_error_function"))
        .and_then(|v| v.as_str())
        .expect("`[binding].exported_error_function` must be set");
    assert_eq!(
        case.get("must_name_offending_function").and_then(|v| v.as_str()),
        Some(exported),
        "`[error_call_case].must_name_offending_function` must equal `[binding].exported_error_function`"
    );
}

#[test]
fn mambalibs_rust_error_crash_guard_pins_all_fail_flags() {
    let doc = load_toml(&manifest_path());
    let guard = doc.get("crash_guard").and_then(|v| v.as_table()).expect(
        "missing `[crash_guard]` block \
         (acceptance: \"Fixture fails if the error crashes the \
         interpreter.\")",
    );

    for flag in &[
        "fail_if_interpreter_segfaults",
        "fail_if_interpreter_aborts",
        "fail_if_rust_panic_aborts_process",
        "fail_if_exit_code_indicates_crash",
        "diagnostic_must_name_crash_mode",
    ] {
        assert_eq!(
            guard.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[crash_guard].{flag}` must be true"
        );
    }

    assert_eq!(
        guard.get("pinned_clean_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[crash_guard].pinned_clean_exit_code` must be 0 — non-zero exits beyond \
         that are crash indicators"
    );
}

#[test]
fn mambalibs_rust_error_isolation_keeps_positive_fixtures_separate() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("isolation_from_positive_cases").and_then(|v| v.as_table()).expect(
        "missing `[isolation_from_positive_cases]` block \
         (acceptance: \"Positive import and call fixtures remain \
         separate.\")",
    );

    assert_eq!(
        block.get("type_roundtrip_fixture_issue").and_then(|v| v.as_integer()),
        Some(2666),
        "`type_roundtrip_fixture_issue` must record #2666"
    );
    assert_eq!(
        block.get("dir_help_fixture_issue").and_then(|v| v.as_integer()),
        Some(2667),
        "`dir_help_fixture_issue` must record #2667"
    );
    for flag in &[
        "type_roundtrip_fixture_must_remain_green",
        "dir_help_fixture_must_remain_green",
        "this_fixture_must_not_affect_positive_cases",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[isolation_from_positive_cases].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_rust_error_isolation_pins_no_global_state() {
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
fn mambalibs_rust_error_runner_contract_declares_keys_and_cases() {
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
        "exception_type",
        "exception_message",
        "error_code",
        "crash_mode",
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
    assert!(
        cases.contains(&"rust_error_propagated"),
        "`[runner_contract].case_values` must include `rust_error_propagated`; \
         got {cases:?}"
    );
}

#[test]
fn mambalibs_rust_error_pins_out_of_scope_per_issue_2672() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("full_exception_hierarchy_design").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].full_exception_hierarchy_design` must be true \
         (issue text: \"Out of scope: full exception hierarchy design.\")"
    );
}
