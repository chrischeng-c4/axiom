//! Schema gate for the stdlib argparse fixture — closes #2639.
//!
//! Acceptance (issue #2639):
//!
//!   1. Fixture fails if option parsing is wrong.
//!      `[failure_on_wrong_parsing_contract]` pins
//!      must_fail_on_incorrect_{namespace_value, flag_default,
//!      positional_value, missing_expected_error} + distinct exit
//!      codes 126/127/128/129 + must_distinguish_each_parsing_
//!      failure_kind.
//!   2. Error path does not terminate the whole test process
//!      unexpectedly. `[error_path_non_terminating_contract]` pins
//!      must_use_exit_on_error_false_or_explicit_handler +
//!      forbid_uncaught_systemexit_during_parse_failure +
//!      forbid_calling_sys_exit_from_runner_on_argparse_error +
//!      must_report_error_via_failure_kind_not_via_exit + exit
//!      code 130.
//!   3. Output is deterministic and locale-independent.
//!      `[deterministic_locale_independent_output_contract]` pins
//!      must_be_reproducible_across_runs +
//!      must_not_consult_locale_setting +
//!      forbid_use_of_LC_ALL_or_LANG_environment_variables +
//!      forbid_locale_dependent_prog_or_usage_string +
//!      must_pin_prog_name_explicitly + distinct exit codes
//!      131 (locale) / 132 (nondeterminism) +
//!      must_distinguish_locale_from_nondeterminism.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("stdlib")
        .join("argparse_behavioral")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path).unwrap();
    raw.parse().unwrap()
}

#[test]
fn header_is_well_formed() {
    let doc = load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("stdlib_argparse_behavioral")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2639));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2529)
    );
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("stdlib_argparse_behavioral")
    );
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = load_toml(&manifest_path());
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
    let doc = load_toml(&manifest_path());
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
fn surface_registers_argparse() {
    let doc = load_toml(&manifest_path());
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
        modules.contains(&"argparse"),
        "covered_modules must include argparse"
    );
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_argparse_in_stdlib_manifest",
        "must_cover_argument_parser",
        "must_cover_add_argument_flag",
        "must_cover_add_argument_positional",
        "must_cover_parse_args",
        "must_cover_parse_args_error_path",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    assert_eq!(
        s.get("import_statement").and_then(|v| v.as_str()),
        Some("import argparse")
    );
}

