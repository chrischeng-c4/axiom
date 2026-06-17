//! Schema gate for the third-party attrs class fixture — closes
//! #2642.
//!
//! Acceptance (issue #2642):
//!
//!   1. Fixture fails if attrs cannot import.
//!      `[import_failure_contract]` pins must_fail_on_import_error +
//!      must_fail_on_missing_attrs_module +
//!      forbid_silent_fallback_when_attrs_missing + exit code 141.
//!   2. Fixture asserts constructor, default, and equality or repr
//!      behavior.
//!      `[constructor_default_equality_repr_contract]` pins
//!      must_fail flags for construction / field default / equality
//!      / repr + distinct exit codes 142/143/144/145 +
//!      must_distinguish_each_attrs_behavior_failure_kind.
//!   3. Unsupported behavior is visible as xfail or blocker with
//!      issue reference.
//!      `[unsupported_behavior_xfail_or_blocker_contract]` pins
//!      must_mark_unsupported_attrs_behavior_with_xfail_or_blocker
//!      + allowed_unsupported_outcome_values=[xfail, blocker] +
//!      forbid_silently_skipping + forbid_falsely_passing +
//!      exit code 146.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("third_party")
        .join("attrs_class_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("third_party_attrs_class_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2642));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("third_party"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("third_party_attrs_class_behavioral"));
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
fn surface_covers_attrs() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["attrs", "attr"] {
        assert!(modules.contains(m), "covered_modules must include {m}");
    }
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_attrs_in_ecosystem_manifest",
        "must_cover_attrs_define",
        "must_cover_attrs_field_default",
        "must_cover_attrs_field_converter_when_supported",
        "must_cover_instance_construction",
        "must_cover_equality",
        "must_cover_repr",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
    assert_eq!(s.get("import_statement").and_then(|v| v.as_str()), Some("import attrs"));
}

#[test]
fn attrs_class_definition_pins_canonical_class() {
    let doc = crate::common::load_toml(&manifest_path());
    let a = doc.get("attrs_class_definition").and_then(|v| v.as_table()).expect("[attrs_class_definition] missing");
    assert_eq!(a.get("class_name").and_then(|v| v.as_str()), Some("Point2642"));
    assert_eq!(a.get("field_a_name").and_then(|v| v.as_str()), Some("x"));
    assert_eq!(a.get("field_b_name").and_then(|v| v.as_str()), Some("label"));
    assert_eq!(a.get("field_c_name").and_then(|v| v.as_str()), Some("n"));
    assert_eq!(a.get("field_c_converter").and_then(|v| v.as_str()), Some("int"));
}

#[test]
fn deterministic_sample_covers_construction_equality_repr() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let specs: &[(&str, &[&str])] = &[
        ("construction_cases", &["call", "expected_x", "expected_label", "expected_n"]),
        ("equality_cases", &["left_call", "right_call", "expected_relation"]),
        ("repr_cases", &["call", "expected_repr"]),
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
}

// Acceptance: "Fixture fails if attrs cannot import."
#[test]
fn fixture_fails_if_attrs_cannot_import() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("import_failure_contract").and_then(|v| v.as_table()).expect(
        "[import_failure_contract] missing — acceptance: \
         \"Fixture fails if attrs cannot import.\"",
    );
    for k in &[
        "must_fail_on_import_error",
        "must_fail_on_missing_attrs_module",
        "must_emit_import_failure_kind_when_attrs_missing",
        "forbid_silent_fallback_when_attrs_missing",
    ] {
        assert_eq!(i.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = i.get("attrs_import_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 141);
    assert_eq!(
        i.get("attrs_import_failure_kind").and_then(|v| v.as_str()),
        Some("third_party_attrs_import_failed"),
    );
}

// Acceptance: "Fixture asserts constructor, default, and equality or
// repr behavior."
#[test]
fn fixture_asserts_constructor_default_and_equality_or_repr() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("constructor_default_equality_repr_contract").and_then(|v| v.as_table()).expect(
        "[constructor_default_equality_repr_contract] missing — acceptance: \
         \"Fixture asserts constructor, default, and equality or repr behavior.\"",
    );
    for k in &[
        "must_fail_on_incorrect_construction",
        "must_fail_on_incorrect_field_default",
        "must_fail_on_incorrect_equality",
        "must_fail_on_incorrect_repr",
        "must_distinguish_each_attrs_behavior_failure_kind",
    ] {
        assert_eq!(c.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exits: Vec<i64> = [
        "construction_mismatch_exit_code",
        "field_default_mismatch_exit_code",
        "equality_mismatch_exit_code",
        "repr_mismatch_exit_code",
    ].iter().map(|k| c.get(*k).and_then(|v| v.as_integer()).unwrap()).collect();
    assert_eq!(exits, vec![142, 143, 144, 145]);
    let mut sorted = exits.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), exits.len(), "attrs behavior exit codes must all differ");
}

// Acceptance: "Unsupported behavior is visible as xfail or blocker
// with issue reference."
#[test]
fn unsupported_behavior_visible_as_xfail_or_blocker_with_issue_reference() {
    let doc = crate::common::load_toml(&manifest_path());
    let u = doc.get("unsupported_behavior_xfail_or_blocker_contract").and_then(|v| v.as_table()).expect(
        "[unsupported_behavior_xfail_or_blocker_contract] missing — acceptance: \
         \"Unsupported behavior is visible as xfail or blocker with issue reference.\"",
    );
    for k in &[
        "must_mark_unsupported_attrs_behavior_with_xfail_or_blocker",
        "must_link_xfail_or_blocker_to_issue",
        "forbid_silently_skipping_unsupported_attrs_behavior",
        "forbid_falsely_passing_unsupported_attrs_behavior",
    ] {
        assert_eq!(u.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let allowed: Vec<&str> = u.get("allowed_unsupported_outcome_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["xfail", "blocker"] {
        assert!(allowed.contains(v), "allowed_unsupported_outcome_values must include {v}");
    }
    assert_eq!(u.get("unsupported_outcome_link_field_name").and_then(|v| v.as_str()), Some("blocker_issue"));
    let exit = u.get("missing_link_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 146);
    assert_eq!(
        u.get("missing_link_failure_kind").and_then(|v| v.as_str()),
        Some("attrs_unsupported_missing_xfail_or_blocker_link"),
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
        "call", "expected_x", "expected_label", "expected_n",
        "expected_repr", "expected_relation",
        "blocker_issue", "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_if_attrs_cannot_import",
        "fixture_asserts_constructor_default_and_equality_or_repr",
        "unsupported_behavior_visible_as_xfail_or_blocker_with_issue_reference",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("full_attrs_feature_coverage").and_then(|v| v.as_bool()), Some(true));
}
