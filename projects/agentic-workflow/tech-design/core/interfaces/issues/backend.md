---
id: sdd-issues-backend
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# SyncReport Type

## Overview
<!-- type: overview lang: markdown -->

Summary type for the issue sync function in
`projects/agentic-workflow/src/issues/backend.rs`. One shape:

- `SyncReport` — `fetched: usize`, `created: usize`, `updated: usize`.
  Derives `[Debug, Clone, Copy]`. Pure data carrier returned by
  `pub async fn sync_issues(...)`.

Codegen replaces only the SyncReport struct declaration. The
`IssueBackend` trait declaration with all its async methods, the
`sync_issues` function, and module imports stay hand-written.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SyncReport:
    type: object
    required: [fetched, created, updated]
    description: |
      Summary of an issue sync operation.
    properties:
      fetched:
        type: integer
        x-rust-type: "usize"
        description: "Number of issues fetched from source."
      created:
        type: integer
        x-rust-type: "usize"
        description: "Number of issues newly created on target."
      updated:
        type: integer
        x-rust-type: "usize"
        description: "Number of issues updated on target."
    x-rust-struct:
      derive: [Debug, Clone, Copy]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/backend.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - SyncReport
    description: |
      Codegen replaces the SyncReport struct declaration only.
  - path: projects/agentic-workflow/src/issues/backend.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, imports, the
      `IssueBackend` trait declaration with all its async methods,
      and the `sync_issues` function.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Pure data carrier with three usize fields and Copy.
- [schema] Standard usize via x-rust-type pattern.
- [changes] Hand-written boundary correctly preserves the trait declaration.
