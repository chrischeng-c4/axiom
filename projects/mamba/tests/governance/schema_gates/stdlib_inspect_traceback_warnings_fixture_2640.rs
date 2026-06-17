//! Schema gate for the stdlib inspect/traceback/warnings fixture —
//! closes #2640.
//!
//! Acceptance (issue #2640):
//!
//!   1. Fixture fails on wrong signature or warning capture
//!      behavior.
//!      `[failure_on_wrong_signature_or_warning_contract]` pins
//!      must_fail flags for signature_str / param_names /
//!      param_kinds / warning_count / warning_category /
//!      warning_message + distinct exit codes 133 (signature) /
//!      134 (warning) + must_distinguish_signature_from_warning_
//!      failure.
//!   2. Traceback assertion avoids line-number brittleness where
//!      possible.
//!      `[traceback_avoids_line_number_brittleness_contract]` pins
//!      must_assert_traceback_by_substring_or_qualname +
//!      forbid_assertion_on_exact_line_number +
//!      must_normalize_or_strip_line_numbers_when_compared +
//!      allowed_softening_strategies ⊇ [substring_match,
//!      regex_match, normalize_line_numbers] + exit code 135.
//!   3. Runner reports diagnostics-module coverage.
//!      `[diagnostics_coverage_reporting_contract]` pins
//!      must_emit_diagnostics_coverage_in_runner_output +
//!      required_diagnostics_modules_covered ⊇ [inspect,
//!      traceback, warnings] +
//!      must_emit_summary_record_with_diagnostics_coverage + exit
//!      code 136.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("stdlib")
        .join("inspect_traceback_warnings_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_inspect_traceback_warnings_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2640));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_inspect_traceback_warnings_behavioral"));
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
fn surface_registers_inspect_traceback_warnings() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["inspect", "traceback", "warnings"] {
        assert!(modules.contains(m), "covered_modules must include {m}");
    }
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_inspect_in_stdlib_manifest",
        "must_register_traceback_in_stdlib_manifest",
        "must_register_warnings_in_stdlib_manifest",
        "must_cover_inspect_signature",
        "must_cover_traceback_format_exception",
        "must_cover_warnings_catch_warnings",
        "must_cover_warnings_simplefilter_always",
        "must_cover_warnings_warn",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

#[test]
fn inspected_function_definition_pins_signature() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("inspected_function_definition").and_then(|v| v.as_table()).expect("[inspected_function_definition] missing");
    assert_eq!(i.get("function_name").and_then(|v| v.as_str()), Some("mamba_2640_target"));
    assert_eq!(i.get("param_a_kind").and_then(|v| v.as_str()), Some("POSITIONAL_OR_KEYWORD"));
    assert_eq!(i.get("param_b_kind").and_then(|v| v.as_str()), Some("POSITIONAL_OR_KEYWORD"));
    assert_eq!(i.get("param_c_kind").and_then(|v| v.as_str()), Some("KEYWORD_ONLY"));
    assert_eq!(
        i.get("expected_signature_str").and_then(|v| v.as_str()),
        Some("(x, y=0, *, label='mamba')"),
    );
}

