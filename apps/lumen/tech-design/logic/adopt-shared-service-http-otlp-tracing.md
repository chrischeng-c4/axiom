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
