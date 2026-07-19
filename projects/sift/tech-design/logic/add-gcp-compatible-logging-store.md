---
id: "1664"
summary: Add a dedicated durable logging projection with GCP jsonPayload and GKE resource compatibility, fixed-field embedded-Lumen search, typed cursor queries, bounded tail resume, retention, and raw rebuild equality.
capability_refs:
  - id: materialized-observability-stores
    role: primary
    gap: logging-store-over-events
    claim: logging-store-over-events
    coverage: full
    rationale: This slice owns the first domain-specific materialized store over the canonical raw journal.
  - id: gcp-cloud-logging-compatibility
    role: primary
    gap: logging-view-query-compatibility
    claim: logging-view-query-compatibility
    coverage: full
    rationale: GCP jsonPayload, k8s_container resources, severity, correlations, and coexistence identity remain queryable in the dedicated log view.
  - id: query-tail-and-replay
    role: contributes
    gap: typed-cross-signal-query
    claim: typed-cross-signal-query
    coverage: partial
    rationale: This slice supplies the log-specific typed query route; unified cross-signal query remains #1671.
  - id: query-tail-and-replay
    role: contributes
    gap: cursor-pagination-and-ordering
    claim: cursor-pagination-and-ordering
    coverage: partial
    rationale: Logging pages are ordered and resumed by raw cursor; unified sort and cross-store cursoring remain #1671.
  - id: query-tail-and-replay
    role: contributes
    gap: live-tail-resume
    claim: live-tail-resume
    coverage: partial
    rationale: GET logs tail exposes a bounded resume primitive; the long-lived streaming transport remains #1671.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-logging-store
entry: raw_log
nodes:
  raw_log: { kind: start, label: "committed raw log event" }
  select: { kind: decision, label: "signal is log?" }
  normalize: { kind: process, label: "normalize jsonPayload and OTel body" }
  dedupe: { kind: process, label: "upsert by event id and raw cursor" }
  index: { kind: process, label: "index fixed text keyword range fields in embedded Lumen" }
  retain: { kind: process, label: "apply independent record retention" }
  checkpoint: { kind: terminal, label: "atomically persist logging snapshot and checkpoint" }
  query: { kind: start, label: "POST logs query or GET logs tail" }
  authorize: { kind: process, label: "authorize project read" }
  wait: { kind: decision, label: "min cursor reached?" }
  lag: { kind: terminal, label: "projection lag with Retry After" }
  candidates: { kind: process, label: "resolve full text candidates" }
  filters: { kind: process, label: "apply typed time project resource severity correlation and attribute filters" }
  page: { kind: terminal, label: "stable raw cursor page and resume cursor" }
edges:
  - { from: raw_log, to: select }
  - { from: select, to: normalize, label: "yes" }
  - { from: select, to: checkpoint, label: "no op" }
  - { from: normalize, to: dedupe }
  - { from: dedupe, to: index }
  - { from: index, to: retain }
  - { from: retain, to: checkpoint }
  - { from: query, to: authorize }
  - { from: authorize, to: wait }
  - { from: wait, to: lag, label: "timeout" }
  - { from: wait, to: candidates, label: "ready" }
  - { from: candidates, to: filters }
  - { from: filters, to: page }
---
flowchart TD
    raw_log([committed raw log]) --> select{signal is log?}
    select -->|yes| normalize[normalize structured body]
    select -->|no| checkpoint([checkpoint unchanged])
    normalize --> dedupe[versioned idempotent upsert]
    dedupe --> index[embedded Lumen fixed fields]
    index --> retain[independent retention]
    retain --> checkpoint[atomic state plus checkpoint]
    query([logs query or tail]) --> authorize[project read authorization]
    authorize --> wait{min cursor reached?}
    wait -->|timeout| lag([projection lag])
    wait -->|ready| candidates[text candidates]
    candidates --> filters[typed filters]
    filters --> page([stable cursor page])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
constants:
  logging_projection: logging-store
  logging_schema_version: 1
  default_retained_records: 1000000
  max_query_limit: 1000
schemas:
  - name: LogRecordV1
    fields:
      - { name: cursor, type: u64, required: true }
      - { name: event_id, type: String, required: true }
      - { name: project, type: String, required: true }
      - { name: environment, type: String, required: true }
      - { name: occurred_at, type: String, required: true, constraints: RFC3339 }
      - { name: observed_at, type: String, required: true, constraints: RFC3339 }
      - { name: severity, type: String, required: false }
      - { name: body_text, type: String, required: true }
      - { name: json_payload, type: JSON, required: true, constraints: object for GCP jsonPayload, structured value otherwise }
      - { name: resource, type: "BTreeMap<String,String>", required: true }
      - { name: attributes, type: "BTreeMap<String,AttributeValue>", required: true }
      - { name: trace_id, type: String, required: false }
      - { name: span_id, type: String, required: false }
      - { name: request_id, type: String, required: false }
      - { name: session_id, type: String, required: false }
      - { name: coexistence_key, type: String, required: true, description: Stable project plus event identity for later Cloud Logging collector dedupe. }
  - name: LogQuery
    fields:
      - { name: project, type: String, required: true }
      - { name: environment, type: String, required: false }
      - { name: start_time, type: String, required: false, constraints: RFC3339 inclusive }
      - { name: end_time, type: String, required: false, constraints: RFC3339 exclusive }
      - { name: severity, type: String, required: false }
      - { name: resource_type, type: String, required: false }
      - { name: service_name, type: String, required: false }
      - { name: trace_id, type: String, required: false }
      - { name: span_id, type: String, required: false }
      - { name: request_id, type: String, required: false }
      - { name: session_id, type: String, required: false }
      - { name: text, type: String, required: false }
      - { name: attribute_equals, type: "BTreeMap<String,AttributeValue>", required: false }
      - { name: after_cursor, type: u64, required: false, default: 0 }
      - { name: min_cursor, type: u64, required: false }
      - { name: limit, type: usize, required: false, default: 100, maximum: 1000 }
  - name: LogPage
    fields:
      - { name: records, type: "Vec<LogRecordV1>", required: true }
      - { name: next_cursor, type: u64, required: true }
      - { name: projection_cursor, type: u64, required: true }
      - { name: has_more, type: bool, required: true }
