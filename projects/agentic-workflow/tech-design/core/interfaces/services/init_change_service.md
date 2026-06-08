---
id: sdd-services-init-change
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Init Change Service Types

## Overview
<!-- type: overview lang: markdown -->

Plain input/output types for the SDD init-change service in
`projects/agentic-workflow/src/services/init_change_service.rs`. Two structs with
**no derives at all** (no Debug, no Clone, no Serialize, no
Deserialize). They are simple data carriers consumed by
`pub fn create_change(...)`.

- `CreateChangeInput` — `change_id: String`, `description: String`,
  `issue_refs: Option<Vec<String>>`, `git_workflow: Option<String>`.
- `CreateChangeResult` — `change_id: String`,
  `artifacts_written: Vec<String>`, `has_issues: bool`.

This spec exercises:

1. **No-derive struct emission** — `x-rust-struct.derive: []` (empty
   list) emits a plain `pub struct Foo { ... }` with no `#[derive(...)]`
   attribute. Verifies the generator gracefully handles structs that
   carry no traits.
2. **`x-rust-type: "Option<Vec<String>>"`** on `issue_refs` — listed in
   `required:` to use the literal type without auto-wrapping. Same
   pattern as `sequence.rs` (sdd-generate-sequence).
3. **Bare `bool` in `required:`** — `has_issues: bool` stays bare
   because it is in `required:` and has no `x-serde-default`.

Codegen replaces the data structs. Companion source templates own the module
documentation, imports, create-change orchestration, issue fetch handoff, state
updates, and regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  CreateChangeInput:
    type: object
    required: [change_id, description, issue_refs, git_workflow]
    description: Input for creating a new change.
    properties:
      change_id:
        type: string
        description: "Change identifier slug."
      description:
        type: string
        description: "Raw user-supplied description."
      issue_refs:
        type: array
        items: { type: string }
        x-rust-type: "Option<Vec<String>>"
        description: "Optional list of issue references to attach."
      git_workflow:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional git-workflow strategy name."
    x-rust-struct:
      derive: []

  CreateChangeResult:
    type: object
    required: [change_id, artifacts_written, has_issues]
    description: Result of creating a new change.
    properties:
      change_id:
        type: string
        description: "Echoed change identifier."
      artifacts_written:
        type: array
        items: { type: string }
        description: "Relative paths of artifacts written by create_change."
      has_issues:
        type: boolean
        description: "True if issue_refs was non-empty and issues were fetched."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/init_change_service.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - CreateChangeInput
      - CreateChangeResult
    description: |
      Codegen replaces both struct declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Correctly identifies both structs, their field types, and all three codegen patterns exercised (no-derive, `x-rust-type` for `Option<Vec<String>>`, bare `bool` in `required:`). Hand-written boundary is clearly stated.
- [schema] Both definitions are well-formed: `x-rust-struct.derive: []` present on each struct; `issue_refs` and `git_workflow` listed in `required:` with `x-rust-type` overrides to prevent auto-wrapping; `has_issues: boolean` in `required:` with no `x-serde-default` so it stays bare `bool`. No inconsistencies found.
- [changes] Two entries correctly split codegen vs hand-written scope for the same file. `replaces` lists both struct names, and the hand-written entry covers docstring, `use` statements, and the `create_change` function body. Sufficient for unambiguous implementation.
