---
id: sdd-fillback-factory-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Fillback Factory Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/fillback/factory.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `StrategyFactory` | projects/agentic-workflow/src/fillback/factory.rs | struct | pub | 17 |  |
| `create` | projects/agentic-workflow/src/fillback/factory.rs | function | pub | 35 | create(strategy_type: &str, source: &Path) -> Result<Box<dyn ImportStrategy>> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap fillback-factory-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/factory.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-factory-runtime>"
    description: "Source template owns fillback factory runtime behavior and tests."
```
