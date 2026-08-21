//! Consolidated umbrella binary for the 34 mambalibs schema-gate fixtures
//! (Phase 4 of the tests/ DDD refactor). Each gate parses a manifest under
//! `tests/mambalibs/fixtures/<gate>/manifest.toml` and asserts the field
//! contract for one mambalibs surface (artifact layout, CLI summary, ABI
//! mismatch, etc.). They have no `src/` owner because they test cross-crate
//! artifact loading rather than the mamba crate's own surface, so they
//! collapse into one umbrella here instead of moving inline under `src/`.
//!
//! Each former top-level `tests/mambalibs_<name>_<issue>.rs` now lives in
//! `tests/mambalibs/` and is registered below with `#[path]`. cargo only
//! compiles `tests/*.rs` as integration binaries, so the files under
//! `tests/mambalibs/` are no longer separate binaries — this umbrella is
//! the sole binary that links them all.
//!
//! G3 / G4.1 selector update: replace
//!     cargo test -p mamba --test 'mambalibs_*'
//! with
//!     cargo test -p mamba --test mambalibs_integration
//! to run every gate in one binary.

#[path = "mambalibs_abi_mismatch_failure_fixture_2582.rs"]
mod abi_mismatch_failure_fixture_2582;

#[path = "mambalibs_array_import_gate_fixture_2842.rs"]
mod array_import_gate_fixture_2842;

#[path = "mambalibs_artifact_layout_validation_fixture_2673.rs"]
mod artifact_layout_validation_fixture_2673;

#[path = "mambalibs_async_export_blocker_fixture_2677.rs"]
mod async_export_blocker_fixture_2677;

#[path = "mambalibs_clean_command_fixture_2674.rs"]
mod clean_command_fixture_2674;

#[path = "mambalibs_cli_summary_json_fixture_2676.rs"]
mod cli_summary_json_fixture_2676;

#[path = "mambalibs_dir_help_introspection_fixture_2667.rs"]
mod dir_help_introspection_fixture_2667;

#[path = "mambalibs_exported_type_roundtrip_fixture_2666.rs"]
mod exported_type_roundtrip_fixture_2666;

#[path = "mambalibs_fetch_import_gate_fixture_2844.rs"]
mod fetch_import_gate_fixture_2844;

#[path = "mambalibs_frame_import_gate_fixture_2840.rs"]
mod frame_import_gate_fixture_2840;

#[path = "mambalibs_from_mambalibs_import_fixture_2576.rs"]
mod from_mambalibs_import_fixture_2576;

#[path = "mambalibs_grid_import_gate_fixture_2847.rs"]
mod grid_import_gate_fixture_2847;

#[path = "mambalibs_learn_import_gate_fixture_2848.rs"]
mod learn_import_gate_fixture_2848;

#[path = "mambalibs_local_mode2_binding_crate_fixture_2577.rs"]
mod local_mode2_binding_crate_fixture_2577;

#[path = "mambalibs_local_path_override_fixture_2671.rs"]
mod local_path_override_fixture_2671;

#[path = "mambalibs_log_import_gate_fixture_2841.rs"]
mod log_import_gate_fixture_2841;

#[path = "mambalibs_mamba_build_e2e_harness_fixture_2578.rs"]
mod mamba_build_e2e_harness_fixture_2578;

#[path = "mambalibs_mamba_toml_mode2_dependency_fixture_2575.rs"]
mod mamba_toml_mode2_dependency_fixture_2575;

#[path = "mambalibs_media_import_gate_fixture_2850.rs"]
mod media_import_gate_fixture_2850;

#[path = "mambalibs_missing_dependency_diagnostic_fixture_2579.rs"]
mod missing_dependency_diagnostic_fixture_2579;

#[path = "mambalibs_mode2_e2e_gate_fixture_2531.rs"]
mod mode2_e2e_gate_fixture_2531;

#[path = "mambalibs_mode2_lockfile_assertion_fixture_2574.rs"]
mod mode2_lockfile_assertion_fixture_2574;

#[path = "mambalibs_multiple_mambalibs_import_fixture_2580.rs"]
mod multiple_mambalibs_import_fixture_2580;

#[path = "mambalibs_pg_import_gate_fixture_2843.rs"]
mod pg_import_gate_fixture_2843;

#[path = "mambalibs_plot_import_gate_fixture_2845.rs"]
mod plot_import_gate_fixture_2845;

#[path = "mambalibs_pyi_stub_generation_fixture_2668.rs"]
mod pyi_stub_generation_fixture_2668;

#[path = "mambalibs_rebuild_cache_invalidation_fixture_2669.rs"]
mod rebuild_cache_invalidation_fixture_2669;

#[path = "mambalibs_registry_metadata_validation_fixture_2670.rs"]
mod registry_metadata_validation_fixture_2670;

#[path = "mambalibs_rust_error_propagation_fixture_2672.rs"]
mod rust_error_propagation_fixture_2672;

#[path = "mambalibs_schema_import_gate_fixture_2839.rs"]
mod schema_import_gate_fixture_2839;

#[path = "mambalibs_sci_import_gate_fixture_2846.rs"]
mod sci_import_gate_fixture_2846;

#[path = "mambalibs_text_import_gate_fixture_2849.rs"]
mod text_import_gate_fixture_2849;

#[path = "mambalibs_toolchain_identity_fixture_2581.rs"]
mod toolchain_identity_fixture_2581;

#[path = "mambalibs_version_pin_conflict_fixture_2675.rs"]
mod version_pin_conflict_fixture_2675;
