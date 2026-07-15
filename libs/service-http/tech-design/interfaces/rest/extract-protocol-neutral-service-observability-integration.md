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
