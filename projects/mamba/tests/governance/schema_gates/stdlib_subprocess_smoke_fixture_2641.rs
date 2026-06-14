//! Schema gate for the stdlib subprocess smoke fixture — closes
//! #2641.
//!
//! Acceptance (issue #2641):
//!
//!   1. Fixture does not invoke shell with untrusted input.
//!      `[no_shell_untrusted_input_contract]` pins
//!      must_set_shell_false + must_pass_argv_as_list_not_string +
//!      forbid_shell_equal_true +
//!      forbid_calling_subprocess_call_with_string_argv +
//!      forbid_string_interpolation_into_command +
//!      forbid_use_of_subprocess_getoutput + forbid_use_of_os_system
//!      + distinct exit codes 137 (shell) / 138 (untrusted input) +
//!      must_distinguish_shell_use_from_untrusted_input.
//!   2. Unsupported behavior is visible as blocker, not silent
//!      skip. `[unsupported_behavior_blocker_contract]` pins
//!      must_mark_unsupported_subprocess_behavior_with_blocker +
//!      must_link_blocker_to_issue +
//!      forbid_silently_skipping +
//!      forbid_falsely_passing_unsupported_subprocess_behavior +
//!      exit code 139.
//!   3. Runner records subprocess status separately.
//!      `[separate_status_recording_contract]` pins
//!      must_record_subprocess_status_in_runner_output +
//!      allowed_status_values=[pass, xfail, blocker] +
//!      forbid_silent_or_implicit_subprocess_status +
//!      must_distinguish_subprocess_status_from_overall_outcome +
//!      exit code 140.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("stdlib")
        .join("subprocess_smoke_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_subprocess_smoke_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2641));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_subprocess_smoke_behavioral"));
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
fn surface_registers_subprocess() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(modules.contains(&"subprocess"), "covered_modules must include subprocess");
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_subprocess_in_stdlib_manifest",
        "must_cover_subprocess_run",
        "must_cover_subprocess_capture_output",
        "must_cover_subprocess_check_returncode",
        "must_use_argv_list_form",
        "must_pass_shell_equal_false",
        "must_use_sys_executable_as_command",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

#[test]
fn deterministic_sample_pins_smoke_commands() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let arr = doc.get("smoke_cases").and_then(|v| v.as_array()).expect("[[smoke_cases]] missing");
    assert!(!arr.is_empty(), "smoke_cases must not be empty");
    for c in arr {
        let t = c.as_table().expect("case must be a table");
        for f in &[
            "argv", "shell", "expected_returncode",
            "expected_stdout", "expected_stderr",
            "runtime_outcome", "timeout_seconds",
        ] {
            assert!(t.get(*f).is_some(), "smoke_cases.{f} missing");
        }
        assert_eq!(t.get("shell").and_then(|v| v.as_bool()), Some(false),
            "shell must be false on every smoke case");
        let argv = t.get("argv").and_then(|v| v.as_array()).expect("argv must be an array");
        assert!(argv.len() >= 1, "argv must be a non-empty list");
    }
}

