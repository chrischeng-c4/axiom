---
id: sdd-interfaces-services-pre-clarifications-service-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Pre Clarifications Service Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/pre_clarifications_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AppendClarificationsInput` | projects/agentic-workflow/src/services/pre_clarifications_service.rs | struct | pub | 20 |  |
| `CreateClarificationsInput` | projects/agentic-workflow/src/services/pre_clarifications_service.rs | struct | pub | 32 |  |
| `QuestionAnswer` | projects/agentic-workflow/src/services/pre_clarifications_service.rs | struct | pub | 42 |  |
| `append_clarifications` | projects/agentic-workflow/src/services/pre_clarifications_service.rs | function | pub | 120 | append_clarifications(     input: AppendClarificationsInput,     project_root: &Path, ) -> Result<String> |
| `create_clarifications` | projects/agentic-workflow/src/services/pre_clarifications_service.rs | function | pub | 58 | create_clarifications(     input: CreateClarificationsInput,     project_root: &Path, ) -> Result<String> |
## Source
<!-- type: source lang: rust -->

```rust
//! Clarifications service - Business logic for creating pre_clarifications.md
//!
//! Provides structured Q&A capture from user interactions during planning.

use crate::models::state::StatePhase;
use crate::state::StateManager;
use crate::Result;
use chrono::Local;
use std::path::Path;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/pre_clarifications_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "<handwrite-gap:pre-clarifications-service-preamble>"
    description: "Source template owns pre-clarifications service docs and imports."
```
