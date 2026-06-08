---
id: sdd-tools-review-preamble-definition
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review preamble definition

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/review.rs | function | pub | 40 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/review.rs | function | pub | 135 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
//! sdd_review_file MCP Tool
//!
//! Unified review tool for decide-stage contexts/gaps and plan-stage proposal/spec.
//! For in-scope artifacts (contexts, gaps, proposal, spec), writes reviews as
//! an inline `# Reviews` section inside the original artifact file.
//! For implementation reviews, writes separate `review_impl*.md` files.
//!
//! Auto-updates STATE.yaml phase after writing the review artifact.

use super::review_helpers::{
    build_review_section, remove_frontmatter_field, strip_review_section, upsert_frontmatter_field,
};
use super::{get_optional_string, get_required_string, ToolDefinition};
use crate::models::state::StatePhase;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Valid review file types
const VALID_FILES: &[&str] = &[
    // Decide stage (8)
    "context_clarifications",
    "spec_clarifications",
    "spec_context",
    "knowledge_context",
    "codebase_context",
    "gap_codebase_spec",
    "gap_codebase_knowledge",
    "gap_spec_knowledge",
    // Plan stage (2)
    "proposal",
    "spec",
    // Impl stage (1)
    "implementation",
];

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_review_file".to_string(),
        description: "Write a structured review artifact (review_{file}.md). Covers decide-stage contexts/gaps and plan-stage proposal/spec.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "file", "verdict", "summary"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID (lowercase, hyphens allowed)"
                },
                "caller": {
                    "type": "string",
                    "enum": ["agent", "mainthread"],
                    "default": "mainthread",
                    "description": "Who is calling: agent (via sdd_delegate_agent) or mainthread. Controls whether next dispatch hint is included in response."
                },
                "file": {
                    "type": "string",
                    "enum": VALID_FILES,
                    "description": "Artifact type being reviewed"
                },
                "verdict": {
                    "type": "string",
                    "enum": ["APPROVED", "REVIEWED", "REJECTED"],
                    "description": "Review verdict"
                },
                "summary": {
                    "type": "string",
                    "minLength": 10,
                    "description": "Summary of review findings"
                },
                "checklist_results": {
                    "type": "array",
                    "description": "Checklist items with pass/fail",
                    "items": {
                        "type": "object",
                        "required": ["item", "passed"],
                        "properties": {
                            "item": { "type": "string" },
                            "passed": { "type": "boolean" },
                            "note": { "type": "string" }
                        }
                    }
                },
                "issues": {
                    "type": "array",
                    "default": [],
                    "description": "List of issues found",
                    "items": {
                        "type": "object",
                        "required": ["severity", "description"],
                        "properties": {
                            "severity": {
                                "type": "string",
                                "enum": ["HIGH", "MEDIUM", "LOW"]
                            },
                            "description": { "type": "string" },
                            "recommendation": { "type": "string" }
                        }
                    }
                },
                "spec_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Spec ID (required when file='spec')"
                },
                "task_id": {
                    "type": "string",
                    "description": "Task ID for per-task review (when file='implementation'). Generates review_impl_{task_id}.md instead of global review_impl.md"
                },
                "iteration": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 1,
                    "description": "Review iteration number"
                }
            }
        }),
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/review.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "definition"
    description: "Module preamble, valid review file list, and review tool definition."
```
