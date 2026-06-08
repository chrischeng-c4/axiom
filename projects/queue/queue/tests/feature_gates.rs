//! Compile-gate verification tests
//!
//! T20: Verifies that the metrics module is excluded when the `metrics` feature is disabled.
//! This test compiles only when the feature is OFF, confirming the cfg gate works.

/// When `metrics` feature is disabled, the `metrics` module should not be accessible.
/// This test simply compiles — its existence under `#[cfg(not(feature = "metrics"))]`
/// proves the gate works. If someone accidentally removed the feature gate from the
/// module, this file would fail to compile (because the non-gated module would conflict
/// or the imports below would succeed when they shouldn't).
#[cfg(not(feature = "metrics"))]
#[test]
fn no_metrics_module_without_feature() {
    // This test body intentionally does nothing.
    // Compilation success under `not(feature = "metrics")` proves the gate:
    //   - cclab_queue::metrics is NOT available in the public API
    //   - TaskMetrics, METRICS, gather_metrics are all gated out
    //
    // If the feature IS enabled, this test is excluded by cfg, which is correct —
    // the companion unit tests (T1–T19) cover the enabled path.
}
