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
