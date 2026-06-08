---
id: sdd-tools-spec-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools spec preamble

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 41 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 238 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_review_spec` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 574 | execute_review_spec(args: &Value, project_root: &Path) -> Result<String> |
| `review_spec_definition` | projects/agentic-workflow/src/tools/spec.rs | function | pub | 476 | review_spec_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! create_spec MCP Tool
//!
//! Creates a validated spec file with requirements and acceptance criteria.
#![allow(deprecated)]
//!
//! ## Structured Diagrams
//!
//! The `diagrams` field accepts structured diagram definitions that are validated
//! against their corresponding Mermaid tool schemas. This ensures diagrams are
//! syntactically correct and enables semantic metadata for code generation.
//!
//! Supported diagram types:
//! - `flowchart` - Process flows, algorithms, decision trees (with semantic extensions)
//! - `sequence` - API interactions, message flows
//! - `class` - Data structures, domain models
//! - `state` - State machines, workflow states
//! - `erd` - Database schemas, entity relationships
//! - `mindmap` - Concept organization
//! - `requirement` - Requirement traceability
//! - `journey` - User journeys

use super::{get_optional_string, get_required_array, get_required_string, ToolDefinition};
use crate::models::spec_rules::{ApiSpecType, SpecType};
use crate::models::state::StatePhase;
use crate::services::spec_service::{
    create_spec, ApiSpecData, CreateSpecInput, DiagramData, RequirementData, ScenarioData,
    SpecChangeData,
};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::str::FromStr;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - <module-preamble>
      - <module-trailer>
    description: "Module preamble and whole-file HANDWRITE edge markers."
```
