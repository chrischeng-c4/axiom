---
id: sdd-tools-implementation-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Implementation Tools Preamble

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/implementation.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `create_merge_review_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 687 | create_merge_review_definition() -> ToolDefinition |
| `create_review_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 477 | create_review_definition() -> ToolDefinition |
| `execute_create_merge_review` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 781 | execute_create_merge_review(args: &Value, project_root: &Path) -> Result<String> |
| `execute_create_review` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 559 | execute_create_review(args: &Value, project_root: &Path) -> Result<String> |
| `execute_list_changed_files` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 356 | execute_list_changed_files(args: &Value, _project_root: &Path) -> Result<String> |
| `execute_read_all_requirements` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 117 | execute_read_all_requirements(args: &Value, project_root: &Path) -> Result<String> |
| `execute_read_implementation_summary` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 232 | execute_read_implementation_summary(args: &Value, _project_root: &Path) -> Result<String> |
| `list_changed_files_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 323 | list_changed_files_definition() -> ToolDefinition |
| `read_all_requirements_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 93 | read_all_requirements_definition() -> ToolDefinition |
| `read_implementation_summary_definition` | projects/agentic-workflow/src/tools/implementation.rs | function | pub | 204 | read_implementation_summary_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
//! Implementation Support MCP Tools
//!
//! Tools for reading requirements and implementation summaries to support
//! the implementation and review workflow.

use super::{get_optional_string, get_required_array, get_required_string, ToolDefinition};
use crate::models::state::StatePhase;
use crate::services::implementation_service::{
    CreateMergeReviewInput, CreateReviewInput, MergeQuality, MergeReviewIssue, MergeReviewVerdict,
    ReviewIssue, ReviewVerdict, Severity, TestResults,
};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/implementation.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
    description: "Module documentation and imports for implementation support MCP tools."
```
