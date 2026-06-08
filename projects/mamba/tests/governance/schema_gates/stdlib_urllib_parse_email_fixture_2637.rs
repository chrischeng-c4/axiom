//! Schema gate for the stdlib urllib.parse + email fixture — closes
//! #2637.
//!
//! Acceptance (issue #2637):
//!
//!   1. Fixture fails on wrong URL component parsing.
//!      `[failure_on_wrong_url_component_contract]` pins
//!      per-component must_fail flags + distinct exit codes
//!      116 (urlparse) / 117 (urlencode) +
//!      must_distinguish_urlparse_from_urlencode.
//!   2. Fixture fails on wrong email header or payload behavior.
//!      `[failure_on_wrong_email_behavior_contract]` pins
//!      must_fail_on_incorrect_header_{from,to,subject} +
//!      must_fail_on_incorrect_payload_text +
//!      must_fail_on_incorrect_content_type + distinct exit codes
//!      118 (header) / 119 (payload) / 120 (content_type) +
//!      must_distinguish_email_header_from_payload_from_content_type.
//!   3. No network I/O is used. `[no_network_io_contract]` pins
//!      must_not_perform_network_io + must_not_perform_smtp +
//!      must_not_perform_dns_resolution +
//!      forbid_use_of_socket_connect + exit code 121.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("stdlib")
        .join("urllib_parse_email_behavioral")
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
        Some("stdlib_urllib_parse_email_behavioral")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2637));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2529)
    );
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("stdlib_urllib_parse_email_behavioral")
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
fn surface_registers_urllib_parse_and_email() {
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
    for m in &["urllib.parse", "email", "email.message"] {
        assert!(modules.contains(m), "covered_modules must include {m}");
    }
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_urllib_parse_in_stdlib_manifest",
        "must_register_email_in_stdlib_manifest",
        "must_cover_urllib_parse_urlparse",
        "must_cover_urllib_parse_urlencode",
        "must_cover_email_message_emailmessage",
        "must_cover_email_set_header",
        "must_cover_email_get_header",
        "must_cover_email_set_content",
        "must_cover_email_get_content",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    assert_eq!(
        s.get("urllib_parse_import_statement")
            .and_then(|v| v.as_str()),
        Some("from urllib.parse import urlparse, urlencode"),
    );
    assert_eq!(
        s.get("email_import_statement").and_then(|v| v.as_str()),
        Some("from email.message import EmailMessage"),
    );
}

#[test]
fn deterministic_sample_covers_urlparse_urlencode_email() {
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

    let specs: &[(&str, &[&str])] = &[
        (
            "urlparse_cases",
            &[
                "input_url",
                "expected_scheme",
                "expected_netloc",
                "expected_path",
            ],
        ),
        ("urlencode_cases", &["input_pairs", "expected_output"]),
        (
            "email_message_cases",
            &[
                "header_from",
                "header_to",
                "header_subject",
                "content_type",
                "charset",
                "payload_text",
                "expected_get_content_value",
                "expected_get_content_type_value",
            ],
        ),
    ];
    for (key, fields) in specs {
        let arr = doc
            .get(*key)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("[[{key}]] missing"));
        assert!(!arr.is_empty(), "[[{key}]] must not be empty");
        for c in arr {
            let t = c.as_table().expect("case must be a table");
            for f in *fields {
                assert!(t.get(*f).is_some(), "{key}.{f} missing");
            }
        }
    }
}

