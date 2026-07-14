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
  - path: apps/lumen/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Keep metrics OpenTelemetry dependencies while making the service-http OTLP tracing feature available and removing the trace-only direct layer dependency.
  - path: apps/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Delegate Lumen trace initialization to service-http with stable Lumen identity while retaining Lumen-owned metrics export.
  - path: apps/lumen/tests/shared_otlp_tracing.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Verify Lumen uses the shared OTLP trace initializer and does not own a duplicate tracer constructor.
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
id: lumen-shared-otlp-contract-verification
requirements:
  metrics_scope_retained:
    id: R2
    text: "The Lumen-owned OTLP metrics exporter remains compiled and available under the existing otel feature after trace extraction."
    kind: regression
    risk: medium
    verify: cargo test -p lumen --features otel --test shared_otlp_tracing -- --exact
  no_local_trace_pipeline:
    id: R3
    text: "The Lumen binary owns no OTLP trace pipeline or tracing-opentelemetry layer after adoption; the shared service-http package owns trace construction and W3C propagation."
    kind: regression
    risk: high
    verify: cargo test -p lumen --test shared_otlp_tracing -- --exact
  shared_trace_contract:
    id: R1
    text: "The Lumen executable passes its resolved endpoint, log settings, and stable identity through the public service-http trace initializer."
    kind: contract
    risk: high
    verify: cargo test -p lumen --test shared_otlp_tracing -- --exact
---
flowchart TD
    r1[R1 shared trace contract] --> cargo_test_p_lumen_test_shared_otlp_tracing_exact[cargo test -p lumen --test shared_otlp_tracing -- --exact]
    r3[R3 no local trace pipeline] --> cargo_test_p_lumen_test_shared_otlp_tracing_exact
    r2[R2 metrics scope retained] --> cargo_test_p_lumen_features_otel_test_shared_otlp_tracing_exact[cargo test -p lumen --features otel --test shared_otlp_tracing -- --exact]
```
