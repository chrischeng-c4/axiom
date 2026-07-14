---
id: "1669"
summary: Store OTel profile payloads in durable content-addressed blobs before raw acknowledgement, materialize bounded profile metadata and stack samples, and expose deterministic flamegraph, top-function, diff, trace-correlation, retention, and rebuild views.
capability_refs:
  - id: profile-observability
    role: primary
    gap: otel-profile-ingest-and-blob-durability
    claim: otel-profile-ingest-and-blob-durability
    coverage: full
    rationale: This slice owns profile blob externalization, reference validation, bounded journal metadata, and blob-before-ack proof.
  - id: profile-observability
    role: primary
    gap: flamegraph-top-functions-and-diff
    claim: flamegraph-top-functions-and-diff
    coverage: full
    rationale: The dedicated profile projection provides deterministic stack aggregation, inclusive/self totals, and comparison deltas.
  - id: profile-observability
    role: primary
    gap: profile-trace-correlation-and-rebuild
    claim: profile-trace-correlation-and-rebuild
    coverage: full
    rationale: Profile records preserve trace/span/resource context, apply retention, checkpoint independently, and rebuild from raw plus durable blobs.
  - id: materialized-observability-stores
    role: contributes
    gap: profile-store-and-analysis
    claim: profile-store-and-analysis
    coverage: full
    rationale: Profiles become an independently rebuildable first-class store with typed analysis queries.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-profile-observability-contract
entry: upload
nodes:
  upload: { kind: start, label: "OTel profile payload and metadata" }
  blob: { kind: process, label: "validate bounds then durably store content-addressed blob" }
  event: { kind: process, label: "write bounded profile event with hash size encoding and correlations" }
  commit: { kind: process, label: "Raft commit and durable raw cursor" }
  projection: { kind: process, label: "materialize samples functions locations mappings labels and time range" }
  analysis: { kind: terminal, label: "flamegraph top functions diff and trace correlation" }
  rebuild: { kind: start, label: "raw replay" }
  verify: { kind: decision, label: "referenced durable blob exists and digest matches" }
  reject: { kind: terminal, label: "typed missing or corrupt blob error" }
edges:
  - { from: upload, to: blob }
  - { from: blob, to: event }
  - { from: event, to: commit }
  - { from: commit, to: projection }
  - { from: projection, to: analysis }
  - { from: rebuild, to: verify }
  - { from: verify, to: projection, label: "yes" }
  - { from: verify, to: reject, label: "no" }
---
flowchart LR
    upload([profile]) --> blob[durable blob] --> event[bounded event] --> commit[commit] --> projection[profile view] --> analysis([analysis])
    rebuild([raw replay]) --> verify{blob valid}
    verify -->|yes| projection
    verify -->|no| reject([typed error])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
constants:
  projection: profile-store
  schema_version: 1
  record_schema: sift.profile.v1
  default_retention_days: 30
schemas:
  - name: ProfileStackSampleV1
    fields:
      - { name: frames, type: Vec<String>, required: true }
      - { name: value, type: f64, required: true }
      - { name: labels, type: Map<String,String>, required: true }
  - name: ProfileRecordV1
    fields:
      - { name: cursor, type: u64, required: true }
      - { name: profile_id, type: String, required: true }
      - { name: project, type: String, required: true }
      - { name: start_time, type: String, required: true }
      - { name: end_time, type: String, required: true }
      - { name: sample_type, type: String, required: true }
      - { name: unit, type: String, required: true }
      - { name: samples, type: Vec<ProfileStackSampleV1>, required: true }
      - { name: blob, type: ContentBlobRef, required: false }
      - { name: trace_id, type: String, required: false }
      - { name: span_id, type: String, required: false }
      - { name: retention_expires_at, type: String, required: true }
  - name: ProfileFunctionValueV1
    fields:
      - { name: function, type: String, required: true }
      - { name: inclusive, type: f64, required: true }
      - { name: self_value, type: f64, required: true }
      - { name: delta, type: f64, required: false }
  - name: ProfileQuery
    fields:
      - { name: project, type: String, required: true }
      - { name: view, type: 'list|flamegraph|top_functions|diff', required: true }
      - { name: profile_id, type: String, required: false }
      - { name: baseline_profile_id, type: String, required: false }
      - { name: comparison_profile_id, type: String, required: false }
      - { name: trace_id, type: String, required: false }
      - { name: span_id, type: String, required: false }
      - { name: min_cursor, type: u64, required: false }
durability:
  order: blob fsync -> bounded metadata event validation -> Raft/raw commit -> acknowledgement
  raw_rule: profile payloads at or above the configured threshold are replaced by one hash/size/encoding reference before raw append
  rebuild_rule: every referenced blob must exist and pass SHA-256 verification before its record is projected
