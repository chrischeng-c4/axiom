---
id: '2415'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: loom-sift-agnostic-telemetry-flow
entry: role
nodes:
  role:
    kind: start
    label: long-running Loom role selected
  tracing:
    kind: process
    label: init shared JSONL and optional OTLP identity=loom
  request:
    kind: process
    label: controller shared HTTP shell receives a request
  completion:
    kind: process
    label: shared HTTP emits W3C-correlated request completion JSONL
  collector:
    kind: process
    label: external Sift collector owns ingest and query
  done:
    kind: terminal
    label: no direct Sift dependency or configuration in Loom
edges:
  - { from: role, to: tracing }
  - { from: tracing, to: request }
  - { from: request, to: completion }
  - { from: completion, to: collector }
  - { from: collector, to: done }
---
flowchart TD
  role[Long-running Loom role] --> tracing[shared service-http tracing identity loom]
  tracing --> request[controller service-http probe and API routes]
  request --> completion[http_request_complete JSONL with W3C context]
  completion --> collector[Sift-owned external collector and query]
  collector --> done[no Sift configuration in Loom]
```

Loom remains a Sift-agnostic producer. The process boundary owns structured axiom.service.log.v1 records, W3C request correlation, Prometheus-compatible probe metrics, and optional non-fatal OTLP export. Sift owns collection, routing, credentials, ingestion, retention, and query.

### Service-role tracing initialization

Add one Loom-local adapter around service_http::HttpConfig, ServiceIdentity, and init_tracing_with_identity. It reads LOOM_LOG_FORMAT (pretty or json, default pretty), LOOM_LOG_LEVEL (default info), and optional LOOM_OTLP_ENDPOINT. The adapter identifies every long-running Loom process as service.name=loom and returns configuration errors before the role begins serving. Controller, worker, run-task, job-controller, schema-layer, and the operator run path invoke the adapter; offline spec, render, llm, upgrade, and issue commands remain byte-clean.

### HTTP completion evidence

The controller keeps its existing service_http standard probe router and trace layer. The shared trace layer emits one INFO http_request_complete record inside the W3C-correlated request span after a response. A valid traceparent keeps trace_id, parent_span_id, trace_flags, and a distinct local span_id. Missing or malformed input creates a valid local root without panic. Loom adds no domain event solely for telemetry.

### Deployment configuration

The static base StatefulSet and the operator-rendered StatefulSet set LOOM_LOG_FORMAT=json for service workloads, while LOOM_LOG_LEVEL remains configurable. This makes production stdout collector-ready without introducing Sift configuration into Loom.

### Failure behavior and boundaries

An invalid LOOM_LOG_FORMAT fails process startup with a remediation message. OTLP setup remains optional and non-fatal according to service-http behavior. No Loom Cargo dependency, environment variable, HTTP header, route, or manifest may name a Sift endpoint, token, or collector. The Sift-owned VAT journey consumes captured stdout out of process and asserts nonzero accepted/query evidence.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/service-http/src/transport.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: pub fn trace_layer
    description: Emit one W3C-correlated http_request_complete JSONL event through the shared response hook so all shared HTTP services have the same generic request evidence.
  - path: libs/service-http/Cargo.toml
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: Add the existing formatter test dependency needed to decode the shared completion event contract.
  - path: libs/service-http/tests/request_completion_event.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Exercise the shared router and JSON formatter for valid, absent, and malformed W3C request context.
  - path: apps/loom/src/main.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: fn main
    description: Initialize shared service tracing only for long-running Loom roles from LOOM_LOG_FORMAT, LOOM_LOG_LEVEL, and optional LOOM_OTLP_ENDPOINT.
  - path: apps/loom/src/operator/render.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: fn extra_env
    description: Set collector-ready LOOM_LOG_FORMAT=json in the operator-rendered service workload environment.
  - path: apps/loom/k8s/base/statefulset.yaml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Set LOOM_LOG_FORMAT=json in the checked-in controller workload base.
  - path: apps/loom/tests/structured_stdout_traceparent.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Spawn the real controller, exercise valid invalid and absent traceparent requests, and assert schema identity W3C correlation and no direct Sift linkage.
```
