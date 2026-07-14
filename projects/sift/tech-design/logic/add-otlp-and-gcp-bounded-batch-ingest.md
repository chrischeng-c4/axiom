---
id: "1658"
summary: Add bounded JSON and OTLP HTTP ingestion, GCP structured-event normalization, ordered partial outcomes, and explicit project admission limits before the shared durable event path.
capability_refs:
  - id: operational-event-ingest
    role: primary
    gap: otlp-log-span-metric-profile-normalization
    claim: otlp-log-span-metric-profile-normalization
    coverage: full
    rationale: This slice owns the bounded transport, normalization, idempotency, quota, and partial-success contract for every v1 ingest route.
  - id: gcp-cloud-logging-compatibility
    role: contributes
    gap: severity-and-trace-context-normalization
    claim: severity-and-trace-context-normalization
    coverage: partial
    rationale: The ingest boundary preserves representative GCP jsonPayload, monitored-resource labels, severity, and trace correlation; the logging store completes query compatibility.
  - id: http2-api-list
    role: contributes
    gap: domain-v1-api-and-client-expansion
    claim: domain-v1-api-and-client-expansion
    coverage: partial
    rationale: These routes expand the served and offline OpenAPI surface; later domain query and operations routes complete the public API.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-otlp-gcp-batch-ingest
entry: client
nodes:
  client: { kind: start, label: "OTLP or GCP client" }
  auth: { kind: process, label: "project scoped authentication" }
  body: { kind: process, label: "bounded body and gzip decode" }
  decode: { kind: decision, label: "endpoint and content type?" }
  normalize: { kind: process, label: "normalize JSON or protobuf into OperationalEventV2" }
  admit: { kind: process, label: "project quota and concurrency admission" }
  govern: { kind: process, label: "validate schema and apply governance" }
  append: { kind: process, label: "append accepted items through RaftStateMachine" }
  batch: { kind: terminal, label: "ordered batch outcomes" }
  partial: { kind: terminal, label: "OTLP partial success" }
edges:
  - { from: client, to: auth }
  - { from: auth, to: body }
  - { from: body, to: decode }
  - { from: decode, to: normalize, label: "events, OTLP, or GCP" }
  - { from: normalize, to: admit }
  - { from: admit, to: govern }
  - { from: govern, to: append }
  - { from: append, to: batch, label: "events write" }
  - { from: append, to: partial, label: "OTLP" }
---
flowchart TD
    client([OTLP or GCP client]) --> auth[project scoped authentication]
    auth --> body[bounded body and gzip decode]
    body --> decode{endpoint and content type?}
    decode --> normalize[normalize JSON or protobuf into OperationalEventV2]
    normalize --> admit[project quota and concurrency admission]
    admit --> govern[validate schema and apply governance]
    govern --> append[append accepted items through RaftStateMachine]
    append -->|events write| batch([ordered batch outcomes])
    append -->|OTLP| partial([OTLP partial success])
```

The transport boundary is bounded and synchronous: authentication, decompression, decoding, normalization, admission, validation, governance, and durable append complete before an item is accepted. Invalid siblings do not block valid items. Every accepted signal becomes `OperationalEventV2` before the shared append path, so transport cannot bypass Raft, privacy policy, idempotency, or raw-journal durability.

## Schema
<!-- type: schema lang: yaml -->

```yaml
schemas:
  - name: EventWriteRequest
    fields:
      - { name: events, type: "Vec<JSON>", required: true, constraints: "1..=1000 items" }
  - name: BatchItemResult
    fields:
      - { name: index, type: usize, required: true }
      - { name: event_id, type: String, required: false }
      - { name: outcome, type: BatchOutcome, required: true }
      - { name: cursor, type: u64, required: false }
      - { name: error, type: ErrorDetail, required: false }
  - name: BatchOutcome
    variants: [accepted, duplicate, rejected]
  - name: EventWriteResponse
    fields:
      - { name: results, type: "Vec<BatchItemResult>", required: true }
      - { name: accepted, type: usize, required: true }
      - { name: duplicates, type: usize, required: true }
      - { name: rejected, type: usize, required: true }
  - name: OtlpPartialSuccess
    fields:
      - { name: rejected_items, type: u64, required: true }
      - { name: error_message, type: String, required: false }
  - name: OtlpExportResponse
    fields:
      - { name: partial_success, type: OtlpPartialSuccess, required: false }
limits:
  compressed_body_bytes: 1048576
  decoded_body_bytes: 8388608
  events_per_batch: 1000
  event_bytes: 262144
  concurrent_requests_per_project: 8
  admitted_items_per_project_window: 10000
normalization:
  otlp: resource attributes and instrumentation scope become V2 common fields; signal bodies remain typed metadata/payload
  gcp: jsonPayload stays structured; resource.type and resource.labels become V2 resource attributes; severity and trace/span/request correlations are preserved
  identity: supplied event ids are retained; OTLP ids are deterministic hashes of project, signal identity, and source timestamp
