//! Schema gate for the stdlib csv/pickle/tomllib fixture — closes
//! #2635.
//!
//! Acceptance (issue #2635):
//!
//!   1. Fixture fails on wrong parsed or roundtripped data.
//!      `[failure_on_incorrect_behavior_contract]` pins
//!      must_fail_on_incorrect_csv_read +
//!      must_fail_on_incorrect_csv_write +
//!      must_fail_on_incorrect_pickle_roundtrip +
//!      must_fail_on_incorrect_tomllib_parse + distinct exit codes
//!      105/106/107/108 + must_distinguish_each_serializer.
//!   2. tomllib behavior is explicitly pass, xfail, or blocker on the
//!      current runtime. `[tomllib_runtime_status_contract]` pins
//!      must_emit_explicit_runtime_status_for_tomllib +
//!      allowed_runtime_status_values=[pass, xfail, blocker] +
//!      must_link_blocker_to_issue + forbid_silently_skipping_tomllib
//!      + forbid_falsely_passing_tomllib + exit code 109.
//!   3. No network or locale dependency.
//!      `[no_network_or_locale_contract]` pins
//!      must_not_perform_network_io + must_not_consult_locale_setting
//!      + forbid_use_of_LC_ALL_or_LANG_environment_variables +
//!      forbid_locale_dependent_csv_dialect + pinned dialect "excel"
//!      + must_use_utf8_for_text_streams + distinct exit codes
//!      110/111 + must_distinguish_network_from_locale_dependency.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("stdlib")
        .join("csv_pickle_tomllib_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("stdlib_csv_pickle_tomllib_behavioral")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2635));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2529)
    );
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("stdlib_csv_pickle_tomllib_behavioral")
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
fn surface_covers_csv_pickle_tomllib() {
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
    for m in &["csv", "pickle", "tomllib"] {
        assert!(modules.contains(m), "covered_modules must include {m}");
    }
    for f in &[
        "must_be_importable_via_import_statement",
        "must_cover_csv_reader",
        "must_cover_csv_writer",
        "must_cover_pickle_dumps",
        "must_cover_pickle_loads",
        "must_cover_tomllib_loads",
        "must_use_in_memory_stringio_for_csv",
        "must_use_in_memory_bytesio_for_pickle",
        "must_use_in_memory_string_for_tomllib",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

#[test]
fn deterministic_sample_covers_csv_pickle_tomllib() {
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

    let specs: &[(&str, &[&str])] = &[
        ("csv_read_cases", &["input_text", "expected_rows"]),
        (
            "csv_write_cases",
            &["input_rows", "expected_output_text", "line_terminator"],
        ),
        (
            "pickle_roundtrip_cases",
            &["input_python_repr", "protocol", "must_match_after_loads"],
        ),
        (
            "tomllib_parse_cases",
            &["input_text", "expected_python_repr", "runtime_outcome"],
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

    let pickle = doc
        .get("pickle_roundtrip_cases")
        .and_then(|v| v.as_array())
        .unwrap();
    for c in pickle {
        let t = c.as_table().unwrap();
        assert_eq!(
            t.get("protocol").and_then(|v| v.as_str()),
            Some("HIGHEST_PROTOCOL")
        );
        assert_eq!(
            t.get("must_match_after_loads").and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}

// Acceptance: "Fixture fails on wrong parsed or roundtripped data."
#[test]
fn fixture_fails_on_wrong_parsed_or_roundtripped_data() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc
        .get("failure_on_incorrect_behavior_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[failure_on_incorrect_behavior_contract] missing — acceptance: \
         \"Fixture fails on wrong parsed or roundtripped data.\"",
        );
    for k in &[
        "must_fail_on_incorrect_csv_read",
        "must_fail_on_incorrect_csv_write",
        "must_fail_on_incorrect_pickle_roundtrip",
        "must_fail_on_incorrect_tomllib_parse",
        "must_distinguish_each_serializer",
    ] {
        assert_eq!(
            f.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exits: Vec<i64> = [
        "csv_read_mismatch_exit_code",
        "csv_write_mismatch_exit_code",
        "pickle_roundtrip_mismatch_exit_code",
        "tomllib_parse_mismatch_exit_code",
    ]
    .iter()
    .map(|k| f.get(*k).and_then(|v| v.as_integer()).unwrap())
    .collect();
    assert_eq!(exits, vec![105, 106, 107, 108]);
    let mut sorted = exits.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        exits.len(),
        "serializer mismatch exit codes must all differ"
    );

    for (k, v) in &[
        ("csv_read_mismatch_failure_kind", "csv_read_mismatch"),
        ("csv_write_mismatch_failure_kind", "csv_write_mismatch"),
        (
            "pickle_roundtrip_mismatch_failure_kind",
            "pickle_roundtrip_mismatch",
        ),
        (
            "tomllib_parse_mismatch_failure_kind",
            "tomllib_parse_mismatch",
        ),
    ] {
        assert_eq!(
            f.get(*k).and_then(|v| v.as_str()),
            Some(*v),
            "{k} must be {v}"
        );
    }
}

// Acceptance: "tomllib behavior is explicitly pass, xfail, or blocker
// on the current runtime."
#[test]
fn tomllib_behavior_is_explicit_pass_xfail_or_blocker() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc
        .get("tomllib_runtime_status_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[tomllib_runtime_status_contract] missing — acceptance: \
         \"tomllib behavior is explicitly pass, xfail, or blocker on the current runtime.\"",
        );
    for k in &[
        "must_emit_explicit_runtime_status_for_tomllib",
        "must_link_blocker_to_issue",
        "forbid_silently_skipping_tomllib",
        "forbid_falsely_passing_tomllib",
    ] {
        assert_eq!(
            t.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        t.get("runtime_status_field_name").and_then(|v| v.as_str()),
        Some("runtime_outcome")
    );
    assert_eq!(
        t.get("blocker_link_field_name").and_then(|v| v.as_str()),
        Some("blocker_issue")
    );

    let allowed: Vec<&str> = t
        .get("allowed_runtime_status_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &["pass", "xfail", "blocker"] {
        assert!(
            allowed.contains(v),
            "allowed_runtime_status_values must include {v}"
        );
    }

    let exit = t
        .get("implicit_tomllib_status_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 109);
    assert_eq!(
        t.get("implicit_tomllib_status_failure_kind")
            .and_then(|v| v.as_str()),
        Some("tomllib_runtime_status_implicit_or_missing"),
    );
}

// Acceptance: "No network or locale dependency."
#[test]
fn no_network_or_locale_dependency() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc
        .get("no_network_or_locale_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[no_network_or_locale_contract] missing — acceptance: \
         \"No network or locale dependency.\"",
        );
    for k in &[
        "must_not_perform_network_io",
        "must_not_consult_locale_setting",
        "forbid_use_of_LC_ALL_or_LANG_environment_variables",
        "forbid_locale_dependent_csv_dialect",
        "must_pin_csv_dialect",
        "must_use_utf8_for_text_streams",
        "must_distinguish_network_from_locale_dependency",
    ] {
        assert_eq!(
            n.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        n.get("pinned_csv_dialect_name").and_then(|v| v.as_str()),
        Some("excel")
    );

    let net = n
        .get("network_io_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let loc = n
        .get("locale_dependency_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(net, 110);
    assert_eq!(loc, 111);
    assert_ne!(
        net, loc,
        "network and locale dependency exit codes must differ"
    );

    assert_eq!(
        n.get("network_io_failure_kind").and_then(|v| v.as_str()),
        Some("csv_pickle_tomllib_network_io_used"),
    );
    assert_eq!(
        n.get("locale_dependency_failure_kind")
            .and_then(|v| v.as_str()),
        Some("csv_pickle_tomllib_locale_dependency_used"),
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
        "csv_input_text",
        "csv_expected_rows",
        "csv_expected_output_text",
        "pickle_input_python_repr",
        "tomllib_input_text",
        "tomllib_expected_python_repr",
        "runtime_outcome",
        "blocker_issue",
        "failure_kind",
        "exit_code",
    ] {
        assert!(
            keys.contains(required),
            "runner_contract.keys must include {required}"
        );
    }

    let outcomes: Vec<&str> = c
        .get("outcome_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &["pass", "fail", "xfail", "blocker", "missing", "skip"] {
        assert!(
            outcomes.contains(v),
            "runner_contract.outcome_values must include {v}"
        );
    }

    let cases: Vec<&str> = c
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "fixture_fails_on_wrong_parsed_or_roundtripped_data",
        "tomllib_behavior_is_explicit_pass_xfail_or_blocker",
        "no_network_or_locale_dependency",
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
        o.get("binary_protocol_compatibility_matrix")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
