---
id: '1777'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-service-observability-contract
entry: configure
nodes:
  configure: { kind: start, label: "Application supplies ObservabilityConfig { log_level, log_format, otlp_endpoint } and ServiceIdentity" }
  validate: { kind: decision, label: "Is OTLP requested with a valid absolute HTTP(S) endpoint and compiled exporter support?" }
  logging: { kind: process, label: "Install one RUST_LOG-first pretty or JSON subscriber" }
  exporter: { kind: process, label: "Attach stable service.name and service.version resources and W3C propagator" }
  fallback: { kind: process, label: "Install logging-only subscriber and emit a redacted fallback reason" }
  provider: { kind: process, label: "MetricsProvider returns canonical Prometheus exposition bytes" }
  connection: { kind: process, label: "LifecycleMetrics implements ConnectionMetrics using metrics-prometheus counters" }
  http_config: { kind: process, label: "service-http HttpConfig projects only its observability fields into ObservabilityConfig" }
  http_adapter: { kind: process, label: "service-http extracts request headers and serves provider bytes without owning protocol-neutral state" }
  non_http: { kind: terminal, label: "Raw TCP and future protocol runtimes consume service-observability directly" }
  compatible: { kind: terminal, label: "Existing service-http names remain additive compatibility re-exports" }
edges:
  - { from: configure, to: validate }
  - { from: validate, to: logging, when: "no exporter requested" }
  - { from: validate, to: exporter, when: "valid and supported" }
  - { from: validate, to: fallback, when: "invalid or unavailable" }
  - { from: configure, to: provider }
  - { from: provider, to: connection }
  - { from: configure, to: http_config }
  - { from: http_config, to: http_adapter }
  - { from: exporter, to: http_adapter }
  - { from: connection, to: http_adapter }
  - { from: connection, to: non_http }
  - { from: http_adapter, to: compatible }
---
flowchart TD
  configure([ObservabilityConfig + ServiceIdentity]) --> validate{OTLP request valid and supported?}
  validate -->|not requested| logging[RUST_LOG-first pretty or JSON subscriber]
  validate -->|yes| exporter[OTLP exporter + stable resource + W3C propagator]
  validate -->|invalid/unavailable| fallback[logging-only + redacted fallback]
  configure --> provider[MetricsProvider canonical bytes]
  provider --> connection[LifecycleMetrics via metrics-prometheus]
  configure --> http_config[HttpConfig projects observability fields]
  http_config --> http_adapter[service-http HTTP request and route adapter]
  exporter --> http_adapter
  connection --> http_adapter
  connection --> non_http([raw TCP and non-HTTP consumers])
  http_adapter --> compatible([compatible service-http imports and bytes])
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
