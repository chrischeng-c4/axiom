---
id: sdd-generate-generators-axum-runtime
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# AxumGenerator Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/generate/generators/axum.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AxumGenerator` | apps/agentic-workflow/src/generate/generators/axum.rs | struct | pub | 16 |  |
| `new` | apps/agentic-workflow/src/generate/generators/axum.rs | function | pub | 24 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-axum-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/generate/generators/axum.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-axum-runtime>"
    description: "Source template owns the Axum generator runtime and regression tests."
```
