---
id: sdd-tools-common-change-impl
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# ImplSubState Type

## Overview
<!-- type: overview lang: markdown -->

Per-spec implementation sub-state enum in
`projects/agentic-workflow/src/tools/common_change_impl.rs`. One shape:

- `ImplSubState` — 11-variant enum with mixed unit + struct variants.
  Derives `[Debug, PartialEq]`.

Codegen replaces the enum declaration. Runtime helpers and regression tests are
managed by source-fragment specs in
`projects/agentic-workflow/tech-design/core/tools/common_change_impl/`.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ImplSubState:
    type: string
    enum: [NoSpecs, ImplementSpecCode, ImplementSpecWithCodegen, BuildCheck, ImplementSpecTests, TestCountCheck, WriteDiff, ReviewSpec, ReviseSpec, TerminalFailure, AdvanceToMerge]
    description: Per-spec implementation sub-state.
    x-rust-enum:
      derive: [Debug, PartialEq]
      variants:
        - { name: NoSpecs, doc: "No change specs found — cannot implement." }
        - name: ImplementSpecCode
          kind: struct
          doc: "Implement production code for a spec (first spec = begin)."
          fields:
            - { name: spec_id, rust_type: String }
            - { name: is_first, rust_type: bool }
        - name: ImplementSpecWithCodegen
          kind: struct
          doc: "Implement with codegen path (has_json_schema/has_api_spec)."
          fields:
            - { name: spec_id, rust_type: String }
        - name: BuildCheck
          kind: struct
          doc: "Gate check: run cargo build before advancing to tests phase."
          fields:
            - { name: spec_id, rust_type: String }
        - name: ImplementSpecTests
          kind: struct
          doc: "Implement test functions for a spec (after build passes)."
          fields:
            - { name: spec_id, rust_type: String }
        - name: TestCountCheck
          kind: struct
          doc: "Gate check: count #[test] in diff vs spec Unit Test section."
          fields:
            - { name: spec_id, rust_type: String }
        - { name: WriteDiff, doc: "All specs implemented, write git diff to implementation.md." }
        - name: ReviewSpec
          kind: struct
          doc: "Review implementation for a spec."
          fields:
            - { name: spec_id, rust_type: String }
        - name: ReviseSpec
          kind: struct
          doc: "Revise implementation for a spec (fix review issues)."
          fields:
            - { name: spec_id, rust_type: String }
        - name: TerminalFailure
          kind: struct
          doc: "Spec exceeded revision limit."
          fields:
            - { name: spec_id, rust_type: String }
            - { name: revisions, rust_type: u32 }
        - { name: AdvanceToMerge, doc: "All specs implemented and approved -> advance to merge." }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/common_change_impl.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ImplSubState
    description: |
      Codegen replaces the enum declaration only.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Single 11-variant enum, mixed unit + struct variants.
- [schema] All variants well-formed.
- [changes] Standard split.
