---
id: sdd-tools-review-change-spec-definitions
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools review change spec definitions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 48 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 186 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 129 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/review_change_spec.rs | function | pub | 24 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// MCP tool definition for sdd_workflow_review_change_spec
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_review_change_spec".to_string(),
        description: "Return review prompt for a change spec (whole-file review)".to_string(),
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

/// MCP tool definition for sdd_artifact_review_change_spec
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_review_change_spec".to_string(),
        description: "Write inline review for a change spec with `# Reviews` section".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "spec_id", "verdict", "summary"],
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
                "spec_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Spec ID being reviewed"
                },
                "verdict": {
                    "type": "string",
                    "enum": ["APPROVED", "REVIEWED", "REJECTED"],
                    "description": "Review verdict"
                },
                "summary": {
                    "type": "string",
                    "description": "Review summary"
                },
                "checklist_results": {
                    "type": "array",
                    "description": "Checklist results",
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
                    "description": "Issues found during review",
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
                "problem_sections": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": ["overview", "requirements", "scenarios", "db-model", "dependency", "state-machine", "logic", "interaction", "mindmap", "rest-api", "rpc-api", "async-api", "cli", "schema", "config", "wireframe", "component", "design-token", "unit-test", "e2e-test", "changes", "doc"]
                    },
                    "description": "Sections that need revision (used by revise flow)"
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
  - path: projects/agentic-workflow/src/tools/review_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:missing-generator:sdd-review-spec-tool-definition-json-schema>"
    description: "Review-change-spec workflow and artifact MCP tool definitions."
```
