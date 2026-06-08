---
id: sdd-generate-diff-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Diff Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diff.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DiffClass` | projects/agentic-workflow/src/generate/diff.rs | enum | pub | 23 |  |
| `DiffReport` | projects/agentic-workflow/src/generate/diff.rs | struct | pub | 37 |  |
| `FileDiff` | projects/agentic-workflow/src/generate/diff.rs | struct | pub | 45 |  |
| `has_drift` | projects/agentic-workflow/src/generate/diff.rs | function | pub | 74 | has_drift(&self) -> bool |
| `overall_drift_pct` | projects/agentic-workflow/src/generate/diff.rs | function | pub | 65 | overall_drift_pct(&self) -> f32 |
| `run_diff` | projects/agentic-workflow/src/generate/diff.rs | function | pub | 87 | run_diff(spec_path: &Path, project_root: &Path) -> crate::generate::Result<DiffReport> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-diff-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diff.rs -->
```rust

//! Diff implementation: compare current target files against what codegen would produce.
//!
//! `run_diff` runs codegen for a spec, compares the generated CODEGEN block content
//! against what is currently in the target file, and classifies the difference.
//!
//! Classification:
//! - `Exact`: Current content matches generated content (no drift)
//! - `MarkerOnly`: CODEGEN markers present but empty content
//! - `Drift`: Content differs from generated output
//! - `Gap`: No CODEGEN markers found in the target file

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-validation.md

use std::path::{Path, PathBuf};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diff.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-diff-preamble>"
    description: "Source template owns diff module docs and imports."
```
