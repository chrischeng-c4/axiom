---
id: '1661'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-shared-otlp-contract
entry: config
nodes:
  config: { kind: start, label: "Lumen ServeArgs owns LUMEN_OTLP_ENDPOINT, log level, and log format" }
  http_config: { kind: process, label: "map resolved values to service_http::HttpConfig" }
  identity: { kind: process, label: "create non-empty ServiceIdentity lumen plus build version" }
  shared: { kind: process, label: "call init_tracing_with_identity before engine startup" }
  trace_boundary: { kind: process, label: "service-http owns OTLP trace pipeline, fallback, and W3C propagation" }
  metrics_boundary: { kind: process, label: "Lumen owns engine-counter OTLP metrics provider and cadence" }
  contract: { kind: terminal, label: "one endpoint flag, shared traces, retained Lumen metrics" }
edges:
  - { from: config, to: http_config }
  - { from: http_config, to: identity }
  - { from: identity, to: shared }
  - { from: shared, to: trace_boundary }
  - { from: trace_boundary, to: metrics_boundary }
  - { from: metrics_boundary, to: contract }
---
flowchart TD
    config([Lumen ServeArgs owns endpoint and logging settings]) --> http_config[map resolved values to service_http HttpConfig]
    http_config --> identity[create non-empty ServiceIdentity with lumen and build version]
    identity --> shared[call init_tracing_with_identity before engine startup]
    shared --> trace_boundary[service-http owns trace pipeline fallback and W3C propagation]
    trace_boundary --> metrics_boundary[Lumen owns engine-counter OTLP metrics provider and cadence]
    metrics_boundary --> contract([one endpoint flag shared traces retained Lumen metrics])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/README.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: Link Lumen observability to the shared service-http trace contract while preserving its Lumen-owned metrics scope.
  - path: apps/lumen/external-contracts/claim-closure/production-claims.md
    action: modify
    section: contract
    impl_mode: hand-written
    description: State that the OTLP observability claim is satisfied by shared trace wiring plus retained Lumen metrics instrumentation.
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
