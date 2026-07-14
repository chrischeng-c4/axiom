---
id: '1640'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: service-http-shared-otlp-tracing
entry: init_tracing
nodes:
  init_tracing:
    kind: start
    label: "service starts shared tracing with log format, service identity, and optional OTLP endpoint"
  endpoint_configured:
    kind: decision
    label: "OTLP endpoint configured?"
  logging_only:
    kind: process
    label: "install existing JSON or pretty formatter with environment filter"
  otlp_feature:
    kind: decision
    label: "OTLP feature compiled?"
  feature_fallback:
    kind: process
    label: "warn once and keep logging-only subscriber; never fail service startup"
  exporter:
    kind: process
    label: "build OTLP exporter with stable service.name and service.version resources"
  exporter_ready:
    kind: decision
    label: "exporter initializes?"
  exporter_fallback:
    kind: process
    label: "record initialization failure and install logging-only subscriber"
  combined_subscriber:
    kind: process
    label: "install formatter plus tracing-opentelemetry layer"
  request_context:
    kind: process
    label: "shared HTTP transport extracts W3C context and creates a request span as child or root"
  ready:
    kind: terminal
    label: "service remains runnable with structured logs and optional OTLP trace export"
edges:
  - { from: init_tracing, to: endpoint_configured }
  - { from: endpoint_configured, to: logging_only, label: "no" }
  - { from: endpoint_configured, to: otlp_feature, label: "yes" }
  - { from: otlp_feature, to: feature_fallback, label: "no" }
  - { from: otlp_feature, to: exporter, label: "yes" }
  - { from: exporter, to: exporter_ready }
  - { from: exporter_ready, to: exporter_fallback, label: "no" }
  - { from: exporter_ready, to: combined_subscriber, label: "yes" }
  - { from: logging_only, to: request_context }
  - { from: feature_fallback, to: request_context }
  - { from: exporter_fallback, to: request_context }
  - { from: combined_subscriber, to: request_context }
  - { from: request_context, to: ready }
---
flowchart TD
    start[service init] --> endpoint{OTLP endpoint configured?}
    endpoint -->|no| logs[install formatter]
    endpoint -->|yes| feature{OTLP feature compiled?}
    feature -->|no| feature_fallback[warn; formatter only]
    feature -->|yes| exporter[build exporter with service resource]
    exporter --> exporter_ok{initialized?}
    exporter_ok -->|no| exporter_fallback[record failure; formatter only]
    exporter_ok -->|yes| combined[formatter plus OTLP layer]
    logs --> request[extract W3C context; create request span]
    feature_fallback --> request
    exporter_fallback --> request
    combined --> request
    request --> ready([runnable; optional export])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-http/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Add an optional OTLP feature and its tracing/export dependencies without changing the default logging-only dependency graph.
  - path: libs/service-http/src/config.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Define explicit stable service identity input alongside the existing optional OTLP endpoint configuration.
  - path: libs/service-http/src/logging.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Replace the OTLP TODO with feature-gated exporter setup, resource attributes, and non-fatal logging-only fallback.
  - path: libs/service-http/src/transport.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Extend the standard request trace layer with W3C trace-context extraction while preserving the existing span shape.
  - path: libs/service-http/src/lib.rs
    action: modify
    section: contract
    impl_mode: hand-written
    description: Export the stable tracing identity and initialization contract for service binaries.
  - path: libs/service-http/README.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: Document logging-only defaults, opt-in OTLP behavior, stable resource fields, and ownership boundaries.
  - path: libs/service-http/external-contracts/behavior/shared-http-service-scaffold-contract.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: Extend the shared service-scaffold claim with optional trace export and context propagation behavior.
  - path: libs/service-http/tests/otlp_tracing.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Verify default logging-only initialization, explicit OTLP service identity, graceful exporter fallback, and propagated request context without a vendor collector.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Refresh semantic source coverage for the tracing identity configuration.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-src-logging-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Refresh semantic source coverage for exporter and fallback behavior.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Refresh semantic source coverage for propagated request tracing.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-src-lib-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Refresh semantic source coverage for the public tracing API.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-tests-otlp-tracing-rs.md
    action: create
    section: source
    impl_mode: hand-written
    description: Add semantic source coverage for OTLP contract tests.
```
