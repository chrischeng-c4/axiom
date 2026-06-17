//! Schema gate for the stdlib tempfile/shutil/glob behavioral fixture —
//! closes #2630.
//!
//! Acceptance (issue #2630):
//!
//!   1. Fixture uses only test temp directories.
//!      `[temp_only_filesystem_contract]` pins must_root_all_writes_
//!      under_a_per_run_tempdir + must_use_tempfile_temporary_
//!      directory_or_mkdtemp + forbid_writes_outside_per_run_tempdir
//!      + tempdir_prefix_value + escape exit_code=81.
//!   2. Fixture fails on wrong glob or copy behavior.
//!      `[failure_on_incorrect_behavior_contract]` pins
//!      must_fail_on_incorrect_glob_match +
//!      must_fail_on_incorrect_copy_contents + distinct exit codes
//!      82/83 + must_distinguish_glob_from_copy_mismatch.
//!   3. Cleanup does not depend on user filesystem state.
//!      `[cleanup_independence_contract]` pins
//!      must_clean_up_inside_per_run_tempdir_only + forbid_calls_to_
//!      user_home_paths + forbid_environment_dependent_paths +
//!      cleanup-dependency exit_code=84.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("stdlib")
        .join("tempfile_shutil_glob_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_tempfile_shutil_glob_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2630));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_tempfile_shutil_glob_behavioral"));
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
fn surface_covers_tempfile_shutil_glob() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["tempfile", "shutil", "glob"] {
        assert!(modules.contains(m), "covered_modules must include {m}");
    }
    for f in &[
        "must_be_importable_via_import_statement",
        "must_cover_tempfile_temporary_directory",
        "must_cover_tempfile_mkdtemp",
        "must_cover_shutil_copy",
        "must_cover_shutil_rmtree",
        "must_cover_glob_glob",
        "must_cover_file_creation",
        "must_cover_cleanup",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

#[test]
fn deterministic_sample_covers_create_copy_glob_cleanup() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("must_isolate_each_case_in_its_own_tempdir").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min, "sample bounds must be sane");
    assert!(max <= 64, "sample_max_records must stay small for per-run check");

    let specs: &[(&str, &[&str])] = &[
        ("file_creation_cases", &["relative_path", "contents"]),
        ("copy_cases", &["source_relative_path", "destination_relative_path", "expected_destination_contents"]),
        ("glob_cases", &["pattern", "created_relative_paths", "expected_match_relative_paths"]),
        ("cleanup_cases", &["relative_paths_to_remove"]),
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

// Acceptance: "Fixture uses only test temp directories."
#[test]
fn fixture_uses_only_test_temp_directories() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc.get("temp_only_filesystem_contract").and_then(|v| v.as_table()).expect(
        "[temp_only_filesystem_contract] missing — acceptance: \
         \"Fixture uses only test temp directories.\"",
    );
    for k in &[
        "must_root_all_writes_under_a_per_run_tempdir",
        "must_use_tempfile_temporary_directory_or_mkdtemp",
        "forbid_writes_outside_per_run_tempdir",
        "forbid_reads_outside_per_run_tempdir",
        "forbid_assertions_on_absolute_tempdir_path",
        "tempdir_prefix_must_be_deterministic",
    ] {
        assert_eq!(t.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(t.get("tempdir_prefix_value").and_then(|v| v.as_str()), Some("mamba_stdlib_2630_"));
    let exit = t.get("write_outside_tempdir_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 81);
    assert_eq!(t.get("write_outside_tempdir_failure_kind").and_then(|v| v.as_str()), Some("tempfile_write_escaped_tempdir"));
}

// Acceptance: "Fixture fails on wrong glob or copy behavior."
#[test]
fn fixture_fails_on_wrong_glob_or_copy_behavior() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc.get("failure_on_incorrect_behavior_contract").and_then(|v| v.as_table()).expect(
        "[failure_on_incorrect_behavior_contract] missing — acceptance: \
         \"Fixture fails on wrong glob or copy behavior.\"",
    );
    for k in &[
        "must_fail_on_incorrect_glob_match",
        "must_fail_on_incorrect_copy_contents",
        "must_fail_on_incorrect_copy_destination_existence",
        "must_distinguish_glob_from_copy_mismatch",
    ] {
        assert_eq!(f.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let glob_exit = f.get("glob_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    let copy_exit = f.get("copy_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(glob_exit, 82);
    assert_eq!(copy_exit, 83);
    assert_ne!(glob_exit, copy_exit, "glob and copy exit codes must differ");
    assert_eq!(f.get("glob_mismatch_failure_kind").and_then(|v| v.as_str()), Some("glob_match_set_mismatch"));
    assert_eq!(f.get("copy_mismatch_failure_kind").and_then(|v| v.as_str()), Some("shutil_copy_mismatch"));
}

// Acceptance: "Cleanup does not depend on user filesystem state."
#[test]
fn cleanup_does_not_depend_on_user_filesystem_state() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("cleanup_independence_contract").and_then(|v| v.as_table()).expect(
        "[cleanup_independence_contract] missing — acceptance: \
         \"Cleanup does not depend on user filesystem state.\"",
    );
    for k in &[
        "must_clean_up_inside_per_run_tempdir_only",
        "must_succeed_without_pre_existing_user_files",
        "must_succeed_without_pre_existing_user_directories",
        "must_remove_per_run_tempdir_on_completion",
        "forbid_calls_to_user_home_paths",
        "forbid_calls_to_current_working_directory_paths",
        "forbid_environment_dependent_paths",
    ] {
        assert_eq!(c.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let forbidden: Vec<&str> = c.get("forbidden_environment_variables").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["HOME", "USERPROFILE", "TMPDIR_FROM_USER_SHELL_PROFILE"] {
        assert!(forbidden.contains(v), "forbidden_environment_variables must include {v}");
    }
    let exit = c.get("cleanup_dependency_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 84);
    assert_eq!(c.get("cleanup_dependency_failure_kind").and_then(|v| v.as_str()), Some("cleanup_depended_on_user_filesystem"));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module_name",
        "relative_path", "source_relative_path", "destination_relative_path",
        "pattern", "expected_match_relative_paths", "actual_match_relative_paths",
        "tempdir_prefix", "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_uses_only_test_temp_directories",
        "fixture_fails_on_wrong_glob_or_copy_behavior",
        "cleanup_does_not_depend_on_user_filesystem_state",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("platform_specific_permission_behavior").and_then(|v| v.as_bool()), Some(true));
}
