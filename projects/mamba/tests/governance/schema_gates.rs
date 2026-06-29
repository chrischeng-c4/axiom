//! Consolidated umbrella binary for 88 TOML schema-gate fixtures
//! (Phase 4.5 of the tests/ DDD refactor). Each gate parses a manifest
//! under `tests/governance/gates/<scope>/<gate>/manifest.toml` and asserts the
//! schema/field contract for one cross-cutting surface — MVP test
//! profile manifests, pkgmgr lockfile/index/install gates, runtime
//! perf shape gates, third-party seed gates, CPython compat seeds, etc.
//!
//! These gates do not spawn the mamba binary (no `CARGO_BIN_EXE_mamba`)
//! and do not exercise crate-internal API (no `use mamba::`), so they
//! are neither Python-perspective integration tests nor inline unit
//! tests. They are *meta* gates over the test profile and on-disk
//! manifest contracts. They have no `src/<domain>/` owner because they
//! describe the test surface itself, not crate code, so they collapse
//! into one umbrella binary instead of moving inline.
//!
//! Each former top-level `tests/<gate>.rs` now lives in
//! `tests/schema_gates/` and is registered below with `#[path]`. cargo
//! only compiles `tests/*.rs` as integration binaries, so the files
//! under `tests/schema_gates/` are no longer separate binaries — this
//! umbrella is the sole binary that links them all.
//!
//! Total: 91 gate modules / 834 `#[test]` functions in one binary.
//!
//! Selector: `cargo test -p mamba --test schema_gates`.

#[path = "common.rs"]
mod common;

#[path = "schema_gates/block_ignored_in_mvp_profile_fixture_2599.rs"]
mod block_ignored_in_mvp_profile_fixture_2599;

#[path = "schema_gates/console_script_entrypoint_fixture_2593.rs"]
mod console_script_entrypoint_fixture_2593;

#[path = "schema_gates/cpython_lib_test_minimal_unittest_dispatch_fixture_2545.rs"]
mod cpython_lib_test_minimal_unittest_dispatch_fixture_2545;

#[path = "schema_gates/ecosystem_fixture_manifest_smoke.rs"]
mod ecosystem_fixture_manifest_smoke;

#[path = "schema_gates/fail_new_ignore_without_work_item_fixture_2603.rs"]
mod fail_new_ignore_without_work_item_fixture_2603;

#[path = "schema_gates/fetch_and_workspace_synth_fixture_2520.rs"]
mod fetch_and_workspace_synth_fixture_2520;

#[path = "schema_gates/frozen_local_simple_index_fixture_2585.rs"]
mod frozen_local_simple_index_fixture_2585;

#[path = "schema_gates/ignore_xfail_skip_inventory_command_fixture_2602.rs"]
mod ignore_xfail_skip_inventory_command_fixture_2602;

#[path = "schema_gates/ignored_test_categorization_fixture_2598.rs"]
mod ignored_test_categorization_fixture_2598;

#[path = "schema_gates/ignored_xfail_skip_debt_manifest_fixture_2533.rs"]
mod ignored_xfail_skip_debt_manifest_fixture_2533;

#[path = "schema_gates/init_add_install_sync_run_e2e_fixture_2588.rs"]
mod init_add_install_sync_run_e2e_fixture_2588;

#[path = "schema_gates/live_pypi_opt_in_fixture_2590.rs"]
mod live_pypi_opt_in_fixture_2590;

#[path = "schema_gates/lockfile_determinism_fixture_2586.rs"]
mod lockfile_determinism_fixture_2586;

#[path = "schema_gates/mode2_lockfile_integration_fixture_2522.rs"]
mod mode2_lockfile_integration_fixture_2522;

#[path = "schema_gates/mvp_ecosystem_profile_manifest_2814.rs"]
mod mvp_ecosystem_profile_manifest_2814;

#[path = "schema_gates/mvp_mambalibs_profile_manifest_2817.rs"]
mod mvp_mambalibs_profile_manifest_2817;

#[path = "schema_gates/mvp_mambalibs_umbrella_gate_fixture_2459.rs"]
mod mvp_mambalibs_umbrella_gate_fixture_2459;

#[path = "schema_gates/mvp_package_manager_profile_manifest_2816.rs"]
mod mvp_package_manager_profile_manifest_2816;

#[path = "schema_gates/mvp_performance_profile_manifest_2815.rs"]
mod mvp_performance_profile_manifest_2815;

#[path = "schema_gates/mvp_release_blocking_profiles_2775.rs"]
mod mvp_release_blocking_profiles_2775;

#[path = "schema_gates/mvp_release_summary_schema_2820.rs"]
mod mvp_release_summary_schema_2820;

#[path = "schema_gates/mvp_required_correctness_profile_manifest_2818.rs"]
mod mvp_required_correctness_profile_manifest_2818;

#[path = "schema_gates/mvp_smoke_gate_compile_and_list_fixture_2527.rs"]
mod mvp_smoke_gate_compile_and_list_fixture_2527;

#[path = "schema_gates/mvp_smoke_profile_manifest_2819.rs"]
mod mvp_smoke_profile_manifest_2819;

