//! Schema gate for the third-party FastAPI route fixture — closes
//! #2650.
//!
//! Acceptance (issue #2650):
//!
//!   1. Fixture appears in ecosystem reports even if blocked.
//!      `[ecosystem_reporting_when_blocked_contract]` pins
//!      must_emit_ecosystem_report_entry_for_every_outcome +
//!      must_emit entries for blocked/xfail/skip/missing +
//!      forbid_suppressing_ecosystem_report_for_blocked_outcomes +
//!      required_ecosystem_dependencies_covered ⊇ [fastapi] +
//!      exit 171.
//!   2. Import and route setup failures are distinguishable.
//!      `[import_vs_route_setup_failure_contract]` pins
//!      must_distinguish_import_failure_from_route_setup_failure +
//!      must_fail_on_import_error +
//!      must_fail_on_route_setup_error +
//!      forbid_silent_fallback_when_fastapi_missing +
//!      forbid_collapsed_or_implicit_failure_phase + distinct exit
//!      codes 172 (import) / 173 (route_setup) +
//!      allowed_failure_phase_values=[import, route_setup].
//!   3. No external network or server process is required.
//!      `[no_external_network_or_server_process_contract]` pins
//!      must_not_perform_network_io + must_not_open_external_socket
//!      + must_not_listen_on_real_port +
//!      must_not_spawn_server_process +
//!      forbid_use_of_socket_socket_bind/listen +
//!      forbid_use_of_uvicorn_run +
//!      forbid_use_of_subprocess_for_server +
//!      must_use_in_process_testclient_or_direct_asgi_call_when_supported
//!      + distinct exit codes 174 (network) / 175 (server process) +
//!      must_distinguish_external_network_from_server_process_use.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("third_party")
        .join("fastapi_route_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("third_party_fastapi_route_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2650));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("third_party"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("third_party_fastapi_route_behavioral"));
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
fn surface_covers_fastapi() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(modules.contains(&"fastapi"), "covered_modules must include fastapi");
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_fastapi_in_ecosystem_manifest",
        "must_cover_fastapi_application_construction",
        "must_cover_route_decoration_with_get",
        "must_cover_handler_function",
        "must_cover_in_process_testclient_or_direct_asgi_call_when_supported",
        "must_assert_response_status",
        "must_assert_response_body",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
    assert_eq!(s.get("import_statement").and_then(|v| v.as_str()), Some("import fastapi"));
}

#[test]
fn route_definition_pins_canonical_route() {
    let doc = crate::common::load_toml(&manifest_path());
    let r = doc.get("route_definition").and_then(|v| v.as_table()).expect("[route_definition] missing");
    assert_eq!(r.get("app_variable").and_then(|v| v.as_str()), Some("app_2650"));
    assert_eq!(r.get("route_path").and_then(|v| v.as_str()), Some("/mamba/2650"));
    assert_eq!(r.get("route_method").and_then(|v| v.as_str()), Some("GET"));
    assert_eq!(r.get("handler_name").and_then(|v| v.as_str()), Some("mamba_2650_handler"));
    assert_eq!(r.get("handler_response_status").and_then(|v| v.as_integer()), Some(200));
    assert_eq!(
        r.get("handler_response_body_python_repr").and_then(|v| v.as_str()),
        Some("{'ok': True, 'n': 2650}"),
    );
}

#[test]
fn deterministic_sample_covers_route_calls() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let arr = doc.get("route_call_cases").and_then(|v| v.as_array()).expect("[[route_call_cases]] missing");
    assert!(!arr.is_empty(), "route_call_cases must not be empty");
    for c in arr {
        let t = c.as_table().expect("case must be a table");
        for f in &[
            "request_method", "request_path",
            "expected_response_status",
            "expected_response_json_python_repr",
            "expected_response_body_substring",
        ] {
            assert!(t.get(*f).is_some(), "route_call_cases.{f} missing");
        }
    }
}

