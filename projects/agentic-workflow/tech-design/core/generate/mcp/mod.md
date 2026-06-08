---
id: projects-sdd-src-generate-mcp-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/mcp/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/mcp/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/mcp/mod.rs -->
```rust
//! MCP Tool Definitions for SDD Generate
//!
//! Exposes diagram and spec generation as MCP tools.

mod handlers;
mod tools;

pub use handlers::*;
pub use tools::SddTools;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/mcp/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete SDD generate MCP module facade.
```
