---
id: '1661'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-adopt-shared-otlp-tracing
entry: start
nodes:
  start: { kind: start, label: "lumen serve resolves log settings and LUMEN_OTLP_ENDPOINT" }
  config: { kind: process, label: "build service_http::HttpConfig" }
  identity: { kind: process, label: "ServiceIdentity::new(lumen, CARGO_PKG_VERSION)" }
  init: { kind: process, label: "service_http::init_tracing_with_identity" }
  mode: { kind: decision, label: "shared tracing mode" }
  logging: { kind: process, label: "install existing JSON or pretty structured logging" }
  exporter: { kind: process, label: "shared tracer has stable resource attributes and W3C propagator" }
  fallback: { kind: process, label: "redacted warning; keep structured logging" }
  engine: { kind: process, label: "start Lumen engine and HTTP server" }
  metrics: { kind: process, label: "retain Lumen local OTLP metrics exporter and instruments" }
  shutdown: { kind: terminal, label: "conditional global tracer shutdown on service exit" }
edges:
  - { from: start, to: config }
  - { from: config, to: identity }
  - { from: identity, to: init }
  - { from: init, to: mode }
  - { from: mode, to: logging, label: "no endpoint" }
  - { from: mode, to: exporter, label: "otlp feature and valid endpoint" }
  - { from: mode, to: fallback, label: "feature missing or invalid/exporter failure" }
  - { from: logging, to: engine }
  - { from: exporter, to: engine }
  - { from: fallback, to: engine }
  - { from: engine, to: metrics }
  - { from: metrics, to: shutdown }
---
flowchart TD
    start([lumen serve resolves log settings and LUMEN_OTLP_ENDPOINT]) --> config[build service_http::HttpConfig]
    config --> identity[ServiceIdentity::new lumen and build version]
    identity --> init[service_http::init_tracing_with_identity]
    init --> mode{shared tracing mode}
    mode -->|no endpoint| logging[install existing JSON or pretty structured logging]
    mode -->|otlp feature and valid endpoint| exporter[shared tracer has stable resource attributes and W3C propagator]
    mode -->|feature missing or invalid/exporter failure| fallback[redacted warning; keep structured logging]
    logging --> engine[start Lumen engine and HTTP server]
    exporter --> engine
    fallback --> engine
    engine --> metrics[retain Lumen local OTLP metrics exporter and instruments]
    metrics --> shutdown([conditional global tracer shutdown on service exit])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Keep the metrics OpenTelemetry dependencies while making the service-http OTLP tracing feature available and removing the trace-only direct layer dependency.
  - path: apps/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Delegate Lumen trace initialization to service-http with stable Lumen identity while retaining Lumen-owned metrics export.
  - path: apps/lumen/tests/shared_otlp_tracing.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Verify the Lumen binary wiring uses the shared OTLP trace initializer without owning a duplicate tracer constructor.
  - path: apps/lumen/README.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: Record the shared service-http tracing adopter under the Lumen observability work root.
  - path: apps/lumen/tech-design/semantic/source/apps-lumen-src-bin-lumen-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Update the semantic source unit to distinguish shared tracing from Lumen-owned metrics export.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-adopt-shared-otlp-tracing-verification
requirements:
  feature_propagation:
    id: R2
    text: "The Lumen otel feature enables the shared service-http otlp feature while keeping the metrics exporter dependencies available."
    kind: contract
    risk: high
    verify: cargo test -p lumen --features otel --test shared_otlp_tracing -- --exact
  no_duplicate_trace_constructor:
    id: R3
    text: "Lumen no longer owns an OTLP trace pipeline or tracing-opentelemetry layer; Lumen-owned OTLP metrics instrumentation remains separate."
    kind: regression
    risk: medium
    verify: cargo test -p lumen --test shared_otlp_tracing -- --exact
  shared_trace_initializer:
    id: R1
    text: "Lumen delegates trace subscriber initialization to service-http with a stable lumen name and build-version identity instead of constructing a local tracer."
    kind: regression
    risk: high
    verify: cargo test -p lumen --test shared_otlp_tracing -- --exact
---
flowchart TD
    r1[R1 shared trace initializer] --> cargo_test_p_lumen_test_shared_otlp_tracing_exact[cargo test -p lumen --test shared_otlp_tracing -- --exact]
    r3[R3 no duplicate trace constructor] --> cargo_test_p_lumen_test_shared_otlp_tracing_exact
    r2[R2 feature propagation] --> cargo_test_p_lumen_features_otel_test_shared_otlp_tracing_exact[cargo test -p lumen --features otel --test shared_otlp_tracing -- --exact]
```
