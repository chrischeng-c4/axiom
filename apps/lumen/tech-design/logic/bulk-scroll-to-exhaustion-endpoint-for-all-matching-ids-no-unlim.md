---
id: '1783'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: search-all-external-ids
entry: request
nodes:
  request: { kind: start, label: "POST search:all with query, optional sort and routing key" }
  authorize: { kind: process, label: "Apply normal read authorization and consistency checks" }
  evaluate: { kind: process, label: "Evaluate complete result through existing local or routed search backend" }
  snapshot: { kind: process, label: "Local read lock snapshot; routed request uses one snapshot per shard" }
  project: { kind: process, label: "Project hits to external_ids with exact total and timing" }
  done: { kind: terminal, label: "Generated-client-visible complete result" }
edges:
  - { from: request, to: authorize }
  - { from: authorize, to: evaluate }
  - { from: evaluate, to: snapshot }
  - { from: snapshot, to: project }
  - { from: project, to: done }
---
flowchart TD
    request([POST search:all]) --> authorize[normal read authorization]
    authorize --> evaluate[complete local or routed search]
    evaluate --> snapshot[document truthful snapshot boundary]
    snapshot --> project[external_ids + exact total + timing]
    project --> done([generated client operation])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/types.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add dedicated search-all request/response wire types with explicit consistency documentation."
  - path: apps/lumen/src/api.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Register POST search:all, reuse authorization/routing, request an exact complete result and project external IDs."
  - path: apps/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Advertise the explicit-cost complete-ID operation and consistency boundary."
  - path: apps/lumen/tests/search_all_e2e.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Prove completeness beyond the default page, sort/filter semantics and generated OpenAPI operation."
  - path: apps/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Lock offline search-all and canonical OpenAPI metadata."
```
