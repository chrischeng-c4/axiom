---
id: '1640'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: service-http-otlp-contract
entry: configure_tracing
nodes:
  configure_tracing:
    kind: start
    label: "service passes HttpConfig and explicit ServiceIdentity to shared initialization"
  identity_valid:
    kind: decision
    label: "service name and version are non-empty stable identifiers?"
  identity_error:
    kind: terminal
    label: "reject invalid local configuration before installing a subscriber"
  endpoint_present:
    kind: decision
    label: "OTLP endpoint present?"
  logging_subscriber:
    kind: process
    label: "install EnvFilter plus selected JSON or pretty formatter"
  feature_present:
    kind: decision
    label: "service-http compiled with otlp feature?"
  no_feature:
    kind: process
    label: "emit one redacted warning then install logging subscriber"
  build_exporter:
    kind: process
    label: "construct batch OTLP tracer with service.name and service.version resource attributes"
  build_result:
    kind: decision
    label: "exporter construction succeeds?"
  build_failure:
    kind: process
    label: "record redacted construction failure then install logging subscriber"
  composite_subscriber:
    kind: process
    label: "install formatter and tracing-opentelemetry layer; later exporter failures remain non-fatal"
  propagation:
    kind: process
    label: "HTTP middleware extracts traceparent/tracestate, sets request span parent, and leaves absent or invalid headers as root spans"
  runnable:
    kind: terminal
    label: "shared service API is runnable with logging and optional trace export"
edges:
  - { from: configure_tracing, to: identity_valid }
  - { from: identity_valid, to: identity_error, label: "no" }
  - { from: identity_valid, to: endpoint_present, label: "yes" }
  - { from: endpoint_present, to: logging_subscriber, label: "no" }
  - { from: endpoint_present, to: feature_present, label: "yes" }
  - { from: feature_present, to: no_feature, label: "no" }
  - { from: feature_present, to: build_exporter, label: "yes" }
  - { from: build_exporter, to: build_result }
  - { from: build_result, to: build_failure, label: "no" }
  - { from: build_result, to: composite_subscriber, label: "yes" }
  - { from: logging_subscriber, to: propagation }
  - { from: no_feature, to: propagation }
  - { from: build_failure, to: propagation }
  - { from: composite_subscriber, to: propagation }
  - { from: propagation, to: runnable }
---
flowchart TD
    start[HttpConfig plus ServiceIdentity] --> identity{valid identity?}
    identity -->|no| invalid([configuration error])
    identity -->|yes| endpoint{endpoint configured?}
    endpoint -->|no| logs[formatter subscriber]
    endpoint -->|yes| feature{OTLP feature?}
    feature -->|no| nofeature[redacted warning; formatter]
    feature -->|yes| export[build batch exporter]
    export --> built{constructed?}
    built -->|no| fallback[redacted failure; formatter]
    built -->|yes| otlp[formatter plus OTLP layer]
    logs --> context[extract W3C context into request span]
    nofeature --> context
    fallback --> context
    otlp --> context
    context --> ready([runnable service])
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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: service-http-shared-otlp-tracing-verification
requirements:
  fallback:
    id: R3
    text: "Malformed or unreachable exporter setup does not abort service startup and falls back to structured logging without leaking endpoint secrets."
    kind: negative
    risk: high
    verify: cargo test -p service-http --features otlp --test otlp_tracing exporter_setup_failure_keeps_logging_available -- --exact
  identity_contract:
    id: R2
    text: "An OTLP-enabled service supplies stable non-empty service name and version resource attributes through the shared configuration API."
    kind: functional
    risk: high
    verify: cargo test -p service-http --features otlp --test otlp_tracing otlp_identity_contract_is_stable -- --exact
  logging_only_default:
    id: R1
    text: "Without an OTLP endpoint or OTLP feature, shared service tracing remains logging-only and preserves the existing initialization contract."
    kind: regression
    risk: high
    verify: cargo test -p service-http --test otlp_tracing logging_only_default_requires_no_exporter -- --exact
  trace_context:
    id: R4
    text: "The shared HTTP trace layer accepts a valid W3C traceparent and makes the resulting request span a child of that propagated context."
    kind: contract
    risk: medium
    verify: cargo test -p service-http --features otlp --test otlp_tracing trace_layer_propagates_w3c_parent_context -- --exact
---
flowchart TD
    r1[R1 logging only default] --> cargo_test_p_service_http_test_otlp_tracing_logging_only_default_requires_no_exporter_exact[cargo test -p service-http --test otlp_tracing logging_only_default_requires_no_exporter -- --exact]
    r2[R2 identity contract] --> cargo_test_p_service_http_features_otlp_test_otlp_tracing_otlp_identity_contract_is_stable_exact[cargo test -p service-http --features otlp --test otlp_tracing otlp_identity_contract_is_stable -- --exact]
    r3[R3 fallback] --> cargo_test_p_service_http_features_otlp_test_otlp_tracing_exporter_setup_failure_keeps_logging_available_exact[cargo test -p service-http --features otlp --test otlp_tracing exporter_setup_failure_keeps_logging_available -- --exact]
    r4[R4 trace context] --> cargo_test_p_service_http_features_otlp_test_otlp_tracing_trace_layer_propagates_w3c_parent_context_exact[cargo test -p service-http --features otlp --test otlp_tracing trace_layer_propagates_w3c_parent_context -- --exact]
```
