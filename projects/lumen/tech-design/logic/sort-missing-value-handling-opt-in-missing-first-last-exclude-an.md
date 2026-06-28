---
id: lumen-sort-missing-value
summary: >
  Add an opt-in `missing: first | last | exclude` to SortSpec. Default `exclude`
  keeps today's behavior and fast keyset planner (rows missing the sort value are
  dropped from results and total). When any key is `first`/`last`, a materialized
  path includes the missing-value rows at the front/back and counts them in an
  exact total.
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: >
      Lets callers sort by an optional field without silently dropping or
      under-counting rows that lack a value (SQL NULLS FIRST/LAST parity).
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sort-missing-dispatch
entry: start
nodes:
  start:       { kind: start,    label: "search with sort" }
  check:       { kind: decision, label: "any sort key missing != exclude?" }
  planner:     { kind: terminal, label: "existing keyset planner (exclude path, unchanged)" }
  materialize: { kind: process,  label: "eval full matched set" }
  build:       { kind: process,  label: "per doc: tuple of Option<value>; drop if missing an exclude key" }
  order:       { kind: process,  label: "missing-aware comparator: first->before, last->after" }
  page:        { kind: process,  label: "exact total; offset-paginate the materialized order" }
  done:        { kind: terminal, label: "SearchResponse incl. missing rows" }
edges:
  - { from: start,       to: check }
  - { from: check,       to: planner,     label: "no" }
  - { from: check,       to: materialize, label: "yes" }
  - { from: materialize, to: build }
  - { from: build,       to: order }
  - { from: order,       to: page }
  - { from: page,        to: done }
---
flowchart TD
    start([search with sort]) --> check{any key missing != exclude?}
    check -->|no| planner([keyset planner, unchanged])
    check -->|yes| materialize[eval full matched set]
    materialize --> build[tuple of Option value; drop exclude-missing]
    build --> order[missing-aware comparator]
    order --> page[exact total; offset-paginate]
    page --> done([SearchResponse incl. missing rows])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sort-missing-verification
requirements:
  missing_last_after:
    id: R1
    text: "missing:last places rows lacking the sort value after present rows and counts them in total"
    kind: functional
    risk: high
    verify: test
  missing_first_before:
    id: R2
    text: "missing:first places rows lacking the sort value before present rows"
    kind: functional
    risk: high
    verify: test
  exclude_default_unchanged:
    id: R3
    text: "default exclude drops missing-value rows from results and total, unchanged from today"
    kind: functional
    risk: medium
    verify: test
  missing_paginates:
    id: R4
    text: "paging the missing-inclusive order returns each row once with an exact total"
    kind: functional
    risk: medium
    verify: test
elements:
  test_missing_last_after:
    kind: test
    type: "rs/#[test]"
  test_missing_first_before:
    kind: test
    type: "rs/#[test]"
  test_exclude_default_unchanged:
    kind: test
    type: "rs/#[test]"
  test_missing_paginates:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: test_missing_last_after,         verifies: missing_last_after }
  - { from: test_missing_first_before,       verifies: missing_first_before }
  - { from: test_exclude_default_unchanged,  verifies: exclude_default_unchanged }
  - { from: test_missing_paginates,          verifies: missing_paginates }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "missing:last after, counted"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "missing:first before"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "exclude default unchanged"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "missing-inclusive pagination exact"
      risk: medium
      verifymethod: test
    }
    element test_missing_last_after {
      type: "rs/#[test]"
    }
    test_missing_last_after - verifies -> R1
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Applicable: control-flow contract for the change.
- [unit-test] Applicable: behavior verified by unit tests.

# Reviews

### Review 1
**Verdict:** approved

- [logic] Correct contract matching the implementation.
- [unit-test] Requirements bound to concrete tests.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/types.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose sort missing-value policy in the request model."
  - path: projects/lumen/src/storage.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Dispatch missing:first/last to materialized sort while keeping exclude on the keyset path."
  - path: projects/lumen/src/storage.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Cover missing:first, missing:last, exclude default, and keyset pagination behavior."
```
