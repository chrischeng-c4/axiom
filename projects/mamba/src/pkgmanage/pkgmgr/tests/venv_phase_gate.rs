//! Inline migration of tests/pkg_mgr_phase_1_5_venv_gate_fixture_1262.rs (#1262).

#![cfg(test)]

use crate::testing::{b, get, i, load_manifest, s, strs};
use toml::Value;

const FIXTURE: &str =
    "tests/governance/gates/package_manager/pkg_mgr_phase_1_5_venv_gate/manifest.toml";
fn m() -> Value {
    load_manifest(FIXTURE)
}

#[test]
fn header_is_well_formed() {
    let m = m();
    assert_eq!(i(&m, "version"), 1);
    assert_eq!(s(&m, "fixture"), "pkg_mgr_phase_1_5_venv_gate");
    assert_eq!(i(&m, "issue"), 1262);
    assert_eq!(i(&m, "parent_issue"), 751);
    assert_eq!(s(&m, "profile"), "package_manager");
    assert_eq!(s(&m, "family"), "pkg_mgr_phase_1_5_venv_gate");
    assert_eq!(s(&m, "network"), "offline");
}

#[test]
fn isolation_pins_no_global_state() {
    let m = m();
    let iso = get(&m, "isolation");
    for key in [
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert!(b(iso, key), "isolation.{key}");
    }
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let m = m();
    let py = get(&m, "python_target");
    assert_eq!(i(py, "python_major"), 3);
    assert_eq!(i(py, "python_minor"), 12);
    assert!(b(py, "must_be_python_3_12"));
}

#[test]
fn surface_pins_all_eight_requirements() {
    let m = m();
    let sf = get(&m, "surface");
    for key in [
        "must_cover_pyvenv_cfg_creation",
        "must_cover_pep_405_directory_layout",
        "must_cover_mamba_venv_create_cli_verb",
        "must_cover_sys_prefix_override_in_mamba_run",
        "must_cover_site_packages_discovery_without_activation",
        "must_cover_mamba_venv_remove_cli_verb",
        "must_cover_interpreter_symlink_or_copy",
        "must_cover_cross_platform_layout_variant",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert!(b(sf, key), "surface.{key}");
    }
}

