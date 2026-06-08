---
id: sdd-tools-create-reference-context-definitions
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create reference context definitions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_reference_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_reference_context.rs | function | pub | 51 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_reference_context.rs | function | pub | 269 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_reference_context.rs | function | pub | 168 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_reference_context.rs | function | pub | 26 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// MCP tool definition for sdd_workflow_create_reference_context
/// @spec projects/agentic-workflow/tech-design/core/logic/remaining-fixes.md#changes
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_create_reference_context".to_string(),
        description: "Orchestrate per-group reference context lifecycle (create/review/revise)"
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

/// MCP tool definition for sdd_artifact_create_reference_context
/// @spec projects/agentic-workflow/tech-design/core/logic/remaining-fixes.md#changes
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_create_reference_context".to_string(),
        description: "Write reference context for a group — supports both legacy (full specs array) and section-loop (section + content) modes".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "group_id"],
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
                    "description": "Group ID to write reference context for"
                },
                "section": {
                    "type": "string",
                    "description": "Section name to fill (section-loop mode). One of: source_refs, related_specs, reproductions, related_issues, first_fix"
                },
                "content": {
                    "type": "string",
                    "description": "Section content (section-loop mode). Used with 'section' parameter."
                },
                "specs": {
                    "type": "array",
                    "description": "Legacy mode: full specs array for one-shot write",
                    "items": {
                        "type": "object",
                        "required": ["spec_id", "spec_group", "relevance"],
                        "properties": {
                            "spec_id": {
                                "type": "string",
                                "description": "Spec ID (e.g. 'create-pre-clarifications')"
                            },
                            "spec_group": {
                                "type": "string",
                                "description": "Spec group path (e.g. 'sdd/tools/workflows')"
                            },
                            "relevance": {
                                "type": "string",
                                "enum": ["high", "medium", "low"],
                                "description": "How relevant this spec is to the group"
                            },
                            "key_requirements": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Key requirement IDs from the spec (e.g. ['R1', 'R3'])"
                            }
                        }
                    }
                },
                "spec_plan": {
                    "type": "array",
                    "description": "Optional spec plan entries for this group",
                    "items": {
                        "type": "object",
                        "required": ["spec_id", "action", "main_spec_ref", "sections"],
                        "properties": {
                            "spec_id": {
                                "type": "string",
                                "description": "Spec ID for the change-spec"
                            },
                            "action": {
                                "type": "string",
                                "enum": ["modify", "create"],
                                "description": "Whether to modify existing spec or create new"
                            },
                            "main_spec_ref": {
                                "type": "string",
                                "description": "Target path in .aw/tech-design/ (REQUIRED — must reside in a named subfolder, min 4 path components: {category}/{crate}/{subdir}/{file}.md)"
                            },
                            "source": {
                                "type": "string",
                                "description": "Source path for modify action (relative to .aw/tech-design/)"
                            },
                            "sections": {
                                "type": "array",
                                "items": {
                                    "type": "string",
                                    "enum": [
                                        "overview", "changes",
                                        "rest-api", "rpc-api", "async-api", "cli",
                                        "schema", "logic", "interaction",
                                        "state-machine", "db-model",
                                        "unit-test", "e2e-test", "dependency",
                                        "wireframe", "component", "design-token",
                                        "config", "runtime-image", "deployment",
                                        "e2e-scenario", "test-fixture", "perf-test",
                                        "threat-model", "auth-matrix", "security-test",
                                        "container", "deploy", "cloud-resource",
                                        "pipeline", "observability",
                                        "grpc", "graphql",
                                        "model", "prompt"
                                    ]
                                },
                                "description": "Section types this spec needs. Determined by rule engine + agent input."
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
  - path: projects/agentic-workflow/src/tools/create_reference_context.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:missing-generator:sdd-reference-context-tool-definition-json-schema>"
    description: "Create-reference-context workflow and artifact MCP tool definitions."
```
