//! Schema gate for the mambalibs `mamba build` E2E harness fixture —
//! closes #2578.
//!
//! Acceptance (issue #2578):
//!
//!   1. Missing build artifact fails the test.
//!      `[artifact_case]` pins must_assert_artifact_path_exists_on_disk
//!      and the artifact_missing failure_kind + exit_code (4).
//!   2. Nonzero build exit fails the test with logs.
//!      `[nonzero_exit_case]` pins clean_exit_code=0 and the
//!      nonzero_exit failure_kind + must_attach_captured_stdout +
//!      must_attach_captured_stderr + exit_code (5) +
//!      must_not_drop_logs.
//!   3. Test is isolated from user-level mamba caches.
//!      `[user_cache_isolation]` pins per-test cache dir +
//!      forbid_reads/writes for user-home and global-mamba-cache +
//!      forbidden_env_vars covering MAMBA_CACHE_DIR / MAMBA_HOME /
//!      XDG_CACHE_HOME.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("mamba_build_e2e_harness")
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
        Some("mambalibs_mamba_build_e2e_harness"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2578));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2531)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("mambalibs")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("mamba_build_e2e_harness")
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
fn fixture_project_targets_the_local_binding_crate() {
    let doc = load_toml(&manifest_path());
    let f = doc
        .get("fixture_project")
        .and_then(|v| v.as_table())
        .expect("[fixture_project] missing");
    assert_eq!(
        f.get("local_binding_crate_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2577),
        "must cross-reference #2577",
    );
    assert_eq!(
        f.get("mamba_toml_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2575),
        "must cross-reference #2575",
    );
    assert!(f.get("relative_path").and_then(|v| v.as_str()).is_some());
    assert_eq!(
        f.get("must_be_offline").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        f.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(f.get("must_be_small").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn build_command_is_the_smallest_e2e_command_and_captures_everything() {
    let doc = load_toml(&manifest_path());
    let b = doc
        .get("build_command")
        .and_then(|v| v.as_table())
        .expect("[build_command] missing");
    assert_eq!(b.get("program").and_then(|v| v.as_str()), Some("mamba"));
    assert_eq!(b.get("subcommand").and_then(|v| v.as_str()), Some("build"));
    for f in &[
        "must_be_smallest_e2e_command",
        "must_target_fixture_project",
        "must_capture_stdout",
        "must_capture_stderr",
        "must_capture_exit_status",
        "must_capture_artifact_path",
    ] {
        assert_eq!(
            b.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

// Acceptance: "Missing build artifact fails the test."
#[test]
fn missing_build_artifact_fails_the_test() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("artifact_case").and_then(|v| v.as_table()).expect(
        "[artifact_case] missing — acceptance: \
         \"Missing build artifact fails the test.\"",
    );
    assert_eq!(
        c.get("must_assert_artifact_path_is_recorded")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_assert_artifact_path_exists_on_disk")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("expected_outcome_when_artifact_present")
            .and_then(|v| v.as_str()),
        Some("pass")
    );
    assert_eq!(
        c.get("expected_outcome_when_artifact_missing")
            .and_then(|v| v.as_str()),
        Some("fail")
    );
    assert_eq!(
        c.get("artifact_missing_failure_kind")
            .and_then(|v| v.as_str()),
        Some("artifact_missing")
    );
    assert_eq!(
        c.get("artifact_missing_exit_code")
            .and_then(|v| v.as_integer()),
        Some(4)
    );
    assert_eq!(
        c.get("artifact_missing_diagnostic_must_name_path")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Nonzero build exit fails the test with logs."
#[test]
fn nonzero_build_exit_fails_with_logs() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("nonzero_exit_case")
        .and_then(|v| v.as_table())
        .expect(
            "[nonzero_exit_case] missing — acceptance: \
         \"Nonzero build exit fails the test with logs.\"",
        );
    assert_eq!(
        c.get("must_assert_clean_exit_code")
            .and_then(|v| v.as_integer()),
        Some(0)
    );
    assert_eq!(
        c.get("expected_outcome_when_build_exits_clean")
            .and_then(|v| v.as_str()),
        Some("pass")
    );
    assert_eq!(
        c.get("expected_outcome_when_build_exits_nonzero")
            .and_then(|v| v.as_str()),
        Some("fail")
    );
    assert_eq!(
        c.get("nonzero_exit_failure_kind").and_then(|v| v.as_str()),
        Some("nonzero_build_exit")
    );
    assert_eq!(
        c.get("nonzero_exit_must_attach_captured_stdout")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("nonzero_exit_must_attach_captured_stderr")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("nonzero_exit_diagnostic_must_name_command")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("nonzero_exit_exit_code").and_then(|v| v.as_integer()),
        Some(5)
    );
    assert_eq!(
        c.get("must_not_drop_logs").and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Test is isolated from user-level mamba caches."
#[test]
fn isolated_from_user_level_mamba_caches() {
    let doc = load_toml(&manifest_path());
    let u = doc
        .get("user_cache_isolation")
        .and_then(|v| v.as_table())
        .expect(
            "[user_cache_isolation] missing — acceptance: \
         \"Test is isolated from user-level mamba caches.\"",
        );
    for f in &[
        "must_isolate_from_user_level_mamba_caches",
        "must_use_per_test_temp_cache_dir",
        "forbid_reads_from_user_home",
        "forbid_writes_to_user_home",
        "forbid_reads_from_global_mamba_cache",
        "forbid_writes_to_global_mamba_cache",
        "must_set_per_test_cache_dir_via_explicit_arg",
    ] {
        assert_eq!(
            u.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    let forbidden: Vec<&str> = u
        .get("forbidden_env_vars")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["MAMBA_CACHE_DIR", "MAMBA_HOME", "XDG_CACHE_HOME"] {
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
        "program",
        "subcommand",
        "stdout_captured",
        "stderr_captured",
        "exit_status",
        "artifact_path",
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
        "build_produces_artifact",
        "build_exits_clean_or_logs_failure",
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
        o.get("publishing_or_global_install")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
