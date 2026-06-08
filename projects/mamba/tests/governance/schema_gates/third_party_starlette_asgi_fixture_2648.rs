//! Schema gate for the third-party starlette ASGI fixture — closes
//! #2648.
//!
//! Acceptance (issue #2648):
//!
//!   1. Fixture fails if starlette cannot import.
//!      `[import_failure_contract]` pins must_fail_on_import_error +
//!      must_fail_on_missing_starlette_module +
//!      forbid_silent_fallback_when_starlette_missing + exit 167.
//!   2. Fixture avoids external sockets.
//!      `[no_external_sockets_contract]` pins
//!      must_not_open_external_socket +
//!      must_not_bind_to_external_interface +
//!      must_not_listen_on_real_port +
//!      forbid_use_of_socket_socket_bind +
//!      forbid_use_of_socket_socket_listen +
//!      forbid_use_of_uvicorn_run +
//!      must_use_in_process_testclient_or_direct_asgi_call +
//!      distinct exit codes 168 (external socket) / 169 (server
//!      lifecycle) +
//!      must_distinguish_external_socket_from_full_server_lifecycle.
//!   3. Summary records ASGI framework coverage.
//!      `[asgi_framework_coverage_reporting_contract]` pins
//!      must_emit_asgi_framework_coverage_in_runner_output +
//!      required_asgi_framework_dependencies_covered ⊇ [starlette] +
//!      must_emit_summary_record_with_asgi_framework_coverage +
//!      forbid_silent_or_implicit_asgi_framework_coverage + exit 170.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("third_party")
        .join("starlette_asgi_behavioral")
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
        Some("third_party_starlette_asgi_behavioral")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2648));
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
        Some("third_party_starlette_asgi_behavioral")
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
fn python_target_is_pinned_to_3_12() {
    let doc = load_toml(&manifest_path());
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
fn surface_covers_starlette() {
    let doc = load_toml(&manifest_path());
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
        modules.contains(&"starlette"),
        "covered_modules must include starlette"
    );
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_starlette_in_ecosystem_manifest",
        "must_cover_starlette_application_construction",
        "must_cover_route_definition",
        "must_cover_handler_function",
        "must_cover_in_process_testclient_or_direct_asgi_call",
        "must_assert_response_status",
        "must_assert_response_body",
        "must_use_synchronous_test_client_or_direct_asgi_call",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    assert_eq!(
        s.get("import_statement").and_then(|v| v.as_str()),
        Some("import starlette")
    );
}

#[test]
fn asgi_application_definition_pins_canonical_app() {
    let doc = load_toml(&manifest_path());
    let a = doc
        .get("asgi_application_definition")
        .and_then(|v| v.as_table())
        .expect("[asgi_application_definition] missing");
    assert_eq!(
        a.get("app_variable").and_then(|v| v.as_str()),
        Some("app_2648")
    );
    assert_eq!(
        a.get("route_path").and_then(|v| v.as_str()),
        Some("/mamba/2648")
    );
    assert_eq!(a.get("route_method").and_then(|v| v.as_str()), Some("GET"));
    assert_eq!(
        a.get("handler_name").and_then(|v| v.as_str()),
        Some("mamba_2648_handler")
    );
    assert_eq!(
        a.get("handler_response_status")
            .and_then(|v| v.as_integer()),
        Some(200)
    );
    assert_eq!(
        a.get("handler_response_body_python_repr")
            .and_then(|v| v.as_str()),
        Some("{'ok': True, 'n': 2648}"),
    );
}

#[test]
fn deterministic_sample_covers_asgi_calls() {
    let doc = load_toml(&manifest_path());
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
        .get("asgi_call_cases")
        .and_then(|v| v.as_array())
        .expect("[[asgi_call_cases]] missing");
    assert!(!arr.is_empty(), "asgi_call_cases must not be empty");
    for c in arr {
        let t = c.as_table().expect("case must be a table");
        for f in &[
            "request_method",
            "request_path",
            "expected_response_status",
            "expected_response_json_python_repr",
            "expected_response_body_substring",
        ] {
            assert!(t.get(*f).is_some(), "asgi_call_cases.{f} missing");
        }
    }
}