#[path = "schema_gates/no_xfail_no_skip_promotion_gate_703.rs"]
mod no_xfail_no_skip_promotion_gate_703;

#[path = "schema_gates/strict_type_accounting_gate_704.rs"]
mod strict_type_accounting_gate_704;

#[path = "schema_gates/platform_readiness_gate_710.rs"]
mod platform_readiness_gate_710;

#[path = "schema_gates/import_readiness_gate_708.rs"]
mod import_readiness_gate_708;

#[path = "schema_gates/third_party_readiness_gate_711.rs"]
mod third_party_readiness_gate_711;

#[path = "schema_gates/perf_allocation_pressure_fixture_2661.rs"]
mod perf_allocation_pressure_fixture_2661;

#[path = "schema_gates/perf_async_scheduling_fixture_2664.rs"]
mod perf_async_scheduling_fixture_2664;

#[path = "schema_gates/perf_class_attr_lookup_fixture_2655.rs"]
mod perf_class_attr_lookup_fixture_2655;

#[path = "schema_gates/perf_dict_insert_lookup_fixture_2654.rs"]
mod perf_dict_insert_lookup_fixture_2654;

#[path = "schema_gates/perf_exception_throw_catch_fixture_2658.rs"]
mod perf_exception_throw_catch_fixture_2658;

#[path = "schema_gates/perf_function_call_overhead_fixture_2656.rs"]
mod perf_function_call_overhead_fixture_2656;

#[path = "schema_gates/perf_generator_iteration_fixture_2662.rs"]
mod perf_generator_iteration_fixture_2662;

#[path = "schema_gates/perf_import_cold_warm_fixture_2660.rs"]
mod perf_import_cold_warm_fixture_2660;

#[path = "schema_gates/perf_json_parse_serialize_fixture_2665.rs"]
mod perf_json_parse_serialize_fixture_2665;

#[path = "schema_gates/perf_list_append_index_fixture_2659.rs"]
mod perf_list_append_index_fixture_2659;

#[path = "schema_gates/perf_regex_findall_fixture_2663.rs"]
mod perf_regex_findall_fixture_2663;

#[path = "schema_gates/perf_string_split_join_fixture_2657.rs"]
mod perf_string_split_join_fixture_2657;

#[path = "schema_gates/pkgmgr_add_fixture_2681.rs"]
mod pkgmgr_add_fixture_2681;

#[path = "schema_gates/pkgmgr_cache_isolation_fixture_2685.rs"]
mod pkgmgr_cache_isolation_fixture_2685;

#[path = "schema_gates/pkgmgr_dependency_group_fixture_2856.rs"]
mod pkgmgr_dependency_group_fixture_2856;

#[path = "schema_gates/pkgmgr_dev_dependency_fixture_2852.rs"]
mod pkgmgr_dev_dependency_fixture_2852;

#[path = "schema_gates/pkgmgr_direct_local_wheel_fixture_2689.rs"]
mod pkgmgr_direct_local_wheel_fixture_2689;

#[path = "schema_gates/pkgmgr_downgrade_fixture_2855.rs"]
mod pkgmgr_downgrade_fixture_2855;

#[path = "schema_gates/pkgmgr_editable_local_project_fixture_2853.rs"]
mod pkgmgr_editable_local_project_fixture_2853;

#[path = "schema_gates/pkgmgr_env_marker_fixture_2688.rs"]
mod pkgmgr_env_marker_fixture_2688;

#[path = "schema_gates/pkgmgr_extras_resolution_fixture_2690.rs"]
mod pkgmgr_extras_resolution_fixture_2690;

#[path = "schema_gates/pkgmgr_hash_verification_fixture_2686.rs"]
mod pkgmgr_hash_verification_fixture_2686;

#[path = "schema_gates/pkgmgr_init_fixture_2679.rs"]
mod pkgmgr_init_fixture_2679;

#[path = "schema_gates/pkgmgr_index_fixture_507.rs"]
mod pkgmgr_index_fixture_507;

#[path = "schema_gates/pkgmgr_json_summary_fixture_2687.rs"]
mod pkgmgr_json_summary_fixture_2687;

#[path = "schema_gates/pkgmgr_lock_fixture_2682.rs"]
mod pkgmgr_lock_fixture_2682;

#[path = "schema_gates/pkgmgr_remove_fixture_2680.rs"]
mod pkgmgr_remove_fixture_2680;

#[path = "schema_gates/pkgmgr_run_fixture_2684.rs"]
mod pkgmgr_run_fixture_2684;

#[path = "schema_gates/pkgmgr_sync_idempotence_fixture_2683.rs"]
mod pkgmgr_sync_idempotence_fixture_2683;

#[path = "schema_gates/pkgmgr_upgrade_fixture_2857.rs"]
mod pkgmgr_upgrade_fixture_2857;

#[path = "schema_gates/pkgmgr_workspace_member_fixture_2854.rs"]
mod pkgmgr_workspace_member_fixture_2854;

