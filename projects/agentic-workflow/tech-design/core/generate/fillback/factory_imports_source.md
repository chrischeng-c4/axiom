---
id: sdd-fillback-factory-imports-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Fillback Factory Imports Source

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
<!-- source-from-target: handwrite-gap fillback-factory-imports -->

<!-- source-snapshot: path=projects/agentic-workflow/src/fillback/factory.rs -->
```rust
use crate::fillback::code::CodeStrategy;
use crate::fillback::openspec::OpenSpecStrategy;
use crate::fillback::speckit::SpeckitStrategy;
use crate::fillback::strategy::ImportStrategy;
use crate::Result;
use colored::Colorize;
use std::path::Path;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/factory.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-factory-imports>"
    description: "Source template owns fillback factory imports."
```
