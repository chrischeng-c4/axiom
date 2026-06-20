//! Schema gate for the minimal unittest dispatch path fixture —
//! closes #2545.
//!
//! Acceptance (issue #2545):
//!
//!   1. The fixture fails when its sentinel assertion is inverted.
//!      `[sentinel_failure_contract]` pins
//!      must_pass_with_original_assertion +
//!      must_fail_with_inverted_assertion +
//!      must_emit_assertion_error_kind_on_inversion +
//!      forbid_silent_pass_on_inverted_assertion +
//!      forbid_inverted_assertion_being_caught_by_runner + exit
//!      176 (assertion failure) / 177 (runner error) +
//!      must_distinguish_assertion_failure_from_runner_error.
//!   2. The fixture is not classified as Stub.
//!      `[non_stub_classification_contract]` pins
//!      must_not_be_classified_as_stub +
//!      must_execute_real_assertion_during_dispatch +
//!      classification_field_name="classification" +
//!      allowed_classification_values=[real, executed] +
//!      forbid_classification_value_stub/passthrough/placeholder +
//!      exit 178.
//!   3. No broad unittest implementation work is included here.
//!      `[narrow_dispatch_scope_contract]` pins
//!      must_limit_scope_to_minimal_dispatch_path + required and
//!      forbidden dispatch surfaces + exit 179.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("harness")
        .join("cpython")
        .join("config")
        .join("seeds")
        .join("minimal_unittest_dispatch")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("cpython_lib_test_minimal_unittest_dispatch")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2545));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2528)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("cpython_lib_test")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("cpython_lib_test_minimal_unittest_dispatch")
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
fn determinism_pins_no_external_state() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc
        .get("determinism")
        .and_then(|v| v.as_table())
        .expect("[determinism] missing");
    for k in &[
        "must_be_deterministic",
        "must_be_offline",
        "must_be_repeatable_across_runs",
        "forbid_wallclock_dependence",
        "forbid_random_seed_dependence",
    ] {
        assert_eq!(
            d.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
}

#[test]
fn seed_module_definition_pins_canonical_assertion() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc
        .get("seed_module_definition")
        .and_then(|v| v.as_table())
        .expect("[seed_module_definition] missing");
    assert_eq!(
        s.get("module_name").and_then(|v| v.as_str()),
        Some("test_mamba_2545_minimal")
    );
    assert_eq!(
        s.get("testcase_class_name").and_then(|v| v.as_str()),
        Some("Mamba2545MinimalTestCase")
    );
    assert_eq!(
        s.get("testcase_method_name").and_then(|v| v.as_str()),
        Some("test_two_plus_two_equals_four")
    );
    assert_eq!(
        s.get("assertion_kind").and_then(|v| v.as_str()),
        Some("assertEqual")
    );
    assert_eq!(
        s.get("assertion_lhs_python_repr").and_then(|v| v.as_str()),
        Some("2 + 2")
    );
    assert_eq!(
        s.get("assertion_rhs_python_repr").and_then(|v| v.as_str()),
        Some("4")
    );
    assert_eq!(
        s.get("sentinel_inversion_lhs_python_repr")
            .and_then(|v| v.as_str()),
        Some("2 + 2")
    );
    assert_eq!(
        s.get("sentinel_inversion_rhs_python_repr")
            .and_then(|v| v.as_str()),
        Some("5")
    );
}

