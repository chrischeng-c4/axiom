---
id: sdd-services-review-service
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# ReviewInput Type

## Overview
<!-- type: overview lang: markdown -->

Input type for writing a review in
`projects/agentic-workflow/src/services/review_service.rs`. One shape:

- `ReviewInput` — 11 fields with **no derives at all**. Pure data
  carrier consumed by `pub fn write_review(...)`.

Codegen replaces the struct declaration. Companion source templates own module
documentation, imports, artifact normalization, phase-transition matrix,
review writing, inline-review mutation, phase updates, and regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ReviewInput:
    type: object
    required: [change_id, file, verdict, summary, checklist, issues, iteration, spec_id, task_id, caller, group_id]
    description: |
      Input for writing a review.
    properties:
      change_id:
        type: string
        description: "Change identifier."
      file:
        type: string
        description: "File being reviewed."
      verdict:
        type: string
        description: "Review verdict (approved/needs-revision)."
      summary:
        type: string
        description: "Review summary."
      checklist:
        type: array
        items: { type: object }
        x-rust-type: "Vec<Value>"
        description: "Checklist entries."
      issues:
        type: array
        items: { type: object }
        x-rust-type: "Vec<Value>"
        description: "Identified issues."
      iteration:
        type: integer
        x-rust-type: "u64"
        description: "Review iteration number."
      spec_id:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional spec ID."
      task_id:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional task ID."
      caller:
        type: string
        description: "Caller identifier."
      group_id:
        type: string
        x-rust-type: "Option<String>"
        description: "Group ID for group-aware artifacts."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/review_service.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ReviewInput
    description: |
      Codegen replaces the struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Single struct with no derives; 11 fields including Vec<Value>, u64, Option<String>.
- [schema] All in `required:`; foreign types via x-rust-type.
- [changes] Standard split.
