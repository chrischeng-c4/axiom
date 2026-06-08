---
id: projects-sdd-src-shared-tools-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Shared workflow utilities are part of the AW Core protocol support surface used across clients and lifecycle phases."
---

# Standardized projects/agentic-workflow/src/shared/tools.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/shared/tools.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/shared/tools.rs -->
```rust
//! Shared tool re-exports
//!
//! Re-exports from the tools module for external access.

pub use crate::tools::analyze;
pub use crate::tools::knowledge;
pub use crate::tools::read;
pub use crate::tools::validate_spec;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/shared/tools.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete shared tool re-export module.
```