// Acceptance: "Fixture does not invoke shell with untrusted input."
#[test]
fn fixture_does_not_invoke_shell_with_untrusted_input() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("no_shell_untrusted_input_contract").and_then(|v| v.as_table()).expect(
        "[no_shell_untrusted_input_contract] missing — acceptance: \
         \"Fixture does not invoke shell with untrusted input.\"",
    );
    for k in &[
        "must_set_shell_false",
        "must_pass_argv_as_list_not_string",
        "forbid_shell_equal_true",
        "forbid_calling_subprocess_call_with_string_argv",
        "forbid_string_interpolation_into_command",
        "forbid_use_of_subprocess_getoutput",
        "forbid_use_of_os_system",
        "must_distinguish_shell_use_from_untrusted_input",
    ] {
        assert_eq!(n.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let sh = n.get("shell_used_exit_code").and_then(|v| v.as_integer()).unwrap();
    let unt = n.get("untrusted_input_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(sh, 137);
    assert_eq!(unt, 138);
    assert_ne!(sh, unt, "shell-used and untrusted-input exit codes must differ");
    assert_eq!(
        n.get("shell_used_failure_kind").and_then(|v| v.as_str()),
        Some("subprocess_shell_used"),
    );
    assert_eq!(
        n.get("untrusted_input_failure_kind").and_then(|v| v.as_str()),
        Some("subprocess_untrusted_input_in_command"),
    );
}

// Acceptance: "Unsupported behavior is visible as blocker, not
// silent skip."
#[test]
fn unsupported_behavior_is_visible_as_blocker_not_silent_skip() {
    let doc = crate::common::load_toml(&manifest_path());
    let u = doc.get("unsupported_behavior_blocker_contract").and_then(|v| v.as_table()).expect(
        "[unsupported_behavior_blocker_contract] missing — acceptance: \
         \"Unsupported behavior is visible as blocker, not silent skip.\"",
    );
    for k in &[
        "must_mark_unsupported_subprocess_behavior_with_blocker",
        "must_link_blocker_to_issue",
        "forbid_silently_skipping_unsupported_subprocess_behavior",
        "forbid_falsely_passing_unsupported_subprocess_behavior",
    ] {
        assert_eq!(u.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(u.get("blocker_outcome_value").and_then(|v| v.as_str()), Some("blocker"));
    assert_eq!(u.get("blocker_link_field_name").and_then(|v| v.as_str()), Some("blocker_issue"));
    let exit = u.get("missing_blocker_link_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 139);
    assert_eq!(
        u.get("missing_blocker_link_failure_kind").and_then(|v| v.as_str()),
        Some("subprocess_unsupported_missing_blocker_link"),
    );
    let allowed: Vec<&str> = u.get("allowed_unsupported_subprocess_paths").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(!allowed.is_empty(), "allowed_unsupported_subprocess_paths must list at least one entry");
}

// Acceptance: "Runner records subprocess status separately."
#[test]
fn runner_records_subprocess_status_separately() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("separate_status_recording_contract").and_then(|v| v.as_table()).expect(
        "[separate_status_recording_contract] missing — acceptance: \
         \"Runner records subprocess status separately.\"",
    );
    for k in &[
        "must_record_subprocess_status_in_runner_output",
        "must_emit_summary_record_with_subprocess_status",
        "forbid_silent_or_implicit_subprocess_status",
        "must_distinguish_subprocess_status_from_overall_outcome",
    ] {
        assert_eq!(s.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(s.get("status_field_name").and_then(|v| v.as_str()), Some("subprocess_status"));
    assert_eq!(s.get("summary_record_format").and_then(|v| v.as_str()), Some("json"));
    let allowed: Vec<&str> = s.get("allowed_status_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["pass", "xfail", "blocker"] {
        assert!(allowed.contains(v), "allowed_status_values must include {v}");
    }
    let exit = s.get("missing_status_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 140);
    assert_eq!(
        s.get("missing_status_failure_kind").and_then(|v| v.as_str()),
        Some("subprocess_status_missing_or_implicit"),
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
        "argv", "shell",
        "expected_returncode", "expected_stdout", "expected_stderr",
        "actual_returncode", "actual_stdout", "actual_stderr",
        "subprocess_status", "blocker_issue",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let outcomes: Vec<&str> = c.get("outcome_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["pass", "fail", "xfail", "blocker", "missing", "skip"] {
        assert!(outcomes.contains(v), "runner_contract.outcome_values must include {v}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_does_not_invoke_shell_with_untrusted_input",
        "unsupported_behavior_is_visible_as_blocker_not_silent_skip",
        "runner_records_subprocess_status_separately",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("shell_pipelines").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(o.get("signal_handling").and_then(|v| v.as_bool()), Some(true));
}
