---
id: sdd-tools-mod-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools module preamble

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/tools/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ToolDefinition` | apps/agentic-workflow/src/tools/mod.rs | struct | pub | 66 |  |
| `ToolRegistry` | apps/agentic-workflow/src/tools/mod.rs | struct | pub | 77 |  |
| `analyze` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 8 |  |
| `artifact_read` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 9 |  |
| `artifact_write` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 10 |  |
| `call_tool` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 238 | call_tool(&self, name: &str, arguments: &Value) -> Result<String> |
| `call_tool_streaming` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 377 | call_tool_streaming(         &self,         name: &str,         arguments: &Value,         _tx: Option<mpsc::Sender<String>>,     ) -> Result<String> |
| `clarifications` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 11 |  |
| `common_change_impl` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 12 |  |
| `common_change_spec` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 13 |  |
| `common_reference_context` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 14 |  |
| `context` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 15 |  |
| `create_change_docs` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 16 |  |
| `create_change_impl` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 17 |  |
| `create_change_merge` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 18 |  |
| `create_change_spec` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 19 |  |
| `create_post_clarifications` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 20 |  |
| `create_pre_clarifications` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 21 |  |
| `create_reference_context` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 22 |  |
| `fetch_issues` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 41 |  |
| `fill_issue_reference_context` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 23 |  |
| `get_optional_string` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 410 | get_optional_string(args: &Value, field: &str) -> Option<String> |
| `get_required_array` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 418 | get_required_array(args: &Value, field: &str) -> Result<Vec<Value>> |
| `get_required_object` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 427 | get_required_object(args: &Value, field: &str) -> Result<Value> |
| `get_required_string` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 396 | get_required_string(args: &Value, field: &str) -> Result<String> |
| `implementation` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 37 |  |
| `init_change` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 24 |  |
| `knowledge` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 38 |  |
| `list_tools` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 219 | list_tools(&self) -> Vec<Value> |
| `merge_git_ops` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 25 |  |
| `new` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 88 | new() -> Self |
| `new_for_stage` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 93 | new_for_stage(stage: &str) -> Self |
| `phase_transition` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 39 |  |
| `platform_sync` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 42 |  |
| `read` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 45 |  |
| `resolve_project_path` | apps/agentic-workflow/src/tools/mod.rs | function | pub | 468 | resolve_project_path(args: &Value) -> Result<PathBuf> |
| `review` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 26 |  |
| `review_change_docs` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 27 |  |
| `review_change_impl` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 28 |  |
| `review_change_spec` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 29 |  |
| `review_helpers` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 46 |  |
| `review_reference_context` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 30 |  |
| `revise_change_docs` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 31 |  |
| `revise_change_impl` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 32 |  |
| `revise_change_spec` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 33 |  |
| `revise_reference_context` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 34 |  |
| `spec` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 47 |  |
| `spec_plan` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 35 |  |
| `task` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 48 |  |
| `validate` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 49 |  |
| `validate_proposal` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 50 |  |
| `validate_spec` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 51 |  |
| `workflow_common` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 43 |  |
| `workflow_validate` | apps/agentic-workflow/src/tools/mod.rs | module | pub | 52 |  |
## Source
<!-- type: source lang: rust -->

````rust
//! MCP Tool Registry and Implementations
//!
//! Each tool provides structured input validation and generates properly formatted
//! markdown files, eliminating format errors from free-form LLM output.

pub mod analyze;
pub mod artifact_read;
pub mod artifact_write;
pub mod clarifications;
pub mod common_change_impl;
pub mod common_change_spec;
pub mod common_reference_context;
pub mod context;
pub mod create_change_docs;
pub mod create_change_impl;
pub mod create_change_merge;
pub mod create_change_spec;
pub mod create_post_clarifications;
pub mod create_pre_clarifications;
pub mod create_reference_context;
pub mod fill_issue_reference_context;
pub mod init_change;
pub mod merge_git_ops;
pub mod review;
pub mod review_change_docs;
pub mod review_change_impl;
pub mod review_change_spec;
pub mod review_reference_context;
pub mod revise_change_docs;
pub mod revise_change_impl;
pub mod revise_change_spec;
pub mod revise_reference_context;
pub mod spec_plan;

pub mod implementation;
pub mod knowledge;
pub mod phase_transition;

pub mod fetch_issues;
pub mod platform_sync;
pub mod workflow_common;

pub mod read;
pub mod review_helpers;
pub mod spec;
pub mod task;
pub mod validate;
pub mod validate_proposal;
pub mod validate_spec;
pub mod workflow_validate;
pub use crate::generate;

use crate::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/tools/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
    description: "Module declarations, re-exports, and imports for the SDD MCP tool registry facade."
```