// Acceptance: "Fixture fails on wrong URL component parsing."
#[test]
fn fixture_fails_on_wrong_url_component_parsing() {
    let doc = load_toml(&manifest_path());
    let f = doc
        .get("failure_on_wrong_url_component_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[failure_on_wrong_url_component_contract] missing — acceptance: \
         \"Fixture fails on wrong URL component parsing.\"",
        );
    for k in &[
        "must_fail_on_incorrect_scheme",
        "must_fail_on_incorrect_netloc",
        "must_fail_on_incorrect_hostname",
        "must_fail_on_incorrect_port",
        "must_fail_on_incorrect_path",
        "must_fail_on_incorrect_query",
        "must_fail_on_incorrect_fragment",
        "must_fail_on_incorrect_urlencode_output",
        "must_distinguish_urlparse_from_urlencode",
    ] {
        assert_eq!(
            f.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let parse_exit = f
        .get("url_component_mismatch_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let encode_exit = f
        .get("urlencode_mismatch_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(parse_exit, 116);
    assert_eq!(encode_exit, 117);
    assert_ne!(
        parse_exit, encode_exit,
        "urlparse and urlencode exit codes must differ"
    );
    assert_eq!(
        f.get("url_component_mismatch_failure_kind")
            .and_then(|v| v.as_str()),
        Some("urlparse_component_mismatch"),
    );
    assert_eq!(
        f.get("urlencode_mismatch_failure_kind")
            .and_then(|v| v.as_str()),
        Some("urlencode_output_mismatch"),
    );
}

// Acceptance: "Fixture fails on wrong email header or payload
// behavior."
#[test]
fn fixture_fails_on_wrong_email_header_or_payload_behavior() {
    let doc = load_toml(&manifest_path());
    let f = doc
        .get("failure_on_wrong_email_behavior_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[failure_on_wrong_email_behavior_contract] missing — acceptance: \
         \"Fixture fails on wrong email header or payload behavior.\"",
        );
    for k in &[
        "must_fail_on_incorrect_header_from",
        "must_fail_on_incorrect_header_to",
        "must_fail_on_incorrect_header_subject",
        "must_fail_on_incorrect_payload_text",
        "must_fail_on_incorrect_content_type",
        "must_distinguish_email_header_from_payload_from_content_type",
    ] {
        assert_eq!(
            f.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exits: Vec<i64> = [
        "email_header_mismatch_exit_code",
        "email_payload_mismatch_exit_code",
        "email_content_type_mismatch_exit_code",
    ]
    .iter()
    .map(|k| f.get(*k).and_then(|v| v.as_integer()).unwrap())
    .collect();
    assert_eq!(exits, vec![118, 119, 120]);
    let mut sorted = exits.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        exits.len(),
        "email failure exit codes must all differ"
    );
    for (k, v) in &[
        (
            "email_header_mismatch_failure_kind",
            "email_header_mismatch",
        ),
        (
            "email_payload_mismatch_failure_kind",
            "email_payload_mismatch",
        ),
        (
            "email_content_type_mismatch_failure_kind",
            "email_content_type_mismatch",
        ),
    ] {
        assert_eq!(
            f.get(*k).and_then(|v| v.as_str()),
            Some(*v),
            "{k} must be {v}"
        );
    }
}

// Acceptance: "No network I/O is used."
#[test]
fn no_network_io_is_used() {
    let doc = load_toml(&manifest_path());
    let n = doc
        .get("no_network_io_contract")
        .and_then(|v| v.as_table())
        .expect("[no_network_io_contract] missing — acceptance: \"No network I/O is used.\"");
    for k in &[
        "must_not_perform_network_io",
        "must_not_perform_smtp",
        "must_not_perform_dns_resolution",
        "forbid_connect_to_mta_host",
        "forbid_use_of_socket_connect",
    ] {
        assert_eq!(
            n.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exit = n
        .get("network_io_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 121);
    assert_eq!(
        n.get("network_io_failure_kind").and_then(|v| v.as_str()),
        Some("urllib_email_network_io_used"),
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
        "input_url",
        "expected_scheme",
        "expected_netloc",
        "expected_path",
        "input_pairs",
        "expected_output",
        "header_from",
        "header_to",
        "header_subject",
        "payload_text",
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
        "fixture_fails_on_wrong_url_component_parsing",
        "fixture_fails_on_wrong_email_header_or_payload_behavior",
        "no_network_io_is_used",
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
        o.get("http_client_behavior").and_then(|v| v.as_bool()),
        Some(true)
    );
}
