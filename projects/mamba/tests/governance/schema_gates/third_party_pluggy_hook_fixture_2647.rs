//! Schema gate for the third-party pluggy hook fixture — closes
//! #2647.
//!
//! Acceptance (issue #2647):
//!
//!   1. Fixture fails if pluggy cannot import.
//!      `[import_failure_contract]` pins must_fail_on_import_error +
//!      must_fail_on_missing_pluggy_module +
//!      forbid_silent_fallback_when_pluggy_missing + exit 163.
//!   2. Fixture asserts hook call result exactly.
//!      `[hook_call_result_contract]` pins
//!      must_assert_hook_call_result_exactly +
//!      must_fail_on_hook_result_mismatch +
//!      must_fail_on_hook_invocation_error +
//!      forbid_partial_or_substring_match_of_hook_result + distinct
//!      exit codes 164 (mismatch) / 165 (invocation_error) +
//!      must_distinguish_hook_result_mismatch_from_invocation_error.
//!   3. Runner records it as plugin-framework coverage.
//!      `[plugin_framework_coverage_reporting_contract]` pins
//!      must_emit_plugin_framework_coverage_in_runner_output +
//!      required_plugin_framework_dependencies_covered ⊇ [pluggy] +
//!      must_emit_summary_record_with_plugin_framework_coverage +
//!      forbid_silent_or_implicit_plugin_framework_coverage +
//!      exit 166.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("third_party")
        .join("pluggy_hook_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("third_party_pluggy_hook_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2647));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("third_party"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("third_party_pluggy_hook_behavioral"));
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
fn surface_covers_pluggy() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(modules.contains(&"pluggy"), "covered_modules must include pluggy");
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_pluggy_in_ecosystem_manifest",
        "must_cover_pluginmanager_construction",
        "must_cover_hookspec_definition",
        "must_cover_hookimpl_definition",
        "must_cover_register_plugin",
        "must_cover_add_hookspecs",
        "must_cover_invoke_hook",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
    assert_eq!(s.get("import_statement").and_then(|v| v.as_str()), Some("import pluggy"));
}

#[test]
fn plugin_definition_pins_canonical_plugin() {
    let doc = crate::common::load_toml(&manifest_path());
    let p = doc.get("plugin_definition").and_then(|v| v.as_table()).expect("[plugin_definition] missing");
    assert_eq!(p.get("project_name").and_then(|v| v.as_str()), Some("mamba2647"));
    assert_eq!(p.get("hook_name").and_then(|v| v.as_str()), Some("mamba_2647_compute"));
    assert_eq!(p.get("hookspec_signature").and_then(|v| v.as_str()), Some("(x: int, y: int) -> int"));
    assert_eq!(p.get("hookimpl_returns_python_repr").and_then(|v| v.as_str()), Some("x + y"));
    assert_eq!(p.get("plugin_class_name").and_then(|v| v.as_str()), Some("Mamba2647Plugin"));
}

#[test]
fn deterministic_sample_covers_hook_calls() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let arr = doc.get("hook_call_cases").and_then(|v| v.as_array()).expect("[[hook_call_cases]] missing");
    assert!(!arr.is_empty(), "hook_call_cases must not be empty");
    for c in arr {
        let t = c.as_table().expect("case must be a table");
        for f in &["call_args_python_repr", "expected_result_python_repr"] {
            assert!(t.get(*f).is_some(), "hook_call_cases.{f} missing");
        }
    }
}

// Acceptance: "Fixture fails if pluggy cannot import."
#[test]
fn fixture_fails_if_pluggy_cannot_import() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("import_failure_contract").and_then(|v| v.as_table()).expect(
        "[import_failure_contract] missing — acceptance: \
         \"Fixture fails if pluggy cannot import.\"",
    );
    for k in &[
        "must_fail_on_import_error",
        "must_fail_on_missing_pluggy_module",
        "must_emit_import_failure_kind_when_pluggy_missing",
        "forbid_silent_fallback_when_pluggy_missing",
    ] {
        assert_eq!(i.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = i.get("pluggy_import_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 163);
    assert_eq!(
        i.get("pluggy_import_failure_kind").and_then(|v| v.as_str()),
        Some("third_party_pluggy_import_failed"),
    );
}

// Acceptance: "Fixture asserts hook call result exactly."
#[test]
fn fixture_asserts_hook_call_result_exactly() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("hook_call_result_contract").and_then(|v| v.as_table()).expect(
        "[hook_call_result_contract] missing — acceptance: \
         \"Fixture asserts hook call result exactly.\"",
    );
    for k in &[
        "must_assert_hook_call_result_exactly",
        "must_fail_on_hook_result_mismatch",
        "must_fail_on_hook_invocation_error",
        "forbid_partial_or_substring_match_of_hook_result",
        "must_distinguish_hook_result_mismatch_from_invocation_error",
    ] {
        assert_eq!(c.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let mismatch = c.get("hook_result_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    let invoc = c.get("hook_invocation_error_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(mismatch, 164);
    assert_eq!(invoc, 165);
    assert_ne!(mismatch, invoc, "mismatch and invocation-error exit codes must differ");
    assert_eq!(
        c.get("hook_result_mismatch_failure_kind").and_then(|v| v.as_str()),
        Some("pluggy_hook_result_mismatch"),
    );
    assert_eq!(
        c.get("hook_invocation_error_failure_kind").and_then(|v| v.as_str()),
        Some("pluggy_hook_invocation_error"),
    );
}

// Acceptance: "Runner records it as plugin-framework coverage."
#[test]
fn runner_records_pluggy_as_plugin_framework_coverage() {
    let doc = crate::common::load_toml(&manifest_path());
    let h = doc.get("plugin_framework_coverage_reporting_contract").and_then(|v| v.as_table()).expect(
        "[plugin_framework_coverage_reporting_contract] missing — acceptance: \
         \"Runner records it as plugin-framework coverage.\"",
    );
    for k in &[
        "must_emit_plugin_framework_coverage_in_runner_output",
        "must_emit_summary_record_with_plugin_framework_coverage",
        "forbid_silent_or_implicit_plugin_framework_coverage",
        "must_distinguish_plugin_framework_coverage_from_overall_outcome",
    ] {
        assert_eq!(h.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(
        h.get("plugin_framework_coverage_field_name").and_then(|v| v.as_str()),
        Some("plugin_framework_dependencies_covered"),
    );
    let req: Vec<&str> = h.get("required_plugin_framework_dependencies_covered").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(req.contains(&"pluggy"), "required_plugin_framework_dependencies_covered must include pluggy");
    assert_eq!(h.get("summary_record_format").and_then(|v| v.as_str()), Some("json"));
    let exit = h.get("missing_plugin_framework_coverage_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 166);
    assert_eq!(
        h.get("missing_plugin_framework_coverage_failure_kind").and_then(|v| v.as_str()),
        Some("pluggy_plugin_framework_coverage_missing"),
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
        "project_name", "hook_name",
        "call_args_python_repr", "expected_result_python_repr",
        "plugin_framework_dependencies_covered",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_if_pluggy_cannot_import",
        "fixture_asserts_hook_call_result_exactly",
        "runner_records_pluggy_as_plugin_framework_coverage",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("pytest_behavior_beyond_pluggy_smoke").and_then(|v| v.as_bool()), Some(true));
}
