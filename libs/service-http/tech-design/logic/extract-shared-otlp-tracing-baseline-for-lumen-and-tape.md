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
    description: Add a default-off otlp feature and optional OpenTelemetry dependencies while preserving the plain logging build.
  - path: libs/service-http/src/config.rs
    action: modify
    section: contract
    impl_mode: hand-written
    description: Add a backward-compatible ServiceIdentity contract with validated stable name and version fields for shared tracing initialization.
  - path: libs/service-http/src/logging.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Construct an optional OTLP tracer from HttpConfig and ServiceIdentity, preserve the formatter, and fall back non-fatally on missing feature or exporter construction failure.
  - path: libs/service-http/src/transport.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Add a shared propagation middleware that extracts W3C traceparent/tracestate into the existing request span without changing route behavior.
  - path: libs/service-http/src/lib.rs
    action: modify
    section: contract
    impl_mode: hand-written
    description: Export the public service identity, initialization, and propagation surface with explicit logging-only defaults.
  - path: libs/service-http/README.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: Document opt-in OTLP, stable resource attributes, fallback semantics, and the Lumen/Tape adoption handoff.
  - path: libs/service-http/external-contracts/behavior/shared-http-service-scaffold-contract.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: Add claim closure for optional OTLP export and propagated request context while retaining no-endpoint behavior.
  - path: libs/service-http/tests/otlp_tracing.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Cover plain startup, identity stability, exporter construction fallback, and W3C parent propagation with local deterministic fixtures.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Refresh configuration semantic source coverage.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-src-logging-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Refresh exporter and fallback semantic source coverage.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Refresh propagation middleware semantic source coverage.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-src-lib-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Refresh public tracing API semantic source coverage.
  - path: libs/service-http/tech-design/semantic/source/libs-service-http-tests-otlp-tracing-rs.md
    action: create
    section: source
    impl_mode: hand-written
    description: Add semantic coverage for OTLP contract verification tests.
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
