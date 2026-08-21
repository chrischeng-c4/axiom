//! Schema gate for the third-party urllib3 response fixture —
//! closes #2643.
//!
//! Acceptance (issue #2643):
//!
//!   1. Fixture fails if urllib3 cannot import.
//!      `[import_failure_contract]` pins must_fail_on_import_error +
//!      must_fail_on_missing_urllib3_module +
//!      forbid_silent_fallback_when_urllib3_missing + exit 147.
//!   2. No external network I/O occurs.
//!      `[no_external_network_io_contract]` pins
//!      must_not_perform_network_io + must_not_open_connection_pool
//!      + must_not_perform_dns_resolution +
//!      forbid_use_of_socket_connect +
//!      forbid_use_of_urllib3_poolmanager_request +
//!      forbid_use_of_urllib3_connectionfromurl +
//!      must_use_in_memory_body_for_response + distinct exit codes
//!      148 (network) / 149 (pool) +
//!      must_distinguish_network_io_from_connection_pool_use.
//!   3. Runner records it under HTTP-client dependency coverage.
//!      `[http_client_coverage_reporting_contract]` pins
//!      must_emit_http_client_dependency_coverage_in_runner_output
//!      + required_http_client_dependencies_covered ⊇ [urllib3] +
//!      must_emit_summary_record_with_http_client_coverage +
//!      forbid_silent_or_implicit_http_client_coverage + exit 150.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("third_party")
        .join("urllib3_response_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("third_party_urllib3_response_behavioral")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2643));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2529)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("third_party")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("third_party_urllib3_response_behavioral")
    );
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
    let p = doc
        .get("python_target")
        .and_then(|v| v.as_table())
        .expect("[python_target] missing");
    assert_eq!(p.get("python_major").and_then(|v| v.as_integer()), Some(3));
    assert_eq!(p.get("python_minor").and_then(|v| v.as_integer()), Some(12));
    assert_eq!(
        p.get("must_be_python_3_12").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn surface_covers_urllib3() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc
        .get("surface")
        .and_then(|v| v.as_table())
        .expect("[surface] missing");
    let modules: Vec<&str> = s
        .get("covered_modules")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        modules.contains(&"urllib3"),
        "covered_modules must include urllib3"
    );
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_urllib3_in_ecosystem_manifest",
        "must_cover_urllib3_httpresponse_construction",
        "must_cover_status_access",
        "must_cover_headers_access",
        "must_cover_data_access",
        "must_construct_response_from_in_memory_body",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    assert_eq!(
        s.get("import_statement").and_then(|v| v.as_str()),
        Some("import urllib3")
    );
}

#[test]
fn deterministic_sample_covers_response_construction() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc
        .get("deterministic_sample")
        .and_then(|v| v.as_table())
        .expect("[deterministic_sample] missing");
    assert_eq!(
        d.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
    let max = d
        .get("sample_max_records")
        .and_then(|v| v.as_integer())
        .unwrap();
    let min = d
        .get("sample_min_records")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert!(
        min >= 1 && max >= min && max <= 64,
        "sample bounds must be sane"
    );

    let arr = doc
        .get("response_cases")
        .and_then(|v| v.as_array())
        .expect("[[response_cases]] missing");
    assert!(!arr.is_empty(), "response_cases must not be empty");
    for c in arr {
        let t = c.as_table().expect("case must be a table");
        for f in &[
            "body_bytes_python_repr",
            "status",
            "headers_python_repr",
            "expected_status",
            "expected_data_bytes_python_repr",
            "expected_content_type_header",
            "expected_content_length_header",
        ] {
            assert!(t.get(*f).is_some(), "response_cases.{f} missing");
        }
    }
}

// Acceptance: "Fixture fails if urllib3 cannot import."
#[test]
fn fixture_fails_if_urllib3_cannot_import() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc
        .get("import_failure_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[import_failure_contract] missing — acceptance: \
         \"Fixture fails if urllib3 cannot import.\"",
        );
    for k in &[
        "must_fail_on_import_error",
        "must_fail_on_missing_urllib3_module",
        "must_emit_import_failure_kind_when_urllib3_missing",
        "forbid_silent_fallback_when_urllib3_missing",
    ] {
        assert_eq!(
            i.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exit = i
        .get("urllib3_import_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 147);
    assert_eq!(
        i.get("urllib3_import_failure_kind")
            .and_then(|v| v.as_str()),
        Some("third_party_urllib3_import_failed"),
    );
}

// Acceptance: "No external network I/O occurs."
#[test]
fn no_external_network_io_occurs() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc
        .get("no_external_network_io_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[no_external_network_io_contract] missing — acceptance: \
         \"No external network I/O occurs.\"",
        );
    for k in &[
        "must_not_perform_network_io",
        "must_not_open_connection_pool",
        "must_not_perform_dns_resolution",
        "forbid_use_of_socket_connect",
        "forbid_use_of_urllib3_poolmanager_request",
        "forbid_use_of_urllib3_connectionfromurl",
        "must_use_in_memory_body_for_response",
        "must_distinguish_network_io_from_connection_pool_use",
    ] {
        assert_eq!(
            n.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let net = n
        .get("network_io_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let pool = n
        .get("connection_pool_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(net, 148);
    assert_eq!(pool, 149);
    assert_ne!(net, pool, "network and pool exit codes must differ");
    assert_eq!(
        n.get("network_io_failure_kind").and_then(|v| v.as_str()),
        Some("urllib3_external_network_io_used"),
    );
    assert_eq!(
        n.get("connection_pool_failure_kind")
            .and_then(|v| v.as_str()),
        Some("urllib3_connection_pool_used"),
    );
}

// Acceptance: "Runner records it under HTTP-client dependency
// coverage."
#[test]
fn runner_records_under_http_client_dependency_coverage() {
    let doc = crate::common::load_toml(&manifest_path());
    let h = doc
        .get("http_client_coverage_reporting_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[http_client_coverage_reporting_contract] missing — acceptance: \
         \"Runner records it under HTTP-client dependency coverage.\"",
        );
    for k in &[
        "must_emit_http_client_dependency_coverage_in_runner_output",
        "must_emit_summary_record_with_http_client_coverage",
        "forbid_silent_or_implicit_http_client_coverage",
        "must_distinguish_http_client_coverage_from_overall_outcome",
    ] {
        assert_eq!(
            h.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        h.get("http_client_coverage_field_name")
            .and_then(|v| v.as_str()),
        Some("http_client_dependencies_covered"),
    );
    let req: Vec<&str> = h
        .get("required_http_client_dependencies_covered")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        req.contains(&"urllib3"),
        "required_http_client_dependencies_covered must include urllib3"
    );
    assert_eq!(
        h.get("summary_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    let exit = h
        .get("missing_http_client_coverage_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 150);
    assert_eq!(
        h.get("missing_http_client_coverage_failure_kind")
            .and_then(|v| v.as_str()),
        Some("urllib3_http_client_coverage_missing"),
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
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
        "module_name",
        "body_bytes_python_repr",
        "status",
        "headers_python_repr",
        "expected_status",
        "expected_data_bytes_python_repr",
        "expected_content_type_header",
        "expected_content_length_header",
        "http_client_dependencies_covered",
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
    for required in &[
        "fixture_fails_if_urllib3_cannot_import",
        "no_external_network_io_occurs",
        "runner_records_under_http_client_dependency_coverage",
    ] {
        assert!(
            cases.contains(required),
            "runner_contract.case_values must include {required}"
        );
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("live_connection_pools").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(o.get("tls").and_then(|v| v.as_bool()), Some(true));
}