#[test]
fn r1_pyvenv_cfg_creation() {
    let m = m();
    let c = get(&m, "r1_pyvenv_cfg_creation_contract");
    assert_eq!(
        s(c, "case"),
        "pyvenv_cfg_is_written_with_home_include_system_site_packages_and_version"
    );
    assert_eq!(s(c, "requirement_id"), "R1");
    assert_eq!(s(c, "priority"), "P1");
    for key in [
        "must_write_pyvenv_cfg_in_venv_root",
        "must_write_home_key",
        "must_write_include_system_site_packages_false",
        "must_write_version_key",
        "must_use_key_equals_value_format",
        "forbid_omitting_required_pyvenv_cfg_keys",
        "must_distinguish_cfg_missing_from_required_key_missing",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(s(c, "expected_pyvenv_cfg_filename"), "pyvenv.cfg");
    assert_eq!(
        strs(c, "required_pyvenv_cfg_keys"),
        vec!["home", "include-system-site-packages", "version"]
    );
    assert_eq!(
        s(c, "pyvenv_cfg_missing_failure_kind"),
        "mvp_package_manager_pyvenv_cfg_missing"
    );
    assert_eq!(i(c, "pyvenv_cfg_missing_exit_code"), 390);
    assert_eq!(
        s(c, "pyvenv_cfg_key_missing_failure_kind"),
        "mvp_package_manager_pyvenv_cfg_required_key_missing"
    );
    assert_eq!(i(c, "pyvenv_cfg_key_missing_exit_code"), 391);
}

#[test]
fn r2_pep_405_directory_layout() {
    let m = m();
    let c = get(&m, "r2_pep_405_directory_layout_contract");
    assert_eq!(
        s(c, "case"),
        "pep_405_directory_tree_bin_lib_site_packages_include_is_created_atomically"
    );
    assert_eq!(s(c, "requirement_id"), "R2");
    assert_eq!(s(c, "priority"), "P1");
    for key in [
        "must_create_bin_dir",
        "must_create_lib_pythonxy_site_packages_dir",
        "must_create_include_dir",
        "must_match_pythonxy_major_minor",
        "must_create_directory_tree_atomically",
        "forbid_leaving_partial_layout_on_error",
        "must_distinguish_partial_layout_from_subdir_missing",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(
        strs(c, "required_subdirs_posix"),
        vec!["bin", "lib/python3.12/site-packages", "include"]
    );
    assert_eq!(
        s(c, "partial_layout_failure_kind"),
        "mvp_package_manager_partial_venv_layout_on_error"
    );
    assert_eq!(i(c, "partial_layout_exit_code"), 392);
    assert_eq!(
        s(c, "required_subdir_missing_failure_kind"),
        "mvp_package_manager_venv_required_subdir_missing"
    );
    assert_eq!(i(c, "required_subdir_missing_exit_code"), 393);
}

#[test]
fn r3_mamba_venv_create_cli() {
    let m = m();
    let c = get(&m, "r3_mamba_venv_create_cli_contract");
    assert_eq!(
        s(c, "case"),
        "mamba_venv_create_cli_verb_creates_venv_refuses_overwrite_supports_python_flag"
    );
    assert_eq!(s(c, "requirement_id"), "R3");
    assert_eq!(s(c, "priority"), "P1");
    for key in [
        "must_register_mamba_venv_create_verb",
        "must_accept_path_positional_argument",
        "must_accept_python_optional_flag",
        "must_refuse_to_overwrite_existing_pyvenv_cfg",
        "forbid_silently_overwriting_existing_venv",
        "must_distinguish_silent_overwrite_from_verb_missing",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(s(c, "expected_verb_name"), "venv create");
    assert_eq!(s(c, "expected_python_flag"), "--python");
    assert_eq!(
        strs(c, "allowed_overwrite_outcome_values"),
        vec!["refused_existing_pyvenv_cfg", "created_new"]
    );
    assert_eq!(
        s(c, "silent_overwrite_failure_kind"),
        "mvp_package_manager_silent_venv_overwrite"
    );
    assert_eq!(i(c, "silent_overwrite_exit_code"), 394);
    assert_eq!(
        s(c, "venv_create_verb_missing_failure_kind"),
        "mvp_package_manager_venv_create_verb_missing"
    );
    assert_eq!(i(c, "venv_create_verb_missing_exit_code"), 395);
}

#[test]
fn r4_sys_prefix_override_in_mamba_run() {
    let m = m();
    let c = get(&m, "r4_sys_prefix_override_in_mamba_run_contract");
    assert_eq!(
        s(c, "case"),
        "mamba_run_overrides_sys_prefix_and_exec_prefix_to_venv_root"
    );
    assert_eq!(s(c, "requirement_id"), "R4");
    assert_eq!(s(c, "priority"), "P1");
    for key in [
        "must_set_sys_prefix_to_venv_root",
        "must_set_sys_exec_prefix_to_venv_root",
        "must_apply_override_inside_mamba_run",
        "forbid_relying_on_environment_variable_activation",
        "forbid_leaking_system_python_prefix_when_venv_active",
        "must_distinguish_prefix_leak_from_env_var_required",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(
        strs(c, "allowed_sys_prefix_target_values"),
        vec!["venv_root", "system_python_root"]
    );
    assert_eq!(
        s(c, "sys_prefix_leak_failure_kind"),
        "mvp_package_manager_sys_prefix_leak_system_python"
    );
    assert_eq!(i(c, "sys_prefix_leak_exit_code"), 396);
    assert_eq!(
        s(c, "env_var_activation_required_failure_kind"),
        "mvp_package_manager_env_var_activation_required"
    );
    assert_eq!(i(c, "env_var_activation_required_exit_code"), 397);
}

#[test]
fn r5_site_packages_discovery_without_activation() {
    let m = m();
    let c = get(&m, "r5_site_packages_discovery_without_activation_contract");
    assert_eq!(
        s(c, "case"),
        "venv_site_packages_is_importable_by_mamba_run_without_activation_step"
    );
    assert_eq!(s(c, "requirement_id"), "R5");
    assert_eq!(s(c, "priority"), "P1");
    for key in [
        "must_prepend_venv_site_packages_to_sys_path",
        "must_resolve_imports_from_venv_site_packages",
        "must_apply_before_user_code_executes",
        "forbid_requiring_shell_activation_for_import",
        "forbid_resolving_imports_from_system_python_first",
        "must_distinguish_shell_required_from_system_shadow",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(s(c, "expected_sys_path_first_entry"), "venv_site_packages");
    assert_eq!(
        strs(c, "allowed_import_resolution_source_values"),
        vec!["venv_site_packages", "system_site_packages"]
    );
    assert_eq!(
        s(c, "shell_activation_required_failure_kind"),
        "mvp_package_manager_venv_shell_activation_required"
    );
    assert_eq!(i(c, "shell_activation_required_exit_code"), 398);
    assert_eq!(
        s(c, "system_site_packages_shadow_failure_kind"),
        "mvp_package_manager_system_site_packages_shadowed_venv"
    );
    assert_eq!(i(c, "system_site_packages_shadow_exit_code"), 399);
}

#[test]
fn r6_mamba_venv_remove_cli() {
    let m = m();
    let c = get(&m, "r6_mamba_venv_remove_cli_contract");
    assert_eq!(
        s(c, "case"),
        "mamba_venv_remove_cleans_tree_only_when_pyvenv_cfg_present"
    );
    assert_eq!(s(c, "requirement_id"), "R6");
    assert_eq!(s(c, "priority"), "P2");
    for key in [
        "must_register_mamba_venv_remove_verb",
        "must_require_pyvenv_cfg_for_removal",
        "must_refuse_to_delete_directory_without_pyvenv_cfg",
        "must_print_removed_top_level_entries",
        "forbid_silently_deleting_non_venv_directory",
        "must_distinguish_silent_delete_from_verb_missing",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(s(c, "expected_remove_verb_name"), "venv remove");
    assert_eq!(
        strs(c, "allowed_removal_outcome_values"),
        vec!["removed", "refused_no_pyvenv_cfg"]
    );
    assert_eq!(
        s(c, "silent_non_venv_delete_failure_kind"),
        "mvp_package_manager_silent_delete_non_venv_directory"
    );
    assert_eq!(i(c, "silent_non_venv_delete_exit_code"), 400);
    assert_eq!(
        s(c, "remove_verb_missing_failure_kind"),
        "mvp_package_manager_venv_remove_verb_missing"
    );
    assert_eq!(i(c, "remove_verb_missing_exit_code"), 401);
}

#[test]
fn r7_interpreter_symlink_or_copy() {
    let m = m();
    let c = get(&m, "r7_interpreter_symlink_or_copy_contract");
    assert_eq!(
        s(c, "case"),
        "bin_python_is_symlink_by_default_or_copy_on_flag_or_no_symlink_fs"
    );
    assert_eq!(s(c, "requirement_id"), "R7");
    assert_eq!(s(c, "priority"), "P2");
    for key in [
        "must_default_to_symlink_for_bin_python",
        "must_support_copies_flag",
        "must_fall_back_to_copy_on_no_symlink_fs",
        "forbid_silently_dropping_bin_python",
        "must_distinguish_bin_python_missing_from_copies_flag_missing",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(s(c, "expected_copies_flag"), "--copies");
    assert_eq!(
        strs(c, "allowed_interpreter_link_kind_values"),
        vec!["symlink", "copy"]
    );
    assert_eq!(
        s(c, "bin_python_missing_failure_kind"),
        "mvp_package_manager_bin_python_missing"
    );
    assert_eq!(i(c, "bin_python_missing_exit_code"), 402);
    assert_eq!(
        s(c, "copies_flag_missing_failure_kind"),
        "mvp_package_manager_copies_flag_missing"
    );
    assert_eq!(i(c, "copies_flag_missing_exit_code"), 403);
}

#[test]
fn r8_cross_platform_layout_variant() {
    let m = m();
    let c = get(&m, "r8_cross_platform_layout_variant_contract");
    assert_eq!(
        s(c, "case"),
        "windows_layout_uses_scripts_and_lib_site_packages_unix_layout_uses_bin_and_lib_pythonxy_site_packages"
    );
    assert_eq!(s(c, "requirement_id"), "R8");
    assert_eq!(s(c, "priority"), "P3");
    for key in [
        "must_select_layout_by_platform",
        "must_use_bin_on_posix",
        "must_use_scripts_on_windows",
        "must_use_lib_pythonxy_site_packages_on_posix",
        "must_use_lib_site_packages_on_windows",
        "forbid_silently_using_posix_layout_on_windows",
        "must_distinguish_wrong_layout_from_selection_skipped",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(strs(c, "allowed_platform_values"), vec!["posix", "windows"]);
    assert_eq!(s(c, "expected_posix_bin_dir"), "bin");
    assert_eq!(s(c, "expected_windows_bin_dir"), "Scripts");
    assert_eq!(
        s(c, "expected_posix_site_packages_dir"),
        "lib/python3.12/site-packages"
    );
    assert_eq!(
        s(c, "expected_windows_site_packages_dir"),
        "Lib/site-packages"
    );
    assert_eq!(
        s(c, "wrong_platform_layout_failure_kind"),
        "mvp_package_manager_wrong_platform_layout"
    );
    assert_eq!(i(c, "wrong_platform_layout_exit_code"), 404);
    assert_eq!(
        s(c, "platform_selection_skipped_failure_kind"),
        "mvp_package_manager_platform_selection_skipped"
    );
    assert_eq!(i(c, "platform_selection_skipped_exit_code"), 405);
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let m = m();
    let r = get(&m, "runner_contract");
    assert_eq!(
        strs(r, "keys"),
        vec![
            "outcome",
            "case",
            "requirement_id",
            "priority",
            "pyvenv_cfg_filename",
            "required_pyvenv_cfg_keys",
            "required_subdirs_posix",
            "atomic_creation",
            "verb_name",
            "python_flag",
            "overwrite_outcome",
            "sys_prefix_target",
            "sys_path_first_entry",
            "import_resolution_source",
            "remove_verb_name",
            "removal_outcome",
            "copies_flag",
            "interpreter_link_kind",
            "platform",
            "posix_bin_dir",
            "windows_bin_dir",
            "posix_site_packages_dir",
            "windows_site_packages_dir",
            "failure_kind",
            "exit_code",
        ]
    );
    assert_eq!(
        strs(r, "outcome_values"),
        vec!["pass", "fail", "missing", "skip"]
    );
    assert_eq!(
        strs(r, "case_values"),
        vec![
            "pyvenv_cfg_is_written_with_home_include_system_site_packages_and_version",
            "pep_405_directory_tree_bin_lib_site_packages_include_is_created_atomically",
            "mamba_venv_create_cli_verb_creates_venv_refuses_overwrite_supports_python_flag",
            "mamba_run_overrides_sys_prefix_and_exec_prefix_to_venv_root",
            "venv_site_packages_is_importable_by_mamba_run_without_activation_step",
            "mamba_venv_remove_cleans_tree_only_when_pyvenv_cfg_present",
            "bin_python_is_symlink_by_default_or_copy_on_flag_or_no_symlink_fs",
            "windows_layout_uses_scripts_and_lib_site_packages_unix_layout_uses_bin_and_lib_pythonxy_site_packages",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let m = m();
    let o = get(&m, "out_of_scope");
    for key in [
        "activation_shell_scripts_for_other_shells",
        "pip_compatibility_shims",
        "conda_environment_compatibility",
        "virtual_environment_cloning_between_machines",
        "python_interpreter_compilation_or_download",
        "windows_full_ci_validation",
        "c_extension_fast_paths",
    ] {
        assert!(b(o, key), "out_of_scope.{key}");
    }
}