```

## Rest Api
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info:
  title: Sift bounded ingest API
  version: 1.0.0
paths:
  /v1/events:write:
    post:
      operationId: writeEvents
      requestBody:
        required: true
        content:
          application/json: { schema: { $ref: "#/components/schemas/EventWriteRequest" } }
      responses:
        "200": { description: Ordered accepted, duplicate, and rejected results }
        "413": { description: Compressed or decoded body limit exceeded }
        "429": { description: Project quota or concurrency admission rejected }
        "503": { description: Service draining or durable path overloaded }
  /v1/logs:
    post:
      operationId: exportLogs
      responses: { "200": { description: OTLP logs export response with optional partial success } }
  /v1/traces:
    post:
      operationId: exportTraces
      responses: { "200": { description: OTLP traces export response with optional partial success } }
  /v1/metrics:
    post:
      operationId: exportMetrics
      responses: { "200": { description: OTLP metrics export response with optional partial success } }
  /v1/profiles:
    post:
      operationId: exportProfiles
      responses: { "200": { description: OTLP profiles export response with optional partial success } }
components:
  schemas:
    EventWriteRequest: { type: object, required: [events] }
content:
  otlp_request_types: [application/json, application/x-protobuf]
  content_encoding: [identity, gzip]
  otlp_response: mirror request media type; protobuf responses use the OTLP partial-success wire shape
errors:
  envelope: shared service HTTP error with stable code, message, retryability, and optional Retry-After
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sift-otlp-gcp-ingest-verification
requirements:
  bounded_batch:
    id: R1
    text: "mixed-validity batches preserve order and return accepted duplicate and rejected outcomes"
    kind: functional
    risk: high
    verify: test
  otlp_wire:
    id: R2
    text: "logs traces metrics and profiles accept JSON protobuf and gzip with OTLP partial success"
    kind: compatibility
    risk: critical
    verify: test
  gcp_normalization:
    id: R3
    text: "GCP structured logs retain jsonPayload resource severity and trace context"
    kind: compatibility
    risk: high
    verify: test
  admission:
    id: R4
    text: "body event project quota and concurrency limits return stable retryable errors"
    kind: security
    risk: high
    verify: test
elements:
  batch_contract: { kind: test, type: "rs/#[tokio::test]" }
  otlp_golden: { kind: test, type: "rs/#[tokio::test]" }
  gcp_golden: { kind: test, type: "rs/#[test]" }
  admission_contract: { kind: test, type: "rs/#[tokio::test]" }
relations:
  - { from: batch_contract, verifies: bounded_batch }
  - { from: otlp_golden, verifies: otlp_wire }
  - { from: gcp_golden, verifies: gcp_normalization }
  - { from: admission_contract, verifies: admission }
---
requirementDiagram
    requirement R1 { id: R1 text: "ordered batch outcomes" risk: high verifymethod: test }
    requirement R2 { id: R2 text: "OTLP wire compatibility" risk: critical verifymethod: test }
    requirement R3 { id: R3 text: "GCP structured normalization" risk: high verifymethod: test }
    requirement R4 { id: R4 text: "explicit admission limits" risk: high verifymethod: test }
    element otlp_golden { type: "rs/#[tokio::test]" }
    otlp_golden - verifies -> R2
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/sift/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    gap: sift-ingest-wire-dependencies
    tracker: "1658"
    description: Add prost, gzip, bytes, hashing, and HTTP header dependencies required by the bounded OTLP transport.
  - path: projects/sift/src/ingest/mod.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-ingest-module
    tracker: "1658"
    description: Export the bounded ingest admission, batch, GCP, and OTLP semantic modules.
  - path: projects/sift/src/ingest/limits.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-ingest-admission-limits
    tracker: "1658"
    description: Enforce compressed and decoded sizes, event count and size, per-project quota, concurrency, draining, and overload errors.
  - path: projects/sift/src/ingest/batch.rs
    action: create
    section: schema
    impl_mode: hand-written
    gap: sift-bounded-event-batch
    tracker: "1658"
    description: Decode bounded event batches and report ordered accepted, duplicate, or rejected per-item outcomes.
  - path: projects/sift/src/ingest/gcp.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-gcp-structured-normalizer
    tracker: "1658"
    description: Normalize representative Cloud Logging structured JSON and GKE monitored resources into OperationalEventV2.
  - path: projects/sift/src/ingest/otlp/mod.rs
    action: create
    section: logic
    impl_mode: hand-written
    gap: sift-otlp-normalizer
    tracker: "1658"
    description: Decode signal endpoint payloads, dispatch wire normalization, and encode media-type-matched partial-success responses.
  - path: projects/sift/src/ingest/otlp/wire.rs
    action: create
    section: schema
    impl_mode: hand-written
    gap: sift-otlp-wire-types
    tracker: "1658"
    description: Define the bounded official OTLP protobuf wire subset shared by log, trace, metric, and profile normalization.
  - path: projects/sift/src/lib.rs
    action: modify
    section: rest-api
    impl_mode: hand-written
    gap: sift-bounded-ingest-routes
    tracker: "1658"
    description: Add events:write and four OTLP routes to the same authenticated ServiceState and OpenAPI boundary.
  - path: projects/sift/src/bin/sift.rs
    action: modify
    section: rest-api
    impl_mode: hand-written
    gap: sift-event-write-import-cli
    tracker: "1658"
    description: Expose event write and import commands with terminal or executable-next output while retaining legacy event invocation.
  - path: projects/sift/tests/otlp_gcp_ingest.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    gap: sift-otlp-gcp-ingest-tests
    tracker: "1658"
    description: Verify golden JSON/protobuf/gzip payloads, partial success, duplicates, auth, body/schema/quota, and overload behavior using the real router and journal.
```
