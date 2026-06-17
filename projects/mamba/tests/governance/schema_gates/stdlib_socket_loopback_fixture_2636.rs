//! Schema gate for the stdlib socket loopback fixture — closes
//! #2636.
//!
//! Acceptance (issue #2636):
//!
//!   1. Fixture never reaches external network.
//!      `[no_external_network_contract]` pins
//!      must_bind_only_to_loopback + allowed_bind_hosts ⊇
//!      [127.0.0.1, ::1] + forbid_bind_to_0_0_0_0 +
//!      forbid_connect_to_non_loopback_host + forbid_dns_resolution +
//!      must_not_use_external_proxy_env_vars + distinct exit codes
//!      112/113 + must_distinguish_external_network_from_dns.
//!   2. Unsupported socket behavior is a blocker with issue
//!      reference. `[unsupported_behavior_blocker_contract]` pins
//!      must_mark_unsupported_socket_path_with_blocker +
//!      must_link_blocker_to_issue +
//!      forbid_silently_skipping_unsupported_socket_behavior +
//!      forbid_falsely_passing_unsupported_socket_behavior + exit
//!      code 114.
//!   3. Passing fixture closes sockets deterministically.
//!      `[deterministic_close_contract]` pins
//!      must_close_listening_socket_on_pass +
//!      must_close_accepted_socket_on_pass +
//!      must_close_client_socket_on_pass +
//!      must_close_sockets_even_on_failure +
//!      forbid_relying_on_gc_to_close_sockets + exit code 115.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("stdlib")
        .join("socket_loopback_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_socket_loopback_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2636));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_socket_loopback_behavioral"));
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("loopback_only"));
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
fn surface_registers_socket() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(modules.contains(&"socket"), "covered_modules must include socket");
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_socket_in_stdlib_manifest",
        "must_cover_socket_socket",
        "must_cover_bind",
        "must_cover_connect",
        "must_cover_send",
        "must_cover_recv",
        "must_cover_close",
        "must_use_af_inet",
        "must_use_sock_stream",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

#[test]
fn deterministic_sample_pins_loopback_roundtrip() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let arr = doc.get("loopback_roundtrip_cases").and_then(|v| v.as_array())
        .expect("[[loopback_roundtrip_cases]] missing");
    assert!(!arr.is_empty(), "loopback_roundtrip_cases must not be empty");
    for c in arr {
        let t = c.as_table().expect("case must be a table");
        for f in &[
            "bind_host", "bind_port", "address_family", "socket_type",
            "payload_bytes_python_repr", "expected_received_bytes_python_repr",
            "recv_buffer_size", "must_send_and_recv_exact_payload",
        ] {
            assert!(t.get(*f).is_some(), "loopback_roundtrip_cases.{f} missing");
        }
        assert_eq!(t.get("bind_host").and_then(|v| v.as_str()), Some("127.0.0.1"));
        assert_eq!(t.get("address_family").and_then(|v| v.as_str()), Some("AF_INET"));
        assert_eq!(t.get("socket_type").and_then(|v| v.as_str()), Some("SOCK_STREAM"));
        assert_eq!(t.get("must_send_and_recv_exact_payload").and_then(|v| v.as_bool()), Some(true));
    }
}

