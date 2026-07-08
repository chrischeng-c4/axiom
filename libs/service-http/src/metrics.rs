// SPEC-MANAGED: libs/service-http/tech-design/semantic/source/libs-service-http-src-metrics-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Metrics seam for `/metrics`.
//!
//! A service supplies a type that renders its Prometheus text-format body; the
//! shared probe router serves it at `GET /metrics` as
//! `text/plain; version=0.0.4`. When a service has no metrics it can omit the
//! provider entirely (the probe router serves an empty body), so the default
//! method returns `String::new()`.

/// Renders the Prometheus text-format `/metrics` body.
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-metrics-rs.md#source
pub trait MetricsProvider: Send + Sync {
    /// The full Prometheus text-format exposition. Defaults to empty.
    fn render_metrics(&self) -> String {
        String::new()
    }
}
// CODEGEN-END
