//! Schema gate for the stdlib logging behavioral fixture — closes #2631.
//!
//! Acceptance (issue #2631):
//!
//!   1. Fixture fails when level filtering or formatting is wrong.
//!      `[failure_on_incorrect_behavior_contract]` pins
//!      must_fail_on_incorrect_level_filtering +
//!      must_fail_on_incorrect_formatting + distinct exit codes
//!      85/86 + must_distinguish_level_filtering_from_formatting.
//!   2. No global logging state leaks across tests.
//!      `[no_global_state_leak_contract]` pins
//!      must_use_isolated_logger_per_case + must_remove/close/clear
//!      handler after case + forbid_use_of_root_logger_directly +
//!      forbid_basic_config_calls + exit_code=87.
//!   3. Fixture output is deterministic.
//!      `[deterministic_output_contract]` pins must_strip_timestamps
//!      + must_strip_process_and_thread_ids + forbid_assertion_on_
//!      asctime/process_id/thread_id/module_path + exit_code=88.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("stdlib")
        .join("logging_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_logging_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2631));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_logging_behavioral"));
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
fn surface_covers_logger_handler_formatter_with_inmemory_sink() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    assert_eq!(s.get("module_name").and_then(|v| v.as_str()), Some("logging"));
    assert_eq!(s.get("must_be_importable_via_import_statement").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("import_statement").and_then(|v| v.as_str()), Some("import logging"));
    for f in &[
        "must_cover_get_logger",
        "must_cover_set_level",
        "must_cover_stream_handler",
        "must_cover_formatter",
        "must_cover_level_filtering",
        "must_cover_message_formatting",
        "must_cover_handler_emit",
        "must_capture_output_via_in_memory_stream",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
    assert_eq!(s.get("in_memory_stream_class_name").and_then(|v| v.as_str()), Some("io.StringIO"));
}

#[test]
fn deterministic_sample_covers_level_filtering_and_formatting() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("must_use_isolated_logger_per_case").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("logger_name_prefix_value").and_then(|v| v.as_str()), Some("mamba_stdlib_2631."));
    assert_eq!(d.get("formatter_format_string").and_then(|v| v.as_str()), Some("%(levelname)s:%(name)s:%(message)s"));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min, "sample bounds must be sane");
    assert!(max <= 64, "sample_max_records must stay small for per-run check");

    let specs: &[(&str, &[&str])] = &[
        ("level_filtering_cases", &["logger_level", "log_call_level", "message", "expected_captured_messages"]),
        ("formatting_cases", &["logger_level", "log_call_level", "message", "formatter_format_string", "expected_formatted_line"]),
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

// Acceptance: "Fixture fails when level filtering or formatting is wrong."
#[test]
fn fixture_fails_on_wrong_level_filtering_or_formatting() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc.get("failure_on_incorrect_behavior_contract").and_then(|v| v.as_table()).expect(
        "[failure_on_incorrect_behavior_contract] missing — acceptance: \
         \"Fixture fails when level filtering or formatting is wrong.\"",
    );
    for k in &[
        "must_fail_on_incorrect_level_filtering",
        "must_fail_on_incorrect_formatting",
        "must_distinguish_level_filtering_from_formatting",
    ] {
        assert_eq!(f.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let lf = f.get("level_filtering_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    let fm = f.get("formatting_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(lf, 85);
    assert_eq!(fm, 86);
    assert_ne!(lf, fm, "level-filtering and formatting exit codes must differ");
    assert_eq!(f.get("level_filtering_mismatch_failure_kind").and_then(|v| v.as_str()), Some("logging_level_filtering_mismatch"));
    assert_eq!(f.get("formatting_mismatch_failure_kind").and_then(|v| v.as_str()), Some("logging_formatting_mismatch"));
}

// Acceptance: "No global logging state leaks across tests."
#[test]
fn no_global_logging_state_leaks_across_tests() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("no_global_state_leak_contract").and_then(|v| v.as_table()).expect(
        "[no_global_state_leak_contract] missing — acceptance: \
         \"No global logging state leaks across tests.\"",
    );
    for k in &[
        "must_use_isolated_logger_per_case",
        "must_create_logger_with_unique_per_case_name",
        "must_remove_handler_after_case",
        "must_close_handler_after_case",
        "must_clear_logger_handlers_after_case",
        "forbid_use_of_root_logger_directly",
        "forbid_basic_config_calls",
        "must_not_mutate_global_logging_module_state",
    ] {
        assert_eq!(n.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let forbidden: Vec<&str> = n.get("forbidden_global_mutations").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &[
        "logging.basicConfig",
        "logging.disable",
        "logging.lastResort_replacement",
        "logging.captureWarnings_toggle",
    ] {
        assert!(forbidden.contains(m), "forbidden_global_mutations must include {m}");
    }
    let exit = n.get("global_state_leak_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 87);
    assert_eq!(n.get("global_state_leak_failure_kind").and_then(|v| v.as_str()), Some("logging_global_state_leaked"));
}

// Acceptance: "Fixture output is deterministic."
#[test]
fn fixture_output_is_deterministic() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_output_contract").and_then(|v| v.as_table()).expect(
        "[deterministic_output_contract] missing — acceptance: \
         \"Fixture output is deterministic.\"",
    );
    for k in &[
        "must_strip_timestamps_from_compared_output",
        "must_strip_process_and_thread_ids_from_compared_output",
        "must_compare_exact_bytes_after_stripping",
        "forbid_assertion_on_asctime",
        "forbid_assertion_on_process_id",
        "forbid_assertion_on_thread_id",
        "forbid_assertion_on_module_path",
    ] {
        assert_eq!(d.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = d.get("nondeterministic_output_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 88);
    assert_eq!(d.get("nondeterministic_output_failure_kind").and_then(|v| v.as_str()), Some("logging_nondeterministic_output"));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module_name",
        "logger_name", "logger_level", "log_call_level", "message",
        "formatter_format_string", "expected_formatted_line",
        "expected_captured_messages", "actual_captured_messages",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_on_wrong_level_filtering_or_formatting",
        "no_global_logging_state_leaks_across_tests",
        "fixture_output_is_deterministic",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("multiprocessing_or_file_logging").and_then(|v| v.as_bool()), Some(true));
}
