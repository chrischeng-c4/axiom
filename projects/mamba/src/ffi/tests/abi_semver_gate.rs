//! Inline migration of tests/abi_semver_gate_fixture_2521.rs (#2521 / #2459).
//! Locks the shape of tests/mambalibs/fixtures/abi_semver_gate/manifest.toml.

#![cfg(test)]

use crate::testing::{b, get, i, load_manifest, s, strs};
use toml::Value;

const FIXTURE: &str = "tests/mambalibs/fixtures/abi_semver_gate/manifest.toml";
fn m() -> Value {
    load_manifest(FIXTURE)
}

#[test]
fn header_is_well_formed() {
    let m = m();
    assert_eq!(i(&m, "version"), 1);
    assert_eq!(s(&m, "fixture"), "abi_semver_gate");
    assert_eq!(i(&m, "issue"), 2521);
    assert_eq!(i(&m, "umbrella_issue"), 2459);
    assert_eq!(s(&m, "profile"), "mambalibs");
    assert_eq!(s(&m, "family"), "abi_semver_gate");
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
fn surface_pins_abi_version_exposure_and_pre_link_compare() {
    let m = m();
    let sf = get(&m, "surface");
    for key in [
        "must_cover_abi_version_constant_exposure",
        "must_cover_pre_link_abi_comparison",
        "must_cover_readable_mismatch_error",
        "must_cover_no_link_time_symbol_failure",
    ] {
        assert!(b(sf, key), "surface.{key}");
    }
}

#[test]
fn cclab_mamba_registry_exposes_abi_version_constants() {
    let m = m();
    let c = get(&m, "abi_version_exposure_contract");
    assert_eq!(
        s(c, "case"),
        "cclab_mamba_registry_exposes_abi_version_constants"
    );
    for key in [
        "must_expose_host_abi_version_constant",
        "must_expose_binding_abi_version_constant",
        "must_record_mb_value_abi_version",
        "must_record_mamba_module_abi_version",
        "forbid_implicit_abi_version_constants",
        "forbid_omitting_required_abi_version_constants",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(s(c, "abi_version_constants_field_name"), "abi_version_constants");
    assert_eq!(
        strs(c, "required_abi_version_constants"),
        vec![
            "host_abi_version",
            "binding_abi_version",
            "mb_value_abi_version",
            "mamba_module_abi_version",
        ]
    );
    assert_eq!(
        s(c, "missing_abi_version_constant_failure_kind"),
        "mambalibs_abi_version_constant_missing"
    );
    assert_eq!(i(c, "missing_abi_version_constant_exit_code"), 229);
}

#[test]
fn abi_mismatch_produces_readable_error_not_link_time_symbol_fail() {
    let m = m();
    let c = get(&m, "abi_mismatch_readable_error_contract");
    assert_eq!(
        s(c, "case"),
        "abi_mismatch_produces_readable_error_not_link_time_symbol_fail"
    );
    for key in [
        "must_fail_before_link_when_abi_mismatched",
        "must_emit_readable_abi_mismatch_error",
        "must_record_host_abi_version_in_error",
        "must_record_binding_abi_version_in_error",
        "must_record_binding_crate_identity_in_error",
        "forbid_link_time_symbol_failure_on_abi_mismatch",
        "forbid_silent_load_on_abi_mismatch",
        "forbid_freeform_or_implicit_abi_mismatch_error",
        "must_distinguish_pre_link_abi_mismatch_from_link_time_symbol_fail",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(
        strs(c, "required_error_fields"),
        vec![
            "host_abi_version",
            "binding_abi_version",
            "binding_crate_name",
            "binding_crate_version",
        ]
    );
    assert_eq!(s(c, "abi_mismatch_error_field_name"), "abi_mismatch_error");
    assert_eq!(s(c, "abi_mismatch_failure_kind"), "mambalibs_abi_mismatch_pre_link");
    assert_eq!(i(c, "abi_mismatch_exit_code"), 230);
    assert_eq!(
        s(c, "link_time_symbol_failure_kind"),
        "mambalibs_abi_mismatch_surfaced_as_link_time_symbol_fail"
    );
    assert_eq!(i(c, "link_time_symbol_failure_exit_code"), 231);
}

#[test]
fn mamba_build_compares_abi_version_before_link() {
    let m = m();
    let c = get(&m, "abi_comparison_phase_contract");
    assert_eq!(s(c, "case"), "mamba_build_compares_abi_version_before_link");
    for key in [
        "must_run_abi_comparison_during_build",
        "must_run_abi_comparison_before_link_phase",
        "forbid_running_abi_comparison_after_link",
        "forbid_skipping_abi_comparison_when_binding_present",
        "must_distinguish_phase_out_of_order_from_phase_skipped",
    ] {
        assert!(b(c, key), "{key}");
    }
    assert_eq!(s(c, "build_phase_field_name"), "build_phase");
    assert_eq!(
        strs(c, "required_build_phases_in_order"),
        vec!["resolve", "fetch", "abi_compare", "link", "emit"]
    );
    assert_eq!(s(c, "abi_compare_phase_name"), "abi_compare");
    assert_eq!(
        s(c, "phase_out_of_order_failure_kind"),
        "mambalibs_abi_compare_phase_out_of_order"
    );
    assert_eq!(i(c, "phase_out_of_order_exit_code"), 232);
    assert_eq!(s(c, "phase_skipped_failure_kind"), "mambalibs_abi_compare_phase_skipped");
    assert_eq!(i(c, "phase_skipped_exit_code"), 233);
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
            "abi_version_constants",
            "host_abi_version",
            "binding_abi_version",
            "mb_value_abi_version",
            "mamba_module_abi_version",
            "abi_mismatch_error",
            "binding_crate_name",
            "binding_crate_version",
            "build_phase",
            "failure_kind",
            "exit_code",
        ]
    );
    assert_eq!(strs(r, "outcome_values"), vec!["pass", "fail", "missing", "skip"]);
    assert_eq!(
        strs(r, "case_values"),
        vec![
            "cclab_mamba_registry_exposes_abi_version_constants",
            "abi_mismatch_produces_readable_error_not_link_time_symbol_fail",
            "mamba_build_compares_abi_version_before_link",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let m = m();
    let o = get(&m, "out_of_scope");
    for key in [
        "public_abi_surface_definition",
        "mb_value_struct_layout",
        "mamba_module_struct_layout",
        "runtime_implementation_of_abi_comparison",
        "runtime_implementation_of_link_phase",
    ] {
        assert!(b(o, key), "out_of_scope.{key}");
    }
}
