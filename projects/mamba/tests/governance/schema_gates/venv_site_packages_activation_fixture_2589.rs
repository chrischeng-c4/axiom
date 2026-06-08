//! Schema gate for the venv site-packages import activation fixture
//! — closes #2589.
//!
//! Acceptance (issue #2589):
//!
//!   1. Import succeeds only after install or sync.
//!      `[install_then_import_case]` pins two-phase run (before
//!      install must fail, after install must succeed) +
//!      allowed_activation_commands = [install, sync] + distinct
//!      failure_kinds and exit codes for both wrong-direction
//!      outcomes.
//!   2. Test fails if site-packages is not on the runtime path.
//!      `[site_packages_path_contract]` pins
//!      must_assert_site_packages_on_runtime_path +
//!      must_assert_site_packages_path_is_under_temp_env +
//!      site_packages_missing_failure_kind +
//!      diagnostic_must_print_sys_path_when_missing.
//!   3. Test avoids global Python or user cache state.
//!      `[global_state_isolation]` pins forbid_reads/writes to
//!      user-home / global-python + forbid_PYTHONPATH_inheritance +
//!      forbidden_env_vars covering PYTHONPATH / PYTHONHOME /
//!      PIP_CACHE_DIR / MAMBA_CACHE_DIR.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("package_manager")
        .join("venv_site_packages_activation")
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
        Some("venv_site_packages_activation"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2589));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2532)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("venv_site_packages_activation")
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
fn index_cross_references_frozen_local_simple_index() {
    let doc = load_toml(&manifest_path());
    let i = doc
        .get("index")
        .and_then(|v| v.as_table())
        .expect("[index] missing");
    assert_eq!(
        i.get("kind").and_then(|v| v.as_str()),
        Some("frozen_local_simple_index")
    );
    assert_eq!(
        i.get("local_simple_index_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2585),
        "must cross-reference frozen local simple-index fixture #2585",
    );
    assert_eq!(
        i.get("must_be_offline").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        i.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn package_under_test_is_pure_python() {
    let doc = load_toml(&manifest_path());
    let p = doc
        .get("package_under_test")
        .and_then(|v| v.as_table())
        .expect("[package_under_test] missing");
    let name = p.get("name").and_then(|v| v.as_str()).unwrap();
    assert!(!name.is_empty());
    assert!(!p
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap()
        .is_empty());
    assert_eq!(
        p.get("must_be_pure_python").and_then(|v| v.as_bool()),
        Some(true)
    );
    let import_name = p.get("import_name").and_then(|v| v.as_str()).unwrap();
    assert_eq!(
        import_name, name,
        "import_name must match package name for a pure-Python package"
    );
    let stmt = p.get("import_statement").and_then(|v| v.as_str()).unwrap();
    assert!(stmt.starts_with("import "));
    assert!(stmt.contains(import_name));
}

#[test]
fn temp_environment_is_per_test_and_isolated() {
    let doc = load_toml(&manifest_path());
    let t = doc
        .get("temp_environment")
        .and_then(|v| v.as_table())
        .expect("[temp_environment] missing");
    for f in &[
        "must_use_per_test_temp_env",
        "must_use_per_test_temp_project",
        "must_clean_up_on_success",
        "must_preserve_on_failure_for_diagnostics",
        "must_record_sys_path_per_run",
    ] {
        assert_eq!(
            t.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

// Acceptance: "Import succeeds only after install or sync."
#[test]
fn import_succeeds_only_after_install_or_sync() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("install_then_import_case")
        .and_then(|v| v.as_table())
        .expect(
            "[install_then_import_case] missing — acceptance: \
         \"Import succeeds only after install or sync.\"",
        );
    for f in &[
        "must_run_import_script_before_install",
        "must_assert_import_fails_before_install",
        "must_run_install_or_sync",
        "must_run_import_script_after_install",
        "must_assert_import_succeeds_after_install",
        "must_be_deterministic",
    ] {
        assert_eq!(
            c.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    let allowed: Vec<&str> = c
        .get("allowed_activation_commands")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for cmd in &["install", "sync"] {
        assert!(
            allowed.contains(cmd),
            "allowed_activation_commands must include {cmd}"
        );
    }
    assert_eq!(
        c.get("pre_install_failure_kind").and_then(|v| v.as_str()),
        Some("import_succeeded_before_install")
    );
    let pre_exit = c
        .get("pre_install_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(pre_exit, 0);
    assert_eq!(pre_exit, 19);
    assert_eq!(
        c.get("post_install_failure_kind").and_then(|v| v.as_str()),
        Some("import_failed_after_install")
    );
    let post_exit = c
        .get("post_install_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(post_exit, 0);
    assert_eq!(post_exit, 20);
    assert_ne!(
        pre_exit, post_exit,
        "pre- and post-install failure exit codes must differ"
    );
}

// Acceptance: "Test fails if site-packages is not on the runtime path."
#[test]
fn test_fails_if_site_packages_not_on_runtime_path() {
    let doc = load_toml(&manifest_path());
    let s = doc
        .get("site_packages_path_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[site_packages_path_contract] missing — acceptance: \
         \"Test fails if site-packages is not on the runtime path.\"",
        );
    assert_eq!(
        s.get("must_assert_site_packages_on_runtime_path")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_assert_site_packages_path_is_under_temp_env")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("site_packages_missing_failure_kind")
            .and_then(|v| v.as_str()),
        Some("site_packages_not_on_runtime_path")
    );
    let exit = s
        .get("site_packages_missing_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 21);
    assert_eq!(
        s.get("diagnostic_must_print_sys_path_when_missing")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_distinguish_from_import_failed_after_install")
            .and_then(|v| v.as_bool()),
        Some(true)
    );

    let post_exit = doc
        .get("install_then_import_case")
        .and_then(|v| v.get("post_install_failure_exit_code"))
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(
        exit, post_exit,
        "site-packages-missing exit code must differ from import-failed-after-install"
    );
}

// Acceptance: "Test avoids global Python or user cache state."
#[test]
fn test_avoids_global_python_or_user_cache_state() {
    let doc = load_toml(&manifest_path());
    let g = doc
        .get("global_state_isolation")
        .and_then(|v| v.as_table())
        .expect(
            "[global_state_isolation] missing — acceptance: \
         \"Test avoids global Python or user cache state.\"",
        );
    for f in &[
        "forbid_reads_from_user_home",
        "forbid_writes_to_user_home",
        "forbid_reads_from_global_python_install",
        "forbid_writes_to_global_python_install",
        "forbid_pythonpath_inheritance_from_parent",
        "forbid_pythonhome_inheritance_from_parent",
        "must_use_clean_environment_for_subprocess",
    ] {
        assert_eq!(
            g.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    let forbidden: Vec<&str> = g
        .get("forbidden_env_vars")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "PYTHONPATH",
        "PYTHONHOME",
        "PIP_CACHE_DIR",
        "MAMBA_CACHE_DIR",
    ] {
        assert!(
            forbidden.contains(required),
            "forbidden_env_vars must include {required}"
        );
    }
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
        "package",
        "version",
        "import_statement",
        "phase",
        "sys_path",
        "site_packages_path",
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
        "import_succeeds_only_after_install_or_sync",
        "site_packages_must_be_on_runtime_path",
        "no_global_python_or_user_cache_state",
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
        o.get("virtualenv_ux_polish").and_then(|v| v.as_bool()),
        Some(true)
    );
}
