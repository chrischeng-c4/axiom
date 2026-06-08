---
id: sdd-tools-common-reference-context
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# GroupSubState Type

## Overview
<!-- type: overview lang: markdown -->

Per-group sub-state enum in
`projects/agentic-workflow/src/tools/common_reference_context.rs`. One shape:

- `GroupSubState` — 5-variant enum with no derives:
  - `Create { group_id: String }` (struct variant)
  - `CreateSection { group_id: String, section: String }` (struct variant)
  - `Review { group_id: String }` (struct variant)
  - `Revise { group_id: String }` (struct variant)
  - `AllDone` (unit variant)

Codegen replaces the enum declaration only. Module imports, the
`resolve_next_group` function and all helpers stay hand-written.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  GroupSubState:
    type: string
    enum: [Create, CreateSection, Review, Revise, AllDone]
    description: |
      Per-group sub-state within the reference context lifecycle.
    x-rust-enum:
      derive: []
      variants:
        - name: Create
          kind: struct
          doc: "No artifact exists — needs creation."
          fields:
            - { name: group_id, rust_type: String }
        - name: CreateSection
          kind: struct
          doc: "Artifact exists with section-loop in progress — fill next section."
          fields:
            - { name: group_id, rust_type: String }
            - { name: section, rust_type: String }
        - name: Review
          kind: struct
          doc: "Artifact exists, no review verdict — needs review."
          fields:
            - { name: group_id, rust_type: String }
        - name: Revise
          kind: struct
          doc: "Reviewed but not approved, revision count < 1 — needs revision."
          fields:
            - { name: group_id, rust_type: String }
        - { name: AllDone, doc: "All groups approved." }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/common_reference_context.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - GroupSubState
    description: |
      Codegen replaces the enum declaration only.
  - path: projects/agentic-workflow/src/tools/common_reference_context.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, imports
      (`crate::state::StateManager`, `crate::workflow::helpers`,
      `crate::Result`, `serde_json::Value`, `std::path::Path`,
      `super::workflow_common`), the `resolve_next_group` function
      and all helpers (`read_fill_sections`, etc.).
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] Single enum with mixed unit + struct variants, no derives.
- [schema] All variants defined; standard struct-variant pattern.
- [changes] Standard split.
