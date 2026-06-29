---
id: lumen-filter-sort-deep-page-gate
summary: >
  Gate the filter + sort + deep-page search chain so keyset cursor pagination
  stays depth-invariant and cannot regress back to O(offset) page walks. The
  slice adds a `sorted_page_deep` lumen-bench cell, wires it into the
  Postgres/OpenSearch competitive gate, adds a rig data-table browse scenario,
  and records the gate in the README capability inventory.
capability_refs:
  - id: "competitor-performance"
    role: primary
    claim: "competitive-regression-gate-beat-pg-os-per-cell-ratcheting"
    coverage: partial
    rationale: >
      The filter + sort + cursor path is the performance promise issue #10
      exists to protect.
  - id: "search-core"
    role: contributes
    claim: "filter-sort-early-termination"
    coverage: partial
    rationale: >
      The same sorted-keyset planner must stay correct and fast for exact
      search browse flows.
fill_sections: [logic, unit-test, e2e-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: filter-sort-deep-page-gate
entry: start
nodes:
  start:      { kind: start,    label: "issue #10: filter + sort + deep page must be gated" }
  bench:      { kind: process,  label: "lumen-bench sorted_page_deep builds corpus and walks to depth via keyset cursors" }
  gate:       { kind: process,  label: "perf_gate_vs_db measures sorted_page_deep vs pg OFFSET at same depth" }
  rig:        { kind: process,  label: "rig data_table_browse drives HTTP filter+sort cursor paging to exhaustion" }
  docs:       { kind: process,  label: "README capability inventory names depth-invariant pagination gate" }
  verify:     { kind: terminal, label: "cargo/rig/vat evidence prevents O(offset) regression" }
edges:
  - { from: start, to: bench }
  - { from: bench, to: gate }
  - { from: gate,  to: rig }
  - { from: rig,   to: docs }
  - { from: docs,  to: verify }
---
flowchart TD
    start([#10 filter + sort + deep page gate]) --> bench[lumen-bench: sorted_page_deep keyset walk]
    bench --> gate[perf_gate_vs_db: compare pg OFFSET at same depth]
    gate --> rig[rig: HTTP data-table browse cursor exhaustion]
    rig --> docs[README: depth-invariant pagination gate row]
    docs --> verify([regression is gated])
```
