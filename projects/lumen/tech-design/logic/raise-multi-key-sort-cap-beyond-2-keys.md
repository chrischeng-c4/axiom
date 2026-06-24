---
id: lumen-multikey-sort-cap
summary: >
  Raise the multi-key sort cap from 2 to 4. The generic materialized sort plan
  and the keyset cursor already carry a full Vec<SortValue> and compare every key
  in order, so the 2-key limit was conservative. Number and keyword keys remain
  the only sortable kinds.
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: >
      Multi-column sort up to 4 keys matches common data-table sort needs; the
      generic plan already supports N keys.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: multikey-sort-cap
entry: start
nodes:
  start:  { kind: start,    label: "validate sort" }
  check:  { kind: decision, label: "sort.len() > MAX_SORT_KEYS (4)?" }
  reject: { kind: terminal, label: "400 UnsupportedSort" }
  plan:   { kind: process,  label: "generic plan: compare every key in order" }
  done:   { kind: terminal, label: "ordered page" }
edges:
  - { from: start,  to: check }
  - { from: check,  to: reject, label: "yes" }
  - { from: check,  to: plan,   label: "no" }
  - { from: plan,   to: done }
---
flowchart TD
    start([validate sort]) --> check{sort.len > 4?}
    check -->|yes| reject([400 UnsupportedSort])
    check -->|no| plan[generic plan compares every key in order]
    plan --> done([ordered page])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: multikey-sort-cap-verification
requirements:
  three_key_sort_orders:
    id: R1
    text: "a 3-key sort orders results by each key in priority with a stable tie-break"
    kind: functional
    risk: high
    verify: test
  cap_rejects_over_four:
    id: R2
    text: "a sort with more than 4 keys is rejected with 400"
    kind: functional
    risk: medium
    verify: test
elements:
  test_three_key_sort_orders:
    kind: test
    type: "rs/#[test]"
  test_cap_rejects_over_four:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: test_three_key_sort_orders, verifies: three_key_sort_orders }
  - { from: test_cap_rejects_over_four, verifies: cap_rejects_over_four }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "3-key sort orders by each key with tie-break"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "more than 4 keys rejected"
      risk: medium
      verifymethod: test
    }
    element test_three_key_sort_orders {
      type: "rs/#[test]"
    }
    test_three_key_sort_orders - verifies -> R1
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
