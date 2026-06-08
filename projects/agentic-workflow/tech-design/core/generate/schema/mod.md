---
id: projects-sdd-src-generate-schema-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/schema/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/schema/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/schema/mod.rs -->
```rust
//! JSON Schema Core Implementation
//!
//! Provides strongly-typed structures for JSON Schema Draft 7 and Draft 2020-12.

mod parser;
mod types;

pub use parser::*;
pub use types::*;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/schema/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete JSON Schema module facade.
```