#[test]
fn deterministic_sample_covers_signature_traceback_warnings() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let specs: &[(&str, &[&str])] = &[
        ("signature_cases", &["target", "expected_param_names", "expected_param_kinds", "expected_signature_str"]),
        ("traceback_cases", &[
            "raise_python_repr",
            "must_assert_exception_type_in_output",
            "expected_exception_type_substring",
            "must_assert_last_frame_qualname_in_output",
            "expected_last_frame_qualname_substring",
            "forbid_assertion_on_exact_line_number",
        ]),
        ("warnings_cases", &[
            "warning_category", "warning_message", "filter",
            "expected_warning_count", "expected_warning_category_name",
            "expected_warning_message_substring",
        ]),
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

// Acceptance: "Fixture fails on wrong signature or warning capture
// behavior."
#[test]
fn fixture_fails_on_wrong_signature_or_warning_capture_behavior() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc.get("failure_on_wrong_signature_or_warning_contract").and_then(|v| v.as_table()).expect(
        "[failure_on_wrong_signature_or_warning_contract] missing — acceptance: \
         \"Fixture fails on wrong signature or warning capture behavior.\"",
    );
    for k in &[
        "must_fail_on_incorrect_signature_str",
        "must_fail_on_incorrect_param_names",
        "must_fail_on_incorrect_param_kinds",
        "must_fail_on_incorrect_warning_count",
        "must_fail_on_incorrect_warning_category",
        "must_fail_on_incorrect_warning_message",
        "must_distinguish_signature_from_warning_failure",
    ] {
        assert_eq!(f.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let sig = f.get("signature_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    let warn = f.get("warning_capture_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(sig, 133);
    assert_eq!(warn, 134);
    assert_ne!(sig, warn, "signature and warning exit codes must differ");
    assert_eq!(
        f.get("signature_mismatch_failure_kind").and_then(|v| v.as_str()),
        Some("inspect_signature_mismatch"),
    );
    assert_eq!(
        f.get("warning_capture_mismatch_failure_kind").and_then(|v| v.as_str()),
        Some("warnings_capture_mismatch"),
    );
}

// Acceptance: "Traceback assertion avoids line-number brittleness
// where possible."
#[test]
fn traceback_assertion_avoids_line_number_brittleness() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc.get("traceback_avoids_line_number_brittleness_contract").and_then(|v| v.as_table()).expect(
        "[traceback_avoids_line_number_brittleness_contract] missing — acceptance: \
         \"Traceback assertion avoids line-number brittleness where possible.\"",
    );
    for k in &[
        "must_assert_traceback_by_substring_or_qualname",
        "forbid_assertion_on_exact_line_number",
        "must_normalize_or_strip_line_numbers_when_compared",
    ] {
        assert_eq!(t.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let allowed: Vec<&str> = t.get("allowed_softening_strategies").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["substring_match", "regex_match", "normalize_line_numbers"] {
        assert!(allowed.contains(v), "allowed_softening_strategies must include {v}");
    }
    let exit = t.get("brittle_traceback_assertion_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 135);
    assert_eq!(
        t.get("brittle_traceback_assertion_failure_kind").and_then(|v| v.as_str()),
        Some("traceback_assertion_brittle_to_line_numbers"),
    );
}

// Acceptance: "Runner reports diagnostics-module coverage."
#[test]
fn runner_reports_diagnostics_module_coverage() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("diagnostics_coverage_reporting_contract").and_then(|v| v.as_table()).expect(
        "[diagnostics_coverage_reporting_contract] missing — acceptance: \
         \"Runner reports diagnostics-module coverage.\"",
    );
    for k in &[
        "must_emit_diagnostics_coverage_in_runner_output",
        "must_emit_summary_record_with_diagnostics_coverage",
    ] {
        assert_eq!(d.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(
        d.get("diagnostics_coverage_field_name").and_then(|v| v.as_str()),
        Some("diagnostics_modules_covered"),
    );
    let req: Vec<&str> = d.get("required_diagnostics_modules_covered").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["inspect", "traceback", "warnings"] {
        assert!(req.contains(m), "required_diagnostics_modules_covered must include {m}");
    }
    assert_eq!(d.get("summary_record_format").and_then(|v| v.as_str()), Some("json"));
    let exit = d.get("missing_diagnostics_coverage_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 136);
    assert_eq!(
        d.get("missing_diagnostics_coverage_failure_kind").and_then(|v| v.as_str()),
        Some("diagnostics_coverage_missing"),
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
        "target", "expected_signature_str", "expected_param_names", "expected_param_kinds",
        "raise_python_repr",
        "warning_category", "warning_message", "expected_warning_count",
        "diagnostics_modules_covered",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_on_wrong_signature_or_warning_capture_behavior",
        "traceback_assertion_avoids_line_number_brittleness",
        "runner_reports_diagnostics_module_coverage",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("full_frame_introspection_compatibility").and_then(|v| v.as_bool()), Some(true));
}
