---
id: '1892'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-operational-hygiene
entry: operational_surface
nodes:
  metrics: { kind: start, label: "Render one consistent pool stats snapshot with escaped Prometheus labels." }
  docs: { kind: process, label: "Serve a self-contained offline admin documentation page." }
  drain: { kind: process, label: "On completed drain, release static allocation and every reserve grant owned by that Pod." }
  batch: { kind: decision, label: "Does static admission batch contain duplicate Pod identities?" }
  reject: { kind: process, label: "Reject duplicate batch before mutating allocation state." }
  done: { kind: terminal, label: "Operational output stays parseable, truthful, offline, and leak-free." }
edges:
  - { from: metrics, to: docs }
  - { from: docs, to: drain }
  - { from: drain, to: batch }
  - { from: batch, to: reject, label: "yes" }
  - { from: batch, to: done, label: "no" }
  - { from: reject, to: done }
---
flowchart TD
    metrics([Escape labels and snapshot once]) --> docs[Serve offline docs]
    docs --> drain[Drain releases Pod reserve grants]
    drain --> batch{Duplicate Pod in batch?}
    batch -->|yes| reject[Reject atomically]
    batch -->|no| done([Operational surface is reliable])
    reject --> done
```

Control-plane Prometheus labels use the shared metrics renderer's escaping rule; admin metrics capture each pool's stats exactly once per render. `/docs` is a self-contained offline HTML index to the same `/openapi.json` contract. Completed drain reaps every grant whose key belongs to the drained Pod only after static allocation releases. Static batch admission rejects a duplicate Pod before calculating or inserting allocation state.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: libs/metrics-prometheus/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: render_labeled
    reason: Expose the shared Prometheus label-value escaping primitive for custom metric families.
  - path: apps/pgpool/src/k8s/control.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: prometheus
    reason: Escape endpoint and Pod labels and reap Pod-owned reserve grants on completed drain.
  - path: apps/pgpool/src/k8s/reserve.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: release_after_close
    reason: Add endpoint-ledger per-Pod reserve grant reaping after physical drain completion.
  - path: apps/pgpool/src/k8s/budget.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: reserve_many
    reason: Reject duplicate Pod identities within one static admission batch before state mutation.
  - path: apps/pgpool/src/admin/metrics.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: render
    reason: Capture one BackendPool stats snapshot per pool per Prometheus render.
  - path: apps/pgpool/src/admin/handlers.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: docs
    reason: Replace remote Swagger CDN assets with a self-contained offline documentation page.
  - path: apps/pgpool/src/admin/router.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: docs_serves_swagger_ui_html_referencing_openapi_json
    reason: Verify docs HTML has no external network dependency.
```
