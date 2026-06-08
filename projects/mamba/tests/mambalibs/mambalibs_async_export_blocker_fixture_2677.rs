//! Schema gate for the mambalibs async export blocker fixture —
//! closes #2677.
//!
//! Acceptance (issue #2677):
//!
//!   1. Async export support status appears in mambalibs gate
//!      summary.
//!      `[gate_summary_contract]` pins
//!      `field_name = "async_export_status"`; the runner contract
//!      carries `async_export_status` as a first-class key.
//!   2. Unsupported behavior is not silently skipped.
//!      `[unsupported_behavior_contract]` pins
//!      `distinguish_blocker_from_skip = true` and
//!      `must_not_silently_skip = true`; "blocked" appears in
//!      `[runner_contract].outcome_values`.
//!   3. Passing fixture avoids wall-clock sleeps.
//!      `[no_wall_clock_sleeps]` pins `max_sleep_seconds = 0`,
//!      `forbid_thread_sleep`, `forbid_asyncio_sleep_above_zero`,
//!      and `pass_path_must_be_compute_only`.
//!
//! Cheap test — single TOML read + field walk. Runs in well under
//! a second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("async_export_blocker")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_async_export_blocker_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_async_export_blocker"),
        "`fixture` must be \"mambalibs_async_export_blocker\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2677),
        "`issue` must record #2677"
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
        Some("async_export_blocker"),
        "`family` must be \"async_export_blocker\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn mambalibs_async_binding_declares_async_function_and_import() {
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
        .get("exported_async_function")
        .and_then(|v| v.as_str())
        .expect("`[binding].exported_async_function` must be set");
    assert!(
        !exported.is_empty(),
        "`[binding].exported_async_function` must be non-empty"
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
        "import statement {import_stmt:?} must reference the async function {exported:?}"
    );

    let signature = bind
        .get("function_signature")
        .and_then(|v| v.as_str())
        .expect("`[binding].function_signature` must be set");
    assert!(
        signature.starts_with("async def"),
        "`[binding].function_signature` must begin with `async def`; got {signature:?}"
    );
    assert!(
        signature.contains(exported),
        "`[binding].function_signature` must name the exported function {exported:?}; got \
         {signature:?}"
    );
}

#[test]
fn mambalibs_async_support_status_enum_is_well_formed() {
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
    assert!(
        allowed.contains(&default),
        "`[support_status].default_status` `{default}` must be in allowed_values; got \
         {allowed:?}"
    );

    let current = block
        .get("current_status")
        .and_then(|v| v.as_str())
        .expect("`[support_status].current_status` must be set");
    assert!(
        allowed.contains(&current),
        "`[support_status].current_status` `{current}` must be in allowed_values; got \
         {allowed:?}"
    );

    assert_eq!(
        block.get("status_must_be_one_of_allowed").and_then(|v| v.as_bool()),
        Some(true),
        "`[support_status].status_must_be_one_of_allowed` must be true"
    );
}

#[test]
fn mambalibs_async_pass_case_is_compute_only_and_deterministic() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("pass_case")
        .and_then(|v| v.as_table())
        .expect("missing `[pass_case]` block");

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("async_export_supported"),
        "`[pass_case].case` must be \"async_export_supported\""
    );
    assert_eq!(
        case.get("status_under_which_applicable").and_then(|v| v.as_str()),
        Some("pass"),
        "`[pass_case].status_under_which_applicable` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[pass_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[pass_case].expected_exit_code` must be 0"
    );

    for flag in &[
        "must_call_async_function",
        "must_use_asyncio_run",
        "must_assert_deterministic_value",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[pass_case].{flag}` must be true"
        );
    }

    // Round-trip determinism — input == expected_return_value.
    let input = case
        .get("deterministic_input")
        .and_then(|v| v.as_integer())
        .expect("`[pass_case].deterministic_input` must be an integer");
    let expected = case
        .get("deterministic_expected_return_value")
        .and_then(|v| v.as_integer())
        .expect("`[pass_case].deterministic_expected_return_value` must be an integer");
    assert_eq!(
        input, expected,
        "`[pass_case].deterministic_input` must equal `deterministic_expected_return_value`"
    );
}

#[test]
fn mambalibs_async_xfail_case_surfaces_exception_not_crash() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("xfail_case")
        .and_then(|v| v.as_table())
        .expect("missing `[xfail_case]` block");

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("async_export_xfail"),
        "`[xfail_case].case` must be \"async_export_xfail\""
    );
    assert_eq!(
        case.get("status_under_which_applicable").and_then(|v| v.as_str()),
        Some("xfail"),
        "`[xfail_case].status_under_which_applicable` must be \"xfail\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[xfail_case].expected_outcome` must be \"fail\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(1),
        "`[xfail_case].expected_exit_code` must be 1"
    );
    for flag in &["must_raise_exception", "must_not_crash_interpreter"] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[xfail_case].{flag}` must be true"
        );
    }
    let exc = case
        .get("expected_exception_type")
        .and_then(|v| v.as_str())
        .expect("`[xfail_case].expected_exception_type` must be set");
    assert!(!exc.is_empty(), "exception type must be non-empty");
}

#[test]
fn mambalibs_async_blocker_case_emits_structured_blocker() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("blocker_case").and_then(|v| v.as_table()).expect(
        "missing `[blocker_case]` block \
         (acceptance: \"Unsupported behavior is not silently skipped.\")",
    );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("async_export_blocker"),
        "`[blocker_case].case` must be \"async_export_blocker\""
    );
    assert_eq!(
        case.get("status_under_which_applicable").and_then(|v| v.as_str()),
        Some("blocker"),
        "`[blocker_case].status_under_which_applicable` must be \"blocker\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("blocked"),
        "`[blocker_case].expected_outcome` must be \"blocked\" — \
         unsupported behavior is NOT silently skipped"
    );
    assert_eq!(
        case.get("linked_blocker_issue").and_then(|v| v.as_integer()),
        Some(2677),
        "`[blocker_case].linked_blocker_issue` must record #2677"
    );
    for flag in &[
        "must_emit_structured_blocker",
        "must_not_attempt_call",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[blocker_case].{flag}` must be true"
        );
    }
    assert_eq!(
        case.get("must_name_offending_feature").and_then(|v| v.as_str()),
        Some("async_export"),
        "`[blocker_case].must_name_offending_feature` must be \"async_export\""
    );

    // The "blocked" outcome MUST appear in runner_contract.outcome_values.
    let outcomes: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("outcome_values"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        outcomes.contains(&"blocked"),
        "`[runner_contract].outcome_values` MUST include `blocked` so the blocker outcome \
         is not silently skipped; got {outcomes:?}"
    );
}

