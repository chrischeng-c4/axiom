//! Schema gate for the third-party httpx mock transport fixture —
//! closes #2645.
//!
//! Acceptance (issue #2645):
//!
//!   1. Fixture fails if httpx cannot import.
//!      `[import_failure_contract]` pins must_fail_on_import_error +
//!      must_fail_on_missing_httpx_module +
//!      forbid_silent_fallback_when_httpx_missing + exit 155.
//!   2. Fixture performs no external network I/O.
//!      `[no_external_network_io_contract]` pins
//!      must_not_perform_network_io + must_not_perform_dns_resolution
//!      + must_not_open_socket + forbid_use_of_socket_connect +
//!      forbid_use_of_httpcore_real_pool +
//!      must_use_mock_transport_for_request + exit 156.
//!   3. Failure output distinguishes transport setup from response
//!      parsing.
//!      `[transport_setup_vs_response_parsing_contract]` pins
//!      must_distinguish_transport_setup_failure_from_response_parsing_failure
//!      + distinct exit codes 157 (transport_setup) / 158
//!      (response_parsing) +
//!      must_emit_failure_phase_field_in_runner_output +
//!      allowed_failure_phase_values=[transport_setup,
//!      response_parsing] + forbid_collapsed_or_implicit_failure_phase.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("third_party")
        .join("httpx_mock_transport_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("third_party_httpx_mock_transport_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2645));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("third_party"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("third_party_httpx_mock_transport_behavioral"));
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = crate::common::load_toml(&manifest_path());
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
fn python_target_is_pinned_to_3_12() {
    let doc = crate::common::load_toml(&manifest_path());
    let p = doc.get("python_target").and_then(|v| v.as_table()).expect("[python_target] missing");
    assert_eq!(p.get("python_major").and_then(|v| v.as_integer()), Some(3));
    assert_eq!(p.get("python_minor").and_then(|v| v.as_integer()), Some(12));
    assert_eq!(p.get("must_be_python_3_12").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn surface_covers_httpx() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(modules.contains(&"httpx"), "covered_modules must include httpx");
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_httpx_in_ecosystem_manifest",
        "must_cover_httpx_mocktransport_construction",
        "must_cover_httpx_client_construction_with_mock_transport",
        "must_cover_request_method",
        "must_cover_request_url",
        "must_cover_response_status",
        "must_cover_response_json_or_text",
        "must_use_synchronous_client",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
    assert_eq!(s.get("import_statement").and_then(|v| v.as_str()), Some("import httpx"));
}

#[test]
fn deterministic_sample_covers_request_flow() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let arr = doc.get("request_cases").and_then(|v| v.as_array()).expect("[[request_cases]] missing");
    assert!(!arr.is_empty(), "request_cases must not be empty");
    for c in arr {
        let t = c.as_table().expect("case must be a table");
        for f in &[
            "request_method", "request_url",
            "mock_response_status", "mock_response_json_python_repr",
            "expected_response_status", "expected_response_json_python_repr",
            "expected_response_text_substring",
        ] {
            assert!(t.get(*f).is_some(), "request_cases.{f} missing");
        }
    }
}

// Acceptance: "Fixture fails if httpx cannot import."
#[test]
fn fixture_fails_if_httpx_cannot_import() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("import_failure_contract").and_then(|v| v.as_table()).expect(
        "[import_failure_contract] missing — acceptance: \
         \"Fixture fails if httpx cannot import.\"",
    );
    for k in &[
        "must_fail_on_import_error",
        "must_fail_on_missing_httpx_module",
        "must_emit_import_failure_kind_when_httpx_missing",
        "forbid_silent_fallback_when_httpx_missing",
    ] {
        assert_eq!(i.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = i.get("httpx_import_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 155);
    assert_eq!(
        i.get("httpx_import_failure_kind").and_then(|v| v.as_str()),
        Some("third_party_httpx_import_failed"),
    );
}

// Acceptance: "Fixture performs no external network I/O."
#[test]
fn fixture_performs_no_external_network_io() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("no_external_network_io_contract").and_then(|v| v.as_table()).expect(
        "[no_external_network_io_contract] missing — acceptance: \
         \"Fixture performs no external network I/O.\"",
    );
    for k in &[
        "must_not_perform_network_io",
        "must_not_perform_dns_resolution",
        "must_not_open_socket",
        "forbid_use_of_socket_connect",
        "forbid_use_of_httpcore_real_pool",
        "must_use_mock_transport_for_request",
    ] {
        assert_eq!(n.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = n.get("network_io_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 156);
    assert_eq!(
        n.get("network_io_failure_kind").and_then(|v| v.as_str()),
        Some("httpx_external_network_io_used"),
    );
}

// Acceptance: "Failure output distinguishes transport setup from
// response parsing."
#[test]
fn failure_output_distinguishes_transport_setup_from_response_parsing() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc.get("transport_setup_vs_response_parsing_contract").and_then(|v| v.as_table()).expect(
        "[transport_setup_vs_response_parsing_contract] missing — acceptance: \
         \"Failure output distinguishes transport setup from response parsing.\"",
    );
    for k in &[
        "must_distinguish_transport_setup_failure_from_response_parsing_failure",
        "must_emit_failure_phase_field_in_runner_output",
        "forbid_collapsed_or_implicit_failure_phase",
    ] {
        assert_eq!(t.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let setup = t.get("transport_setup_exit_code").and_then(|v| v.as_integer()).unwrap();
    let parse = t.get("response_parsing_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(setup, 157);
    assert_eq!(parse, 158);
    assert_ne!(setup, parse, "transport-setup and response-parsing exit codes must differ");
    assert_eq!(
        t.get("transport_setup_failure_kind").and_then(|v| v.as_str()),
        Some("httpx_mock_transport_setup_failed"),
    );
    assert_eq!(
        t.get("response_parsing_failure_kind").and_then(|v| v.as_str()),
        Some("httpx_response_parsing_failed"),
    );
    assert_eq!(
        t.get("failure_phase_field_name").and_then(|v| v.as_str()),
        Some("failure_phase"),
    );
    let allowed: Vec<&str> = t.get("allowed_failure_phase_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["transport_setup", "response_parsing"] {
        assert!(allowed.contains(v), "allowed_failure_phase_values must include {v}");
    }
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module_name",
        "request_method", "request_url",
        "mock_response_status", "mock_response_json_python_repr",
        "expected_response_status", "expected_response_json_python_repr",
        "expected_response_text_substring",
        "failure_phase", "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_if_httpx_cannot_import",
        "fixture_performs_no_external_network_io",
        "failure_output_distinguishes_transport_setup_from_response_parsing",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("async_http").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(o.get("tls_behavior").and_then(|v| v.as_bool()), Some(true));
}