#[test]
fn runner_command_pins_canonical_invocation() {
    let doc = crate::common::load_toml(&manifest_path());
    let r = doc
        .get("runner_command")
        .and_then(|v| v.as_table())
        .expect("[runner_command] missing");
    for k in &[
        "must_record_runner_command_in_summary",
        "forbid_invocation_via_pytest",
        "forbid_invocation_via_nose",
        "forbid_invocation_via_ad_hoc_harness",
    ] {
        assert_eq!(
            r.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        r.get("runner_command_field_name").and_then(|v| v.as_str()),
        Some("runner_command")
    );
    assert_eq!(
        r.get("canonical_runner_command").and_then(|v| v.as_str()),
        Some("mamba -m unittest -v test_mamba_2545_minimal"),
    );
}

// Acceptance: "The fixture fails when its sentinel assertion is
// inverted."
#[test]
fn fixture_fails_when_sentinel_assertion_is_inverted() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc
        .get("sentinel_failure_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[sentinel_failure_contract] missing — acceptance: \
         \"The fixture fails when its sentinel assertion is inverted.\"",
        );
    for k in &[
        "must_pass_with_original_assertion",
        "must_fail_with_inverted_assertion",
        "must_emit_assertion_error_kind_on_inversion",
        "forbid_silent_pass_on_inverted_assertion",
        "forbid_inverted_assertion_being_caught_by_runner",
        "must_distinguish_assertion_failure_from_runner_error",
    ] {
        assert_eq!(
            c.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        c.get("original_assertion_expected_outcome")
            .and_then(|v| v.as_str()),
        Some("pass"),
    );
    assert_eq!(
        c.get("inverted_assertion_expected_outcome")
            .and_then(|v| v.as_str()),
        Some("fail"),
    );
    let assert_exit = c
        .get("inverted_assertion_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let runner_exit = c
        .get("runner_error_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(assert_exit, 176);
    assert_eq!(runner_exit, 177);
    assert_ne!(
        assert_exit, runner_exit,
        "assertion-failure and runner-error exit codes must differ"
    );
    assert_eq!(
        c.get("inverted_assertion_failure_kind")
            .and_then(|v| v.as_str()),
        Some("unittest_assertion_failed"),
    );
    assert_eq!(
        c.get("runner_error_failure_kind").and_then(|v| v.as_str()),
        Some("unittest_runner_error"),
    );
}

// Acceptance: "The fixture is not classified as Stub."
#[test]
fn fixture_is_not_classified_as_stub() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc
        .get("non_stub_classification_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[non_stub_classification_contract] missing — acceptance: \
         \"The fixture is not classified as Stub.\"",
        );
    for k in &[
        "must_not_be_classified_as_stub",
        "must_execute_real_assertion_during_dispatch",
        "must_not_be_marked_as_passthrough",
        "must_not_be_marked_as_placeholder",
        "forbid_classification_value_stub",
        "forbid_classification_value_passthrough",
        "forbid_classification_value_placeholder",
        "forbid_silent_or_implicit_classification",
    ] {
        assert_eq!(
            n.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        n.get("classification_field_name").and_then(|v| v.as_str()),
        Some("classification")
    );
    let allowed: Vec<&str> = n
        .get("allowed_classification_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &["real", "executed"] {
        assert!(
            allowed.contains(v),
            "allowed_classification_values must include {v}"
        );
    }
    assert!(
        !allowed.contains(&"stub"),
        "allowed_classification_values must NOT include 'stub'"
    );
    assert!(
        !allowed.contains(&"passthrough"),
        "allowed_classification_values must NOT include 'passthrough'"
    );
    assert!(
        !allowed.contains(&"placeholder"),
        "allowed_classification_values must NOT include 'placeholder'"
    );
    let exit = n
        .get("stub_classification_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 178);
    assert_eq!(
        n.get("stub_classification_failure_kind")
            .and_then(|v| v.as_str()),
        Some("unittest_seed_classified_as_stub"),
    );
}

// Acceptance: "No broad unittest implementation work is included
// here."
#[test]
fn no_broad_unittest_implementation_work_is_included_here() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc
        .get("narrow_dispatch_scope_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[narrow_dispatch_scope_contract] missing — acceptance: \
         \"No broad unittest implementation work is included here.\"",
        );
    for k in &[
        "must_limit_scope_to_minimal_dispatch_path",
        "must_not_implement_full_unittest_runner",
        "must_not_implement_test_loader_full_protocol",
        "must_not_implement_test_suite_full_protocol",
        "must_not_implement_test_result_full_protocol",
    ] {
        assert_eq!(
            s.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let required: Vec<&str> = s
        .get("required_dispatch_surface")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &[
        "unittest.TestCase",
        "unittest.TestCase.setUp",
        "unittest.TestCase.tearDown",
        "unittest.TestCase.run",
        "unittest.TestCase.assertEqual",
    ] {
        assert!(
            required.contains(v),
            "required_dispatch_surface must include {v}"
        );
    }
    let forbidden: Vec<&str> = s
        .get("forbidden_surface")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &[
        "unittest.TestLoader.loadTestsFromModule_full_protocol",
        "unittest.TestSuite.run_full_protocol",
        "unittest.TextTestRunner_full_features",
        "unittest.mock_full_protocol",
    ] {
        assert!(forbidden.contains(v), "forbidden_surface must include {v}");
    }
    let exit = s
        .get("scope_overshoot_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 179);
    assert_eq!(
        s.get("scope_overshoot_failure_kind")
            .and_then(|v| v.as_str()),
        Some("unittest_dispatch_scope_overshoot"),
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
        "testcase_class_name",
        "testcase_method_name",
        "assertion_kind",
        "runner_command",
        "classification",
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
        "fixture_fails_when_sentinel_assertion_is_inverted",
        "fixture_is_not_classified_as_stub",
        "no_broad_unittest_implementation_work_is_included_here",
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
        o.get("full_unittest_compatibility_beyond_dispatch_path")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
