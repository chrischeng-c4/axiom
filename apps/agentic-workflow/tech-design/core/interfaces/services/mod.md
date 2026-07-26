---
id: projects-sdd-src-services-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Standardized apps/agentic-workflow/src/services/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/services/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `file_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 10 |  |
| `implementation_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 11 |  |
| `init_change_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 12 |  |
| `issue_parser` | apps/agentic-workflow/src/services/mod.rs | module | pub | 13 |  |
| `knowledge_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 14 |  |
| `path_scope` | apps/agentic-workflow/src/services/mod.rs | module | pub | 15 |  |
| `platform_sync` | apps/agentic-workflow/src/services/mod.rs | module | pub | 16 |  |
| `post_clarifications_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 17 |  |
| `pre_clarifications_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 18 |  |
| `project_discovery` | apps/agentic-workflow/src/services/mod.rs | module | pub | 19 |  |
| `project_registry` | apps/agentic-workflow/src/services/mod.rs | module | pub | 20 |  |
| `python_artifact` | apps/agentic-workflow/src/services/mod.rs | module | pub | 21 |  |
| `python_artifact_code_check` | apps/agentic-workflow/src/services/mod.rs | module | pub | 22 |  |
| `python_artifact_readiness` | apps/agentic-workflow/src/services/mod.rs | module | pub | 23 |  |
| `python_ec` | apps/agentic-workflow/src/services/mod.rs | module | pub | 24 |  |
| `python_td` | apps/agentic-workflow/src/services/mod.rs | module | pub | 25 |  |
| `python_td_rust_target` | apps/agentic-workflow/src/services/mod.rs | module | pub | 26 |  |
| `python_td_target` | apps/agentic-workflow/src/services/mod.rs | module | pub | 27 |  |
| `python_td_typescript_target` | apps/agentic-workflow/src/services/mod.rs | module | pub | 28 |  |
| `reference_context_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 29 |  |
| `review_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 30 |  |
| `spec_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 31 |  |
| `tech_stack_service` | apps/agentic-workflow/src/services/mod.rs | module | pub | 32 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=apps/agentic-workflow/src/services/mod.rs -->
```rust
//! Service layer for SDD
//!
//! This module contains the business logic extracted from MCP tools.
#![allow(deprecated)]
//! Services are shared between MCP tools and CLI commands to ensure
//! consistency and avoid code duplication.

pub mod file_service;
pub mod implementation_service;
pub mod init_change_service;
pub mod issue_parser;
pub mod knowledge_service;
pub mod path_scope;
pub mod platform_sync;
pub mod post_clarifications_service;
pub mod pre_clarifications_service;
pub mod project_discovery;
pub mod project_registry;
pub mod python_artifact;
pub mod python_artifact_code_check;
pub mod python_artifact_readiness;
pub mod python_ec;
pub mod python_td;
pub mod python_td_mutation;
pub mod python_td_openapi_target;
pub mod python_td_rust_target;
pub mod python_td_target;
pub mod python_td_typescript_target;
pub mod python_ui_td;
pub mod reference_context_service;
pub mod review_service;
pub mod spec_service;
pub mod tech_stack_service;

// Re-export commonly used types
pub use crate::models::spec_rules::{ApiSpecType, SpecType};
pub use file_service::{list_specs, read_file};
pub use implementation_service::{list_changed_files, read_all_requirements};
pub use knowledge_service::write_main_spec;
pub use platform_sync::{
    GitHubProvider, PlatformConfig, PlatformSyncService, SyncPayload, SyncResult, SyncStatus,
};
pub use spec_service::{
    create_spec, resolve_section_rules, ApiSpecData, CreateSpecInput, DiagramData, RequirementData,
    ScenarioData,
};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/services/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete services module facade.
```
