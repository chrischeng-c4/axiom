//! Schema gate for the stdlib hashlib/hmac/secrets fixture — closes
//! #2638.
//!
//! Acceptance (issue #2638):
//!
//!   1. Fixture fails on wrong digest or comparison result.
//!      `[failure_on_wrong_digest_or_comparison_contract]` pins
//!      must_fail_on_incorrect_sha256_hexdigest +
//!      must_fail_on_incorrect_sha256_digest_length +
//!      must_fail_on_incorrect_hmac_compare_digest_result + distinct
//!      exit codes 122 (sha256) / 123 (hmac).
//!   2. Random output is tested only by shape or length, not exact
//!      value. `[shape_only_random_contract]` pins
//!      must_assert_secrets_output_length_only +
//!      must_assert_secrets_output_type_only +
//!      must_assert_token_hex_alphabet_only +
//!      forbid_asserting_exact_secrets_token_value +
//!      forbid_pinning_pseudorandom_seed + exit code 124.
//!   3. No external services are used.
//!      `[no_external_services_contract]` pins
//!      must_not_perform_network_io +
//!      must_not_use_external_random_provider_endpoint +
//!      forbid_use_of_socket_connect +
//!      forbid_use_of_external_kms_or_hsm +
//!      forbid_calling_openssl_engine + exit code 125.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("stdlib")
        .join("hashlib_hmac_secrets_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_hashlib_hmac_secrets_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2638));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_hashlib_hmac_secrets_behavioral"));
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
fn surface_registers_hashlib_hmac_secrets() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["hashlib", "hmac", "secrets"] {
        assert!(modules.contains(m), "covered_modules must include {m}");
    }
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_hashlib_in_stdlib_manifest",
        "must_register_hmac_in_stdlib_manifest",
        "must_register_secrets_in_stdlib_manifest",
        "must_cover_hashlib_sha256",
        "must_cover_hmac_compare_digest",
        "must_cover_secrets_token_bytes",
        "must_cover_secrets_token_hex",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

#[test]
fn deterministic_sample_covers_sha256_hmac_secrets() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let specs: &[(&str, &[&str])] = &[
        ("sha256_cases", &["input_bytes_python_repr", "expected_hexdigest", "expected_digest_len"]),
        ("hmac_compare_digest_cases", &["left_bytes_python_repr", "right_bytes_python_repr", "expected_result"]),
        ("secrets_shape_cases", &["helper", "input_nbytes", "expected_output_type", "expected_output_len", "must_assert_shape_only", "forbid_assert_exact_value"]),
    ];
    for (key, fields) in specs {
        let arr = doc.get(*key).and_then(|v| v.as_array()).unwrap_or_else(|| panic!("[[{key}]] missing"));
        assert!(!arr.is_empty(), "[[{key}]] must not be empty");
        for c in arr {
            let t = c.as_table().expect("case must be a table");
            for f in *fields {
                assert!(t.get(*f).is_some(), "{key}.{f} missing");
            }
        }
    }

    let sha256 = doc.get("sha256_cases").and_then(|v| v.as_array()).unwrap();
    for c in sha256 {
        let t = c.as_table().unwrap();
        let len = t.get("expected_digest_len").and_then(|v| v.as_integer()).unwrap();
        assert_eq!(len, 32, "sha256 digest length must be 32 bytes");
        let hex = t.get("expected_hexdigest").and_then(|v| v.as_str()).unwrap();
        assert_eq!(hex.len(), 64, "sha256 hexdigest must be 64 chars");
    }
}

// Acceptance: "Fixture fails on wrong digest or comparison result."
#[test]
fn fixture_fails_on_wrong_digest_or_comparison_result() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc.get("failure_on_wrong_digest_or_comparison_contract").and_then(|v| v.as_table()).expect(
        "[failure_on_wrong_digest_or_comparison_contract] missing — acceptance: \
         \"Fixture fails on wrong digest or comparison result.\"",
    );
    for k in &[
        "must_fail_on_incorrect_sha256_hexdigest",
        "must_fail_on_incorrect_sha256_digest_length",
        "must_fail_on_incorrect_hmac_compare_digest_result",
        "must_distinguish_sha256_from_hmac_failure",
    ] {
        assert_eq!(f.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let sha = f.get("sha256_digest_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    let hmac = f.get("hmac_compare_digest_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(sha, 122);
    assert_eq!(hmac, 123);
    assert_ne!(sha, hmac, "sha256 and hmac exit codes must differ");
    assert_eq!(
        f.get("sha256_digest_mismatch_failure_kind").and_then(|v| v.as_str()),
        Some("sha256_digest_mismatch"),
    );
    assert_eq!(
        f.get("hmac_compare_digest_mismatch_failure_kind").and_then(|v| v.as_str()),
        Some("hmac_compare_digest_mismatch"),
    );
}

// Acceptance: "Random output is tested only by shape or length, not
// exact value."
#[test]
fn random_output_is_tested_only_by_shape_or_length() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("shape_only_random_contract").and_then(|v| v.as_table()).expect(
        "[shape_only_random_contract] missing — acceptance: \
         \"Random output is tested only by shape or length, not exact value.\"",
    );
    for k in &[
        "must_assert_secrets_output_length_only",
        "must_assert_secrets_output_type_only",
        "must_assert_token_hex_alphabet_only",
        "forbid_asserting_exact_secrets_token_value",
        "forbid_pinning_pseudorandom_seed",
        "must_distinguish_shape_only_violation_from_other_failures",
    ] {
        assert_eq!(s.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = s.get("exact_value_assertion_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 124);
    assert_eq!(
        s.get("exact_value_assertion_failure_kind").and_then(|v| v.as_str()),
        Some("secrets_exact_value_asserted"),
    );
}

// Acceptance: "No external services are used."
#[test]
fn no_external_services_are_used() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("no_external_services_contract").and_then(|v| v.as_table()).expect(
        "[no_external_services_contract] missing — acceptance: \
         \"No external services are used.\"",
    );
    for k in &[
        "must_not_perform_network_io",
        "must_not_use_external_random_provider_endpoint",
        "forbid_use_of_socket_connect",
        "forbid_use_of_external_kms_or_hsm",
        "forbid_calling_openssl_engine",
    ] {
        assert_eq!(n.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = n.get("external_service_use_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 125);
    assert_eq!(
        n.get("external_service_use_failure_kind").and_then(|v| v.as_str()),
        Some("hashlib_hmac_secrets_external_service_used"),
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
        "input_bytes_python_repr", "expected_hexdigest", "expected_digest_len",
        "left_bytes_python_repr", "right_bytes_python_repr", "expected_result",
        "helper", "input_nbytes", "expected_output_type", "expected_output_len",
        "allowed_alphabet",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_on_wrong_digest_or_comparison_result",
        "random_output_is_tested_only_by_shape_or_length",
        "no_external_services_are_used",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("cryptographic_performance").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(o.get("openssl_provider_coverage").and_then(|v| v.as_bool()), Some(true));
}
