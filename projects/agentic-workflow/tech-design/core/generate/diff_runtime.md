---
id: sdd-generate-diff-runtime
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Diff Runtime Source

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
<!-- source-from-target: handwrite-gap generate-diff-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diff.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-diff-runtime>"
    description: "Source template owns diff runtime helpers and regression tests."
```
