---
id: lumen-kw-term-early-stop-claim
summary: >
  Close issue #45 by claiming the current standalone keyword-term planner:
  `storage::try_plan` already serves no-sort `Term` queries from the posting
  window (`take(limit)`) while deriving total from the bitmap cardinality, so a
  high-selectivity `kw_term` page no longer materializes every matching hit or
  sorts a full scored map. The slice records the claim and perf evidence rather
  than changing the hot path.
capability_refs:
  - id: "competitor-performance"
    role: primary
    claim: "competitive-regression-gate-beat-pg-os-per-cell-ratcheting"
    coverage: partial
    rationale: >
      The reported regression is a competitor-performance cell tracked by
      perf_gate_vs_db and the vat ec-efficiency-meter runner.
  - id: "search-core"
    role: contributes
    claim: "filter-sort-early-termination"
    coverage: partial
    rationale: >
      The fix belongs to search-core planner early termination for exact term
      pages.
fill_sections: [logic, unit-test, e2e-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: kw-term-early-stop-applicability
entry: start
nodes:
  start:    { kind: start, label: "issue #45: kw_term must not materialize every high-selectivity hit" }
  inspect:  { kind: process, label: "Inspect storage planner for standalone Term queries" }
  planner:  { kind: process, label: "Use posting-window path: first limit docids + posting len total" }
  evidence: { kind: process, label: "Verify with perf_gate and vat ec-efficiency-meter" }
  close:    { kind: terminal, label: "Close stale perf regression with TD evidence" }
edges:
  - { from: start, to: inspect }
  - { from: inspect, to: planner }
  - { from: planner, to: evidence }
  - { from: evidence, to: close }
---
flowchart TD
    start([#45 high-selectivity kw_term regression]) --> inspect[Inspect storage::try_plan]
    inspect --> planner[Standalone Term planner returns posting.take(limit) + posting.len]
    planner --> evidence[perf_gate + vat ec-efficiency-meter evidence]
    evidence --> close([Issue closed by existing implementation claim])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: kw-term-early-stop-unit-evidence
requirements:
  standalone_term_uses_posting_window:
    id: R1
    text: "No-sort keyword Term queries return the first limit entries directly from the posting list"
    kind: design-constraint
    risk: high
    verify: inspection
  high_selectivity_term_has_latency_floor:
    id: R2
    text: "Term query latency stays covered by the lumen perf gate"
    kind: performance
    risk: medium
    verify: test
  competitor_gate_covers_kw_term:
    id: R3
    text: "The release competitive gate reports kw_term under pg-native/OpenSearch peer evidence"
    kind: performance
    risk: high
    verify: test
elements:
  storage_try_plan_term_page:
    kind: source
    path: projects/lumen/src/storage.rs
  test_term_query_latency_floor:
    kind: test
    path: projects/lumen/tests/perf_gate.rs
  test_competitive_perf_gate:
    kind: test
    path: projects/lumen/tests/perf_gate_vs_db.rs
relations:
  - { from: storage_try_plan_term_page, verifies: standalone_term_uses_posting_window }
  - { from: test_term_query_latency_floor, verifies: high_selectivity_term_has_latency_floor }
  - { from: test_competitive_perf_gate, verifies: competitor_gate_covers_kw_term }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "standalone keyword Term uses posting window"
      risk: high
      verifymethod: inspection
    }
    requirement R2 {
      id: R2
      text: "term latency floor remains covered"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "kw_term has peer perf evidence"
      risk: high
      verifymethod: test
    }
    element storage_try_plan_term_page {
      type: "rs/function"
    }
    element test_term_query_latency_floor {
      type: "rs/#[test]"
    }
    element test_competitive_perf_gate {
      type: "rs/#[test]"
    }
    storage_try_plan_term_page - verifies -> R1
    test_term_query_latency_floor - verifies -> R2
    test_competitive_perf_gate - verifies -> R3
```
