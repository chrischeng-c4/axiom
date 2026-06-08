// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN

//! T12 — `jet_parity_oracle::run_fixture` resolves at link time.
//!
//! Downstream parity tests (#2151 pixel, #2160 a11y, #2167 pointer,
//! #2174 IME) will build on this re-export to compare jet's WASM output
//! against the oracle's `ArtifactBundle`.

/// @spec parity-dom-reference-runner.md#Test Plan (jet_harness_reexport, T12)
#[test]
fn run_fixture_symbol_resolves() {
    // Reference the entry point through a type-erased pointer-to-address
    // check. If the symbol fails to resolve, the test binary won't link.
    let addr = jet_parity_oracle::run_fixture as *const ();
    assert!(!addr.is_null());
    // Touch the other public re-exports the spec promises (R10).
    let _cfg = jet_parity_oracle::RunnerConfig::default();
    let _matrix = jet_parity_oracle::MatrixEntry {
        browser: jet_parity_oracle::BrowserKind::Chromium,
        dpr: 1.0,
    };
}
// CODEGEN-END
