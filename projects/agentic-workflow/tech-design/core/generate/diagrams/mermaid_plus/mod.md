---
id: projects-sdd-src-generate-diagrams-mermaid-plus-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `migrate` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs | module | pub | 9 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs -->
```rust
//! Mermaid+ State Machine Format
//!
//! XState-compatible state machine definitions with Mermaid stateDiagram-v2 output.
//! This module provides the core types and generator, independent of Lens IR.

mod generator;
pub mod migrate;
mod schema;
mod validator;

pub use generator::*;
pub use migrate::{
    apply_block_payload, detect_diagram_kind, enumerate_envelopes, mermaid_equivalent,
    run_migration, DiagramKind, MigrateState, MigrationEnvelope, MigrationOptions,
    MIGRATE_TOOL_VERSION, PAYLOAD_DIR,
};
pub use schema::*;
pub use validator::*;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/mermaid_plus/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Mermaid+ module declaration and exports.
```
