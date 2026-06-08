---
id: sdd-fillback-speckit-imports-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Fillback Speckit Imports Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/fillback/speckit.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SpeckitStrategy` | projects/agentic-workflow/src/fillback/speckit.rs | struct | pub | 13 |  |
| `new` | projects/agentic-workflow/src/fillback/speckit.rs | function | pub | 20 | new() -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap fillback-speckit-imports -->

<!-- source-snapshot: path=projects/agentic-workflow/src/fillback/speckit.rs -->
```rust
use crate::fillback::strategy::ImportStrategy;
use crate::Result;
use async_trait::async_trait;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::path::Path;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/speckit.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:fillback-speckit-imports>"
    description: "Source template owns fillback Speckit imports."
```
