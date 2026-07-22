---
id: aw-ddd-meta-projection-contract
summary: "Project stable DDD identities into one-owner CAPABILITIES, README, CONTRIBUTING, EC, TD, and source narratives."
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: capability-control-plane
    role: primary
    gap: ddd-meta-projection-contract
    claim: ddd-meta-projection-contract
    coverage: full
    rationale: "The control plane needs reproducible narrative surfaces that reference one stable DDD identity instead of paths and headings."
---

# DDD Meta-Projection Contract

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-ddd-meta-projection-validation
entry: load
nodes:
  load: { kind: start, label: "parse aw.ddd-meta-projection.v1" }
  stable: { kind: decision, label: "each projection identity exists in the DDD context map?" }
  role: { kind: decision, label: "surface owns every declared narrative fact?" }
  ownership: { kind: decision, label: "identity and fact have exactly one owner?" }
  index: { kind: process, label: "render deterministic projection index from owned facts" }
  accepted: { kind: terminal, label: "valid disposable narrative projections" }
  reject: { kind: terminal, label: "reject unresolved, forbidden, or duplicate ownership" }
edges:
  - { from: load, to: stable }
  - { from: stable, to: role, label: "yes" }
  - { from: stable, to: reject, label: "no" }
  - { from: role, to: ownership, label: "yes" }
  - { from: role, to: reject, label: "no" }
  - { from: ownership, to: index, label: "yes" }
  - { from: ownership, to: reject, label: "no" }
  - { from: index, to: accepted }
---
flowchart TD
  load([parse aw.ddd-meta-projection.v1]) --> stable{each projection identity exists in the DDD context map?}
  stable -->|yes| role{surface owns every declared narrative fact?}
  stable -->|no| reject([reject unresolved, forbidden, or duplicate ownership])
  role -->|yes| ownership{identity and fact have exactly one owner?}
  role -->|no| reject
  ownership -->|yes| index[render deterministic projection index from owned facts]
  ownership -->|no| reject
  index --> accepted([valid disposable narrative projections])
```

The DDD meta-projection contract has six fixed narrative surfaces. The ownership
matrix is canonical and also supplies the forbidden examples: CAPABILITIES owns
`promise`; README owns `overview` and `journey`; CONTRIBUTING owns `boundary`
and `authoring-rule`; EC owns `external-truth`; TD owns
`executable-construction`; and `src/*` owns `implementation` and `unit-test`.
No surface may claim another surface's fact, and the tuple `(stable identity,
fact)` has one owner only.

Each projection must reference an identity already validated by
`aw.ddd-context-map.v1`. A `rendered_markdown` value is intentionally
non-canonical: validation and `render_projection_index` ignore it, then render a
stable index from declared ownership. Consequently a document move, heading
rename, or regenerated wording is disposable; it cannot become a second source
of truth or redefine a DDD identity.

This contract defines narrative ownership without rewriting existing meta docs,
compiling Python AST, generating source, or changing Mamba/mambalibs. Later
adapters may materialize each surface only from these validated facts and their
domain-specific payloads.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-ddd-meta-projection-unit-tests
requirements:
  role_matrix:
    id: R1
    text: "All six surfaces expose one declared narrative role and accept only their owned fact types."
    kind: contract
    risk: high
    verify: "cargo test -p agentic-workflow --test ddd_meta_projection_test ddd_meta_projection_assigns_each_surface_its_declared_narrative_role -- --nocapture"
  one_owner:
    id: R2
    text: "Duplicate ownership and unresolved stable IDs fail validation."
    kind: regression
    risk: high
    verify: "cargo test -p agentic-workflow --test ddd_meta_projection_test ddd_meta_projection_rejects_duplicate_ownership -- --nocapture"
  disposable_rendering:
    id: R3
    text: "Changed Markdown renders reproduce the same deterministic ownership index."
    kind: contract
    risk: medium
    verify: "cargo test -p agentic-workflow --test ddd_meta_projection_test ddd_meta_projection_regenerates_the_same_index_after_markdown_changes -- --nocapture"
elements:
  ddd_meta_projection_assigns_each_surface_its_declared_narrative_role: { kind: test, type: "rs/#[test]" }
  ddd_meta_projection_rejects_duplicate_ownership: { kind: test, type: "rs/#[test]" }
  ddd_meta_projection_rejects_unresolved_stable_identity_references: { kind: test, type: "rs/#[test]" }
  ddd_meta_projection_rejects_forbidden_surface_ownership: { kind: test, type: "rs/#[test]" }
  ddd_meta_projection_regenerates_the_same_index_after_markdown_changes: { kind: test, type: "rs/#[test]" }
relations:
  - { from: ddd_meta_projection_assigns_each_surface_its_declared_narrative_role, verifies: role_matrix }
  - { from: ddd_meta_projection_rejects_duplicate_ownership, verifies: one_owner }
  - { from: ddd_meta_projection_rejects_unresolved_stable_identity_references, verifies: one_owner }
  - { from: ddd_meta_projection_rejects_forbidden_surface_ownership, verifies: role_matrix }
  - { from: ddd_meta_projection_regenerates_the_same_index_after_markdown_changes, verifies: disposable_rendering }
---
requirementDiagram
  requirement R1 {
    id: R1
    text: "single role per narrative surface"
    risk: high
    verifymethod: test
  }
  requirement R2 {
    id: R2
    text: "one owner and resolved stable ID"
    risk: high
    verifymethod: test
  }
  requirement R3 {
    id: R3
    text: "Markdown is disposable projection output"
    risk: medium
    verifymethod: test
  }
  element ddd_meta_projection_assigns_each_surface_its_declared_narrative_role {
    type: "rs/#[test]"
  }
  element ddd_meta_projection_rejects_duplicate_ownership {
    type: "rs/#[test]"
  }
  element ddd_meta_projection_rejects_unresolved_stable_identity_references {
    type: "rs/#[test]"
  }
  element ddd_meta_projection_rejects_forbidden_surface_ownership {
    type: "rs/#[test]"
  }
  element ddd_meta_projection_regenerates_the_same_index_after_markdown_changes {
    type: "rs/#[test]"
  }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/context_map/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose narrative projection parsing, validation, model, and deterministic rendering."
  - path: apps/agentic-workflow/src/context_map/narrative_projection.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Own the six-surface narrative-role matrix and stable-ID projection validator."
  - path: apps/agentic-workflow/tests/ddd_meta_projection_test.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Prove roles, forbidden ownership, unresolved IDs, duplicates, and reproducible rendering."
  - path: apps/agentic-workflow/tests/fixtures/ddd_meta_projection
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Static valid, rerendered, duplicate, unresolved, and forbidden narrative projections."
```