#[test]
fn mambalibs_async_unsupported_behavior_contract_blocks_silent_skip() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("unsupported_behavior_contract").and_then(|v| v.as_table()).expect(
        "missing `[unsupported_behavior_contract]` block \
         (acceptance: \"Unsupported behavior is not silently skipped.\")",
    );

    for flag in &[
        "distinguish_blocker_from_skip",
        "distinguish_blocker_from_fail",
        "must_not_silently_skip",
        "must_not_silently_pass",
        "must_surface_blocker_in_gate_summary",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[unsupported_behavior_contract].{flag}` must be true"
        );
    }

    assert_eq!(
        block.get("linked_blocker_issue").and_then(|v| v.as_integer()),
        Some(2677),
        "`[unsupported_behavior_contract].linked_blocker_issue` must record #2677"
    );
}

#[test]
fn mambalibs_async_no_wall_clock_sleeps_pins_zero_sleep() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("no_wall_clock_sleeps").and_then(|v| v.as_table()).expect(
        "missing `[no_wall_clock_sleeps]` block \
         (acceptance: \"Passing fixture avoids wall-clock sleeps.\")",
    );

    for flag in &[
        "forbid_thread_sleep",
        "forbid_asyncio_sleep_above_zero",
        "pass_path_must_be_compute_only",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[no_wall_clock_sleeps].{flag}` must be true"
        );
    }

    assert_eq!(
        block.get("max_sleep_seconds").and_then(|v| v.as_integer()),
        Some(0),
        "`[no_wall_clock_sleeps].max_sleep_seconds` must be 0"
    );

    let rationale = block
        .get("rationale")
        .and_then(|v| v.as_str())
        .expect("`[no_wall_clock_sleeps].rationale` must be set");
    assert!(
        !rationale.is_empty(),
        "`[no_wall_clock_sleeps].rationale` must be non-empty"
    );
}

#[test]
fn mambalibs_async_gate_summary_contract_pins_async_export_status_field() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("gate_summary_contract").and_then(|v| v.as_table()).expect(
        "missing `[gate_summary_contract]` block \
         (acceptance: \"Async export support status appears in mambalibs gate summary.\")",
    );

    assert_eq!(
        block.get("field_name").and_then(|v| v.as_str()),
        Some("async_export_status"),
        "`[gate_summary_contract].field_name` must be \"async_export_status\""
    );
    assert_eq!(
        block.get("field_source_block").and_then(|v| v.as_str()),
        Some("support_status"),
        "`[gate_summary_contract].field_source_block` must be \"support_status\""
    );
    assert_eq!(
        block.get("field_source_key").and_then(|v| v.as_str()),
        Some("current_status"),
        "`[gate_summary_contract].field_source_key` must be \"current_status\""
    );

    // Allowed field values MUST match support_status.allowed_values.
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

    // The field name MUST appear in runner_contract.keys.
    let contract_keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        contract_keys.contains(&"async_export_status"),
        "`[runner_contract].keys` must include `async_export_status`; got {contract_keys:?}"
    );
}

#[test]
fn mambalibs_async_isolation_pins_no_global_state() {
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
fn mambalibs_async_runner_contract_declares_keys_and_cases() {
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
        "exported_async_function",
        "async_export_status",
        "linked_blocker_issue",
        "asyncio_run_invoked",
        "wall_clock_seconds",
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
        "async_export_supported",
        "async_export_xfail",
        "async_export_blocker",
    ] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_async_pins_out_of_scope_per_issue_2677() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("full_async_runtime_integration").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].full_async_runtime_integration` must be true \
         (issue text: \"Out of scope: full async runtime integration.\")"
    );
}
