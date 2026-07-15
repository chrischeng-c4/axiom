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
// SPEC-MANAGED: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#logic
// CODEGEN-BEGIN
pub fn configure() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Decision: Is OTLP requested with a valid absolute HTTP(S) endpoint and compiled exporter support?
    if todo!("decision: Is OTLP requested with a valid absolute HTTP(S) endpoint and compiled exporter support?") /* branch */ {
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-logging
        // TODO: Implement process step: Install one RUST_LOG-first pretty or JSON subscriber
        todo!("process: Install one RUST_LOG-first pretty or JSON subscriber");
    } else if todo!("decision branch: {}", "branch") { /* branch */
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-exporter
        // TODO: Implement process step: Attach stable service.name and service.version resources and W3C propagator
        todo!("process: Attach stable service.name and service.version resources and W3C propagator");
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_adapter
        // TODO: Implement process step: service-http extracts request headers and serves provider bytes without owning protocol-neutral state
        todo!("process: service-http extracts request headers and serves provider bytes without owning protocol-neutral state");
        todo!("terminal: Existing service-http names remain additive compatibility re-exports");
    } else { /* branch */
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-fallback
        // TODO: Implement process step: Install logging-only subscriber and emit a redacted fallback reason
        todo!("process: Install logging-only subscriber and emit a redacted fallback reason");
    }
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-provider
    // TODO: Implement process step: MetricsProvider returns canonical Prometheus exposition bytes
    todo!("process: MetricsProvider returns canonical Prometheus exposition bytes");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-connection
    // TODO: Implement process step: LifecycleMetrics implements ConnectionMetrics using metrics-prometheus counters
    todo!("process: LifecycleMetrics implements ConnectionMetrics using metrics-prometheus counters");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_adapter
    // TODO: Implement process step: service-http extracts request headers and serves provider bytes without owning protocol-neutral state
    todo!("process: service-http extracts request headers and serves provider bytes without owning protocol-neutral state");
    todo!("terminal: Existing service-http names remain additive compatibility re-exports");
    todo!("terminal: Raw TCP and future protocol runtimes consume service-observability directly");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_config
    // TODO: Implement process step: service-http HttpConfig projects only its observability fields into ObservabilityConfig
    todo!("process: service-http HttpConfig projects only its observability fields into ObservabilityConfig");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_adapter
    // TODO: Implement process step: service-http extracts request headers and serves provider bytes without owning protocol-neutral state
    todo!("process: service-http extracts request headers and serves provider bytes without owning protocol-neutral state");
    todo!("terminal: Existing service-http names remain additive compatibility re-exports");
    // Terminal: compatible -> Existing service-http names remain additive compatibility re-exports
    // Terminal: non_http -> Raw TCP and future protocol runtimes consume service-observability directly
}
// CODEGEN-END
