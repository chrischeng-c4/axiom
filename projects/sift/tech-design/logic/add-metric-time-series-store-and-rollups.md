---
id: "1667"
summary: Materialize direct metric signals into Sift-owned cardinality-bounded time-series chunks with OTel temporality, histogram, exemplar, reset, late-point, rollup, query, checkpoint, and raw-rebuild semantics.
capability_refs:
  - id: materialized-observability-stores
    role: primary
    gap: metric-store-direct-points-and-exemplars
    claim: metric-store-direct-points-and-exemplars
    coverage: full
    rationale: This slice owns metric identity, chunks, temporality, histograms, exemplars, cardinality overflow, rollups, query, and rebuild equality.
  - id: slo-and-error-budget
    role: contributes
    gap: sli-objective-and-error-budget
    claim: sli-objective-and-error-budget
    coverage: partial
    rationale: Stable typed metric aggregation is the input contract for later SLI and burn-rate evaluation.
fill_sections: [logic, schema, rest-api, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-metric-store-contract
entry: metric
nodes:
  metric: { kind: start, label: "validated direct metric signal" }
  identity: { kind: process, label: "hash canonical resource attributes name unit temporality" }
  budget: { kind: decision, label: "new identity within project budget" }
  normal: { kind: process, label: "retain exact series identity" }
  overflow: { kind: process, label: "route to deterministic overflow identity and diagnostic" }
  point: { kind: process, label: "insert timestamp cursor ordered point" }
  semantics: { kind: process, label: "reset histogram exemplar and fixed-window rollup semantics" }
  checkpoint: { kind: terminal, label: "independent durable checkpoint" }
  query: { kind: start, label: "typed bounded metric query" }
  aggregate: { kind: process, label: "gauge last delta sum cumulative increase histogram merge" }
  page: { kind: terminal, label: "stable page and projection cursor" }
edges:
  - { from: metric, to: identity }
  - { from: identity, to: budget }
  - { from: budget, to: normal, when: yes }
  - { from: budget, to: overflow, when: no }
  - { from: normal, to: point }
  - { from: overflow, to: point }
  - { from: point, to: semantics }
  - { from: semantics, to: checkpoint }
  - { from: query, to: aggregate }
  - { from: aggregate, to: page }
---
flowchart LR
    metric([metric]) --> identity[series identity] --> budget{cardinality}
    budget -->|yes| normal[exact identity]
    budget -->|no| overflow[overflow identity]
    normal --> point[ordered point]
    overflow --> point --> semantics[metric semantics] --> checkpoint([checkpoint])
    query([query]) --> aggregate[typed aggregate] --> page([page])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
constants:
  projection: metric-store
  schema_version: 1
  chunk_points: 256
  rollup_windows_seconds: [60, 3600]
enums:
  MetricAggregation: [raw, sum, avg, min, max, count, rate]
  HistogramKind: [explicit, exponential]
schemas:
  - name: MetricHistogramV1
    fields:
      - { name: kind, type: HistogramKind, required: true }
      - { name: count, type: u64, required: true }
      - { name: sum, type: f64, required: true }
      - { name: explicit_bounds, type: "Vec<f64>", required: false }
      - { name: bucket_counts, type: "Vec<u64>", required: true }
      - { name: scale, type: i32, required: false }
      - { name: zero_count, type: u64, required: false }
      - { name: positive_offset, type: i32, required: false }
      - { name: negative_offset, type: i32, required: false }
  - name: MetricPointV1
    fields:
      - { name: cursor, type: u64, required: true }
      - { name: occurred_at, type: String, required: true }
      - { name: value, type: f64, required: true }
      - { name: histogram, type: MetricHistogramV1, required: false }
      - { name: exemplars, type: "Vec<MetricExemplar>", required: true }
  - name: MetricSeriesResultV1
    fields:
      - { name: series_id, type: String, required: true }
      - { name: project, type: String, required: true }
      - { name: name, type: String, required: true }
      - { name: temporality, type: MetricTemporality, required: true }
      - { name: resource, type: map, required: true }
      - { name: attributes, type: map, required: true }
      - { name: overflow, type: bool, required: true }
      - { name: points, type: "Vec<MetricPointV1>", required: true }
      - { name: aggregate, type: f64, required: false }
      - { name: histogram, type: MetricHistogramV1, required: false }
      - { name: reset_count, type: u64, required: true }
      - { name: rollups, type: "Vec<MetricRollupV1>", required: true }
cardinality:
  identity: canonical SHA256 of project, metric name, unit, temporality, resource, and attributes
  overflow: one deterministic project and metric scoped series; raw event remains unchanged
```

## REST API
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info: { title: Sift Metric Store Slice, version: 1.0.0 }
paths:
  /v1/metrics:query:
    post:
      operationId: queryMetrics
      summary: Query bounded metric series, points, rollups, and typed aggregates.
      responses:
        "200": { description: Stable metric series page. }
        "400": { description: Invalid filter, aggregation, or time range. }
        "403": { description: Project read denied. }
        "503": { description: Shared projection_lag. }
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sift-metric-store-verification
requirements:
  identity: { id: R1, text: "Canonical dimensions create stable series identities", kind: functional, risk: critical, verify: test }
  temporality: { id: R2, text: "Gauge delta and cumulative/reset aggregates are correct", kind: functional, risk: critical, verify: test }
  histogram: { id: R3, text: "Compatible explicit and exponential histograms merge deterministically", kind: functional, risk: high, verify: test }
  exemplar: { id: R4, text: "Exemplar trace and span correlation survives query and rebuild", kind: functional, risk: high, verify: test }
  overflow: { id: R5, text: "Cardinality budget routes new identities to a stable overflow series with diagnostics", kind: reliability, risk: critical, verify: test }
  late: { id: R6, text: "Late points reorder by event time without changing cursor identity", kind: reliability, risk: high, verify: test }
  rollup: { id: R7, text: "Minute and hour rollups are deterministic across restart", kind: reliability, risk: high, verify: test }
  rebuild: { id: R8, text: "Raw rebuild equals the live metric projection", kind: reliability, risk: critical, verify: test }
  api: { id: R9, text: "Typed query enforces project auth pagination and projection lag", kind: security, risk: high, verify: test }
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
  - { from: store_test, to: R7, type: verifies }
  - { from: store_test, to: R8, type: verifies }
  - { from: api_test, to: R9, type: verifies }
---
graph LR
    store_test --> R1
    store_test --> R2
    store_test --> R3
    store_test --> R4
    store_test --> R5
    store_test --> R6
    store_test --> R7
    store_test --> R8
    api_test --> R9
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - { path: projects/sift/src/projection/metric.rs, action: create, section: schema, impl_mode: hand-written, gap: sift-metric-projection, tracker: "1667", description: "Define series identity, chunks, temporality, histograms, exemplars, overflow, rollups, typed query, snapshot, and rebuild." }
  - { path: projects/sift/src/projection/mod.rs, action: modify, section: logic, impl_mode: hand-written, gap: sift-projection-module, tracker: "1660", description: "Export the metric projection and query contracts." }
  - { path: projects/sift/src/projection/runtime.rs, action: modify, section: logic, impl_mode: hand-written, gap: sift-projection-runtime, tracker: "1660", description: "Register the independent metric projection and typed query." }
  - { path: projects/sift/src/lib.rs, action: modify, section: rest-api, impl_mode: hand-written, gap: sift-service-core, tracker: "1576", description: "Add the authorized typed metric query route." }
  - { path: projects/sift/tests/metric_store.rs, action: create, section: unit-test, impl_mode: hand-written, gap: sift-metric-store-tests, tracker: "1667", description: "Verify temporality, resets, histograms, exemplars, overflow, late points, rollups, and rebuild equality." }
  - { path: projects/sift/tests/metric_api.rs, action: create, section: unit-test, impl_mode: hand-written, gap: sift-metric-api-tests, tracker: "1667", description: "Verify authorized typed query, pagination, and projection lag." }
```