#[path = "schema_gates/pure_python_wheel_install_fixture_2591.rs"]
mod pure_python_wheel_install_fixture_2591;

#[path = "schema_gates/python_language_semantics_acceptance_suite_fixture_2774.rs"]
mod python_language_semantics_acceptance_suite_fixture_2774;

#[path = "schema_gates/require_issue_refs_in_ignored_fixture_2601.rs"]
mod require_issue_refs_in_ignored_fixture_2601;

#[path = "schema_gates/resolver_conflict_fixture_2587.rs"]
mod resolver_conflict_fixture_2587;

#[path = "schema_gates/resolver_yanked_version_fixture_2584.rs"]
mod resolver_yanked_version_fixture_2584;

#[path = "schema_gates/skip_debt_counts_smoke_summary_fixture_2600.rs"]
mod skip_debt_counts_smoke_summary_fixture_2600;

#[path = "schema_gates/stdlib_argparse_fixture_2639.rs"]
mod stdlib_argparse_fixture_2639;

#[path = "schema_gates/stdlib_collections_itertools_functools_fixture_2632.rs"]
mod stdlib_collections_itertools_functools_fixture_2632;

#[path = "schema_gates/stdlib_csv_pickle_tomllib_fixture_2635.rs"]
mod stdlib_csv_pickle_tomllib_fixture_2635;

#[path = "schema_gates/stdlib_dataclasses_typing_fixture_2633.rs"]
mod stdlib_dataclasses_typing_fixture_2633;

#[path = "schema_gates/stdlib_datetime_behavioral_fixture_2626.rs"]
mod stdlib_datetime_behavioral_fixture_2626;

#[path = "schema_gates/stdlib_hashlib_hmac_secrets_fixture_2638.rs"]
mod stdlib_hashlib_hmac_secrets_fixture_2638;

#[path = "schema_gates/stdlib_import_smoke_matrix_fixture_2627.rs"]
mod stdlib_import_smoke_matrix_fixture_2627;

#[path = "schema_gates/stdlib_importlib_metadata_fixture_2634.rs"]
mod stdlib_importlib_metadata_fixture_2634;

#[path = "schema_gates/stdlib_inspect_traceback_warnings_fixture_2640.rs"]
mod stdlib_inspect_traceback_warnings_fixture_2640;

#[path = "schema_gates/stdlib_json_behavioral_fixture_2628.rs"]
mod stdlib_json_behavioral_fixture_2628;

#[path = "schema_gates/stdlib_logging_behavioral_fixture_2631.rs"]
mod stdlib_logging_behavioral_fixture_2631;

#[path = "schema_gates/stdlib_pathlib_behavioral_fixture_2629.rs"]
mod stdlib_pathlib_behavioral_fixture_2629;

#[path = "schema_gates/stdlib_required_module_manifest_fixture_2624.rs"]
mod stdlib_required_module_manifest_fixture_2624;

#[path = "schema_gates/stdlib_socket_loopback_fixture_2636.rs"]
mod stdlib_socket_loopback_fixture_2636;

#[path = "schema_gates/stdlib_subprocess_smoke_fixture_2641.rs"]
mod stdlib_subprocess_smoke_fixture_2641;

#[path = "schema_gates/stdlib_tempfile_shutil_glob_fixture_2630.rs"]
mod stdlib_tempfile_shutil_glob_fixture_2630;

#[path = "schema_gates/stdlib_urllib_parse_email_fixture_2637.rs"]
mod stdlib_urllib_parse_email_fixture_2637;

#[path = "schema_gates/third_party_attrs_class_fixture_2642.rs"]
mod third_party_attrs_class_fixture_2642;

#[path = "schema_gates/third_party_fastapi_route_fixture_2650.rs"]
mod third_party_fastapi_route_fixture_2650;

#[path = "schema_gates/third_party_httpx_mock_transport_fixture_2645.rs"]
mod third_party_httpx_mock_transport_fixture_2645;

#[path = "schema_gates/third_party_pluggy_hook_fixture_2647.rs"]
mod third_party_pluggy_hook_fixture_2647;

#[path = "schema_gates/third_party_pyyaml_load_dump_fixture_2646.rs"]
mod third_party_pyyaml_load_dump_fixture_2646;

#[path = "schema_gates/third_party_sqlalchemy_expression_fixture_2644.rs"]
mod third_party_sqlalchemy_expression_fixture_2644;

#[path = "schema_gates/third_party_starlette_asgi_fixture_2648.rs"]
mod third_party_starlette_asgi_fixture_2648;

#[path = "schema_gates/third_party_urllib3_response_fixture_2643.rs"]
mod third_party_urllib3_response_fixture_2643;

#[path = "schema_gates/transitive_dependency_resolution_fixture_2592.rs"]
mod transitive_dependency_resolution_fixture_2592;

#[path = "schema_gates/uv_like_pkgmgr_offline_e2e_fixture_2532.rs"]
mod uv_like_pkgmgr_offline_e2e_fixture_2532;

#[path = "schema_gates/venv_site_packages_activation_fixture_2589.rs"]
mod venv_site_packages_activation_fixture_2589;
