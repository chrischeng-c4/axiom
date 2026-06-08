---
id: sdd-tools-common-change-spec
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# SpecSubState Type

## Overview
<!-- type: overview lang: markdown -->

Per-spec sub-state enum in
`projects/agentic-workflow/src/tools/common_change_spec.rs`. One shape:

- `SpecSubState` — 5-variant enum:
  - `Create { spec_id, depends }` (struct variant)
  - `Review { spec_id }` (struct variant)
  - `Revise { spec_id }` (struct variant)
  - `MainthreadMustFix { spec_id }` (struct variant)
  - `AdvanceToImplementation` (unit)
  Derives `[Debug]`.

Codegen replaces the enum declaration. Source fragments in
`tools/common_change_spec/` own the universal skeleton, helper runtime, and
regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SpecSubState:
    type: string
    enum: [Create, Review, Revise, MainthreadMustFix, AdvanceToImplementation]
    description: Per-spec sub-state within the change-spec lifecycle.
    x-rust-enum:
      derive: [Debug]
      variants:
        - name: Create
          kind: struct
          doc: "No spec file — needs skeleton + create loop."
          fields:
            - { name: spec_id, rust_type: String }
            - { name: depends, rust_type: "Vec<String>" }
        - name: Review
          kind: struct
          doc: "Spec exists with create_complete, no review — needs review."
          fields:
            - { name: spec_id, rust_type: String }
        - name: Revise
          kind: struct
          doc: "Reviewed with issues — re-fill flagged sections."
          fields:
            - { name: spec_id, rust_type: String }
        - name: MainthreadMustFix
          kind: struct
          doc: "REJECTED after revision limit — mainthread must intervene."
          fields:
            - { name: spec_id, rust_type: String }
        - { name: AdvanceToImplementation, doc: "All specs created + approved." }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/common_change_spec.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - SpecSubState
    description: |
      Codegen replaces the enum declaration only.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] Single enum, mixed unit + struct variants.
- [schema] All variants well-formed.
- [changes] Standard split.
