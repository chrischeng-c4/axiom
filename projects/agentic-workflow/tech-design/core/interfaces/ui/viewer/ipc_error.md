---
id: sdd-ui-viewer-ipc-error
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# IpcError

## Overview
<!-- type: overview lang: markdown -->

Single thiserror enum `IpcError` in `projects/agentic-workflow/src/ui/viewer/ipc.rs`.
3 tuple variants. Carries `#[allow(dead_code)]` (only one variant is
actively constructed; others retained for completeness as the legacy
IPC API is phased out).

This spec exercises the **`x-rust-attrs`** feature (just shipped) for
arbitrary type-level attributes — `#[allow(dead_code)]` is emitted
above the derive list.

Hand-written outside CODEGEN: module preamble, `use super::manager::ViewerError`,
and the `#[cfg(test)] mod tests` block.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  IpcError:
    type: object
    description: Errors that can occur during IPC handling.
    x-rust-attrs:
      - "allow(dead_code)"
    x-rust-enum:
      derive: [Debug, "thiserror::Error"]
      variants:
        - name: ParseError
          kind: tuple
          error: "Failed to parse IPC message: {0}"
          fields:
            - { rust_type: String }
        - name: AnnotationError
          kind: tuple
          error: "Annotation error: {0}"
          fields:
            - { rust_type: String }
        - name: ViewerError
          kind: tuple
          error: "Viewer error: {0}"
          fields:
            - { rust_type: "super::manager::ViewerError", error_from: true }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/ui/viewer/ipc.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - IpcError
    description: |
      Codegen replaces the IpcError enum declaration only. thiserror's
      derive macro auto-generates Display + From<ViewerError> impls.
  - path: projects/agentic-workflow/src/ui/viewer/ipc.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module preamble, `use super::manager::ViewerError`,
      and the `#[cfg(test)] mod tests` block.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] First x-rust-attrs use; thiserror enum precedent re-applied.
- [schema] Three tuple variants with error templates; ViewerError variant uses error_from for #[from].
- [changes] Codegen replaces only the enum decl; hand-written use stmt + tests preserved.
