---
id: sdd-generate-generators-fastapi-runtime
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# FastAPIGenerator Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/fastapi.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FastAPIGenerator` | projects/agentic-workflow/src/generate/generators/fastapi.rs | struct | pub | 35 |  |
| `new` | projects/agentic-workflow/src/generate/generators/fastapi.rs | function | pub | 43 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-fastapi-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/fastapi.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-fastapi-runtime>"
    description: "Source template owns the FastAPI generator runtime and regression tests."
```