// Acceptance: "Fixture fails if starlette cannot import."
#[test]
fn fixture_fails_if_starlette_cannot_import() {
    let doc = load_toml(&manifest_path());
    let i = doc
        .get("import_failure_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[import_failure_contract] missing — acceptance: \
         \"Fixture fails if starlette cannot import.\"",
        );
    for k in &[
        "must_fail_on_import_error",
        "must_fail_on_missing_starlette_module",
        "must_emit_import_failure_kind_when_starlette_missing",
        "forbid_silent_fallback_when_starlette_missing",
    ] {
        assert_eq!(
            i.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exit = i
        .get("starlette_import_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 167);
    assert_eq!(
        i.get("starlette_import_failure_kind")
            .and_then(|v| v.as_str()),
        Some("third_party_starlette_import_failed"),
    );
}

// Acceptance: "Fixture avoids external sockets."
#[test]
fn fixture_avoids_external_sockets() {
    let doc = load_toml(&manifest_path());
    let n = doc
        .get("no_external_sockets_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[no_external_sockets_contract] missing — acceptance: \
         \"Fixture avoids external sockets.\"",
        );
    for k in &[
        "must_not_open_external_socket",
        "must_not_bind_to_external_interface",
        "must_not_listen_on_real_port",
        "forbid_use_of_socket_socket_bind",
        "forbid_use_of_socket_socket_listen",
        "forbid_use_of_uvicorn_run",
        "must_use_in_process_testclient_or_direct_asgi_call",
        "must_distinguish_external_socket_from_full_server_lifecycle",
    ] {
        assert_eq!(
            n.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let sock = n
        .get("external_socket_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let life = n
        .get("server_lifecycle_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(sock, 168);
    assert_eq!(life, 169);
    assert_ne!(
        sock, life,
        "external-socket and server-lifecycle exit codes must differ"
    );
    assert_eq!(
        n.get("external_socket_failure_kind")
            .and_then(|v| v.as_str()),
        Some("starlette_external_socket_used"),
    );
    assert_eq!(
        n.get("server_lifecycle_failure_kind")
            .and_then(|v| v.as_str()),
        Some("starlette_full_server_lifecycle_used"),
    );
}

// Acceptance: "Summary records ASGI framework coverage."
#[test]
fn summary_records_asgi_framework_coverage() {
    let doc = load_toml(&manifest_path());
    let h = doc
        .get("asgi_framework_coverage_reporting_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[asgi_framework_coverage_reporting_contract] missing — acceptance: \
         \"Summary records ASGI framework coverage.\"",
        );
    for k in &[
        "must_emit_asgi_framework_coverage_in_runner_output",
        "must_emit_summary_record_with_asgi_framework_coverage",
        "forbid_silent_or_implicit_asgi_framework_coverage",
        "must_distinguish_asgi_framework_coverage_from_overall_outcome",
    ] {
        assert_eq!(
            h.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        h.get("asgi_framework_coverage_field_name")
            .and_then(|v| v.as_str()),
        Some("asgi_framework_dependencies_covered"),
    );
    let req: Vec<&str> = h
        .get("required_asgi_framework_dependencies_covered")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        req.contains(&"starlette"),
        "required_asgi_framework_dependencies_covered must include starlette"
    );
    assert_eq!(
        h.get("summary_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    let exit = h
        .get("missing_asgi_framework_coverage_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 170);
    assert_eq!(
        h.get("missing_asgi_framework_coverage_failure_kind")
            .and_then(|v| v.as_str()),
        Some("starlette_asgi_framework_coverage_missing"),
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
        "module_name",
        "app_variable",
        "route_path",
        "route_method",
        "request_method",
        "request_path",
        "expected_response_status",
        "expected_response_json_python_repr",
        "expected_response_body_substring",
        "asgi_framework_dependencies_covered",
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
        "fixture_fails_if_starlette_cannot_import",
        "fixture_avoids_external_sockets",
        "summary_records_asgi_framework_coverage",
    ] {
        assert!(
            cases.contains(required),
            "runner_contract.case_values must include {required}"
        );
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("full_server_lifecycle").and_then(|v| v.as_bool()),
        Some(true)
    );
}
