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
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: search-all-external-ids-verification
requirements:
  complete_ids:
    id: R1
    text: "search:all returns every matching external ID exactly once beyond the normal page size."
    kind: functional
    risk: high
    verify: cargo test -p lumen --test search_all_e2e -- --nocapture
  generated_client_operation:
    id: R3
    text: "Canonical OpenAPI exposes a typed search_all operation for generated clients."
    kind: contract
    risk: medium
    verify: cargo test -p lumen --test spec_cli -- --nocapture
  semantic_parity:
    id: R2
    text: "Filter and sort results match ordinary exact search semantics and disclose local versus routed snapshot boundaries."
    kind: integration
    risk: high
    verify: cargo test -p lumen --test search_all_e2e -- --nocapture
---
flowchart TD
    r1[R1 complete ids] --> cargo_test_p_lumen_test_search_all_e2e_nocapture[cargo test -p lumen --test search_all_e2e -- --nocapture]
    r2[R2 semantic parity] --> cargo_test_p_lumen_test_search_all_e2e_nocapture
    r3[R3 generated client operation] --> cargo_test_p_lumen_test_spec_cli_nocapture[cargo test -p lumen --test spec_cli -- --nocapture]
```
