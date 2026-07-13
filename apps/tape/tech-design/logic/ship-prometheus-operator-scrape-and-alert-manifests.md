---
id: "1588"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-prometheus-operator-observability
entry: service
nodes:
  service:
    kind: start
    label: "Tape server Service exposes /metrics with app=tape and role=server labels"
  monitor:
    kind: process
    label: "Optional ServiceMonitor selects the Service, scrapes /metrics, and copies app and role target labels"
  rules:
    kind: process
    label: "PrometheusRule evaluates existing append/replay latency sums and counts plus kube restart telemetry"
  optional:
    kind: terminal
    label: "Component applies only where Prometheus Operator CRDs exist; OTLP stays a shared-library follow-up"
edges:
  - { from: service, to: monitor }
  - { from: monitor, to: rules }
  - { from: rules, to: optional }
---
flowchart TD
    service[Tape Service exposes metrics and stable labels] --> monitor[ServiceMonitor scrapes metrics and copies labels]
    monitor --> rules[PrometheusRule evaluates existing Tape series]
    rules --> optional([Optional component when Prometheus Operator CRDs exist])
```
