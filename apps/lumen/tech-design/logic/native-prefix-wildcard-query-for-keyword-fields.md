---
id: '1792'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keyword-prefix-query
entry: request
nodes:
  request: { kind: start, label: "Evaluate prefix field/value query node" }
  nonempty: { kind: decision, label: "prefix value non-empty?" }
  type: { kind: decision, label: "target field is keyword?" }
  reject: { kind: terminal, label: "400 invalid prefix query" }
  terms: { kind: process, label: "Enumerate segment plus live-tail keyword dictionary in lexical order" }
  union: { kind: process, label: "Union postings while term starts_with prefix; subtract tombstones through live_terms" }
  compose: { kind: process, label: "Compose bitmap under boolean and sort planners" }
  done: { kind: terminal, label: "Return exact prefix matches" }
edges:
  - { from: request, to: nonempty }
  - { from: nonempty, to: reject, label: "no" }
  - { from: nonempty, to: type, label: "yes" }
  - { from: type, to: reject, label: "no" }
  - { from: type, to: terms, label: "yes" }
  - { from: terms, to: union }
  - { from: union, to: compose }
  - { from: compose, to: done }
---
flowchart TD
    request([prefix field/value]) --> nonempty{value non-empty?}
    nonempty -->|no| reject([400 invalid prefix])
    nonempty -->|yes| type{keyword field?}
    type -->|no| reject
    type -->|yes| terms[lexical live term dictionary]
    terms --> union[union starts_with postings]
    union --> compose[boolean + sort composition]
    compose --> done([exact prefix matches])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/types.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add PrefixQuery, QueryNode::Prefix and the keyword prefix capability bit."
  - path: apps/lumen/src/storage.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Validate prefix inputs, union lexical keyword postings, and integrate boolean/sort predicate paths."
  - path: apps/lumen/src/dx.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Project prefix support from FieldType capabilities into the generated field catalogue."
  - path: apps/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the canonical hierarchical keyword prefix query shape."
  - path: apps/lumen/tests/prefix_query.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Cover Taiwanese UTF-8 paths, boolean/sort composition, invalid field types, empty values, segment tail and delete parity."
  - path: apps/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Lock runtime-derived field catalogue and canonical OpenAPI prefix metadata."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keyword-prefix-query-verification
requirements:
  segment_and_contract_parity:
    id: R3
    text: "Segment plus live-tail and delete results match live memory, and field/OpenAPI catalogues advertise the same prefix capability."
    kind: integration
    risk: high
    verify: cargo test -p lumen --test prefix_query -- --nocapture && cargo test -p lumen --test spec_cli -- --nocapture
  unicode_prefix:
    id: R1
    text: "A case-sensitive Taiwanese UTF-8 path prefix returns exact starts-with keyword matches and composes under boolean and sort."
    kind: functional
    risk: high
    verify: cargo test -p lumen --test prefix_query -- --nocapture
  validation:
    id: R2
    text: "Empty prefixes and prefix queries against non-keyword fields fail with a clear 400-class error."
    kind: regression
    risk: medium
    verify: cargo test -p lumen --test prefix_query -- --nocapture
---
flowchart TD
    r1[R1 unicode prefix] --> cargo_test_p_lumen_test_prefix_query_nocapture[cargo test -p lumen --test prefix_query -- --nocapture]
    r2[R2 validation] --> cargo_test_p_lumen_test_prefix_query_nocapture
    r3[R3 segment and contract parity] --> cargo_test_p_lumen_test_prefix_query_nocapture_cargo_test_p_lumen_test_spec_cli_nocapture[cargo test -p lumen --test prefix_query -- --nocapture && cargo test -p lumen --test spec_cli -- --nocapture]
```
