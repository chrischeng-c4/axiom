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