// Acceptance: "Fixture never reaches external network."
#[test]
fn fixture_never_reaches_external_network() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("no_external_network_contract").and_then(|v| v.as_table()).expect(
        "[no_external_network_contract] missing — acceptance: \
         \"Fixture never reaches external network.\"",
    );
    for k in &[
        "must_bind_only_to_loopback",
        "forbid_bind_to_0_0_0_0",
        "forbid_connect_to_non_loopback_host",
        "forbid_dns_resolution",
        "must_not_consult_resolv_conf",
        "must_not_use_external_proxy_env_vars",
        "must_distinguish_external_network_from_dns",
    ] {
        assert_eq!(n.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let allowed: Vec<&str> = n.get("allowed_bind_hosts").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for h in &["127.0.0.1", "::1"] {
        assert!(allowed.contains(h), "allowed_bind_hosts must include {h}");
    }
    let forbidden: Vec<&str> = n.get("forbidden_env_vars").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"] {
        assert!(forbidden.contains(v), "forbidden_env_vars must include {v}");
    }
    let ext = n.get("external_network_use_exit_code").and_then(|v| v.as_integer()).unwrap();
    let dns = n.get("dns_resolution_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(ext, 112);
    assert_eq!(dns, 113);
    assert_ne!(ext, dns, "external-network and dns exit codes must differ");
    assert_eq!(
        n.get("external_network_use_failure_kind").and_then(|v| v.as_str()),
        Some("socket_external_network_used"),
    );
    assert_eq!(
        n.get("dns_resolution_failure_kind").and_then(|v| v.as_str()),
        Some("socket_dns_resolution_used"),
    );
}

// Acceptance: "Unsupported socket behavior is a blocker with issue
// reference."
#[test]
fn unsupported_socket_behavior_is_blocker_with_issue_reference() {
    let doc = crate::common::load_toml(&manifest_path());
    let u = doc.get("unsupported_behavior_blocker_contract").and_then(|v| v.as_table()).expect(
        "[unsupported_behavior_blocker_contract] missing — acceptance: \
         \"Unsupported socket behavior is a blocker with issue reference.\"",
    );
    for k in &[
        "must_mark_unsupported_socket_path_with_blocker",
        "must_link_blocker_to_issue",
        "forbid_silently_skipping_unsupported_socket_behavior",
        "forbid_falsely_passing_unsupported_socket_behavior",
    ] {
        assert_eq!(u.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(u.get("blocker_outcome_value").and_then(|v| v.as_str()), Some("blocker"));
    assert_eq!(u.get("blocker_link_field_name").and_then(|v| v.as_str()), Some("blocker_issue"));
    let exit = u.get("missing_blocker_link_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 114);
    assert_eq!(
        u.get("missing_blocker_link_failure_kind").and_then(|v| v.as_str()),
        Some("socket_unsupported_missing_blocker_link"),
    );
    let allowed: Vec<&str> = u.get("allowed_unsupported_socket_paths").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(!allowed.is_empty(), "allowed_unsupported_socket_paths must list at least one entry");
}

// Acceptance: "Passing fixture closes sockets deterministically."
#[test]
fn passing_fixture_closes_sockets_deterministically() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("deterministic_close_contract").and_then(|v| v.as_table()).expect(
        "[deterministic_close_contract] missing — acceptance: \
         \"Passing fixture closes sockets deterministically.\"",
    );
    for k in &[
        "must_close_listening_socket_on_pass",
        "must_close_accepted_socket_on_pass",
        "must_close_client_socket_on_pass",
        "must_close_sockets_even_on_failure",
        "must_use_with_statement_or_explicit_close",
        "forbid_relying_on_gc_to_close_sockets",
    ] {
        assert_eq!(c.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = c.get("leaked_socket_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 115);
    assert_eq!(
        c.get("leaked_socket_failure_kind").and_then(|v| v.as_str()),
        Some("socket_not_closed_after_run"),
    );
    let order: Vec<&str> = c.get("close_order_after_pass").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["accepted_socket", "client_socket", "listening_socket"] {
        assert!(order.contains(v), "close_order_after_pass must include {v}");
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
        "bind_host", "bind_port", "address_family", "socket_type",
        "payload_bytes_python_repr", "received_bytes_python_repr",
        "blocker_issue", "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let outcomes: Vec<&str> = c.get("outcome_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["pass", "fail", "blocker", "missing", "skip"] {
        assert!(outcomes.contains(v), "runner_contract.outcome_values must include {v}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_never_reaches_external_network",
        "unsupported_socket_behavior_is_blocker_with_issue_reference",
        "passing_fixture_closes_sockets_deterministically",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("tls").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(o.get("async_networking").and_then(|v| v.as_bool()), Some(true));
}
