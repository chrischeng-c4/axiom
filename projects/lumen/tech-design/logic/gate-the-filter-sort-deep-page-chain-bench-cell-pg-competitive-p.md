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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: filter-sort-deep-page-unit-verification
requirements:
  bench_cli_accepts_sorted_page_deep:
    id: R1
    text: "`lumen-bench run --types sorted_page_deep` executes the keyset deep-page cell and reports p50/p99"
    kind: functional
    risk: high
    verify: test
  competitive_gate_knows_sorted_page_deep:
    id: R2
    text: "perf_gate_vs_db includes sorted_page_deep with a pg OFFSET peer and ratcheted baseline entry"
    kind: design-constraint
    risk: high
    verify: test
  readme_lists_depth_invariant_gate:
    id: R3
    text: "README performance/search-core inventory names the depth-invariant pagination gate and evidence"
    kind: design-constraint
    risk: medium
    verify: inspection
elements:
  test_lumen_bench_sorted_page_deep_smoke:
    kind: test
    type: "rs/#[test]"
  test_competitive_gate_sorted_page_deep_registered:
    kind: test
    type: "rs/#[test]"
  test_readme_gate_inventory_updated:
    kind: test
    type: "inspection"
relations:
  - { from: test_lumen_bench_sorted_page_deep_smoke, verifies: bench_cli_accepts_sorted_page_deep }
  - { from: test_competitive_gate_sorted_page_deep_registered, verifies: competitive_gate_knows_sorted_page_deep }
  - { from: test_readme_gate_inventory_updated, verifies: readme_lists_depth_invariant_gate }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "lumen-bench sorted_page_deep runs"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "competitive gate includes pg OFFSET cell"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "README lists depth-invariant gate"
      risk: medium
      verifymethod: inspection
    }
    element test_lumen_bench_sorted_page_deep_smoke {
      type: "rs/#[test]"
    }
    test_lumen_bench_sorted_page_deep_smoke - verifies -> R1
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: rig-data-table-browse
    name: "rig data-table browse"
    runner: rig
    path: projects/lumen/tests/rig/cases/load/data_table_browse.toml
    command: "cd projects/lumen && ../../target/debug/rig run --dir tests/rig/cases/load --case data_table_browse"
    verifies:
      - "Seeds a browse corpus over the HTTP API."
      - "Pages through filter+sort cursor results to exhaustion."
      - "Asserts page concatenation matches a one-shot sorted oracle."
      - "Asserts deep-page p99 stays within the shallow-page tolerance."
  - id: vat-rig-data-table-browse
    name: "vat rig data-table browse"
    runner: vat
    path: projects/lumen/vat.toml
    command: "cd projects/lumen && ../../target/debug/vat run rig-load"
    verifies:
      - "The load runner includes the data_table_browse rig case when vat provisions the lumen service."
  - id: vat-ec-efficiency-meter
    name: "vat efficiency meter"
    runner: vat
    path: projects/lumen/vat.toml
    command: "cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter"
    verifies:
      - "The strict competitive gate can be run in a vat-owned pg/OpenSearch environment."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Register the lumen-bench binary used by the issue #10 bench and vat meter-profile runner."
  - path: projects/lumen/src/bin/lumen-bench.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Implement `lumen-bench run --types sorted_page_deep` with keyset deep-page p50/p99 reporting."
  - path: projects/lumen/tests/lumen_bench_cli.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Smoke-test the sorted_page_deep bench CLI and output fields."
  - path: projects/lumen/tests/perf_gate_vs_db.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the sorted_page_deep competitive cell with a Postgres OFFSET peer query at the same depth."
  - path: projects/lumen/tests/perf-baseline.json
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Record the ratcheted sorted_page_deep pg baseline entry."
  - path: projects/lumen/tests/rig/cases/load/data_table_browse.toml
    action: create
    section: e2e-test
    impl_mode: hand-written
    description: "Add the HTTP browse rig scenario for filter+sort cursor exhaustion and latency flatness."
  - path: projects/lumen/vat.toml
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: "Keep vat load/meter runners aligned with the new browse and lumen-bench surfaces."
  - path: projects/lumen/README.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: "Add the depth-invariant pagination promise and gate inventory row."
```
