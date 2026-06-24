//! Schema gate for the third-party PyYAML load/dump fixture —
//! closes #2646.
//!
//! Acceptance (issue #2646):
//!
//!   1. Fixture fails if yaml cannot import.
//!      `[import_failure_contract]` pins must_fail_on_import_error +
//!      must_fail_on_missing_yaml_module +
//!      forbid_silent_fallback_when_yaml_missing + exit 159.
//!   2. Fixture fails on wrong parsed or dumped shape.
//!      `[shape_mismatch_contract]` pins
//!      must_fail_on_incorrect_parsed_shape +
//!      must_fail_on_incorrect_dumped_shape + distinct exit codes
//!      160 (parsed) / 161 (dumped) +
//!      must_distinguish_parsed_from_dumped_shape_failure.
//!   3. Native-extension limitations are recorded explicitly if
//!      present.
//!      `[native_extension_limitation_reporting_contract]` pins
//!      must_record_libyaml_availability +
//!      libyaml_availability_field_name="libyaml_available" +
//!      allowed_libyaml_availability_values=[available, unavailable]
//!      + must_record_native_extension_limitations_when_unavailable
//!      + forbid_silently_hiding_native_extension_limitations +
//!      forbid_falsely_reporting_libyaml_available + exit 162.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("third_party")
        .join("pyyaml_load_dump_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("third_party_pyyaml_load_dump_behavioral")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2646));
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
        Some("third_party_pyyaml_load_dump_behavioral")
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
fn surface_covers_pyyaml() {
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
        modules.contains(&"yaml"),
        "covered_modules must include yaml"
    );
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_pyyaml_in_ecosystem_manifest",
        "must_cover_yaml_safe_load",
        "must_cover_yaml_safe_dump",
        "must_cover_mapping_round_trip",
        "must_cover_list_round_trip",
        "must_use_only_safe_load_and_safe_dump",
        "forbid_use_of_yaml_load_without_loader",
        "forbid_use_of_unsafe_loader",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    assert_eq!(
        s.get("import_statement").and_then(|v| v.as_str()),
        Some("import yaml")
    );
}

#[test]
fn deterministic_sample_covers_load_and_dump() {
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

    let load = doc
        .get("load_cases")
        .and_then(|v| v.as_array())
        .expect("[[load_cases]] missing");
    assert!(!load.is_empty(), "load_cases must not be empty");
    for c in load {
        let t = c.as_table().expect("case must be a table");
        for f in &["yaml_input", "expected_parsed_python_repr"] {
            assert!(t.get(*f).is_some(), "load_cases.{f} missing");
        }
    }
    let dump = doc
        .get("dump_cases")
        .and_then(|v| v.as_array())
        .expect("[[dump_cases]] missing");
    assert!(!dump.is_empty(), "dump_cases must not be empty");
    for c in dump {
        let t = c.as_table().expect("case must be a table");
        assert!(
            t.get("python_value_repr").is_some(),
            "dump_cases.python_value_repr missing"
        );
    }
}

// Acceptance: "Fixture fails if yaml cannot import."
#[test]
fn fixture_fails_if_yaml_cannot_import() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc
        .get("import_failure_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[import_failure_contract] missing — acceptance: \
         \"Fixture fails if yaml cannot import.\"",
        );
    for k in &[
        "must_fail_on_import_error",
        "must_fail_on_missing_yaml_module",
        "must_emit_import_failure_kind_when_yaml_missing",
        "forbid_silent_fallback_when_yaml_missing",
    ] {
        assert_eq!(
            i.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exit = i
        .get("yaml_import_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 159);
    assert_eq!(
        i.get("yaml_import_failure_kind").and_then(|v| v.as_str()),
        Some("third_party_pyyaml_import_failed"),
    );
}

// Acceptance: "Fixture fails on wrong parsed or dumped shape."
#[test]
fn fixture_fails_on_wrong_parsed_or_dumped_shape() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc
        .get("shape_mismatch_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[shape_mismatch_contract] missing — acceptance: \
         \"Fixture fails on wrong parsed or dumped shape.\"",
        );
    for k in &[
        "must_fail_on_incorrect_parsed_shape",
        "must_fail_on_incorrect_dumped_shape",
        "must_distinguish_parsed_from_dumped_shape_failure",
    ] {
        assert_eq!(
            c.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let parsed = c
        .get("parsed_shape_mismatch_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let dumped = c
        .get("dumped_shape_mismatch_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(parsed, 160);
    assert_eq!(dumped, 161);
    assert_ne!(parsed, dumped, "parsed and dumped exit codes must differ");
    assert_eq!(
        c.get("parsed_shape_mismatch_failure_kind")
            .and_then(|v| v.as_str()),
        Some("pyyaml_parsed_shape_mismatch"),
    );
    assert_eq!(
        c.get("dumped_shape_mismatch_failure_kind")
            .and_then(|v| v.as_str()),
        Some("pyyaml_dumped_shape_mismatch"),
    );
}

// Acceptance: "Native-extension limitations are recorded explicitly
// if present."
#[test]
fn native_extension_limitations_recorded_explicitly_if_present() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc
        .get("native_extension_limitation_reporting_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[native_extension_limitation_reporting_contract] missing — acceptance: \
         \"Native-extension limitations are recorded explicitly if present.\"",
        );
    for k in &[
        "must_record_libyaml_availability",
        "must_record_native_extension_limitations_when_unavailable",
        "forbid_silently_hiding_native_extension_limitations",
        "forbid_falsely_reporting_libyaml_available",
    ] {
        assert_eq!(
            n.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        n.get("libyaml_availability_field_name")
            .and_then(|v| v.as_str()),
        Some("libyaml_available"),
    );
    let allowed: Vec<&str> = n
        .get("allowed_libyaml_availability_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &["available", "unavailable"] {
        assert!(
            allowed.contains(v),
            "allowed_libyaml_availability_values must include {v}"
        );
    }
    assert_eq!(
        n.get("native_extension_limitation_field_name")
            .and_then(|v| v.as_str()),
        Some("native_extension_limitations"),
    );
    let exit = n
        .get("missing_native_extension_report_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 162);
    assert_eq!(
        n.get("missing_native_extension_report_failure_kind")
            .and_then(|v| v.as_str()),
        Some("pyyaml_native_extension_report_missing"),
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
        "yaml_input",
        "expected_parsed_python_repr",
        "python_value_repr",
        "libyaml_available",
        "native_extension_limitations",
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
        "fixture_fails_if_yaml_cannot_import",
        "fixture_fails_on_wrong_parsed_or_dumped_shape",
        "native_extension_limitations_recorded_explicitly_if_present",
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
        o.get("yaml_spec_edge_cases").and_then(|v| v.as_bool()),
        Some(true)
    );
}
