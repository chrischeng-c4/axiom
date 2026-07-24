---
id: '2521'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: surface-microdom-v1-contract
entry: construct
nodes:
  construct: { kind: start, label: "MicroDom owns Vec<MicroNode>, stable_id_index, and role_index; NodeId is a checked u32 arena index" }
  specify: { kind: process, label: "NodeSpec supplies stable_id, typed SemanticRole, optional accessible name, compact NodeState, and ActionSet" }
  preflight: { kind: decision, label: "Validate parent index and stable-id uniqueness before any arena or index mutation" }
  invalid_parent: { kind: terminal, label: "Return MicroDomError::InvalidParent with original len and indexes unchanged" }
  duplicate: { kind: terminal, label: "Return MicroDomError::DuplicateStableId with original len and indexes unchanged" }
  append: { kind: process, label: "Append one MicroNode value to the contiguous arena; store parent, first_child, last_child, and next_sibling as Option<NodeId>" }
  link_parent: { kind: process, label: "Link the parent last child to the new sibling or set first child; update last child" }
  index: { kind: process, label: "Insert stable id to NodeId and append NodeId to its role bucket in arena order" }
  select: { kind: decision, label: "Selector is stable id, role, or role plus exact accessible name" }
  id_lookup: { kind: process, label: "Stable-id lookup is direct through stable_id_index" }
  role_lookup: { kind: process, label: "Role lookup visits only the indexed role bucket; role-and-name applies exact name filtering in insertion order" }
  traverse: { kind: process, label: "Children iterator follows first_child then next_sibling and validates every NodeId through the arena" }
  snapshot: { kind: terminal, label: "Canonical snapshot clones nodes in arena order with schema version 1 and excludes unordered indexes" }
edges:
  - { from: construct, to: specify }
  - { from: specify, to: preflight }
  - { from: preflight, to: invalid_parent, label: "parent missing" }
  - { from: preflight, to: duplicate, label: "stable id exists" }
  - { from: preflight, to: append, label: "valid" }
  - { from: append, to: link_parent }
  - { from: link_parent, to: index }
  - { from: index, to: select }
  - { from: select, to: id_lookup, label: "id" }
  - { from: select, to: role_lookup, label: "role or role-name" }
  - { from: id_lookup, to: traverse }
  - { from: role_lookup, to: traverse }
  - { from: traverse, to: snapshot }
---
flowchart TD
  construct([Contiguous MicroDom arenas and indexes]) --> specify[Typed NodeSpec]
  specify --> preflight{Parent valid and stable id unique?}
  preflight -->|bad parent| invalid_parent([InvalidParent, atomic])
  preflight -->|duplicate| duplicate([DuplicateStableId, atomic])
  preflight -->|valid| append[Append value and assign NodeId u32]
  append --> link_parent[Update indexed sibling links]
  link_parent --> index[Index stable id and role]
  index --> select{Semantic selector}
  select -->|id| id_lookup[Direct id lookup]
  select -->|role/name| role_lookup[Indexed role bucket plus exact name filter]
  id_lookup --> traverse[Checked deterministic traversal]
  role_lookup --> traverse
  traverse --> snapshot([Schema v1 arena-order snapshot])
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
    description: Export pub mod microdom adjacent to the existing renderer-neutral Element contract; do not alter Element behavior.
  - path: libs/surface/src/microdom.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: Implement NodeId(u32), typed SemanticRole, compact NodeState and ActionSet, NodeSpec, contiguous Vec<MicroNode> storage, index-based parent/first-child/last-child/next-sibling links, stable-id and role indexes, SemanticSelector, typed atomic insertion errors, and schema-v1 canonical snapshot types.
  - path: libs/surface/tests/microdom_contract.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Add the four contract tests named by the unit-test section, including exact canonical JSON evidence and pre/post mutation equality for error cases.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: surface-microdom-v1-verification
requirements:
  atomic_errors:
    id: R4
    text: "Duplicate stable ids and invalid parent NodeId values return typed errors without partially mutating nodes, relationships, or selector indexes."
    kind: functional
    risk: high
    verify: cargo test -p cclab-surface --test microdom_contract invalid_insertions_are_typed_and_atomic -- --nocapture
  canonical_snapshot:
    id: R3
    text: "Identical MicroDOM inputs emit byte-identical schema-versioned canonical snapshots in arena order and do not serialize unordered selector indexes."
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
    text: "Stable id, semantic role, and role-plus-accessible-name selectors return deterministic NodeId matches through the id or role index."
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
