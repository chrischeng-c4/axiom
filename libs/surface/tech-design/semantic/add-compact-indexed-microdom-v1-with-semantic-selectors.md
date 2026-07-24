---
id: '2521'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: surface-microdom-v1-flow
entry: create
nodes:
  create: { kind: start, label: "Create an empty MicroDom with compact node and relationship arenas" }
  insert: { kind: process, label: "Insert a typed node with optional parent, stable semantic id, role, name, state, and actions" }
  validate_parent: { kind: decision, label: "Does the requested parent NodeId exist" }
  reject_parent: { kind: terminal, label: "Return InvalidParent without mutating the tree" }
  validate_semantic_id: { kind: decision, label: "Is the stable semantic id absent from the id index" }
  reject_duplicate: { kind: terminal, label: "Return DuplicateSemanticId without mutating the tree" }
  link: { kind: process, label: "Append the node to the arena and link it through indexed first-child and next-sibling relationships" }
  index: { kind: process, label: "Index semantic id plus semantic role and accessible name" }
  query: { kind: decision, label: "Lookup by id, role, or role-and-name selector" }
  traverse: { kind: process, label: "Traverse parent and children in deterministic insertion order" }
  snapshot: { kind: terminal, label: "Emit canonical schema-versioned nodes in arena order" }
edges:
  - { from: create, to: insert }
  - { from: insert, to: validate_parent }
  - { from: validate_parent, to: reject_parent, label: "no" }
  - { from: validate_parent, to: validate_semantic_id, label: "yes or root" }
  - { from: validate_semantic_id, to: reject_duplicate, label: "duplicate" }
  - { from: validate_semantic_id, to: link, label: "unique" }
  - { from: link, to: index }
  - { from: index, to: query }
  - { from: query, to: traverse, label: "match" }
  - { from: query, to: snapshot, label: "snapshot request" }
  - { from: traverse, to: snapshot }
---
flowchart TD
  create([Create MicroDom]) --> insert[Insert typed node]
  insert --> validate_parent{Parent valid?}
  validate_parent -->|no| reject_parent([InvalidParent])
  validate_parent -->|yes or root| validate_semantic_id{Stable id unique?}
  validate_semantic_id -->|no| reject_duplicate([DuplicateSemanticId])
  validate_semantic_id -->|yes| link[Append arena node and indexed links]
  link --> index[Index id and semantic role/name]
  index --> query{Query or traverse}
  query -->|match| traverse[Deterministic parent/children traversal]
  query -->|snapshot| snapshot([Canonical snapshot])
  traverse --> snapshot
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/surface/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: Element
    description: Export the focused microdom module from the existing renderer-neutral Surface crate without changing the Element authoring contract.
  - path: libs/surface/src/microdom.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: Define the compact indexed MicroDOM v1 arena, typed node identity and semantics, deterministic traversal, indexed selectors, typed construction errors, and canonical snapshot.
  - path: libs/surface/tests/microdom_contract.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Prove stable node identity, insertion-order traversal, id and semantic selectors, canonical serialization, and atomic rejection of invalid parent or duplicate stable id.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: surface-microdom-v1-verification
requirements:
  atomic_errors:
    id: R4
    text: "Duplicate semantic ids and invalid parent NodeId values return typed errors without partially mutating nodes, relationships, or selector indexes."
    kind: functional
    risk: high
    verify: cargo test -p cclab-surface --test microdom_contract invalid_insertions_are_typed_and_atomic -- --nocapture
  canonical_snapshot:
    id: R3
    text: "Identical MicroDOM inputs emit byte-identical schema-versioned canonical snapshots in arena order."
    kind: regression
    risk: medium
    verify: cargo test -p cclab-surface --test microdom_contract canonical_snapshot_is_byte_stable -- --nocapture
  compact_indexed_arena:
    id: R1
    text: "MicroDOM stores typed NodeId values in one compact node arena and preserves parent/child insertion order without one heap object per node."
    kind: functional
    risk: high
    verify: cargo test -p cclab-surface --test microdom_contract compact_arena_preserves_typed_identity_and_child_order -- --nocapture
  semantic_selectors:
    id: R2
    text: "Stable id, semantic role, and role-plus-accessible-name selectors return deterministic NodeId matches."
    kind: functional
    risk: high
    verify: cargo test -p cclab-surface --test microdom_contract selectors_resolve_stable_id_role_and_name -- --nocapture
---
flowchart TD
    r1[R1 compact indexed arena] --> cargo_test_p_cclab_surface_test_microdom_contract_compact_arena_preserves_typed_identity_and_child_order_nocapture[cargo test -p cclab-surface --test microdom_contract compact_arena_preserves_typed_identity_and_child_order -- --nocapture]
    r2[R2 semantic selectors] --> cargo_test_p_cclab_surface_test_microdom_contract_selectors_resolve_stable_id_role_and_name_nocapture[cargo test -p cclab-surface --test microdom_contract selectors_resolve_stable_id_role_and_name -- --nocapture]
    r3[R3 canonical snapshot] --> cargo_test_p_cclab_surface_test_microdom_contract_canonical_snapshot_is_byte_stable_nocapture[cargo test -p cclab-surface --test microdom_contract canonical_snapshot_is_byte_stable -- --nocapture]
    r4[R4 atomic errors] --> cargo_test_p_cclab_surface_test_microdom_contract_invalid_insertions_are_typed_and_atomic_nocapture[cargo test -p cclab-surface --test microdom_contract invalid_insertions_are_typed_and_atomic -- --nocapture]
```
