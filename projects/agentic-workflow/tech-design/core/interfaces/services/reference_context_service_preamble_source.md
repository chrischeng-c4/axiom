---
id: sdd-interfaces-services-reference-context-service-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Reference Context Service Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/reference_context_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CreateCodebaseContextInput` | projects/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 24 |  |
| `CreateContextInput` | projects/agentic-workflow/src/services/reference_context_service.rs | enum | pub | 44 |  |
| `CreateKnowledgeContextInput` | projects/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 59 |  |
| `CreateSpecContextInput` | projects/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 79 |  |
| `create_context` | projects/agentic-workflow/src/services/reference_context_service.rs | function | pub | 105 | create_context(input: CreateContextInput, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

```rust
//! Context service - Business logic for structured context artifact creation
//!
//! Each context type (spec, knowledge, codebase) has its own input struct
//! with type-specific validation and markdown rendering. The output is a
//! structured index (what was scanned, what was found, where it lives)
//! rather than a free-form copy of content.

use crate::models::context::{DocRef, FileRef, LensResult, PatternRef, SpecRef};
use crate::Result;
use chrono::Utc;
use std::path::Path;

// ---------------------------------------------------------------------------
// Input structs
// ---------------------------------------------------------------------------
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/reference_context_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
    description: "Source template owns reference-context service documentation and imports."
```
