---
id: projects-sdd-src-generate-diagrams-erd-plus-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/diagrams/erd_plus/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/erd_plus/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/erd_plus/mod.rs -->
```rust
//! ERD+ Diagram Format
//!
//! Enhanced Entity-Relationship diagram definitions with validation and YAML frontmatter.
//! Supports entities, attributes (PK/FK/UK), and relationships with cardinality.

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/erd_plus/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete ERD+ module facade.
```
