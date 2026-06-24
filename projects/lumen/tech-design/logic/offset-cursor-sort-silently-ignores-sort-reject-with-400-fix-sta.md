---
id: lumen-offset-sort-reject-and-sort-doc
summary: >
  Reject `offset` cursor combined with `sort` (currently the sort is silently
  ignored and results fall back to score ranking) with a 400 UnsupportedSort,
  and correct the stale `SearchRequest.sort` doc comment that claims only number
  fields are sortable.
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: >
      Hardens the search sort/pagination contract of the query planner: an
      offset cursor combined with sort must error instead of silently
      mis-ordering, and the sortable-field documentation must be accurate.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: offset-sort-reject
entry: start
nodes:
  start:        { kind: start,    label: "search() entry" }
  parse_cursor: { kind: process,  label: "parse cursor to offset" }
  guard:        { kind: decision, label: "offset>0 AND sort present?" }
  reject:       { kind: terminal, label: "400 UnsupportedSort" }
  proceed:      { kind: process,  label: "existing plan / score path" }
  done:         { kind: terminal, label: "SearchResponse" }
edges:
  - { from: start,        to: parse_cursor }
  - { from: parse_cursor, to: guard }
  - { from: guard,        to: reject,  label: "yes" }
  - { from: guard,        to: proceed, label: "no" }
  - { from: proceed,      to: done }
---
flowchart TD
    start([search entry]) --> parse_cursor[parse cursor to offset]
    parse_cursor --> guard{offset>0 AND sort?}
    guard -->|yes| reject([400 UnsupportedSort])
    guard -->|no| proceed[existing plan / score path]
    proceed --> done([SearchResponse])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: offset-sort-reject-verification
requirements:
  reject_offset_sort:
    id: R1
    text: "search with an offset cursor (N>0) and a non-empty sort returns 400 UnsupportedSort"
    kind: functional
    risk: high
    verify: test
  offset_unsorted_ok:
    id: R2
    text: "an offset cursor without sort still paginates relevance-ranked results"
    kind: functional
    risk: medium
    verify: test
  keyset_sort_ok:
    id: R3
    text: "a keyset cursor combined with sort paginates correctly"
    kind: functional
    risk: medium
    verify: test
  sort_doc_corrected:
    id: R4
    text: "the OpenAPI SearchRequest.sort doc no longer claims only number fields are sortable"
    kind: design-constraint
    risk: low
    verify: inspection
elements:
  test_offset_sort_rejected:
    kind: test
    type: "rs/#[test]"
  test_offset_unsorted_paginates:
    kind: test
    type: "rs/#[test]"
  test_keyset_sort_paginates:
    kind: test
    type: "rs/#[test]"
  inspect_openapi_sort_doc:
    kind: inspection
    type: "rs/#[test]"
relations:
  - { from: test_offset_sort_rejected,      verifies: reject_offset_sort }
  - { from: test_offset_unsorted_paginates, verifies: offset_unsorted_ok }
  - { from: test_keyset_sort_paginates,     verifies: keyset_sort_ok }
  - { from: inspect_openapi_sort_doc,       verifies: sort_doc_corrected }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "offset cursor with sort returns 400"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "offset cursor without sort paginates"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "keyset cursor with sort paginates"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "sort doc corrected"
      risk: low
      verifymethod: inspection
    }
    element test_offset_sort_rejected {
      type: "rs/#[test]"
    }
    test_offset_sort_rejected - verifies -> R1
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Correct contract: guard sits after cursor→offset parse; offset>0 AND sort present → 400 UnsupportedSort, else the existing plan/score path is unchanged. Matches the silent-ignore site at storage.rs:7558 / 3390-3397.
- [unit-test] Requirements R1–R4 cover the reject path, the offset-without-sort regression, the keyset+sort happy path, and the doc-correctness inspection, each bound to a concrete test element.
