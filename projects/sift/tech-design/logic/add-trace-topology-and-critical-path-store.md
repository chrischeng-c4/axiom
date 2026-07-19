---
id: "1665"
summary: Materialize OpenTelemetry span events into deterministic trace topology, explicit partial/cycle diagnostics, critical paths, and project-authorized trace retrieval.
capability_refs:
  - id: materialized-observability-stores
    role: primary
    gap: trace-store-topology-and-correlation
    claim: trace-store-topology-and-correlation
    coverage: full
    rationale: This slice owns the dedicated trace projection, topology, critical path, correlations, checkpoint, and raw rebuild equality.
  - id: query-tail-and-replay
    role: contributes
    gap: typed-cross-signal-query
    claim: typed-cross-signal-query
    coverage: partial
    rationale: GET trace supplies the trace-specific typed retrieval surface; unified multi-store query remains 1671.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-trace-store
entry: raw_span
nodes:
  raw_span: { kind: start, label: "committed span event" }
  normalize: { kind: process, label: "normalize timing status links and events" }
  upsert: { kind: process, label: "idempotent span upsert" }
  topology: { kind: process, label: "resolve parent child topology and gaps" }
  critical: { kind: process, label: "compute deterministic critical path" }
  checkpoint: { kind: terminal, label: "persist trace store checkpoint" }
  query: { kind: start, label: "authorized trace query" }
  wait: { kind: decision, label: "projection cursor ready?" }
  lag: { kind: terminal, label: "projection lag" }
  result: { kind: terminal, label: "complete or explicit partial trace" }
edges:
  - { from: raw_span, to: normalize }
  - { from: normalize, to: upsert }
  - { from: upsert, to: topology }
  - { from: topology, to: critical }
  - { from: critical, to: checkpoint }
  - { from: query, to: wait }
  - { from: wait, to: lag, label: "no" }
  - { from: wait, to: result, label: "yes" }
---
flowchart TD
    raw_span([committed span]) --> normalize[normalize span]
    normalize --> upsert[idempotent upsert]
    upsert --> topology[topology and gaps]
    topology --> critical[critical path]
    critical --> checkpoint([checkpoint])
    query([authorized query]) --> wait{cursor ready?}
    wait -->|no| lag([projection lag])
    wait -->|yes| result([trace result])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
constants:
  projection: trace-store
  schema_version: 1
schemas:
  - name: SpanLinkV1
    fields:
      - { name: trace_id, type: String, required: true }
      - { name: span_id, type: String, required: true }
      - { name: attributes, type: "BTreeMap<String,AttributeValue>", required: true }
  - name: SpanEventV1
    fields:
      - { name: name, type: String, required: true }
      - { name: time_unix_nano, type: u64, required: true }
      - { name: attributes, type: "BTreeMap<String,AttributeValue>", required: true }
  - name: SpanRecordV1
    fields:
      - { name: cursor, type: u64, required: true }
      - { name: project, type: String, required: true }
      - { name: environment, type: String, required: true }
      - { name: trace_id, type: String, required: true }
      - { name: span_id, type: String, required: true }
      - { name: parent_span_id, type: String, required: false }
      - { name: name, type: String, required: true }
      - { name: kind, type: String, required: false }
      - { name: start_time_unix_nano, type: u64, required: true }
      - { name: end_time_unix_nano, type: u64, required: true }
      - { name: status_code, type: String, required: false }
      - { name: status_message, type: String, required: false }
      - { name: links, type: "Vec<SpanLinkV1>", required: true }
      - { name: events, type: "Vec<SpanEventV1>", required: true }
      - { name: resource, type: "BTreeMap<String,String>", required: true }
      - { name: attributes, type: "BTreeMap<String,AttributeValue>", required: true }
      - { name: request_id, type: String, required: false }
      - { name: session_id, type: String, required: false }
  - name: TraceResultV1
    fields:
      - { name: project, type: String, required: true }
      - { name: trace_id, type: String, required: true }
      - { name: spans, type: "Vec<SpanRecordV1>", required: true }
      - { name: root_span_ids, type: "Vec<String>", required: true }
      - { name: partial, type: bool, required: true }
      - { name: gaps, type: "Vec<String>", required: true }
      - { name: cycles, type: "Vec<Vec<String>>", required: true }
      - { name: critical_path_span_ids, type: "Vec<String>", required: true }
      - { name: duration_unix_nano, type: u64, required: true }
      - { name: correlation_ids, type: "BTreeMap<String,Vec<String>>", required: true }
      - { name: projection_cursor, type: u64, required: true }
