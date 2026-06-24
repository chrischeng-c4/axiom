---
id: lumen-ids-query
summary: >
  Add a native `ids` query node that filters by a set of external_ids. It
  resolves each id through the collection interner to a docid bitmap (unknown
  ids skipped), is constant-scored and predicable, and composes under and/or/not
  like term/terms. Removes the need to index a redundant `_row_id` keyword field.
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: >
      Native row-id-set filtering (row_id_in) without a redundant companion
      keyword field, since lumen already holds external_id in its interner.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: ids-eval
entry: start
nodes:
  start:   { kind: start,    label: "eval ids query {values}" }
  loop:    { kind: process,  label: "for each external_id: interner.id(eid)" }
  known:   { kind: decision, label: "id known?" }
  insert:  { kind: process,  label: "bitmap.insert(docid)" }
  skip:    { kind: process,  label: "skip unknown id" }
  done:    { kind: terminal, label: "constant-score docid bitmap" }
edges:
  - { from: start,  to: loop }
  - { from: loop,   to: known }
  - { from: known,  to: insert, label: "yes" }
  - { from: known,  to: skip,   label: "no" }
  - { from: insert, to: done }
  - { from: skip,   to: done }
---
flowchart TD
    start([eval ids query]) --> loop[for each external_id: interner.id]
    loop --> known{id known?}
    known -->|yes| insert[bitmap.insert docid]
    known -->|no| skip[skip unknown id]
    insert --> done([constant-score bitmap])
    skip --> done
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: ids-query-verification
requirements:
  ids_returns_set:
    id: R1
    text: "an ids query returns exactly the existing docs named, skipping unknown ids"
    kind: functional
    risk: high
    verify: test
  ids_composes_boolean:
    id: R2
    text: "an ids query composes under and/or/not with other clauses"
    kind: functional
    risk: high
    verify: test
  ids_sortable:
    id: R3
    text: "an ids query can be combined with sort (it is a predicable filter)"
    kind: functional
    risk: medium
    verify: test
elements:
  test_ids_returns_set:
    kind: test
    type: "rs/#[test]"
  test_ids_composes_boolean:
    kind: test
    type: "rs/#[test]"
  test_ids_sortable:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: test_ids_returns_set,      verifies: ids_returns_set }
  - { from: test_ids_composes_boolean, verifies: ids_composes_boolean }
  - { from: test_ids_sortable,         verifies: ids_sortable }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "ids returns the named set, skips unknown"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "ids composes under and/or/not"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "ids combines with sort"
      risk: medium
      verifymethod: test
    }
    element test_ids_returns_set {
      type: "rs/#[test]"
    }
    test_ids_returns_set - verifies -> R1
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
