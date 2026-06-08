//! Schema gate for the stdlib import smoke matrix fixture —
//! closes #2627.
//!
//! Acceptance (issue #2627):
//!
//!   1. Removing a required stdlib module from the import matrix
//!      fails validation. `[required_module_coverage_contract]` pins
//!      must_compare_matrix_modules_to_required_module_manifest +
//!      must_fail_when_required_module_is_missing_from_matrix +
//!      missing-required-module exit_code=67.
//!   2. Import failures name the exact module.
//!      `[import_failure_naming_contract]` pins
//!      must_emit_module_name_on_import_failure +
//!      must_emit_python_exception_type_on_import_failure +
//!      forbid_generic_unnamed_import_failure +
//!      import-failure exit_code=68.
//!   3. The smoke summary includes stdlib import totals.
//!      `[smoke_summary_totals_contract]` pins
//!      must_emit_stdlib_import_totals_in_smoke_summary +
//!      totals_required_fields (pass/fail/xfail/skip/total) +
//!      missing-stdlib-totals exit_code=69.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("stdlib")
        .join("import_smoke_matrix")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_import_smoke_matrix"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2627));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_import_smoke_matrix"));
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
fn cross_references_required_module_manifest_fixture() {
    let doc = crate::common::load_toml(&manifest_path());
    let x = doc.get("required_module_manifest_cross_reference").and_then(|v| v.as_table())
        .expect("[required_module_manifest_cross_reference] missing");
    assert_eq!(x.get("fixture_issue").and_then(|v| v.as_integer()), Some(2624));
    assert_eq!(x.get("must_consume_required_module_manifest").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(x.get("must_match_required_module_set_exactly").and_then(|v| v.as_bool()), Some(true));
    let shared: Vec<&str> = x.get("shared_required_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["json", "datetime", "pathlib", "logging", "argparse"] {
        assert!(shared.contains(m), "shared_required_modules must include {m}");
    }
}

#[test]
fn matrix_record_format_is_pinned() {
    let doc = crate::common::load_toml(&manifest_path());
    let m = doc.get("matrix_record").and_then(|v| v.as_table()).expect("[matrix_record] missing");
    assert_eq!(m.get("record_format").and_then(|v| v.as_str()), Some("json"));
    let required: Vec<&str> = m.get("record_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &["module_name", "outcome", "duration_ms", "error_message", "exit_code"] {
        assert!(required.contains(f), "record_required_fields must include {f}");
    }
    let outcomes: Vec<&str> = m.get("outcome_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for o in &["pass", "fail", "xfail", "skip"] {
        assert!(outcomes.contains(o), "outcome_values must include {o}");
    }
    assert_eq!(m.get("must_emit_one_record_per_required_module").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(m.get("must_emit_one_record_per_optional_module").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(m.get("must_distinguish_required_from_optional_in_record").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(m.get("required_flag_field_name").and_then(|v| v.as_str()), Some("required"));
}

#[test]
fn smoke_budget_is_bounded() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("smoke_budget").and_then(|v| v.as_table()).expect("[smoke_budget] missing");
    assert_eq!(s.get("must_run_in_smoke_profile").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("smoke_profile_fixture_issue").and_then(|v| v.as_integer()), Some(2527));
    let total = s.get("max_total_runtime_ms").and_then(|v| v.as_integer()).unwrap();
    let per = s.get("max_per_module_runtime_ms").and_then(|v| v.as_integer()).unwrap();
    assert!(total > 0 && per > 0 && per <= total, "runtime bounds must be positive and per≤total");
    assert!(total <= 10_000, "smoke total runtime must stay light, got {total}ms");
    assert_eq!(s.get("must_run_imports_in_isolated_subprocess_or_subinterpreter").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Removing a required stdlib module from the import matrix fails validation."
#[test]
fn removing_required_module_from_matrix_fails_validation() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("required_module_coverage_contract").and_then(|v| v.as_table()).expect(
        "[required_module_coverage_contract] missing — acceptance: \
         \"Removing a required stdlib module from the import matrix fails validation.\"",
    );
    for k in &[
        "must_compare_matrix_modules_to_required_module_manifest",
        "must_fail_when_required_module_is_missing_from_matrix",
        "must_emit_missing_module_name_in_diagnostic",
        "must_emit_required_module_manifest_path_in_diagnostic",
    ] {
        assert_eq!(c.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = c.get("missing_required_module_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 67);
    assert_eq!(c.get("missing_required_module_failure_kind").and_then(|v| v.as_str()), Some("required_stdlib_module_missing_from_import_matrix"));
}

// Acceptance: "Import failures name the exact module."
#[test]
fn import_failure_names_the_exact_module() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("import_failure_naming_contract").and_then(|v| v.as_table()).expect(
        "[import_failure_naming_contract] missing — acceptance: \
         \"Import failures name the exact module.\"",
    );
    for k in &[
        "must_emit_module_name_on_import_failure",
        "must_emit_python_exception_type_on_import_failure",
        "must_emit_python_exception_message_on_import_failure",
        "forbid_generic_unnamed_import_failure",
    ] {
        assert_eq!(n.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let required: Vec<&str> = n.get("import_failure_record_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &["module_name", "exception_type", "exception_message", "outcome", "exit_code"] {
        assert!(required.contains(f), "import_failure_record_required_fields must include {f}");
    }
    let exit = n.get("import_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 68);
    assert_eq!(n.get("import_failure_failure_kind").and_then(|v| v.as_str()), Some("stdlib_module_import_failed"));
}

// Acceptance: "The smoke summary includes stdlib import totals."
#[test]
fn smoke_summary_includes_stdlib_import_totals() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("smoke_summary_totals_contract").and_then(|v| v.as_table()).expect(
        "[smoke_summary_totals_contract] missing — acceptance: \
         \"The smoke summary includes stdlib import totals.\"",
    );
    assert_eq!(s.get("must_emit_stdlib_import_totals_in_smoke_summary").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("totals_record_format").and_then(|v| v.as_str()), Some("json"));
    let required: Vec<&str> = s.get("totals_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &[
        "stdlib_import_total",
        "stdlib_import_pass",
        "stdlib_import_fail",
        "stdlib_import_xfail",
        "stdlib_import_skip",
    ] {
        assert!(required.contains(f), "totals_required_fields must include {f}");
    }
    assert_eq!(s.get("must_be_consumable_by_smoke_summary_emitter").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("smoke_summary_fixture_issue").and_then(|v| v.as_integer()), Some(2527));
    let exit = s.get("missing_stdlib_totals_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 69);
    assert_eq!(s.get("missing_stdlib_totals_failure_kind").and_then(|v| v.as_str()), Some("smoke_summary_missing_stdlib_import_totals"));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module_name", "required", "duration_ms",
        "exception_type", "exception_message",
        "stdlib_import_total", "stdlib_import_pass",
        "stdlib_import_fail", "stdlib_import_xfail", "stdlib_import_skip",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "removing_required_module_from_matrix_fails_validation",
        "import_failure_names_the_exact_module",
        "smoke_summary_includes_stdlib_import_totals",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("behavioral_correctness_for_each_module").and_then(|v| v.as_bool()), Some(true));
}