```

## REST API
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info: { title: Sift Profile Observability Slice, version: 1.0.0 }
paths:
  /v1/profiles:
    post:
      operationId: ingestOtlpProfiles
      responses:
        "200": { description: OTLP partial-success after blob and raw durability. }
        "400": { description: Invalid profile or missing/corrupt blob reference. }
        "413": { description: Bounded request limit exceeded. }
  /v1/profiles:query:
    post:
      operationId: queryProfiles
      responses:
        "200": { description: Authorized list, flamegraph, top-functions, or diff response. }
        "403": { description: Project read denied. }
        "503": { description: Shared projection_lag with current cursor and Retry-After. }
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sift-profile-observability-verification
requirements:
  golden: { id: R1, text: "OTel JSON profile fixture preserves functions stacks labels time range and sample unit", kind: functional, risk: high, verify: test }
  blob: { id: R2, text: "Large payload blob is durable before the raw cursor is acknowledged", kind: reliability, risk: critical, verify: test }
  missing: { id: R3, text: "Missing or digest-mismatched referenced blobs fail projection and rebuild", kind: reliability, risk: critical, verify: test }
  bounded: { id: R4, text: "Raw journal contains bounded metadata and content reference instead of large payload bytes", kind: security, risk: high, verify: test }
  analysis: { id: R5, text: "Flamegraph top functions and profile diff are deterministic", kind: functional, risk: high, verify: test }
  correlation: { id: R6, text: "Profile records filter by trace span project resource and time", kind: functional, risk: high, verify: test }
  retention: { id: R7, text: "Expired profiles leave the hot view without deleting raw/blob source facts", kind: compliance, risk: high, verify: test }
  rebuild: { id: R8, text: "Raw plus blob replay equals the live profile projection", kind: reliability, risk: critical, verify: test }
elements:
  store_test: { kind: test, type: "rs/#[test]" }
  blob_test: { kind: test, type: "rs/#[test]" }
  api_test: { kind: test, type: "rs/#[tokio::test]" }
relations:
  - { from: store_test, to: R1, type: verifies }
  - { from: blob_test, to: R2, type: verifies }
  - { from: blob_test, to: R3, type: verifies }
  - { from: blob_test, to: R4, type: verifies }
  - { from: store_test, to: R5, type: verifies }
  - { from: api_test, to: R6, type: verifies }
  - { from: store_test, to: R7, type: verifies }
  - { from: store_test, to: R8, type: verifies }
---
graph LR
    store_test --> R1
    blob_test --> R2
    blob_test --> R3
    blob_test --> R4
    store_test --> R5
    api_test --> R6
    store_test --> R7
    store_test --> R8
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - { path: projects/sift/src/projection/profile.rs, action: create, section: schema, impl_mode: hand-written, gap: sift-profile-projection, tracker: "1669", description: "Define blob-backed profile records, OTel normalization, retention, typed analysis, snapshot, and rebuild semantics." }
  - { path: projects/sift/src/projection/mod.rs, action: modify, section: logic, impl_mode: hand-written, gap: sift-projection-module, tracker: "1660", description: "Export profile projection and query contracts." }
  - { path: projects/sift/src/projection/runtime.rs, action: modify, section: logic, impl_mode: hand-written, gap: sift-projection-runtime, tracker: "1660", description: "Register the independent profile projection with journal blob access and typed reads." }
  - { path: projects/sift/src/storage/blob.rs, action: modify, section: logic, impl_mode: hand-written, gap: sift-content-addressed-blob-store, tracker: "1659", description: "Externalize large complete profile payloads and validate referenced bytes before raw acknowledgement/rebuild." }
  - { path: projects/sift/src/storage/mod.rs, action: modify, section: logic, impl_mode: hand-written, gap: sift-sharded-storage-module, tracker: "1659", description: "Expose profile-safe content-addressed blob operations to the journal and projection." }
  - { path: projects/sift/src/lib.rs, action: modify, section: rest-api, impl_mode: hand-written, gap: sift-service-core, tracker: "1576", description: "Add project-scoped profile list/flamegraph/top/diff route with shared projection-lag contract." }
  - { path: projects/sift/tests/profile_store.rs, action: create, section: unit-test, impl_mode: hand-written, gap: sift-profile-store-tests, tracker: "1669", description: "Verify normalization, analysis, correlations, retention, snapshot, and raw rebuild equality." }
  - { path: projects/sift/tests/profile_blob_durability.rs, action: create, section: unit-test, impl_mode: hand-written, gap: sift-profile-blob-tests, tracker: "1669", description: "Verify blob-before-ack, bounded raw bytes, missing/corrupt rejection, and OTLP API authorization." }
```
