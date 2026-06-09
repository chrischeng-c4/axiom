//! Schema gate for the stdlib importlib/importlib.metadata fixture —
//! closes #2634.
//!
//! Acceptance (issue #2634):
//!
//!   1. Fixture fails when dynamic import by name is broken.
//!      `[failure_on_broken_dynamic_import_contract]` pins
//!      must_fail_when_import_module_returns_none +
//!      must_fail_when_import_module_raises_unexpected_exception +
//!      must_fail_when_imported_module_missing_expected_attribute +
//!      distinct exit codes 100/101/102 +
//!      must_distinguish_each_dynamic_import_failure_mode.
//!   2. Metadata not-found behavior is deterministic.
//!      `[deterministic_not_found_contract]` pins
//!      must_raise_packagenotfounderror_for_unknown_package +
//!      must_not_perform_network_io_on_not_found + exit_code=103.
//!   3. Failure output distinguishes importlib from metadata
//!      behavior. `[failure_kind_separation_contract]` pins
//!      must_emit_subject_module_in_failure_output +
//!      allowed_subject_module_values + forbid_shared_failure_kind_
//!      across_subjects + mixed-subject exit_code=104.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("stdlib")
        .join("importlib_metadata_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_importlib_metadata_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2634));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_importlib_metadata_behavioral"));
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
fn surface_covers_importlib_and_importlib_metadata() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["importlib", "importlib.metadata"] {
        assert!(modules.contains(m), "covered_modules must include {m}");
    }
    assert_eq!(s.get("import_statement").and_then(|v| v.as_str()), Some("import importlib"));
    assert_eq!(s.get("metadata_import_statement").and_then(|v| v.as_str()), Some("from importlib import metadata as importlib_metadata"));
    for f in &[
        "must_be_importable_via_import_statement",
        "must_cover_importlib_import_module",
        "must_cover_importlib_metadata_lookup",
        "must_cover_metadata_packagenotfounderror_path",
        "must_cover_module_object_has_attribute_after_import",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

#[test]
fn deterministic_sample_covers_import_and_not_found() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let specs: &[(&str, &[&str])] = &[
        ("import_by_name_cases", &["target_module_name", "expected_module_attribute"]),
        ("metadata_not_found_cases", &["target_package_name", "expected_exception_type", "must_not_perform_network_io"]),
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
    let not_found = doc.get("metadata_not_found_cases").and_then(|v| v.as_array()).unwrap();
    for c in not_found {
        let t = c.as_table().unwrap();
        assert_eq!(t.get("expected_exception_type").and_then(|v| v.as_str()), Some("PackageNotFoundError"));
        assert_eq!(t.get("must_not_perform_network_io").and_then(|v| v.as_bool()), Some(true));
    }
}

// Acceptance: "Fixture fails when dynamic import by name is broken."
#[test]
fn fixture_fails_when_dynamic_import_by_name_is_broken() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc.get("failure_on_broken_dynamic_import_contract").and_then(|v| v.as_table()).expect(
        "[failure_on_broken_dynamic_import_contract] missing — acceptance: \
         \"Fixture fails when dynamic import by name is broken.\"",
    );
    for k in &[
        "must_fail_when_import_module_returns_none",
        "must_fail_when_import_module_raises_unexpected_exception",
        "must_fail_when_imported_module_missing_expected_attribute",
        "must_call_importlib_import_module_explicitly",
        "forbid_use_of_top_level_import_statement_for_dynamic_path",
        "must_distinguish_each_dynamic_import_failure_mode",
    ] {
        assert_eq!(f.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exits: Vec<i64> = [
        "dynamic_import_exit_code",
        "unexpected_exception_exit_code",
        "missing_attribute_exit_code",
    ].iter().map(|k| f.get(*k).and_then(|v| v.as_integer()).unwrap()).collect();
    assert_eq!(exits, vec![100, 101, 102]);
    let mut sorted = exits.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), exits.len(), "dynamic import failure exit codes must all differ");
}

// Acceptance: "Metadata not-found behavior is deterministic."
#[test]
fn metadata_not_found_behavior_is_deterministic() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_not_found_contract").and_then(|v| v.as_table()).expect(
        "[deterministic_not_found_contract] missing — acceptance: \
         \"Metadata not-found behavior is deterministic.\"",
    );
    for k in &[
        "must_raise_packagenotfounderror_for_unknown_package",
        "must_not_perform_network_io_on_not_found",
        "must_not_consult_user_site_packages_for_not_found",
        "must_be_reproducible_across_runs",
        "forbid_silent_fallback_to_none",
    ] {
        assert_eq!(d.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(d.get("not_found_exception_type_field_name").and_then(|v| v.as_str()), Some("exception_type"));
    assert_eq!(d.get("not_found_exception_type_value").and_then(|v| v.as_str()), Some("PackageNotFoundError"));
    let exit = d.get("nondeterministic_not_found_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 103);
    assert_eq!(d.get("nondeterministic_not_found_failure_kind").and_then(|v| v.as_str()), Some("metadata_not_found_nondeterministic"));
}

// Acceptance: "Failure output distinguishes importlib from metadata behavior."
#[test]
fn failure_output_distinguishes_importlib_from_metadata() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("failure_kind_separation_contract").and_then(|v| v.as_table()).expect(
        "[failure_kind_separation_contract] missing — acceptance: \
         \"Failure output distinguishes importlib from metadata behavior.\"",
    );
    for k in &[
        "must_emit_subject_module_in_failure_output",
        "must_distinguish_importlib_failure_kinds_from_metadata_failure_kinds",
        "forbid_shared_failure_kind_across_subjects",
    ] {
        assert_eq!(s.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(s.get("subject_module_field_name").and_then(|v| v.as_str()), Some("subject_module"));
    let allowed: Vec<&str> = s.get("allowed_subject_module_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["importlib", "importlib.metadata"] {
        assert!(allowed.contains(m), "allowed_subject_module_values must include {m}");
    }
    let exit = s.get("mixed_subject_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 104);
    assert_eq!(s.get("mixed_subject_failure_kind").and_then(|v| v.as_str()), Some("import_failure_subject_module_mixed"));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "subject_module",
        "target_module_name", "target_package_name",
        "expected_module_attribute",
        "expected_exception_type", "actual_exception_type",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_when_dynamic_import_by_name_is_broken",
        "metadata_not_found_behavior_is_deterministic",
        "failure_output_distinguishes_importlib_from_metadata",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("full_installed_distribution_metadata_coverage").and_then(|v| v.as_bool()), Some(true));
}
