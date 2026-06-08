//! Schema gate for the mambalibs multiple-import fixture — closes
//! #2580.
//!
//! Acceptance (issue #2580):
//!
//!   1. Both imports succeed in a single interpreter run.
//!      `[multi_import_case]` pins must_import_both_in_single_interpreter
//!      and the pass/fail outcome pair.
//!   2. A namespace collision fails with a clear test failure.
//!      `[namespace_collision_case]` pins must_detect_namespace_collision
//!      + a distinct failure_kind and exit_code, and the diagnostic
//!      must name the colliding modules.
//!   3. Fixture stays offline and deterministic.
//!      `[offline_and_deterministic_invariant]` pins must_be_offline,
//!      must_be_deterministic, forbid_network_access, and
//!      forbid_nondeterministic_inputs.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("multiple_mambalibs_import")
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
        Some("mambalibs_multiple_mambalibs_import"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2580));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2531));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("mambalibs"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("multiple_mambalibs_import"));
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
fn binding_runs_in_single_interpreter_and_is_offline_deterministic() {
    let doc = load_toml(&manifest_path());
    let b = doc.get("binding").and_then(|v| v.as_table()).expect("[binding] missing");
    assert_eq!(b.get("module_name").and_then(|v| v.as_str()), Some("mambalibs"));
    assert_eq!(b.get("must_run_in_single_interpreter").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(b.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(b.get("must_be_offline").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(b.get("local_binding_crate_fixture_issue").and_then(|v| v.as_integer()), Some(2577));
}

#[test]
fn modules_declare_two_distinct_fixtures_with_distinct_values() {
    let doc = load_toml(&manifest_path());
    let mods = doc.get("modules").and_then(|v| v.as_array()).expect("[[modules]] missing");
    assert_eq!(mods.len(), 2, "exactly two modules must be declared");

    let names: Vec<&str> = mods.iter()
        .filter_map(|m| m.as_table().and_then(|t| t.get("fixture_module")).and_then(|v| v.as_str()))
        .collect();
    assert_eq!(names.len(), 2);
    assert_ne!(names[0], names[1], "fixture_module names must differ");

    let funcs: Vec<&str> = mods.iter()
        .filter_map(|m| m.as_table().and_then(|t| t.get("exported_function")).and_then(|v| v.as_str()))
        .collect();
    assert_eq!(funcs.len(), 2);
    assert_ne!(funcs[0], funcs[1], "exported_function names must differ");

    let values: Vec<i64> = mods.iter()
        .filter_map(|m| m.as_table().and_then(|t| t.get("expected_return_value")).and_then(|v| v.as_integer()))
        .collect();
    assert_eq!(values.len(), 2);
    assert_ne!(values[0], values[1], "expected_return_value must differ to prove no collision");

    for m in mods {
        let t = m.as_table().unwrap();
        let stmt = t.get("import_statement").and_then(|v| v.as_str()).unwrap();
        assert!(stmt.starts_with("from mambalibs import "));
        let fixture_module = t.get("fixture_module").and_then(|v| v.as_str()).unwrap();
        assert!(stmt.contains(fixture_module));
    }
}

// Acceptance: "Both imports succeed in a single interpreter run."
#[test]
fn both_imports_succeed_in_single_interpreter() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("multi_import_case").and_then(|v| v.as_table()).expect(
        "[multi_import_case] missing — acceptance: \
         \"Both imports succeed in a single interpreter run.\"",
    );
    assert_eq!(c.get("must_import_both_in_single_interpreter").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_assert_each_exported_function_returns_own_value").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("expected_outcome_when_both_imports_succeed").and_then(|v| v.as_str()), Some("pass"));
    assert_eq!(c.get("expected_outcome_when_either_import_fails").and_then(|v| v.as_str()), Some("fail"));
    assert_eq!(c.get("both_imports_failure_kind").and_then(|v| v.as_str()), Some("import_failure"));
    assert_eq!(c.get("both_imports_failure_exit_code").and_then(|v| v.as_integer()), Some(2));
}

// Acceptance: "A namespace collision fails with a clear test failure."
#[test]
fn namespace_collision_fails_with_clear_test_failure() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("namespace_collision_case").and_then(|v| v.as_table()).expect(
        "[namespace_collision_case] missing — acceptance: \
         \"A namespace collision fails with a clear test failure.\"",
    );
    assert_eq!(c.get("must_detect_namespace_collision").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_emit_clear_failure_diagnostic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("collision_failure_kind").and_then(|v| v.as_str()), Some("namespace_collision"));
    let exit = c.get("collision_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_ne!(exit, 0);
    assert_ne!(exit, 2, "collision exit code must differ from import_failure to distinguish failure modes");
    assert_eq!(c.get("collision_diagnostic_must_name_colliding_modules").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("collision_diagnostic_must_distinguish_from_import_failure").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_not_silently_overwrite_module_binding").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("expected_outcome_when_collision_detected").and_then(|v| v.as_str()), Some("fail"));
}

// Acceptance: "Fixture stays offline and deterministic."
#[test]
fn fixture_stays_offline_and_deterministic() {
    let doc = load_toml(&manifest_path());
    let i = doc.get("offline_and_deterministic_invariant").and_then(|v| v.as_table()).expect(
        "[offline_and_deterministic_invariant] missing — acceptance: \
         \"Fixture stays offline and deterministic.\"",
    );
    for f in &[
        "must_be_offline",
        "must_be_deterministic",
        "forbid_network_access",
        "forbid_nondeterministic_inputs",
        "must_only_use_local_binding_crates",
    ] {
        assert_eq!(i.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module", "module_a", "module_b",
        "returned_value_a", "returned_value_b",
        "expected_returned_value_a", "expected_returned_value_b",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &["both_modules_import_in_one_run", "namespace_collision_is_clear_failure"] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("testing_all_cclab_libraries").and_then(|v| v.as_bool()), Some(true));
}
