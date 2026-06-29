---
id: lumen-has-child-sort
summary: >
  Allow a `has_child` query to be combined with `sort`. has_child cannot drive
  the per-doc keyset planner, so a sorted query containing has_child routes to
  the materialized sort path (eval_query resolves the parent docids, which are
  then sorted by their parent fields). Validation still rejects sort with
  knn/rrf/hamming.
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: >
      Unlocks nested data-table search: filter parents by a child match
      (has_child) AND sort those parents by a parent field with an accurate
      total, in a single query.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: has-child-sort-dispatch
entry: start
nodes:
  start:       { kind: start,    label: "search with sort" }
  validate:    { kind: process,  label: "validate: reject sort+knn/rrf/hamming, ALLOW has_child" }
  check:       { kind: decision, label: "sort present AND query has has_child?" }
  keyset:      { kind: terminal, label: "keyset planner (no has_child, unchanged)" }
  materialize: { kind: process,  label: "eval_query resolves parent docids (join)" }
  order:       { kind: process,  label: "sort parents by parent field; exact total" }
  done:        { kind: terminal, label: "SearchResponse" }
edges:
  - { from: start,       to: validate }
  - { from: validate,    to: check }
  - { from: check,       to: keyset,      label: "no" }
  - { from: check,       to: materialize, label: "yes" }
  - { from: materialize, to: order }
  - { from: order,       to: done }
---
flowchart TD
    start([search with sort]) --> validate[reject sort+knn/rrf/hamming; allow has_child]
    validate --> check{sort AND has_child?}
    check -->|no| keyset([keyset planner, unchanged])
    check -->|yes| materialize[eval_query resolves parent docids]
    materialize --> order[sort parents by parent field; exact total]
    order --> done([SearchResponse])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: has-child-sort-verification
requirements:
  has_child_sort_orders_parents:
    id: R1
    text: "a has_child query combined with sort returns matching parents ordered by the parent sort field"
    kind: functional
    risk: high
    verify: test
  has_child_sort_composes_with_filter:
    id: R2
    text: "has_child AND a parent-field filter, sorted, returns the intersection ordered by the sort field with an exact total"
    kind: functional
    risk: high
    verify: test
  knn_sort_still_rejected:
    id: R3
    text: "sort combined with knn/rrf/hamming is still rejected with 400"
    kind: functional
    risk: medium
    verify: test
elements:
  test_has_child_sort_orders_parents:
    kind: test
    type: "rs/#[test]"
  test_has_child_sort_with_filter:
    kind: test
    type: "rs/#[test]"
  test_knn_sort_rejected:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: test_has_child_sort_orders_parents, verifies: has_child_sort_orders_parents }
  - { from: test_has_child_sort_with_filter,    verifies: has_child_sort_composes_with_filter }
  - { from: test_knn_sort_rejected,             verifies: knn_sort_still_rejected }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "has_child + sort orders parents"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "has_child + filter + sort with exact total"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "sort + knn/rrf/hamming still rejected"
      risk: medium
      verifymethod: test
    }
    element test_has_child_sort_orders_parents {
      type: "rs/#[test]"
    }
    test_has_child_sort_orders_parents - verifies -> R1
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/storage.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Materialize parent bitmaps so has_child filters can participate in sorted parent queries."
  - path: projects/lumen/tests/collapse_nested.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    reason: "Verify has_child queries combined with parent sorting and nested result accounting."
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
  - path: projects/lumen/src/storage.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Route sorted has_child queries through the materialized parent sort path while preserving incompatible sort rejections."
  - path: projects/lumen/src/storage.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Cover has_child sort ordering, filter composition, and knn/sort rejection regressions."
```
