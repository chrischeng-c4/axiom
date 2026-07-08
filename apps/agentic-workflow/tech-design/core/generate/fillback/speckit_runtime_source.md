---
id: sdd-fillback-speckit-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Fillback Speckit Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/fillback/speckit.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SpeckitStrategy` | apps/agentic-workflow/src/fillback/speckit.rs | struct | pub | 13 |  |
| `new` | apps/agentic-workflow/src/fillback/speckit.rs | function | pub | 20 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap fillback-speckit-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/fillback/speckit.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-speckit-runtime>"
    description: "Source template owns fillback Speckit runtime behavior and tests."
```
