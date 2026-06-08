---
id: projects-sdd-src-fillback-strategy-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: brownfield-takeover-surface
    claim: brownfield-takeover-surface
    coverage: full
    rationale: "Fillback interfaces support brownfield takeover by deriving TD/spec coverage from existing project artifacts."
---

# Standardized projects/agentic-workflow/src/fillback/strategy.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/fillback/strategy.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/fillback/strategy.rs -->
```rust
use crate::Result;
use async_trait::async_trait;
use std::path::Path;

/// Common interface for all import strategies
///
/// Each strategy (OpenSpec, Speckit, Code) implements this trait to provide
/// a consistent way to execute imports and detect if they can handle a given source.
#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/fillback/strategy.md#source
pub trait ImportStrategy: Send + Sync {
    /// Execute the import strategy
    ///
    /// # Arguments
    /// * `source` - Path to the source directory or file to import from
    /// * `change_id` - The change ID to create/populate in .aw/changes/
    ///
    /// # Errors
    /// Returns an error if the import fails for any reason (parsing, file I/O, etc.)
    async fn execute(&self, source: &Path, change_id: &str) -> Result<()>;

    /// Check if this strategy can handle the given source
    ///
    /// Used for auto-detection when strategy is set to "auto".
    /// Each strategy implements its own detection logic.
    ///
    /// # Arguments
    /// * `source` - Path to check
    ///
    /// # Returns
    /// `true` if this strategy can handle the source, `false` otherwise
    fn can_handle(&self, source: &Path) -> bool;

    /// Get the name of this strategy for display purposes
    fn name(&self) -> &'static str;
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/strategy.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete fillback import strategy trait module.
```
