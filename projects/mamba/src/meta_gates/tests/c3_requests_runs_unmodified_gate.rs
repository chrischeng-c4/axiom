#![cfg(test)]

// Locks the shape of the C3 requests-runs-unmodified fixture pinned
// by tests/governance/gates/third_party/c3_requests_runs_unmodified_gate/
// manifest.toml. Closes #1259.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/third_party/c3_requests_runs_unmodified_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(
        m["fixture"].as_str(),
        Some("c3_requests_runs_unmodified_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(1259));
    assert_eq!(m["profile"].as_str(), Some("third_party"));
    assert_eq!(
        m["family"].as_str(),
        Some("c3_requests_runs_unmodified_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
    let related: Vec<_> = m["related_issues"]
        .as_array()
        .expect("related_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(related, vec![1263, 1234, 1257, 1265]);
}

#[test]
fn isolation_pins_no_global_state() {
    let iso = &manifest()["isolation"];
    for key in [
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(iso[key].as_bool(), Some(true), "isolation.{key}");
    }
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let py = &manifest()["python_target"];
    assert_eq!(py["python_major"].as_integer(), Some(3));
    assert_eq!(py["python_minor"].as_integer(), Some(12));
    assert_eq!(py["must_be_python_3_12"].as_bool(), Some(true));
}

#[test]
fn surface_pins_all_five_requirements() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_ssl_fidelity_spike_estimate",
        "must_cover_requests_import",
        "must_cover_https_get_completes_handshake_and_request",
        "must_cover_response_fields_match_cpython",
        "must_cover_conformance_fixture_under_3p_requests_https_get",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r0_ssl_fidelity_spike_must_complete_with_written_estimate() {
    let c = &manifest()["r0_ssl_fidelity_spike_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("ssl_fidelity_spike_must_complete_with_written_estimate")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R0"));
    for key in [
        "must_audit_current_ssl_mod_surface",
        "must_attempt_real_tls_handshake_against_known_server",
        "must_emit_written_estimate_multi_week_vs_multi_month",
        "forbid_skipping_spike_before_full_implementation",
        "forbid_collapsing_spike_into_implementation_phase",
        "must_distinguish_audit_missing_from_estimate_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["ssl_mod_relative_path"].as_str(),
        Some("crates/mamba/src/runtime/stdlib/ssl_mod.rs")
    );
    assert_eq!(
        c["ssl_mod_relative_path_field_name"].as_str(),
        Some("ssl_mod_path")
    );
    assert_eq!(
        c["spike_estimate_kind_field_name"].as_str(),
        Some("ssl_spike_estimate_kind")
    );
    let kinds: Vec<_> = c["allowed_spike_estimate_kinds"]
        .as_array()
        .expect("allowed_spike_estimate_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(kinds, vec!["multi_week", "multi_month"]);
    assert_eq!(
        c["spike_target_handshake_host_field_name"].as_str(),
        Some("ssl_spike_target_host")
    );
    assert_eq!(
        c["ssl_spike_audit_missing_failure_kind"].as_str(),
        Some("c3_requests_ssl_spike_audit_missing")
    );
    assert_eq!(
        c["ssl_spike_audit_missing_exit_code"].as_integer(),
        Some(285)
    );
    assert_eq!(
        c["ssl_spike_estimate_missing_failure_kind"].as_str(),
        Some("c3_requests_ssl_spike_estimate_missing")
    );
    assert_eq!(
        c["ssl_spike_estimate_missing_exit_code"].as_integer(),
        Some(286)
    );
}

#[test]
fn r1_import_requests_resolves() {
    let c = &manifest()["r1_requests_import_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("import_requests_resolves_and_module_object_present")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_install_requests_unmodified_from_pypi",
        "must_resolve_import_requests",
        "must_expose_requests_module_object",
        "forbid_modifying_requests_on_disk_before_run",
        "forbid_substituting_urllib3_with_mamba_shim",
        "must_distinguish_import_failure_from_shim_substitution",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["canonical_import_statement"].as_str(),
        Some("import requests")
    );
    assert_eq!(
        c["canonical_import_statement_field_name"].as_str(),
        Some("canonical_import_statement")
    );
    assert_eq!(
        c["module_object_field_name"].as_str(),
        Some("requests_module")
    );
    assert_eq!(
        c["requests_import_failure_kind"].as_str(),
        Some("c3_requests_import_failed")
    );
    assert_eq!(
        c["requests_import_failure_exit_code"].as_integer(),
        Some(287)
    );
    assert_eq!(
        c["requests_shim_substitution_failure_kind"].as_str(),
        Some("c3_requests_urllib3_substituted_with_mamba_shim")
    );
    assert_eq!(
        c["requests_shim_substitution_failure_exit_code"].as_integer(),
        Some(288)
    );
}

#[test]
fn r2_https_get_completes_handshake_and_request() {
    let c = &manifest()["r2_https_get_completes_handshake_and_request_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("requests_get_over_https_completes_dns_tls_request_and_response")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_resolve_dns_for_target_host",
        "must_complete_tls_handshake_against_target_host",
        "must_send_http_get_after_handshake",
        "must_receive_http_response_after_send",
        "forbid_skipping_tls_handshake",
        "forbid_falling_back_to_plain_http_when_https_requested",
        "must_distinguish_tls_handshake_failure_from_plain_http_fallback",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["default_loopback_target_url"].as_str(),
        Some("https://127.0.0.1:8443/get")
    );
    assert_eq!(
        c["default_loopback_target_url_field_name"].as_str(),
        Some("loopback_target_url")
    );
    assert_eq!(
        c["target_url_field_name"].as_str(),
        Some("https_target_url")
    );
    assert_eq!(
        c["phases_completed_field_name"].as_str(),
        Some("https_phases_completed")
    );
    let phases: Vec<_> = c["required_phases"]
        .as_array()
        .expect("required_phases")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        phases,
        vec![
            "dns_resolve",
            "tls_handshake",
            "request_send",
            "response_receive",
        ]
    );
    assert_eq!(
        c["tls_handshake_failure_kind"].as_str(),
        Some("c3_requests_tls_handshake_failure")
    );
    assert_eq!(c["tls_handshake_failure_exit_code"].as_integer(), Some(289));
    assert_eq!(
        c["http_plain_fallback_failure_kind"].as_str(),
        Some("c3_requests_plain_http_fallback_when_https_requested")
    );
    assert_eq!(
        c["http_plain_fallback_failure_exit_code"].as_integer(),
        Some(290)
    );
}

#[test]
fn r3_response_fields_match_cpython() {
    let c = &manifest()["r3_response_fields_match_cpython_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("response_status_code_headers_json_and_text_match_cpython")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_record_status_code_int",
        "must_record_headers_mapping",
        "must_record_json_dict_when_application_json",
        "must_record_text_str",
        "forbid_dropping_status_code_field",
        "forbid_collapsing_headers_into_string_blob",
        "forbid_silently_returning_bytes_for_text",
        "must_distinguish_status_code_divergence_from_body_divergence",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_response_fields"]
        .as_array()
        .expect("required_response_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(fields, vec!["status_code", "headers", "json", "text"]);
    assert_eq!(
        c["response_fields_field_name"].as_str(),
        Some("response_fields")
    );
    assert_eq!(
        c["status_code_field_name"].as_str(),
        Some("response_status_code")
    );
    assert_eq!(c["headers_field_name"].as_str(), Some("response_headers"));
    assert_eq!(c["json_field_name"].as_str(), Some("response_json"));
    assert_eq!(c["text_field_name"].as_str(), Some("response_text"));
    assert_eq!(
        c["response_status_code_divergence_failure_kind"].as_str(),
        Some("c3_requests_response_status_code_divergence")
    );
    assert_eq!(
        c["response_status_code_divergence_exit_code"].as_integer(),
        Some(291)
    );
    assert_eq!(
        c["response_body_divergence_failure_kind"].as_str(),
        Some("c3_requests_response_body_divergence")
    );
    assert_eq!(
        c["response_body_divergence_exit_code"].as_integer(),
        Some(292)
    );
}

#[test]
fn r4_conformance_fixture_runs_against_httpbin_mirror() {
    let c = &manifest()["r4_conformance_fixture_runs_against_httpbin_mirror_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("conformance_fixture_runs_against_httpbin_mirror_or_vendored_server")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_provide_httpbin_style_mirror_or_vendored_server",
        "must_pin_fixture_relative_path",
        "must_assert_response_status_in_fixture",
        "must_assert_response_body_in_fixture",
        "must_tear_down_test_server_at_end_of_fixture",
        "forbid_resolving_to_live_internet_in_default_gate",
        "forbid_fixture_passing_when_request_did_not_run",
        "must_distinguish_request_skipped_from_live_internet_used",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["fixture_relative_path"].as_str(),
        Some("projects/mamba/tests/cpython/fixtures/3rd-libs/requests_https_get")
    );
    assert_eq!(
        c["fixture_relative_path_field_name"].as_str(),
        Some("fixture_relative_path")
    );
    let kinds: Vec<_> = c["allowed_test_server_kinds"]
        .as_array()
        .expect("allowed_test_server_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(kinds, vec!["vendored_https_server", "httpbin_mirror"]);
    assert_eq!(
        c["test_server_kind_field_name"].as_str(),
        Some("test_server_kind")
    );
    assert_eq!(
        c["fixture_request_skipped_failure_kind"].as_str(),
        Some("c3_requests_fixture_request_skipped")
    );
    assert_eq!(
        c["fixture_request_skipped_exit_code"].as_integer(),
        Some(293)
    );
    assert_eq!(
        c["fixture_live_internet_used_failure_kind"].as_str(),
        Some("c3_requests_fixture_live_internet_used_in_default_gate")
    );
    assert_eq!(
        c["fixture_live_internet_used_exit_code"].as_integer(),
        Some(294)
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let r = &manifest()["runner_contract"];
    let keys: Vec<_> = r["keys"]
        .as_array()
        .expect("keys")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        keys,
        vec![
            "outcome",
            "case",
            "requirement_id",
            "ssl_mod_path",
            "ssl_spike_estimate_kind",
            "ssl_spike_target_host",
            "canonical_import_statement",
            "requests_module",
            "https_target_url",
            "loopback_target_url",
            "https_phases_completed",
            "response_fields",
            "response_status_code",
            "response_headers",
            "response_json",
            "response_text",
            "fixture_relative_path",
            "test_server_kind",
            "failure_kind",
            "exit_code",
        ]
    );
    let outcomes: Vec<_> = r["outcome_values"]
        .as_array()
        .expect("outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(outcomes, vec!["pass", "fail", "missing", "skip"]);
    let cases: Vec<_> = r["case_values"]
        .as_array()
        .expect("case_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        cases,
        vec![
            "ssl_fidelity_spike_must_complete_with_written_estimate",
            "import_requests_resolves_and_module_object_present",
            "requests_get_over_https_completes_dns_tls_request_and_response",
            "response_status_code_headers_json_and_text_match_cpython",
            "conformance_fixture_runs_against_httpbin_mirror_or_vendored_server",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "flask_werkzeug_wsgi",
        "pytest_harness",
        "pkgmgr_and_venv",
        "performance_gates",
        "c_extension_fast_paths",
        "runtime_implementation_of_ssl_context",
        "runtime_implementation_of_socket_ssl_wrap",
        "runtime_implementation_of_http_client_https_connection",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
