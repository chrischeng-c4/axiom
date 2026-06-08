---
id: sdd-tools-clarifications-definition
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools clarifications definition

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/clarifications.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 15 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 68 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_append` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 312 | execute_append(args: &Value, project_root: &Path) -> Result<String> |
| `execute_post_clarifications` | projects/agentic-workflow/src/tools/clarifications.rs | function | pub | 198 | execute_post_clarifications(     args: &Value,     project_root: &Path,     _action: &str, ) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
use super::{get_required_array, get_required_string, ToolDefinition};
use crate::models::state::StatePhase;
use crate::services::pre_clarifications_service::{
    append_clarifications as service_append, AppendClarificationsInput, QuestionAnswer,
};
use crate::state::StateManager;
use crate::Result;
use chrono::Local;
use serde_json::{json, Value};
use std::path::Path;

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_create_clarifications".to_string(),
        description: "Create context_clarifications.md with structured Q&A from user".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "questions"],
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
                "questions": {
                    "type": "array",
                    "minItems": 1,
                    "description": "Array of Q&A pairs",
                    "items": {
                        "type": "object",
                        "required": ["topic", "question", "answer", "rationale"],
                        "properties": {
                            "topic": {
                                "type": "string",
                                "description": "Short topic label (e.g., 'Authentication Method')"
                            },
                            "question": {
                                "type": "string",
                                "description": "The question asked to the user"
                            },
                            "answer": {
                                "type": "string",
                                "description": "User's answer"
                            },
                            "rationale": {
                                "type": "string",
                                "description": "Why this choice was made"
                            }
                        }
                    }
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
  - path: projects/agentic-workflow/src/tools/clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
      - "definition"
    description: "Clarifications tool imports and MCP tool schema definition."
```
