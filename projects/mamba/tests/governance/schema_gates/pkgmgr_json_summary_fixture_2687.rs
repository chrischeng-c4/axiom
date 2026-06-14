//! Schema gate for the package-manager JSON summary fixture —
//! closes #2687.
//!
//! Acceptance (issue #2687):
//!
//!   1. JSON summary can be parsed without log scraping.
//!      `[envelope]` pins `format = "json"`, single object per
//!      invocation, strict JSON.
//!   2. Missing required fields fail the test.
//!      `[missing_field_assertion].fail_on_missing_required = true`,
//!      plus a report-all-missing + name-in-diagnostic invariant.
//!   3. Failure path includes command, exit status, and diagnostic
//!      message.
//!      `[failure_case]` pins all three `must_include_*` flags and
//!      a substring the diagnostic must carry.
//!
//! Out of scope (per issue body): UI or report rendering.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("json_summary")
        .join("manifest.toml")
}

#[test]
fn pkgmgr_json_summary_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_json_summary"),
        "`fixture` must be \"pkgmgr_json_summary\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2687),
        "`issue` must record #2687"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("json_summary"),
        "`family` must be \"json_summary\""
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager"),
        "`profile` must be \"package_manager\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn pkgmgr_json_summary_envelope_is_strict_json() {
    let doc = crate::common::load_toml(&manifest_path());
    let env = doc.get("envelope").and_then(|v| v.as_table()).expect(
        "missing `[envelope]` block \
         (acceptance: \"JSON summary can be parsed without log scraping.\")",
    );

    assert_eq!(
        env.get("format").and_then(|v| v.as_str()),
        Some("json"),
        "`[envelope].format` must be \"json\""
    );
    assert_eq!(
        env.get("stream").and_then(|v| v.as_str()),
        Some("stdout"),
        "`[envelope].stream` must be \"stdout\" — workers parse stdout, not stderr"
    );
    assert_eq!(
        env.get("one_object_per_invocation").and_then(|v| v.as_bool()),
        Some(true),
        "`[envelope].one_object_per_invocation` must be true"
    );
    assert_eq!(
        env.get("strict_json").and_then(|v| v.as_bool()),
        Some(true),
        "`[envelope].strict_json` must be true — no JSON5/comments/trailing commas"
    );

    let schema_version = env
        .get("schema_version")
        .and_then(|v| v.as_integer())
        .expect("`[envelope].schema_version` must be set");
    assert!(
        schema_version >= 1,
        "`[envelope].schema_version` must be >= 1; got {schema_version}"
    );
}

#[test]
fn pkgmgr_json_summary_covers_all_required_verbs() {
    let doc = crate::common::load_toml(&manifest_path());
    let covered = doc.get("covered_workflows").and_then(|v| v.as_table()).expect(
        "missing `[covered_workflows]` block \
         (acceptance: \"Emit or assert JSON summary fields for init, lock, sync, \
         install, run, and diagnostics.\")",
    );

    let verbs: Vec<&str> = covered
        .get("verbs")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["init", "lock", "sync", "install", "run", "diagnostics"] {
        assert!(
            verbs.contains(required),
            "`[covered_workflows].verbs` must include `{required}`; got {verbs:?}"
        );
    }
}

#[test]
fn pkgmgr_json_summary_required_fields_pin_shared_and_per_verb() {
    let doc = crate::common::load_toml(&manifest_path());
    let req = doc.get("required_fields").and_then(|v| v.as_table()).expect(
        "missing `[required_fields]` block \
         (acceptance: \"Include package count, environment path, lockfile path, \
         and offline source identity.\")",
    );

    let shared: Vec<&str> = req
        .get("shared")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "verb",
        "outcome",
        "exit_code",
        "package_count",
        "environment_path",
        "lockfile_path",
        "offline_source",
        "schema_version",
    ] {
        assert!(
            shared.contains(required),
            "`[required_fields].shared` must include `{required}`; got {shared:?}"
        );
    }

    // Each per-verb list must be non-empty.
    for verb in &["init", "lock", "sync", "install", "run", "diagnostics"] {
        let extra: Vec<&str> = req
            .get(*verb)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        assert!(
            !extra.is_empty(),
            "`[required_fields].{verb}` must list at least one per-verb field; got {extra:?}"
        );
    }
}

#[test]
fn pkgmgr_json_summary_missing_field_assertion_fails_loud() {
    let doc = crate::common::load_toml(&manifest_path());
    let mfa = doc.get("missing_field_assertion").and_then(|v| v.as_table()).expect(
        "missing `[missing_field_assertion]` block \
         (acceptance: \"Missing required fields fail the test.\")",
    );

    for flag in &[
        "fail_on_missing_required",
        "report_all_missing_fields",
        "must_name_missing_field_in_diagnostic",
    ] {
        assert_eq!(
            mfa.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[missing_field_assertion].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_json_summary_success_case_covers_happy_path() {
    let doc = crate::common::load_toml(&manifest_path());
    let success = doc
        .get("success_case")
        .and_then(|v| v.as_table())
        .expect("missing `[success_case]` block");

    let workflow = success
        .get("workflow")
        .and_then(|v| v.as_str())
        .expect("`[success_case].workflow` must name the verb");
    assert!(
        !workflow.is_empty(),
        "success workflow name must be non-empty"
    );

    assert_eq!(
        success.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[success_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        success.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[success_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        success
            .get("must_include_package_count_greater_than_zero")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[success_case].must_include_package_count_greater_than_zero` must be true"
    );
}

#[test]
fn pkgmgr_json_summary_failure_case_includes_command_status_and_diagnostic() {
    let doc = crate::common::load_toml(&manifest_path());
    let fail = doc.get("failure_case").and_then(|v| v.as_table()).expect(
        "missing `[failure_case]` block \
         (acceptance: \"Failure path includes command, exit status, and diagnostic message.\")",
    );

    assert_eq!(
        fail.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[failure_case].expected_outcome` must be \"fail\""
    );

    let exit = fail
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[failure_case].expected_exit_code` must be set");
    assert_ne!(exit, 0, "`[failure_case].expected_exit_code` must be non-zero; got {exit}");

    for flag in &[
        "must_include_command",
        "must_include_exit_status",
        "must_include_diagnostic_message",
    ] {
        assert_eq!(
            fail.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[failure_case].{flag}` must be true"
        );
    }

    let substring = fail
        .get("diagnostic_message_substring")
        .and_then(|v| v.as_str())
        .expect("`[failure_case].diagnostic_message_substring` must be set");
    assert!(
        !substring.is_empty(),
        "`[failure_case].diagnostic_message_substring` must be non-empty"
    );
}

#[test]
fn pkgmgr_json_summary_isolation_pins_no_global_state() {
    let doc = crate::common::load_toml(&manifest_path());
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
fn pkgmgr_json_summary_runner_contract_declares_outcome_keys() {
    let doc = crate::common::load_toml(&manifest_path());
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
        "verb",
        "schema_version",
        "exit_code",
        "package_count",
        "environment_path",
        "lockfile_path",
        "offline_source",
        "diagnostic_message",
        "command",
    ] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }
}

#[test]
fn pkgmgr_json_summary_pins_out_of_scope_per_issue_2687() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("ui_or_report_rendering").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].ui_or_report_rendering` must be true \
         (issue text: \"Out of scope: UI or report rendering.\")"
    );
}
