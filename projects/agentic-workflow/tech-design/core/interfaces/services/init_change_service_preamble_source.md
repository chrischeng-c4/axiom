---
id: sdd-interfaces-services-init-change-service-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Init Change Service Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/init_change_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CreateChangeInput` | projects/agentic-workflow/src/services/init_change_service.rs | struct | pub | 15 |  |
| `CreateChangeResult` | projects/agentic-workflow/src/services/init_change_service.rs | struct | pub | 28 |  |
| `create_change` | projects/agentic-workflow/src/services/init_change_service.rs | function | pub | 48 | create_change(input: CreateChangeInput, project_root: &Path) -> Result<CreateChangeResult> |
## Source
<!-- type: source lang: rust -->

```rust
//! Init change service — business logic for creating new changes.
//!
//! Extracted from `mcp/tools/init_change.rs`.

use crate::state::StateManager;
use crate::Result;
use std::path::Path;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/init_change_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "<handwrite-gap:init-change-service-preamble>"
    description: "Source template owns the init-change service docs and imports."
```
