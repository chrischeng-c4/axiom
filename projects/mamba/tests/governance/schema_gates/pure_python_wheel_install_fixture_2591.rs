//! Schema gate for the pure-Python wheel install fixture — closes
//! #2591.
//!
//! Acceptance (issue #2591):
//!
//!   1. Missing wheel file fails fixture validation.
//!      `[validation_case]` pins must_validate_wheel_file_exists +
//!      validation_must_run_before_install_attempt +
//!      missing_wheel_failure_kind + exit_code=23.
//!   2. Install succeeds offline. `[offline_install_case]` pins
//!      must_be_offline + forbid_network_access +
//!      must_extract_wheel_into_temp_env + expected_install_exit_
//!      code=0 + must_assert_dist_info_present + RECORD file.
//!   3. Imported package returns an expected sentinel value.
//!      `[import_sentinel_case]` pins
//!      must_assert_sentinel_attribute_present +
//!      must_assert_sentinel_value_equals_expected + distinct
//!      failure_kinds + exit codes for sentinel mismatch (24) vs
//!      import failure (25).

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("package_manager")
        .join("pure_python_wheel_install")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pure_python_wheel_install"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2591));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2532));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("package_manager"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("pure_python_wheel_install"));
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
fn index_cross_references_frozen_local_simple_index() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("index").and_then(|v| v.as_table()).expect("[index] missing");
    assert_eq!(i.get("kind").and_then(|v| v.as_str()), Some("frozen_local_simple_index"));
    assert_eq!(
        i.get("local_simple_index_fixture_issue").and_then(|v| v.as_integer()),
        Some(2585),
        "must cross-reference frozen local simple-index fixture #2585",
    );
    assert_eq!(i.get("must_be_offline").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(i.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn wheel_is_minimal_pure_python_py3_none_any() {
    let doc = crate::common::load_toml(&manifest_path());
    let w = doc.get("wheel").and_then(|v| v.as_table()).expect("[wheel] missing");
    let name = w.get("package_name").and_then(|v| v.as_str()).unwrap();
    let ver = w.get("package_version").and_then(|v| v.as_str()).unwrap();
    let filename = w.get("wheel_filename").and_then(|v| v.as_str()).unwrap();
    assert!(filename.contains(name) && filename.contains(ver));
    assert!(filename.ends_with(".whl"));
    assert!(filename.contains("py3-none-any"));
    assert_eq!(w.get("must_be_pure_python").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(w.get("must_be_py3_none_any").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(w.get("must_be_minimal").and_then(|v| v.as_bool()), Some(true));
    let sentinel_attr = w.get("sentinel_attribute").and_then(|v| v.as_str()).unwrap();
    assert!(!sentinel_attr.is_empty());
    assert!(w.get("sentinel_value").and_then(|v| v.as_integer()).is_some());
    let stmt = w.get("import_statement").and_then(|v| v.as_str()).unwrap();
    assert!(stmt.starts_with("import "));
    assert!(stmt.contains(name));
    let tmpl = w.get("import_assert_template").and_then(|v| v.as_str()).unwrap();
    assert!(tmpl.contains(name));
    assert!(tmpl.contains(sentinel_attr));
    assert!(tmpl.contains("=="));
}

#[test]
fn temp_environment_is_per_test_and_isolated() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc.get("temp_environment").and_then(|v| v.as_table()).expect("[temp_environment] missing");
    for f in &[
        "must_use_per_test_temp_env",
        "must_use_per_test_temp_project",
        "must_clean_up_on_success",
        "must_preserve_on_failure_for_diagnostics",
    ] {
        assert_eq!(t.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

// Acceptance: "Missing wheel file fails fixture validation."
#[test]
fn missing_wheel_file_fails_fixture_validation() {
    let doc = crate::common::load_toml(&manifest_path());
    let v = doc.get("validation_case").and_then(|v| v.as_table()).expect(
        "[validation_case] missing — acceptance: \
         \"Missing wheel file fails fixture validation.\"",
    );
    assert_eq!(v.get("must_validate_wheel_file_exists_on_disk").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(v.get("must_validate_wheel_file_matches_index_metadata").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(v.get("validation_must_run_before_install_attempt").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(v.get("missing_wheel_failure_kind").and_then(|v| v.as_str()), Some("wheel_file_missing"));
    let exit = v.get("missing_wheel_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 23);
    assert_eq!(v.get("missing_wheel_diagnostic_must_name_path").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(v.get("must_distinguish_from_resolver_failures").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Install succeeds offline."
#[test]
fn install_succeeds_offline() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("offline_install_case").and_then(|v| v.as_table()).expect(
        "[offline_install_case] missing — acceptance: \"Install succeeds offline.\"",
    );
    assert_eq!(c.get("must_run_install_command").and_then(|v| v.as_bool()), Some(true));
    let tmpl = c.get("install_command_template").and_then(|v| v.as_str()).unwrap();
    assert!(tmpl.starts_with("mamba "), "install_command_template must invoke mamba");
    let pkg = doc.get("wheel").and_then(|v| v.get("package_name")).and_then(|v| v.as_str()).unwrap();
    assert!(tmpl.contains(pkg), "install_command_template must reference the package");
    assert_eq!(c.get("must_be_offline").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("forbid_network_access").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_extract_wheel_into_temp_env").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_capture_install_stdout").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_capture_install_stderr").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_capture_install_exit_status").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("expected_install_exit_code").and_then(|v| v.as_integer()), Some(0));
    assert_eq!(c.get("must_assert_dist_info_present_after_install").and_then(|v| v.as_bool()), Some(true));
    let dist_info = c.get("dist_info_dir_pattern").and_then(|v| v.as_str()).unwrap();
    assert!(dist_info.contains(pkg));
    assert!(dist_info.contains(".dist-info"));
    assert_eq!(c.get("must_assert_record_file_present").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("record_file_path_within_dist_info").and_then(|v| v.as_str()), Some("RECORD"));
}

// Acceptance: "Imported package returns an expected sentinel value."
#[test]
fn imported_package_returns_expected_sentinel_value() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("import_sentinel_case").and_then(|v| v.as_table()).expect(
        "[import_sentinel_case] missing — acceptance: \
         \"Imported package returns an expected sentinel value.\"",
    );
    assert_eq!(c.get("must_run_import_script_after_install").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_assert_sentinel_attribute_present").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_assert_sentinel_value_equals_expected").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("sentinel_mismatch_failure_kind").and_then(|v| v.as_str()), Some("sentinel_value_mismatch"));
    let mismatch_exit = c.get("sentinel_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_ne!(mismatch_exit, 0);
    assert_eq!(mismatch_exit, 24);
    assert_eq!(c.get("import_failure_failure_kind").and_then(|v| v.as_str()), Some("import_failed_after_install"));
    let import_exit = c.get("import_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_ne!(import_exit, 0);
    assert_eq!(import_exit, 25);
    assert_ne!(mismatch_exit, import_exit, "sentinel-mismatch and import-failure exit codes must differ");
    assert_eq!(c.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("must_distinguish_sentinel_mismatch_from_import_failure").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "package", "version", "wheel_filename",
        "dist_info_dir", "sentinel_attribute",
        "sentinel_expected_value", "sentinel_observed_value",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "missing_wheel_file_fails_fixture_validation",
        "install_succeeds_offline",
        "imported_package_returns_expected_sentinel",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("native_wheel_support").and_then(|v| v.as_bool()), Some(true));
}
