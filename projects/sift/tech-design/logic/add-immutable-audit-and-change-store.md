---
id: "1668"
summary: Materialize audit and change signals into a cursor-ordered per-project tamper-evident timeline with durable legal holds, retention precedence, scoped query, controlled export manifests, and audit evidence for every control mutation.
capability_refs:
  - id: security-audit-and-governance
    role: primary
    gap: immutable-audit-event-projection
    claim: immutable-audit-event-projection
    coverage: full
    rationale: This slice owns normalized records, hash-chain integrity, retention, legal holds, controlled export, auth, checkpoints, and rebuild equality.
  - id: security-audit-and-governance
    role: primary
    gap: change-event-causality-timeline
    claim: change-event-causality-timeline
    coverage: full
    rationale: Change records preserve causal trace, request, session, version, resource, and target context in the same ordered timeline.
  - id: security-audit-and-governance
    role: primary
    gap: audit-retention-hold-and-export-controls
    claim: audit-retention-hold-and-export-controls
    coverage: full
    rationale: Durable legal holds override hot retention and controlled exports are admin scoped, hashed, and audited.
  - id: materialized-observability-stores
    role: contributes
    gap: audit-and-change-store-timeline
    claim: audit-and-change-store-timeline
    coverage: full
    rationale: Audit and change become a dedicated independently rebuildable store over the raw journal.
  - id: security-hardening
    role: contributes
    gap: audit-event-retention-policy
    claim: audit-event-retention-policy
    coverage: full
    rationale: Project-scoped query, admin-only hold/export, retention precedence, and mutation evidence close the audit retention control boundary.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-audit-change-contract
entry: event
nodes:
  event: { kind: start, label: "durable audit or change event" }
  normalize: { kind: process, label: "normalize actor subject action target and correlations" }
  append: { kind: process, label: "append cursor ordered project hash-chain record" }
  checkpoint: { kind: terminal, label: "immutable projection checkpoint" }
  control: { kind: start, label: "admin hold or export request" }
  commit: { kind: process, label: "single Sift state machine commit" }
  evidence: { kind: process, label: "append audit and change control evidence" }
  output: { kind: terminal, label: "held timeline or content-hash export manifest" }
edges:
  - { from: event, to: normalize }
  - { from: normalize, to: append }
  - { from: append, to: checkpoint }
  - { from: control, to: commit }
  - { from: commit, to: evidence }
  - { from: evidence, to: output }
---
flowchart LR
    event([audit/change]) --> normalize[normalize] --> append[hash chain] --> checkpoint([checkpoint])
    control([hold/export]) --> commit[state machine] --> evidence[audit/change evidence] --> output([manifest])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
constants:
  projection: audit-change-store
  schema_version: 1
  record_schema: sift.audit.v1
  default_retention_days: 365
schemas:
  - name: AuditChangeRecordV1
    fields:
      - { name: cursor, type: u64, required: true }
      - { name: event_id, type: String, required: true }
      - { name: project, type: String, required: true }
      - { name: signal, type: SignalKind, required: true }
      - { name: actor, type: String, required: true }
      - { name: subject, type: String, required: false }
      - { name: action, type: String, required: true }
      - { name: target, type: String, required: false }
      - { name: payload, type: json, required: true }
      - { name: previous_hash, type: String, required: true }
      - { name: record_hash, type: String, required: true }
  - name: AuditLegalHoldV1
    fields:
      - { name: id, type: String, required: true }
      - { name: project, type: String, required: true }
      - { name: start_time, type: String, required: true }
      - { name: end_time, type: String, required: true }
      - { name: reason, type: String, required: true }
      - { name: actor, type: String, required: true }
      - { name: active, type: bool, required: true }
      - { name: commit_index, type: u64, required: true }
  - name: AuditExportManifestV1
    fields:
      - { name: id, type: String, required: true }
      - { name: project, type: String, required: true }
      - { name: record_count, type: u64, required: true }
      - { name: content_sha256, type: String, required: true }
      - { name: actor, type: String, required: true }
      - { name: commit_index, type: u64, required: true }
immutability:
  record_identity: event_id
  chain_scope: project
  retention: expired records remain in raw journal and are hidden from the hot view unless an active legal hold covers occurred_at
