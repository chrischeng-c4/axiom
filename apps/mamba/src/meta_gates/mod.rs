//! Test-only home for cross-domain manifest schema gates.
//!
//! Hosts the umbrella/epic/MVP/conformance-meta gate fixtures inlined
//! from `tests/*_gate_fixture_*.rs` that have no single-domain owner.
//! Every file here is a pure TOML-schema test of a `tests/governance/gates/.../manifest.toml`
//! and is gated `#[cfg(test)]` so it contributes zero to release builds.

#![cfg(test)]

mod tests {
    mod c3_3p_libs_conformance_umbrella_gate;
    mod c3_flask_runs_unmodified_gate;
    mod c3_pytest_runs_unmodified_gate;
    mod c3_requests_runs_unmodified_gate;
    mod cclab_qc_mamba_binding_gate;
    mod cloud_sdk_umbrella_gate;
    mod cpython_lib_test_denominator_gate;
    mod cpython_lib_test_real_assertions_gate;
    mod mvp_perf_10x_umbrella_gate;
    mod mvp_py312_ecosystem_umbrella_gate;
    mod mvp_test_completeness_epic_gate;
    mod perf_10x_cpython_312_gate;
    mod py313_conformance_epic_gate;
    mod py314_conformance_epic_gate;
    mod t1_forcetyped_completeness_denominator_gate;
    mod t1_free_threaded_state_denominator_gate;
    mod t1_generic_binding_denominator_gate;
    mod t1_race_leak_matrix_denominator_gate;
    mod t1_type_wall_denominator_gate;
    mod t2_calls_binding_denominator_gate;
    mod t2_codegen_lowering_denominator_gate;
    mod t2_exceptions_denominator_gate;
    mod t2_frames_dispatch_denominator_gate;
    mod t2_generators_async_denominator_gate;
    mod t2_object_model_denominator_gate;
    mod t2_parser_syntaxerror_denominator_gate;
    mod t2_peephole_denominator_gate;
    mod t2_scope_closures_denominator_gate;
    mod typeshed_surface_gate;
}