// Acceptance: "Fixture appears in ecosystem reports even if blocked."
#[test]
fn fixture_appears_in_ecosystem_reports_even_if_blocked() {
    let doc = crate::common::load_toml(&manifest_path());
    let e = doc.get("ecosystem_reporting_when_blocked_contract").and_then(|v| v.as_table()).expect(
        "[ecosystem_reporting_when_blocked_contract] missing — acceptance: \
         \"Fixture appears in ecosystem reports even if blocked.\"",
    );
    for k in &[
        "must_emit_ecosystem_report_entry_for_every_outcome",
        "must_emit_ecosystem_report_entry_when_blocked",
        "must_emit_ecosystem_report_entry_when_xfail",
        "must_emit_ecosystem_report_entry_when_skip",
        "must_emit_ecosystem_report_entry_when_missing",
        "forbid_suppressing_ecosystem_report_for_blocked_outcomes",
    ] {
        assert_eq!(e.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(
        e.get("ecosystem_report_field_name").and_then(|v| v.as_str()),
        Some("ecosystem_dependencies_covered"),
    );
    let req: Vec<&str> = e.get("required_ecosystem_dependencies_covered").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(req.contains(&"fastapi"), "required_ecosystem_dependencies_covered must include fastapi");
    let exit = e.get("missing_ecosystem_report_entry_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 171);
    assert_eq!(
        e.get("missing_ecosystem_report_entry_failure_kind").and_then(|v| v.as_str()),
        Some("fastapi_ecosystem_report_entry_missing"),
    );
}

// Acceptance: "Import and route setup failures are distinguishable."
#[test]
fn import_and_route_setup_failures_are_distinguishable() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("import_vs_route_setup_failure_contract").and_then(|v| v.as_table()).expect(
        "[import_vs_route_setup_failure_contract] missing — acceptance: \
         \"Import and route setup failures are distinguishable.\"",
    );
    for k in &[
        "must_distinguish_import_failure_from_route_setup_failure",
        "must_fail_on_import_error",
        "must_fail_on_route_setup_error",
        "forbid_silent_fallback_when_fastapi_missing",
        "forbid_collapsed_or_implicit_failure_phase",
    ] {
        assert_eq!(c.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let import_exit = c.get("fastapi_import_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    let route_exit = c.get("fastapi_route_setup_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(import_exit, 172);
    assert_eq!(route_exit, 173);
    assert_ne!(import_exit, route_exit, "import and route-setup exit codes must differ");
    assert_eq!(
        c.get("fastapi_import_failure_kind").and_then(|v| v.as_str()),
        Some("third_party_fastapi_import_failed"),
    );
    assert_eq!(
        c.get("fastapi_route_setup_failure_kind").and_then(|v| v.as_str()),
        Some("fastapi_route_setup_failed"),
    );
    assert_eq!(c.get("failure_phase_field_name").and_then(|v| v.as_str()), Some("failure_phase"));
    let allowed: Vec<&str> = c.get("allowed_failure_phase_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["import", "route_setup"] {
        assert!(allowed.contains(v), "allowed_failure_phase_values must include {v}");
    }
}

// Acceptance: "No external network or server process is required."
#[test]
fn no_external_network_or_server_process_is_required() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("no_external_network_or_server_process_contract").and_then(|v| v.as_table()).expect(
        "[no_external_network_or_server_process_contract] missing — acceptance: \
         \"No external network or server process is required.\"",
    );
    for k in &[
        "must_not_perform_network_io",
        "must_not_open_external_socket",
        "must_not_listen_on_real_port",
        "must_not_spawn_server_process",
        "forbid_use_of_socket_socket_bind",
        "forbid_use_of_socket_socket_listen",
        "forbid_use_of_uvicorn_run",
        "forbid_use_of_subprocess_for_server",
        "must_use_in_process_testclient_or_direct_asgi_call_when_supported",
        "must_distinguish_external_network_from_server_process_use",
    ] {
        assert_eq!(n.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let net = n.get("external_network_exit_code").and_then(|v| v.as_integer()).unwrap();
    let proc_ = n.get("server_process_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(net, 174);
    assert_eq!(proc_, 175);
    assert_ne!(net, proc_, "external-network and server-process exit codes must differ");
    assert_eq!(
        n.get("external_network_failure_kind").and_then(|v| v.as_str()),
        Some("fastapi_external_network_used"),
    );
    assert_eq!(
        n.get("server_process_failure_kind").and_then(|v| v.as_str()),
        Some("fastapi_external_server_process_used"),
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module_name",
        "app_variable", "route_path", "route_method",
        "request_method", "request_path",
        "expected_response_status",
        "expected_response_json_python_repr",
        "expected_response_body_substring",
        "ecosystem_dependencies_covered",
        "failure_phase",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_appears_in_ecosystem_reports_even_if_blocked",
        "import_and_route_setup_failures_are_distinguishable",
        "no_external_network_or_server_process_is_required",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
    let outcomes: Vec<&str> = c.get("outcome_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &["pass", "fail", "xfail", "blocker", "missing", "skip"] {
        assert!(outcomes.contains(required), "runner_contract.outcome_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("full_pydantic_coverage").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(o.get("openapi_generation_coverage").and_then(|v| v.as_bool()), Some(true));
}
