---
id: sdd-fillback-openspec-imports-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Fillback OpenSpec Imports Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/fillback/openspec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `OpenSpecStrategy` | projects/agentic-workflow/src/fillback/openspec.rs | struct | pub | 13 |  |
| `new` | projects/agentic-workflow/src/fillback/openspec.rs | function | pub | 54 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap fillback-openspec-imports -->

<!-- source-snapshot: path=projects/agentic-workflow/src/fillback/openspec.rs -->
```rust
use crate::fillback::strategy::ImportStrategy;
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/openspec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-openspec-imports>"
    description: "Source template owns fillback OpenSpec imports."
```
