---
id: sdd-issues-backends-local
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# LocalBackend Type

## Overview
<!-- type: overview lang: markdown -->

Local filesystem issue backend in `projects/agentic-workflow/src/issues/backends/local.rs`.
It can be rooted at the repository lifecycle directory (`.aw/issues`) or at
an ephemeral remote read cache under `/tmp/aw/issues`. One shape:

- `LocalBackend` — single private `issues_dir: PathBuf` field, no derives.

Codegen replaces the struct declaration. Companion source templates own module
imports, helper functions, `impl LocalBackend { ... }`, trait impls,
frontmatter persistence adapters, and regression tests.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  LocalBackend:
    type: object
    required: [issues_dir]
    description: Backend that stores issues as files under an issue directory.
    properties:
      issues_dir:
        type: string
        x-rust-type: "PathBuf"
        x-rust-visibility: private
        description: "Directory containing issue files."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/backends/local.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - LocalBackend
    description: |
      Codegen replaces the struct only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Single struct with private PathBuf.
- [schema] Standard pattern.
- [changes] Standard split.
