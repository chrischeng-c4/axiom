---
id: sdd-tools-create-pre-clarifications-workflow-definitions
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create pre clarifications workflow definitions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_pre_clarifications.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 287 | artifact_definition() -> ToolDefinition |
| `definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 17 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 70 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `execute_append` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 199 | execute_append(args: &Value, project_root: &Path) -> Result<String> |
| `execute_artifact_pre_clarifications` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 418 | execute_artifact_pre_clarifications(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow_pre_clarifications` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 343 | execute_workflow_pre_clarifications(     args: &Value,     project_root: &Path, ) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_pre_clarifications.rs | function | pub | 262 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// MCP tool definition for sdd_workflow_create_pre_clarifications
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_pre_clarifications".to_string(),
        description: "Return prompt for mainthread to clarify the next incomplete group"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID"
                }
            }
        }),
    }
}

/// MCP tool definition for sdd_artifact_create_pre_clarifications
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_create_pre_clarifications".to_string(),
        description: "Write answers for pre-generated questions in a group".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "group_id", "answers"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID"
                },
                "group_id": {
                    "type": "string",
                    "description": "Group ID to write answers for"
                },
                "answers": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["topic", "answer"],
                        "properties": {
                            "topic": {
                                "type": "string",
                                "description": "Topic matching the pre-generated question"
                            },
                            "answer": {
                                "type": "string",
                                "description": "User's answer to the question"
                            },
                            "follow_up_questions": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Optional follow-up questions raised by the answer"
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
  - path: projects/agentic-workflow/src/tools/create_pre_clarifications.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "workflow_definition"
      - "artifact_definition"
    description: "Group-aware workflow and artifact MCP tool definitions."
```
