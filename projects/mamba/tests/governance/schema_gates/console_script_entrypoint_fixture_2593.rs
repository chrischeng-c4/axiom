//! Schema gate for the console script entrypoint fixture — closes
//! #2593.
//!
//! Acceptance (issue #2593):
//!
//!   1. Entry point executable is created or made invokable by the
//!      run command. `[entry_point_install_contract]` pins
//!      must_install_console_script + run_command_template ("mamba
//!      run gamma-hello") + must_resolve_entry_point_from_project_env
//!      + missing entry point exit_code=31 + not-invokable exit=32.
//!   2. Command output matches the fixture sentinel.
//!      `[command_output_contract]` pins must_capture_stdout +
//!      expected_run_exit_code=0 + sentinel mismatch exit_code=33 +
//!      run failure exit_code=34.
//!   3. Test avoids user-level PATH mutation.
//!      `[path_isolation_contract]` pins forbid_modifying_user_path +
//!      forbid_writing_to_user_home_bin + must_run_through_project_
//!      env_only + path mutation exit_code=35 + allowed_path_
//!      resolution_modes.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("package_manager")
        .join("console_script_entrypoint")
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
        Some("console_script_entrypoint")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2593));
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
        Some("console_script_entrypoint")
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
fn package_declares_exactly_one_console_script_with_consistent_entry_point() {
    let doc = load_toml(&manifest_path());
    let p = doc
        .get("package")
        .and_then(|v| v.as_table())
        .expect("[package] missing");
    let name = p.get("package_name").and_then(|v| v.as_str()).unwrap();
    let ver = p.get("package_version").and_then(|v| v.as_str()).unwrap();
    let filename = p.get("wheel_filename").and_then(|v| v.as_str()).unwrap();
    assert!(filename.contains(name) && filename.contains(ver));
    assert!(filename.ends_with(".whl"));
    assert!(filename.contains("py3-none-any"));
    assert_eq!(
        p.get("must_be_pure_python").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("must_be_py3_none_any").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        p.get("must_declare_exactly_one_console_script")
            .and_then(|v| v.as_bool()),
        Some(true)
    );

    let script = p
        .get("console_script_name")
        .and_then(|v| v.as_str())
        .unwrap();
    let module = p
        .get("console_script_module")
        .and_then(|v| v.as_str())
        .unwrap();
    let callable = p
        .get("console_script_callable")
        .and_then(|v| v.as_str())
        .unwrap();
    let spec = p
        .get("console_script_entry_point_spec")
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(
        module.contains(name),
        "module {module} must reference package {name}"
    );
    assert_eq!(
        spec,
        format!("{script} = {module}:{callable}").as_str(),
        "entry_point_spec {spec} must be in the canonical `name = module:callable` form"
    );
}

// Acceptance: "Entry point executable is created or made invokable by the run command."
#[test]
fn entry_point_is_invokable_via_run_command() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("entry_point_install_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[entry_point_install_contract] missing — acceptance: \
         \"Entry point executable is created or made invokable by the run command.\"",
        );
    assert_eq!(
        c.get("must_install_console_script")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_make_entry_point_invokable_via_run_command")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let tmpl = c
        .get("run_command_template")
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(
        tmpl.starts_with("mamba "),
        "run_command_template must invoke mamba"
    );
    let script = doc
        .get("package")
        .and_then(|v| v.get("console_script_name"))
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(
        tmpl.contains(script),
        "run_command_template must reference console_script_name {script}"
    );
    assert!(
        tmpl.contains("run"),
        "run_command_template must use the run verb"
    );
    assert_eq!(
        c.get("must_resolve_entry_point_from_project_env")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_not_depend_on_global_console_script_registration")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let missing = c
        .get("entry_point_missing_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(missing, 31);
    let not_inv = c
        .get("entry_point_not_invokable_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(not_inv, 32);
    assert_ne!(missing, not_inv);
}

// Acceptance: "Command output matches the fixture sentinel."
#[test]
fn command_output_matches_sentinel() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("command_output_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[command_output_contract] missing — acceptance: \
         \"Command output matches the fixture sentinel.\"",
        );
    assert_eq!(
        c.get("must_capture_stdout").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_capture_stderr").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_capture_exit_status").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("expected_run_exit_code").and_then(|v| v.as_integer()),
        Some(0)
    );
    assert_eq!(
        c.get("must_assert_stdout_matches_sentinel")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let mismatch = c
        .get("sentinel_mismatch_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(mismatch, 33);
    let run_fail = c
        .get("run_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(run_fail, 34);
    assert_ne!(mismatch, run_fail);

    let s = doc
        .get("sentinel")
        .and_then(|v| v.as_table())
        .expect("[sentinel] missing");
    let expected = s.get("expected_stdout").and_then(|v| v.as_str()).unwrap();
    let script = doc
        .get("package")
        .and_then(|v| v.get("console_script_name"))
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(
        expected.contains(script),
        "expected_stdout must reference {script}"
    );
    assert_eq!(
        s.get("expected_stdout_match_mode").and_then(|v| v.as_str()),
        Some("exact_line")
    );
    assert_eq!(
        s.get("must_match_after_trim").and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Test avoids user-level PATH mutation."
#[test]
fn test_avoids_user_level_path_mutation() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("path_isolation_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[path_isolation_contract] missing — acceptance: \
         \"Test avoids user-level PATH mutation.\"",
        );
    for f in &[
        "forbid_modifying_user_path",
        "forbid_writing_to_user_home_bin",
        "forbid_writing_to_system_bin",
        "must_run_through_project_env_only",
        "must_not_require_shell_rc_changes",
    ] {
        assert_eq!(
            c.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    let exit = c
        .get("path_mutation_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 35);
    let modes: Vec<&str> = c
        .get("allowed_path_resolution_modes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(modes.contains(&"project_local_entry_point"));
    assert!(modes.contains(&"managed_run_command_dispatch"));
    assert!(
        !modes.contains(&"user_home_bin"),
        "user_home_bin must NOT be allowed"
    );
    assert!(
        !modes.contains(&"system_bin"),
        "system_bin must NOT be allowed"
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
        "package",
        "version",
        "console_script_name",
        "console_script_entry_point_spec",
        "run_command",
        "stdout_capture",
        "stderr_capture",
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
        "entry_point_is_invokable_via_run_command",
        "command_output_matches_sentinel",
        "test_avoids_user_level_path_mutation",
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
        o.get("shell_completion").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        o.get("per_platform_script_shims").and_then(|v| v.as_bool()),
        Some(true)
    );
}
