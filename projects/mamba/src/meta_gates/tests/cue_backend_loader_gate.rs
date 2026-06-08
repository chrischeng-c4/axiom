#![cfg(test)]

// Locks the shape of the Cue backend loader bug fixture
// pinned by
// tests/governance/gates/loader/cue_backend_loader_gate/manifest.toml.
// Closes #2939.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/loader/cue_backend_loader_gate/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(
        m["fixture"].as_str(),
        Some("cue_backend_loader_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(2939));
    assert_eq!(m["profile"].as_str(), Some("conformance"));
    assert_eq!(
        m["family"].as_str(),
        Some("cue_backend_loader_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let iso = &manifest()["isolation"];
    for key in [
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(iso[key].as_bool(), Some(true), "isolation.{key}");
    }
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let py = &manifest()["python_target"];
    assert_eq!(py["python_major"].as_integer(), Some(3));
    assert_eq!(py["python_minor"].as_integer(), Some(12));
    assert_eq!(py["must_be_python_3_12"].as_bool(), Some(true));
}

#[test]
fn surface_pins_all_five_requirements() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_mamba_run_loads_cue_backend_or_emits_narrow_diagnostic",
        "must_cover_narrow_diagnostic_carries_construct_and_location",
        "must_cover_api_py_generic_syntax_blocker_is_resolved_or_narrow_diagnostic",
        "must_cover_cue_backend_binds_canonical_local_dev_address",
        "must_cover_bridge_fallback_does_not_satisfy_mamba_product_target",
        "must_be_offline_or_loopback_only",
        "must_be_deterministic",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_mamba_run_loads_or_narrow_diagnostic() {
    let c = &manifest()["r1_mamba_run_loads_cue_backend_or_narrow_diagnostic_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("mamba_run_config_loads_cue_backend_or_emits_narrow_unsupported_feature_diagnostic")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_succeed_load_or_emit_narrow_diagnostic",
        "must_use_config_flag_for_mamba_toml",
        "forbid_silent_failure_without_diagnostic",
        "forbid_collapsing_diagnostic_into_generic_parse_error",
        "forbid_collapsing_diagnostic_into_generic_type_error",
        "must_distinguish_silent_from_broad_parse_from_broad_type",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_cue_backend_root_relative"].as_str(),
        Some("projects/cue/backend")
    );
    assert_eq!(
        c["expected_mamba_run_config_invocation"].as_str(),
        Some("mamba run --config projects/cue/backend/mamba.toml")
    );
    let outcomes: Vec<_> = c["allowed_load_outcome_values"]
        .as_array()
        .expect("allowed_load_outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        outcomes,
        vec![
            "loaded",
            "narrow_unsupported_feature",
            "broad_parse_error",
            "broad_type_error",
            "silent_failure",
        ]
    );
    assert_eq!(
        c["silent_failure_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_silent_failure")
    );
    assert_eq!(c["silent_failure_exit_code"].as_integer(), Some(406));
    assert_eq!(
        c["broad_parse_error_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_broad_parse_error")
    );
    assert_eq!(c["broad_parse_error_exit_code"].as_integer(), Some(407));
    assert_eq!(
        c["broad_type_error_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_broad_type_error")
    );
    assert_eq!(c["broad_type_error_exit_code"].as_integer(), Some(408));
}

#[test]
fn r2_narrow_diagnostic_schema() {
    let c = &manifest()["r2_narrow_diagnostic_schema_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("narrow_diagnostic_carries_kind_file_line_column_byte_span_and_construct_name")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_include_diagnostic_kind",
        "must_include_file_relative_to_cue_backend_root",
        "must_include_line_number",
        "must_include_column_number",
        "must_include_byte_span",
        "must_include_construct_name",
        "forbid_omitting_construct_name",
        "forbid_using_absolute_path_in_diagnostic",
        "must_distinguish_construct_missing_from_absolute_path",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let required: Vec<_> = c["required_diagnostic_fields"]
        .as_array()
        .expect("required_diagnostic_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        required,
        vec![
            "diagnostic_kind",
            "file_relative",
            "line",
            "column",
            "byte_span",
            "construct_name",
        ]
    );
    let kinds: Vec<_> = c["allowed_diagnostic_kind_values"]
        .as_array()
        .expect("allowed_diagnostic_kind_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        kinds,
        vec![
            "unsupported_syntax",
            "unsupported_type",
            "unsupported_runtime_feature",
        ]
    );
    assert_eq!(
        c["construct_name_missing_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_diagnostic_construct_name_missing")
    );
    assert_eq!(
        c["construct_name_missing_exit_code"].as_integer(),
        Some(409)
    );
    assert_eq!(
        c["absolute_path_in_diagnostic_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_diagnostic_absolute_path")
    );
    assert_eq!(
        c["absolute_path_in_diagnostic_exit_code"].as_integer(),
        Some(410)
    );
}

#[test]
fn r3_concrete_api_and_main_blockers() {
    let c = &manifest()["r3_concrete_api_and_main_blockers_resolved_or_narrow_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("api_py_generic_syntax_and_main_py_unknown_type_blockers_resolve_or_emit_narrow_diagnostic")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_resolve_api_py_generic_syntax_blocker_or_emit_narrow",
        "must_resolve_main_py_unknown_type_blocker_or_emit_narrow",
        "forbid_silently_skipping_api_py_blocker",
        "forbid_silently_skipping_main_py_blocker",
        "must_distinguish_api_py_skip_from_main_py_skip",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_api_py_blocker_relative_path"].as_str(),
        Some("src/mambalibs/api.py")
    );
    assert_eq!(
        c["expected_api_py_blocker_byte_span"].as_str(),
        Some("1107..1108")
    );
    assert_eq!(
        c["expected_api_py_blocker_construct"].as_str(),
        Some("subscript_after_type_annotation")
    );
    assert_eq!(
        c["expected_main_py_blocker_relative_path"].as_str(),
        Some("src/main.py")
    );
    assert_eq!(
        c["expected_main_py_blocker_byte_span"].as_str(),
        Some("2096..2111")
    );
    assert_eq!(
        c["expected_main_py_blocker_construct"].as_str(),
        Some("unknown_type_WorkstreamError")
    );
    assert_eq!(
        c["expected_main_py_blocker_line"].as_integer(),
        Some(81)
    );
    assert_eq!(
        c["expected_main_py_blocker_column"].as_integer(),
        Some(19)
    );
    assert_eq!(
        c["api_py_blocker_silently_skipped_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_api_py_blocker_silently_skipped")
    );
    assert_eq!(
        c["api_py_blocker_silently_skipped_exit_code"].as_integer(),
        Some(411)
    );
    assert_eq!(
        c["main_py_blocker_silently_skipped_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_main_py_blocker_silently_skipped")
    );
    assert_eq!(
        c["main_py_blocker_silently_skipped_exit_code"].as_integer(),
        Some(412)
    );
}

#[test]
fn r4_canonical_local_dev_address_binding() {
    let c = &manifest()["r4_canonical_local_dev_address_binding_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("cue_backend_dev_entrypoint_binds_127_0_0_1_43219")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_bind_loopback_address",
        "must_bind_canonical_dev_port",
        "forbid_binding_non_loopback_address_in_dev",
        "forbid_binding_alternate_port_silently",
        "must_distinguish_non_loopback_from_alternate_port",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_canonical_dev_address"].as_str(),
        Some("127.0.0.1:43219")
    );
    assert_eq!(
        c["expected_canonical_dev_host"].as_str(),
        Some("127.0.0.1")
    );
    assert_eq!(
        c["expected_canonical_dev_port"].as_integer(),
        Some(43219)
    );
    assert_eq!(
        c["non_loopback_bind_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_non_loopback_bind_in_dev")
    );
    assert_eq!(c["non_loopback_bind_exit_code"].as_integer(), Some(413));
    assert_eq!(
        c["alternate_port_silently_bound_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_alternate_port_silently_bound")
    );
    assert_eq!(
        c["alternate_port_silently_bound_exit_code"].as_integer(),
        Some(414)
    );
}

#[test]
fn r5_bridge_fallback_does_not_satisfy_product_target() {
    let c = &manifest()["r5_bridge_fallback_does_not_satisfy_product_target_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("cue_backend_mode_bridge_cpython_fallback_does_not_satisfy_mamba_product_target_gate")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_forbid_treating_bridge_mode_as_mamba_green",
        "must_record_bridge_mode_consultation_distinct_from_load",
        "forbid_silently_falling_back_to_bridge_in_product_target",
        "forbid_silently_swallowing_mamba_loader_error_via_bridge",
        "must_distinguish_bridge_green_from_silent_bridge_fallback",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["expected_bridge_mode_env_var"].as_str(),
        Some("CUE_BACKEND_MODE")
    );
    assert_eq!(
        c["expected_bridge_mode_env_value"].as_str(),
        Some("bridge")
    );
    let targets: Vec<_> = c["allowed_product_target_values"]
        .as_array()
        .expect("allowed_product_target_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(targets, vec!["mamba", "bridge"]);
    assert_eq!(c["expected_product_target"].as_str(), Some("mamba"));
    assert_eq!(
        c["bridge_marked_as_green_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_bridge_marked_as_green")
    );
    assert_eq!(
        c["bridge_marked_as_green_exit_code"].as_integer(),
        Some(415)
    );
    assert_eq!(
        c["silent_bridge_fallback_failure_kind"].as_str(),
        Some("mvp_cue_backend_loader_silent_bridge_fallback")
    );
    assert_eq!(
        c["silent_bridge_fallback_exit_code"].as_integer(),
        Some(416)
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let r = &manifest()["runner_contract"];
    let keys: Vec<_> = r["keys"]
        .as_array()
        .expect("keys")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        keys,
        vec![
            "outcome",
            "case",
            "requirement_id",
            "cue_backend_root_relative",
            "mamba_run_config_invocation",
            "load_outcome",
            "required_diagnostic_fields",
            "api_py_blocker_relative_path",
            "api_py_blocker_byte_span",
            "api_py_blocker_construct",
            "main_py_blocker_relative_path",
            "main_py_blocker_byte_span",
            "main_py_blocker_construct",
            "main_py_blocker_line",
            "main_py_blocker_column",
            "canonical_dev_address",
            "canonical_dev_host",
            "canonical_dev_port",
            "bridge_mode_env_var",
            "bridge_mode_env_value",
            "product_target",
            "failure_kind",
            "exit_code",
        ]
    );
    let outcomes: Vec<_> = r["outcome_values"]
        .as_array()
        .expect("outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(outcomes, vec!["pass", "fail", "missing", "skip"]);
    let cases: Vec<_> = r["case_values"]
        .as_array()
        .expect("case_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        cases,
        vec![
            "mamba_run_config_loads_cue_backend_or_emits_narrow_unsupported_feature_diagnostic",
            "narrow_diagnostic_carries_kind_file_line_column_byte_span_and_construct_name",
            "api_py_generic_syntax_and_main_py_unknown_type_blockers_resolve_or_emit_narrow_diagnostic",
            "cue_backend_dev_entrypoint_binds_127_0_0_1_43219",
            "cue_backend_mode_bridge_cpython_fallback_does_not_satisfy_mamba_product_target_gate",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "implementation_of_missing_parser_or_type_support",
        "cue_product_ux",
        "local_dev_tui",
        "cpython_bridge_implementation",
        "c_extension_fast_paths",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
