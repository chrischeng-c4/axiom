#![cfg(test)]

// Locks the shape of the C3 Flask-runs-unmodified fixture pinned by
// tests/governance/gates/third_party/c3_flask_runs_unmodified_gate/
// manifest.toml. Closes #1257.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/third_party/c3_flask_runs_unmodified_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(m["fixture"].as_str(), Some("c3_flask_runs_unmodified_gate"));
    assert_eq!(m["issue"].as_integer(), Some(1257));
    assert_eq!(m["profile"].as_str(), Some("third_party"));
    assert_eq!(m["family"].as_str(), Some("c3_flask_runs_unmodified_gate"));
    assert_eq!(m["network"].as_str(), Some("offline"));
    let related: Vec<_> = m["related_issues"]
        .as_array()
        .expect("related_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(related, vec![1263, 1234, 1265]);
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
fn surface_pins_all_four_requirements() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_flask_import_and_app_construction",
        "must_cover_route_decorator_registration",
        "must_cover_dev_server_responds_to_http_get",
        "must_cover_conformance_fixture_spawns_server_curls_and_asserts",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_flask_import_and_app_construction() {
    let c = &manifest()["r1_flask_import_and_app_construction_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("from_flask_import_flask_then_flask_dunder_name_constructs_app")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_install_flask_unmodified_from_pypi",
        "must_resolve_from_flask_import_flask",
        "must_construct_app_with_dunder_name",
        "forbid_modifying_flask_on_disk_before_run",
        "forbid_substituting_werkzeug_with_mamba_shim",
        "must_distinguish_import_failure_from_construction_failure",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["canonical_import_statement"].as_str(),
        Some("from flask import Flask")
    );
    assert_eq!(
        c["canonical_import_statement_field_name"].as_str(),
        Some("canonical_import_statement")
    );
    assert_eq!(c["app_object_field_name"].as_str(), Some("flask_app"));
    assert_eq!(
        c["flask_import_failure_kind"].as_str(),
        Some("c3_flask_import_failed")
    );
    assert_eq!(c["flask_import_failure_exit_code"].as_integer(), Some(277));
    assert_eq!(
        c["flask_app_construction_failure_kind"].as_str(),
        Some("c3_flask_app_construction_failed")
    );
    assert_eq!(
        c["flask_app_construction_failure_exit_code"].as_integer(),
        Some(278)
    );
}

#[test]
fn r2_route_decorator_registration() {
    let c = &manifest()["r2_route_decorator_registration_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("app_route_slash_decorator_registers_endpoint_for_get")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_resolve_app_route_decorator",
        "must_register_endpoint_for_root_path",
        "must_register_endpoint_for_http_get_method",
        "forbid_silently_dropping_route_registration",
        "forbid_collapsing_multiple_routes_into_single_endpoint",
        "must_distinguish_registration_missing_from_method_mismatched",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["canonical_route_path"].as_str(), Some("/"));
    assert_eq!(c["canonical_route_method"].as_str(), Some("GET"));
    assert_eq!(
        c["endpoint_path_field_name"].as_str(),
        Some("endpoint_path")
    );
    assert_eq!(
        c["endpoint_method_field_name"].as_str(),
        Some("endpoint_method")
    );
    assert_eq!(
        c["route_registration_missing_failure_kind"].as_str(),
        Some("c3_flask_route_registration_missing")
    );
    assert_eq!(
        c["route_registration_missing_exit_code"].as_integer(),
        Some(279)
    );
    assert_eq!(
        c["route_method_mismatched_failure_kind"].as_str(),
        Some("c3_flask_route_method_mismatched")
    );
    assert_eq!(
        c["route_method_mismatched_exit_code"].as_integer(),
        Some(280)
    );
}

#[test]
fn r3_dev_server_responds_to_http_get() {
    let c = &manifest()["r3_dev_server_responds_to_http_get_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("app_run_starts_dev_server_and_responds_to_http_get")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_start_werkzeug_dev_server",
        "must_bind_to_loopback_only_in_default_gate",
        "must_respond_to_http_1_1_get",
        "must_emit_200_status_for_registered_route",
        "must_emit_hello_world_body_for_registered_route",
        "forbid_binding_to_non_loopback_in_default_gate",
        "forbid_silently_falling_back_to_non_wsgi_server",
        "must_distinguish_bind_failure_from_response_mismatch",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(c["default_bind_host"].as_str(), Some("127.0.0.1"));
    assert_eq!(c["default_bind_port"].as_integer(), Some(5000));
    assert_eq!(c["bind_host_field_name"].as_str(), Some("bind_host"));
    assert_eq!(c["bind_port_field_name"].as_str(), Some("bind_port"));
    assert_eq!(c["http_status_field_name"].as_str(), Some("http_status"));
    assert_eq!(c["http_body_field_name"].as_str(), Some("http_body"));
    assert_eq!(c["expected_http_status"].as_integer(), Some(200));
    assert_eq!(c["expected_http_body"].as_str(), Some("hello world"));
    assert_eq!(
        c["dev_server_bind_failure_kind"].as_str(),
        Some("c3_flask_dev_server_bind_failure")
    );
    assert_eq!(
        c["dev_server_bind_failure_exit_code"].as_integer(),
        Some(281)
    );
    assert_eq!(
        c["dev_server_response_mismatch_failure_kind"].as_str(),
        Some("c3_flask_dev_server_response_mismatch")
    );
    assert_eq!(
        c["dev_server_response_mismatch_exit_code"].as_integer(),
        Some(282)
    );
}

#[test]
fn r4_conformance_fixture_spawns_server_curls_and_asserts() {
    let c = &manifest()["r4_conformance_fixture_spawns_server_curls_and_asserts_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("conformance_fixture_spawns_server_curls_and_asserts_response")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_spawn_dev_server_inside_fixture",
        "must_curl_or_equivalent_against_dev_server",
        "must_assert_response_status_in_fixture",
        "must_assert_response_body_in_fixture",
        "must_tear_down_dev_server_at_end_of_fixture",
        "forbid_dev_server_leaked_between_fixtures",
        "forbid_fixture_passing_when_curl_did_not_run",
        "must_distinguish_curl_skipped_from_server_leaked",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["fixture_relative_path"].as_str(),
        Some("projects/mamba/tests/cpython/fixtures/3rd-libs/flask_hello_world")
    );
    assert_eq!(
        c["fixture_relative_path_field_name"].as_str(),
        Some("fixture_relative_path")
    );
    assert_eq!(
        c["http_client_kind_field_name"].as_str(),
        Some("http_client_kind")
    );
    let kinds: Vec<_> = c["allowed_http_client_kinds"]
        .as_array()
        .expect("allowed_http_client_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(kinds, vec!["curl", "wsgi_test_client", "urllib"]);
    assert_eq!(
        c["fixture_curl_skipped_failure_kind"].as_str(),
        Some("c3_flask_fixture_curl_skipped")
    );
    assert_eq!(c["fixture_curl_skipped_exit_code"].as_integer(), Some(283));
    assert_eq!(
        c["fixture_server_leaked_failure_kind"].as_str(),
        Some("c3_flask_fixture_dev_server_leaked")
    );
    assert_eq!(c["fixture_server_leaked_exit_code"].as_integer(), Some(284));
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
            "canonical_import_statement",
            "flask_app",
            "endpoint_path",
            "endpoint_method",
            "bind_host",
            "bind_port",
            "http_status",
            "http_body",
            "fixture_relative_path",
            "http_client_kind",
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
            "from_flask_import_flask_then_flask_dunder_name_constructs_app",
            "app_route_slash_decorator_registers_endpoint_for_get",
            "app_run_starts_dev_server_and_responds_to_http_get",
            "conformance_fixture_spawns_server_curls_and_asserts_response",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "requests_urllib3_tls_ssl",
        "pytest_harness",
        "flask_extensions_beyond_core",
        "c_extension_fast_paths",
        "performance_gates",
        "runtime_implementation_of_werkzeug_dev_server",
        "runtime_implementation_of_socket_listen_accept",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