ordering: start_time_unix_nano_then_span_id_then_cursor
partial_rules:
  - missing parent
  - duplicate conflicting span id
  - parent cycle
  - no root
```

## REST API
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info: { title: Sift Trace Store Slice, version: 1.0.0 }
paths:
  /v1/traces/{id}:
    get:
      operationId: getTrace
      summary: Return a stable complete or explicitly partial trace topology.
      parameters:
        - { name: id, in: path, required: true, schema: { type: string } }
        - { name: project, in: query, required: true, schema: { type: string } }
        - { name: min_cursor, in: query, required: false, schema: { type: integer, format: uint64 } }
      responses:
        "200": { description: Trace topology and critical path. }
        "400": { description: Invalid project or trace id. }
        "403": { description: Project read denied. }
        "404": { description: Trace absent from the authorized project. }
        "503": { description: Shared projection_lag with Retry-After. }
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sift-trace-store-verification
requirements:
  out_of_order: { id: R1, text: "Out of order spans form deterministic parent child topology", kind: functional, risk: high, verify: test }
  missing_parent: { id: R2, text: "Missing parents produce explicit partial gaps", kind: reliability, risk: high, verify: test }
  links_events: { id: R3, text: "Links events status resources and scope survive projection", kind: functional, risk: high, verify: test }
  cycle: { id: R4, text: "Cycles are detected without recursion failure", kind: reliability, risk: critical, verify: test }
  critical: { id: R5, text: "Critical path and trace duration are deterministic", kind: functional, risk: high, verify: test }
  correlations: { id: R6, text: "Request session linked trace and span ids remain machine queryable", kind: functional, risk: high, verify: test }
  api: { id: R7, text: "Trace retrieval enforces project read and min cursor", kind: security, risk: critical, verify: test }
  rebuild: { id: R8, text: "Raw journal rebuild equals the live trace store", kind: reliability, risk: critical, verify: test }
elements:
  store_test: { kind: test, type: "rs/#[test]" }
  api_test: { kind: test, type: "rs/#[tokio::test]" }
relations:
  - { from: store_test, to: R1, type: verifies }
  - { from: store_test, to: R2, type: verifies }
  - { from: store_test, to: R3, type: verifies }
  - { from: store_test, to: R4, type: verifies }
  - { from: store_test, to: R5, type: verifies }
  - { from: store_test, to: R6, type: verifies }
  - { from: api_test, to: R7, type: verifies }
  - { from: store_test, to: R8, type: verifies }
---
graph LR
    store_test --> R1
    store_test --> R2
    store_test --> R3
    store_test --> R4
    store_test --> R5
    store_test --> R6
    api_test --> R7
    store_test --> R8
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/src/projection/trace.rs
    action: create
    section: schema
    impl_mode: hand-written
    gap: sift-trace-projection
    tracker: "1665"
    description: Define span/link/event schemas, trace topology, partial diagnostics, critical path, correlations, snapshot, and rebuild semantics.
  - path: projects/sift/src/projection/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-projection-module
    tracker: "1660"
    description: Export trace projection contracts.
  - path: projects/sift/src/projection/runtime.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-projection-runtime
    tracker: "1660"
    description: Register the independent trace projection and typed read access.
  - path: projects/sift/src/lib.rs
    action: modify
    section: rest-api
    impl_mode: hand-written
    gap: sift-service-core
    tracker: "1576"
    description: Add project-authorized trace retrieval with shared min-cursor lag behavior and OpenAPI schemas.
  - path: projects/sift/tests/trace_store.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-trace-store-tests
    tracker: "1665"
    description: Verify topology, missing parents, cycles, links/events, critical path, correlations, ordering, and rebuild equality.
  - path: projects/sift/tests/trace_api.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-trace-api-tests
    tracker: "1665"
    description: Verify authorized trace retrieval, not found, and projection lag.
```