logging_lumen_fields:
  body: text
  project: keyword
  environment: keyword
  severity: keyword
  resource_type: keyword
  service_name: keyword
  trace_id: keyword
  span_id: keyword
  request_id: keyword
  session_id: keyword
  occurred_at: keyword
  coexistence_key: keyword
  cursor: number
retention:
  unit: records
  behavior: Remove the oldest record rows after the configured bound; raw events remain authoritative and rebuildable.
  independence: Logging snapshot and checkpoint are separate from every other projection.
```

## REST API
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info: { title: Sift Logging Store Slice, version: 1.0.0 }
paths:
  /v1/logs:query:
    post:
      operationId: queryLogs
      summary: Query the dedicated logging projection with typed filters and stable raw-cursor pagination.
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: "#/components/schemas/LogQuery" }
      responses:
        "200": { description: Stable log page. }
        "400": { description: Invalid time, limit, or filter. }
        "403": { description: Project read denied. }
        "503": { description: projection_lag with Retry-After. }
  /v1/logs:tail:
    get:
      operationId: tailLogs
      summary: Return a bounded cursor-resumable log page for a caller-owned tail loop.
      parameters:
        - { name: project, in: query, required: true, schema: { type: string } }
        - { name: after_cursor, in: query, required: false, schema: { type: integer, format: uint64 } }
        - { name: min_cursor, in: query, required: false, schema: { type: integer, format: uint64 } }
        - { name: limit, in: query, required: false, schema: { type: integer, maximum: 1000 } }
      responses:
        "200": { description: Bounded page carrying next_cursor for resume. }
        "403": { description: Project read denied. }
        "503": { description: projection_lag with Retry-After. }
components:
  schemas:
    LogQuery: { type: object, required: [project] }
    LogPage: { type: object, required: [records, next_cursor, projection_cursor, has_more] }
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sift-logging-store-verification
requirements:
  golden_gcp: { id: R1, text: "GCP jsonPayload and k8s_container labels normalize deterministically", kind: functional, risk: high, verify: test }
  golden_otel: { id: R2, text: "OTel structured and string bodies become searchable records", kind: functional, risk: high, verify: test }
  correlation: { id: R3, text: "severity trace span request session and resource filters preserve correlation", kind: functional, risk: high, verify: test }
  full_text: { id: R4, text: "embedded Lumen text search returns only retained matching log records", kind: functional, risk: high, verify: test }
  cursor_tail: { id: R5, text: "pagination and tail resume are stable by raw cursor", kind: reliability, risk: critical, verify: test }
  lag: { id: R6, text: "min cursor timeout returns shared projection lag and Retry After", kind: reliability, risk: high, verify: test }
  retention: { id: R7, text: "logging retention is independent and deterministic", kind: reliability, risk: high, verify: test }
  rebuild: { id: R8, text: "fresh raw rebuild equals the live logging projection", kind: reliability, risk: critical, verify: test }
  auth: { id: R9, text: "log reads enforce project scoped authorization", kind: security, risk: critical, verify: test }
  coexistence: { id: R10, text: "stable coexistence keys require no collector coupling", kind: compatibility, risk: high, verify: test }
elements:
  store_test: { kind: test, type: "rs/#[test]" }
  api_test: { kind: test, type: "rs/#[tokio::test]" }
relations:
  - { from: store_test, to: R1, type: verifies }
  - { from: store_test, to: R2, type: verifies }
  - { from: store_test, to: R3, type: verifies }
  - { from: store_test, to: R4, type: verifies }
  - { from: api_test, to: R5, type: verifies }
  - { from: api_test, to: R6, type: verifies }
  - { from: store_test, to: R7, type: verifies }
  - { from: store_test, to: R8, type: verifies }
  - { from: api_test, to: R9, type: verifies }
  - { from: store_test, to: R10, type: verifies }
---
graph LR
    store_test --> R1
    store_test --> R2
    store_test --> R3
    store_test --> R4
    api_test --> R5
    api_test --> R6
    store_test --> R7
    store_test --> R8
    api_test --> R9
    store_test --> R10
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/src/projection/logging.rs
    action: create
    section: schema
    impl_mode: hand-written
    gap: sift-logging-projection
    tracker: "1664"
    description: Define the log record/query/page schema, fixed-field embedded Lumen index, retention, snapshot, restore, and typed query behavior.
  - path: projects/sift/src/projection/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-projection-module
    tracker: "1660"
    description: Export logging projection contracts.
  - path: projects/sift/src/projection/runtime.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-projection-runtime
    tracker: "1660"
    description: Register the logging projection factory and expose typed read access without a second service boundary.
  - path: projects/sift/src/lib.rs
    action: modify
    section: rest-api
    impl_mode: hand-written
    gap: sift-service-core
    tracker: "1576"
    description: Add project-authorized logs query/tail routes, shared min-cursor lag mapping, and OpenAPI schemas.
  - path: projects/sift/tests/logging_store.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-logging-store-tests
    tracker: "1664"
    description: Verify golden GCP/OTel records, filters, text, retention, coexistence identity, and rebuild equality.
  - path: projects/sift/tests/logging_api.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-logging-api-tests
    tracker: "1664"
    description: Verify typed query, stable tail resume, min cursor lag, and project read authorization.
```