#[test]
fn parser_definition_pins_canonical_parser() {
    let doc = load_toml(&manifest_path());
    let p = doc
        .get("parser_definition")
        .and_then(|v| v.as_table())
        .expect("[parser_definition] missing");
    assert_eq!(p.get("prog").and_then(|v| v.as_str()), Some("mamba_2639"));
    assert_eq!(
        p.get("flag_name").and_then(|v| v.as_str()),
        Some("--verbose")
    );
    assert_eq!(
        p.get("flag_action").and_then(|v| v.as_str()),
        Some("store_true")
    );
    assert_eq!(p.get("flag_default").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(
        p.get("positional_name").and_then(|v| v.as_str()),
        Some("name")
    );
    assert_eq!(
        p.get("positional_required").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("exit_on_error").and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn deterministic_sample_covers_success_and_failure() {
    let doc = load_toml(&manifest_path());
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
        (
            "parse_success_cases",
            &["argv", "expected_name", "expected_verbose"],
        ),
        (
            "parse_failure_cases",
            &[
                "argv",
                "expected_error_kind",
                "must_not_call_systemexit_on_host",
            ],
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
    let fail = doc
        .get("parse_failure_cases")
        .and_then(|v| v.as_array())
        .unwrap();
    for c in fail {
        let t = c.as_table().unwrap();
        assert_eq!(
            t.get("must_not_call_systemexit_on_host")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}

// Acceptance: "Fixture fails if option parsing is wrong."
#[test]
fn fixture_fails_if_option_parsing_is_wrong() {
    let doc = load_toml(&manifest_path());
    let f = doc
        .get("failure_on_wrong_parsing_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[failure_on_wrong_parsing_contract] missing — acceptance: \
         \"Fixture fails if option parsing is wrong.\"",
        );
    for k in &[
        "must_fail_on_incorrect_namespace_value",
        "must_fail_on_incorrect_flag_default",
        "must_fail_on_incorrect_positional_value",
        "must_fail_on_missing_expected_error",
        "must_distinguish_each_parsing_failure_kind",
    ] {
        assert_eq!(
            f.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exits: Vec<i64> = [
        "namespace_mismatch_exit_code",
        "flag_default_mismatch_exit_code",
        "positional_mismatch_exit_code",
        "missing_expected_error_exit_code",
    ]
    .iter()
    .map(|k| f.get(*k).and_then(|v| v.as_integer()).unwrap())
    .collect();
    assert_eq!(exits, vec![126, 127, 128, 129]);
    let mut sorted = exits.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        exits.len(),
        "argparse parsing-failure exit codes must all differ"
    );
}

// Acceptance: "Error path does not terminate the whole test process
// unexpectedly."
#[test]
fn error_path_does_not_terminate_host_process_unexpectedly() {
    let doc = load_toml(&manifest_path());
    let e = doc
        .get("error_path_non_terminating_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[error_path_non_terminating_contract] missing — acceptance: \
         \"Error path does not terminate the whole test process unexpectedly.\"",
        );
    for k in &[
        "must_use_exit_on_error_false_or_explicit_handler",
        "forbid_uncaught_systemexit_during_parse_failure",
        "forbid_calling_sys_exit_from_runner_on_argparse_error",
        "must_report_error_via_failure_kind_not_via_exit",
    ] {
        assert_eq!(
            e.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exit = e
        .get("host_process_terminated_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 130);
    assert_eq!(
        e.get("host_process_terminated_failure_kind")
            .and_then(|v| v.as_str()),
        Some("argparse_error_path_terminated_host_process"),
    );
}

// Acceptance: "Output is deterministic and locale-independent."
#[test]
fn output_is_deterministic_and_locale_independent() {
    let doc = load_toml(&manifest_path());
    let d = doc
        .get("deterministic_locale_independent_output_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[deterministic_locale_independent_output_contract] missing — acceptance: \
         \"Output is deterministic and locale-independent.\"",
        );
    for k in &[
        "must_be_reproducible_across_runs",
        "must_not_consult_locale_setting",
        "forbid_use_of_LC_ALL_or_LANG_environment_variables",
        "forbid_locale_dependent_prog_or_usage_string",
        "must_pin_prog_name_explicitly",
        "must_distinguish_locale_from_nondeterminism",
    ] {
        assert_eq!(
            d.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        d.get("pinned_prog_name").and_then(|v| v.as_str()),
        Some("mamba_2639")
    );
    let loc = d
        .get("locale_dependency_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let nd = d
        .get("nondeterministic_output_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(loc, 131);
    assert_eq!(nd, 132);
    assert_ne!(loc, nd, "locale and nondeterminism exit codes must differ");
    assert_eq!(
        d.get("locale_dependency_failure_kind")
            .and_then(|v| v.as_str()),
        Some("argparse_locale_dependency_used"),
    );
    assert_eq!(
        d.get("nondeterministic_output_failure_kind")
            .and_then(|v| v.as_str()),
        Some("argparse_nondeterministic_output"),
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = load_toml(&manifest_path());
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
        "argv",
        "expected_name",
        "expected_verbose",
        "expected_error_kind",
        "actual_error_kind",
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
        "fixture_fails_if_option_parsing_is_wrong",
        "error_path_does_not_terminate_host_process_unexpectedly",
        "output_is_deterministic_and_locale_independent",
    ] {
        assert!(
            cases.contains(required),
            "runner_contract.case_values must include {required}"
        );
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("full_help_formatting_compatibility")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
