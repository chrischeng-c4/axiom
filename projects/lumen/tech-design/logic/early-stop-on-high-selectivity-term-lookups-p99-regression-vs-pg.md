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
id: kw-term-early-stop-contract
entry: request
nodes:
  request: { kind: start, label: "POST /search Term(city=taipei), no sort, first page" }
  plan:    { kind: process, label: "try_plan detects standalone Term" }
  page:    { kind: process, label: "Read posting iterator and take limit docids" }
  total:   { kind: process, label: "Use posting.len as exact total; do not build scored HashMap" }
  output:  { kind: terminal, label: "Return constant-score page in posting order" }
edges:
  - { from: request, to: plan }
  - { from: plan, to: page }
  - { from: page, to: total }
  - { from: total, to: output }
---
flowchart TD
    request([Term query, no sort]) --> plan[Planner standalone Term branch]
    plan --> page[posting.iter().take(limit)]
    page --> total[posting.len exact total]
    total --> output([No full materialization or ranking])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: kw-term-early-stop-contract-tests
requirements:
  posting_window_contract:
    id: R1
    text: "`storage::try_plan` keeps standalone keyword Term queries on the posting-window path"
    kind: design-constraint
    risk: high
    verify: inspection
  term_latency_floor_contract:
    id: R2
    text: "`term_query_latency_floor` stays below the exact term lookup budget"
    kind: performance
    risk: medium
    verify: test
  release_peer_gate_contract:
    id: R3
    text: "`ec-efficiency-meter` can run the release competitive gate cleanly with real pg/OpenSearch peers"
    kind: performance
    risk: high
    verify: test
elements:
  storage_try_plan:
    kind: source
    path: projects/lumen/src/storage.rs
  perf_gate_term_query_latency_floor:
    kind: test
    path: projects/lumen/tests/perf_gate.rs
  vat_ec_efficiency_meter:
    kind: runner
    path: projects/lumen/vat.toml
relations:
  - { from: storage_try_plan, verifies: posting_window_contract }
  - { from: perf_gate_term_query_latency_floor, verifies: term_latency_floor_contract }
  - { from: vat_ec_efficiency_meter, verifies: release_peer_gate_contract }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "Term page stays on posting-window planner"
      risk: high
      verifymethod: inspection
    }
    requirement R2 {
      id: R2
      text: "term latency floor passes"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "release peer perf gate passes"
      risk: high
      verifymethod: test
    }
    element storage_try_plan {
      type: "rs/function"
    }
    element perf_gate_term_query_latency_floor {
      type: "rs/#[test]"
    }
    element vat_ec_efficiency_meter {
      type: "vat/runner"
    }
    storage_try_plan - verifies -> R1
    perf_gate_term_query_latency_floor - verifies -> R2
    vat_ec_efficiency_meter - verifies -> R3
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: local-term-latency-floor
    name: "local term latency floor"
    runner: cargo
    path: projects/lumen/tests/perf_gate.rs
    command: "cargo test -p lumen --test perf_gate term_query_latency_floor -- --exact --nocapture"
    verifies:
      - "Exact term lookup stays below the local p99 regression budget."
  - id: release-competitive-peer-gate
    name: "release competitive peer gate"
    runner: vat
    path: projects/lumen/vat.toml
    command: "cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter"
    verifies:
      - "Postgres and OpenSearch peers are real services provisioned by vat."
      - "The release competitive gate exits clean under meter."
      - "kw_term remains represented in the gate and README performance table."
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: projects/lumen/src/storage.rs
    action: claim
    section: logic
    impl_mode: hand-written
    reason: "Standalone Term planner already early-stops the returned page by reading only the requested posting window."
  - path: projects/lumen/tests/perf_gate.rs
    action: verify
    section: unit-test
    impl_mode: hand-written
    reason: "Term latency floor is the local regression guard."
  - path: projects/lumen/tests/perf_gate_vs_db.rs
    action: verify
    section: e2e-test
    impl_mode: hand-written
    reason: "Release competitive gate covers keyword term performance claims."
  - path: projects/lumen/vat.toml
    action: verify
    section: e2e-test
    impl_mode: hand-written
    reason: "ec-efficiency-meter is the real-service peer runner used as closing evidence."
  - path: projects/lumen/README.md
    action: claim
    section: changes
    impl_mode: hand-written
    reason: "Performance contract already records kw_term green margins through pg-native/OpenSearch evidence."
```
