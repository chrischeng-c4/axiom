---
id: mamba-language-core-gate-closure-call-hot-with-perf-pin
summary: Bring release closure-call throughput to an enforceable CPython-relative perf gate without weakening closure cell semantics.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-closure-call-hot-applicability
entry: closure_call_hot release workload
nodes:
  call: { kind: start, label: closure call with one captured integer }
  module: { kind: process, label: callable module context dispatch }
  safepoint: { kind: process, label: gc safepoint and dynamic call checks }
  cells: { kind: process, label: install captured cell context }
  jit: { kind: process, label: invoke JIT frame }
  gate: { kind: terminal, label: CPU and RSS perf pin against CPython baseline }
  preserve: { kind: terminal, label: closure semantics and ownership preserved }
edges:
  - { from: call, to: module }
  - { from: module, to: safepoint }
  - { from: safepoint, to: cells }
  - { from: cells, to: jit }
  - { from: jit, to: gate }
  - { from: cells, to: preserve }
---
flowchart TD
    call([closure call with one captured integer]) --> module[callable module context dispatch]
    module --> safepoint[gc safepoint and dynamic call checks]
    safepoint --> cells[install captured cell context]
    cells --> jit[invoke JIT frame]
    jit --> gate([CPU and RSS perf pin against CPython baseline])
    cells --> preserve([closure semantics and ownership preserved])
```

Applicability is confined to release-mode repeated calls of a stable closure. The observed release workload is 3.96 user seconds for Mamba versus 0.10 for CPython. The candidate hot path is `mb_call1_val` plus `with_closure_cells`: it traverses module context, safepoint, closure lookup, and active-cell map setup for every call. The contract must measure and remove only redundant work while retaining per-call capture identity, exception behavior, and ownership.