```

## REST API
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info: { title: Sift Audit And Change Store Slice, version: 1.0.0 }
paths:
  /v1/audit:query:
    post:
      operationId: queryAuditTimeline
      responses:
        "200": { description: Authorized immutable timeline page. }
        "403": { description: Project read denied. }
        "503": { description: Shared projection_lag. }
  /v1/audit/holds/{id}:
    put:
      operationId: upsertAuditLegalHold
      responses:
        "200": { description: Durable admin-only hold and audit/change evidence. }
    delete:
      operationId: releaseAuditLegalHold
      responses:
        "200": { description: Durable hold release and audit/change evidence. }
  /v1/audit:export:
    post:
      operationId: exportAuditTimeline
      responses:
        "200": { description: Bounded records plus committed content-hash manifest. }
        "403": { description: Project admin denied. }
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sift-audit-change-verification
requirements:
  normalize: { id: R1, text: "Actor subject action target resource and correlations are normalized", kind: functional, risk: high, verify: test }
  immutable: { id: R2, text: "Duplicate mutation cannot replace an existing event record", kind: security, risk: critical, verify: test }
  chain: { id: R3, text: "Per-project hash chain detects record or ordering tampering", kind: security, risk: critical, verify: test }
  hold: { id: R4, text: "Active legal hold overrides hot retention and release restores expiry", kind: compliance, risk: critical, verify: test }
  auth: { id: R5, text: "Reads are project scoped and hold export require admin", kind: security, risk: critical, verify: test }
  export: { id: R6, text: "Controlled export has stable records count hash actor and audit evidence", kind: compliance, risk: high, verify: test }
  correlation: { id: R7, text: "Change versions correlate with trace request session and resource", kind: functional, risk: high, verify: test }
  rebuild: { id: R8, text: "Raw rebuild equals the live audit/change projection", kind: reliability, risk: critical, verify: test }
elements:
  store_test: { kind: test, type: "rs/#[test]" }
  api_test: { kind: test, type: "rs/#[tokio::test]" }
relations:
  - { from: store_test, to: R1, type: verifies }
  - { from: store_test, to: R2, type: verifies }
  - { from: store_test, to: R3, type: verifies }
  - { from: api_test, to: R4, type: verifies }
  - { from: api_test, to: R5, type: verifies }
  - { from: api_test, to: R6, type: verifies }
  - { from: store_test, to: R7, type: verifies }
  - { from: store_test, to: R8, type: verifies }
---
graph LR
    store_test --> R1
    store_test --> R2
    store_test --> R3
    api_test --> R4
    api_test --> R5
    api_test --> R6
    store_test --> R7
    store_test --> R8
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - { path: projects/sift/src/projection/audit_change.rs, action: create, section: schema, impl_mode: hand-written, gap: sift-audit-change-projection, tracker: "1668", description: "Define normalized hash-chained records, retention/hold query, export records, snapshot, integrity verification, and rebuild." }
  - { path: projects/sift/src/projection/model.rs, action: modify, section: schema, impl_mode: hand-written, gap: sift-projection-model, tracker: "1660", description: "Persist legal holds and export manifests in the durable control state." }
  - { path: projects/sift/src/projection/mod.rs, action: modify, section: logic, impl_mode: hand-written, gap: sift-projection-module, tracker: "1660", description: "Export audit/change projection and control contracts." }
  - { path: projects/sift/src/projection/runtime.rs, action: modify, section: logic, impl_mode: hand-written, gap: sift-projection-runtime, tracker: "1660", description: "Register the independent audit/change projection and typed reads." }
  - { path: projects/sift/src/durability.rs, action: modify, section: logic, impl_mode: hand-written, gap: sift-framed-journal-state-machine, tracker: "1605", description: "Commit legal holds and export manifests and append control evidence through the one state machine." }
  - { path: projects/sift/src/lib.rs, action: modify, section: rest-api, impl_mode: hand-written, gap: sift-service-core, tracker: "1576", description: "Add scoped audit query and admin-only hold/export routes." }
  - { path: projects/sift/tests/audit_change_store.rs, action: create, section: unit-test, impl_mode: hand-written, gap: sift-audit-change-store-tests, tracker: "1668", description: "Verify normalization, immutability, chain integrity, correlation, and rebuild equality." }
  - { path: projects/sift/tests/audit_change_api.rs, action: create, section: unit-test, impl_mode: hand-written, gap: sift-audit-change-api-tests, tracker: "1668", description: "Verify retention/hold precedence, scoped authorization, controlled export, and mutation evidence." }
```
