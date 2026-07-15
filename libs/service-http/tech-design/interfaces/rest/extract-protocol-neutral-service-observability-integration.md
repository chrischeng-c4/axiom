---
id: '1777'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-service-observability-integration
entry: configure
nodes:
  configure: { kind: start, label: "Application supplies ObservabilityConfig and stable ServiceIdentity" }
  mode: { kind: decision, label: "Resolve logging-only, OTLP, or safe fallback mode without HTTP dependencies" }
  subscriber: { kind: process, label: "service-observability installs the shared tracing subscriber and optional exporter" }
  metrics: { kind: process, label: "Domain metrics implement the shared MetricsProvider seam and render with metrics-prometheus" }
  lifecycle: { kind: process, label: "LifecycleMetrics implements server-lifecycle ConnectionMetrics with accepted, rejected, and closed counters" }
  http: { kind: process, label: "service-http only adapts W3C request headers and exposes provider bytes at /metrics" }
  other: { kind: terminal, label: "Raw TCP and future non-HTTP services compose the same observability contract" }
  compatible: { kind: terminal, label: "Existing service-http imports remain additive re-exports with byte-compatible probe output" }
edges:
  - { from: configure, to: mode }
  - { from: mode, to: subscriber, when: "logging or exporter mode selected" }
  - { from: configure, to: metrics }
  - { from: metrics, to: lifecycle }
  - { from: subscriber, to: http }
  - { from: lifecycle, to: http }
  - { from: lifecycle, to: other }
  - { from: http, to: compatible }
