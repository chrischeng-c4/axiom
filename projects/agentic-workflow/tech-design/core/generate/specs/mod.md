---
id: projects-sdd-src-generate-specs-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/specs/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/specs/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `asyncapi` | projects/agentic-workflow/src/generate/specs/mod.rs | module | pub | 7 |  |
| `openapi` | projects/agentic-workflow/src/generate/specs/mod.rs | module | pub | 8 |  |
| `openrpc` | projects/agentic-workflow/src/generate/specs/mod.rs | module | pub | 9 |  |
| `serverless` | projects/agentic-workflow/src/generate/specs/mod.rs | module | pub | 10 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/specs/mod.rs -->
```rust
//! API Specification Generation
//!
//! Provides functions for generating various API specification formats.

pub mod asyncapi;
pub mod openapi;
pub mod openrpc;
pub mod serverless;

pub use asyncapi::{generate_asyncapi, AsyncApiInput};
pub use openapi::{generate_openapi, OpenApiInput};
pub use openrpc::{generate_openrpc, OpenRpcInput};
pub use serverless::{generate_serverless_workflow, ServerlessWorkflowInput};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/specs/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete API specification generation module aggregator.
```
