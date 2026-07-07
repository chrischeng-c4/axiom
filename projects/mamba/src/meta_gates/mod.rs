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
    mod typeshed_surface_gate;
}
