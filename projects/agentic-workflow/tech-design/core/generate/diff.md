---
id: sdd-generate-diff
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Diff Output Types

## Overview
<!-- type: overview lang: markdown -->

Diff classification and report types in
`projects/agentic-workflow/src/generate/diff.rs`. Three shapes:

- `DiffClass` — 4-variant unit enum (`Exact`, `MarkerOnly`, `Drift`, `Gap`).
  Derives `[Debug, Clone, PartialEq]`.
- `FileDiff` — per-file result (path, classification, drift_pct,
  marker_pct, coverage_pct). Derives `[Debug, Clone]`.
- `DiffReport` — collection of FileDiff. Derives `[Debug, Clone]`.

Codegen replaces all three type declarations. Companion source templates own
the module preamble and the runtime helpers that previously lived in managed
HANDWRITE gaps.

This spec exercises:

1. **`f32` field** — drift/marker/coverage percentages use bare f32 in `required:`.
2. **`PathBuf`** field via x-rust-type.
3. **Foreign-type sibling reference** — `classification: DiffClass`.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  DiffClass:
    type: string
    enum: [Exact, MarkerOnly, Drift, Gap]
    description: |
      Classification of drift between generated and current file content.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq]
      variants:
        - { name: Exact, doc: "Content matches generated output exactly." }
        - { name: MarkerOnly, doc: "CODEGEN markers present but block is empty." }
        - { name: Drift, doc: "Content differs from generated output." }
        - { name: Gap, doc: "No CODEGEN markers found in target file." }

  FileDiff:
    type: object
    required: [path, classification, drift_pct, marker_pct, coverage_pct]
    description: |
      Per-file diff result.
    properties:
      path:
        type: string
        x-rust-type: "PathBuf"
        description: "Target file path (relative to project root)."
      classification:
        type: string
        x-rust-type: "DiffClass"
        description: "Classification of the diff."
      drift_pct:
        type: number
        x-rust-type: "f32"
        description: "Percentage of content that has drifted (0.0–100.0)."
      marker_pct:
        type: number
        x-rust-type: "f32"
        description: "Percentage of CODEGEN blocks that have SPEC-MANAGED markers."
      coverage_pct:
        type: number
        x-rust-type: "f32"
        description: "Percentage of spec requirements covered by CODEGEN blocks."
    x-rust-struct:
      derive: [Debug, Clone]

  DiffReport:
    type: object
    required: [files]
    description: |
      Diff report for a spec file.
    properties:
      files:
        type: array
        items: { $ref: "#/definitions/FileDiff" }
        x-rust-type: "Vec<FileDiff>"
        description: "Per-file diff results."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diff.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - DiffClass
      - FileDiff
      - DiffReport
    description: |
      Codegen replaces all three type declarations.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Three data carriers; standard derives.
- [schema] All in `required:`; PathBuf + f32 + DiffClass via x-rust-type.
- [changes] Standard split with all three in `replaces`.