---
flowchart TD
  configure([ObservabilityConfig + ServiceIdentity]) --> mode{resolve trace mode}
  mode --> subscriber[service-observability subscriber + optional OTLP]
  configure --> metrics[MetricsProvider + metrics-prometheus encoder]
  metrics --> lifecycle[LifecycleMetrics implements ConnectionMetrics]
  subscriber --> http[service-http request/header + /metrics adapter only]
  lifecycle --> http
  lifecycle --> other([raw TCP and non-HTTP consumers])
  http --> compatible([byte-compatible existing service surface])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - { path: libs/service-observability/Cargo.toml, action: create, section: logic, impl_mode: hand-written, description: "Declare the protocol-neutral observability integration crate and optional OTLP feature." }
  - { path: libs/service-observability/README.md, action: create, section: logic, impl_mode: hand-written, description: "Define the shared service observability capability and ownership boundary." }
  - { path: libs/service-observability/src/lib.rs, action: create, section: logic, impl_mode: hand-written, description: "Export typed configuration, tracing, metric provider, and lifecycle metrics." }
  - { path: libs/service-observability/src/config.rs, action: create, section: logic, impl_mode: hand-written, description: "Own LogFormat, ObservabilityConfig, and ServiceIdentity." }
  - { path: libs/service-observability/src/logging.rs, action: create, section: logic, impl_mode: hand-written, description: "Own logging, OTLP resolution, subscriber installation, and W3C extraction." }
  - { path: libs/service-observability/src/metrics.rs, action: create, section: logic, impl_mode: hand-written, description: "Own MetricsProvider and Prometheus-backed lifecycle connection counters." }
  - { path: libs/service-http/src/config.rs, action: modify, section: logic, impl_mode: codegen, description: "Retain HTTP knobs while adapting them into ObservabilityConfig." }
  - { path: libs/service-http/src/logging.rs, action: modify, section: logic, impl_mode: codegen, description: "Replace protocol-neutral implementation with compatibility delegation." }
  - { path: libs/service-http/src/metrics.rs, action: modify, section: logic, impl_mode: codegen, description: "Re-export the shared protocol-neutral MetricsProvider." }
  - { path: libs/service-http/src/transport.rs, action: modify, section: logic, impl_mode: codegen, description: "Keep only HTTP request-context propagation adaptation." }
  - { path: libs/service-http/src/lib.rs, action: modify, section: logic, impl_mode: codegen, description: "Expose compatibility re-exports without claiming observability ownership." }
  - { path: libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md, action: modify, section: logic, impl_mode: hand-written, description: "Author the HTTP-to-observability config adapter source." }
  - { path: libs/service-http/tech-design/semantic/source/libs-service-http-src-logging-rs.md, action: modify, section: logic, impl_mode: hand-written, description: "Author compatibility delegation to service-observability." }
  - { path: libs/service-http/tech-design/semantic/source/libs-service-http-src-metrics-rs.md, action: modify, section: logic, impl_mode: hand-written, description: "Author the provider re-export source." }
  - { path: libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md, action: modify, section: logic, impl_mode: hand-written, description: "Author the HTTP-only propagation adapter source." }
  - { path: libs/service-http/tech-design/semantic/source/libs-service-http-src-lib-rs.md, action: modify, section: logic, impl_mode: hand-written, description: "Author the narrowed service-http public surface." }
  - { path: libs/service-http/tests/otlp_tracing.rs, action: modify, section: unit-test, impl_mode: hand-written, description: "Verify compatibility delegation and OTLP feature behavior." }
  - { path: libs/service-http/Cargo.toml, action: modify, section: logic, impl_mode: hand-written, description: "Delegate protocol-neutral dependencies and forward the OTLP feature." }
  - { path: libs/service-http/README.md, action: modify, section: logic, impl_mode: hand-written, description: "Document HTTP-only policy and adaptation ownership." }
  - { path: Cargo.toml, action: modify, section: logic, impl_mode: hand-written, description: "Register service-observability in the workspace." }
  - { path: aw.toml, action: modify, section: logic, impl_mode: codegen, description: "Register service-observability as an AW project." }
  - { path: Cargo.lock, action: modify, section: logic, impl_mode: codegen, description: "Refresh the dependency graph." }
  - { path: README.md, action: modify, section: logic, impl_mode: hand-written, description: "Name one owner for every observability layer." }
  - { path: CONTRIBUTING.md, action: modify, section: logic, impl_mode: hand-written, description: "Define service-observability, metrics-prometheus, server lifecycle, and HTTP adapter boundaries." }
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: shared-service-observability-integration-verification
requirements:
  consumer_observability_regression:
    id: R5
    text: "Lumen, Tape, Keep, Relay, and Courier continue compiling and serving their existing metrics and request-tracing policy after ownership extraction."
    kind: regression
    risk: medium
    verify: cargo check -p lumen -p tape -p keep -p relay -p courier
  http_compatibility:
    id: R3
    text: "service-http keeps byte-compatible probe metrics and OpenAPI responses while delegating protocol-neutral configuration, tracing, and MetricsProvider ownership."
    kind: regression
    risk: high
    verify: cargo test -p service-http
  lifecycle_prometheus_bridge:
    id: R2
    text: "LifecycleMetrics emits accepted, rejected, and closed callbacks through server-lifecycle and renders their counters with the canonical metrics-prometheus encoder."
    kind: functional
    risk: high
    verify: cargo test -p service-observability lifecycle_metrics
  otlp_feature_compatibility:
    id: R4
    text: "Default and OTLP-enabled builds preserve logging-only, valid exporter, invalid endpoint, and W3C propagation behavior through the compatibility surface."
    kind: regression
    risk: high
    verify: cargo test -p service-observability --features otlp; cargo test -p service-http --features otlp --test otlp_tracing
  protocol_neutral_configuration:
    id: R1
    text: "service-observability owns typed logging format, stable identity, observability configuration, trace-mode resolution, and subscriber installation without depending on service-http or axum."
    kind: functional
    risk: high
    verify: cargo test -p service-observability
---
flowchart TD
    r1[R1 protocol neutral configuration] --> cargo_test_p_service_observability[cargo test -p service-observability]
    r2[R2 lifecycle prometheus bridge] --> cargo_test_p_service_observability_lifecycle_metrics[cargo test -p service-observability lifecycle_metrics]
    r3[R3 http compatibility] --> cargo_test_p_service_http[cargo test -p service-http]
    r4[R4 otlp feature compatibility] --> cargo_test_p_service_observability_features_otlp_cargo_test_p_service_http_features_otlp_test_otlp_tracing[cargo test -p service-observability --features otlp; cargo test -p service-http --features otlp --test otlp_tracing]
    r5[R5 consumer observability regression] --> cargo_check_p_lumen_p_tape_p_keep_p_relay_p_courier[cargo check -p lumen -p tape -p keep -p relay -p courier]
```
