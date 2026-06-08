---
id: sdd-tools-spec-definition
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools spec definition

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
/// Get the tool definition for create_spec
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_create_spec".to_string(),
        description: "Create a validated spec file with requirements and acceptance criteria. Supports structured diagrams for spec-to-code generation."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "spec_id", "title", "overview", "spec_type", "requirements", "scenarios"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "The change ID this spec belongs to"
                },
                "caller": {
                    "type": "string",
                    "enum": ["agent", "mainthread"],
                    "default": "mainthread",
                    "description": "Who is calling: agent (via sdd_delegate_agent) or mainthread. Controls whether next dispatch hint is included in response."
                },
                "spec_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Unique identifier for this spec (lowercase, hyphens allowed)"
                },
                "title": {
                    "type": "string",
                    "description": "Human-readable title for the spec"
                },
                "overview": {
                    "type": "string",
                    "minLength": 50,
                    "description": "Overview of what this spec covers"
                },
                "spec_type": {
                    "type": "string",
                    "enum": ["http-api", "event-driven", "data-model", "algorithm", "integration", "utility", "rpc-api", "workflow"],
                    "description": "Spec type classification. Determines required elements: http-api (sequence diagram + OpenAPI), event-driven (sequence + AsyncAPI), data-model (erd/class diagram), algorithm (flowchart/state), integration (sequence), utility (none), rpc-api (class diagram + OpenRPC), workflow (state/flowchart + Serverless Workflow)"
                },
                "requirements": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["id", "title", "description"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "pattern": "^R\\d+$",
                                "description": "Requirement ID (e.g., R1, R2)"
                            },
                            "title": {
                                "type": "string",
                                "description": "Short requirement title"
                            },
                            "description": {
                                "type": "string",
                                "description": "Detailed requirement description"
                            },
                            "priority": {
                                "enum": ["high", "medium", "low"],
                                "default": "medium",
                                "description": "Requirement priority"
                            }
                        }
                    },
                    "description": "List of requirements"
                },
                "scenarios": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["name", "when", "then"],
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Scenario name"
                            },
                            "given": {
                                "type": "string",
                                "description": "Optional precondition"
                            },
                            "when": {
                                "type": "string",
                                "description": "Action or trigger"
                            },
                            "then": {
                                "type": "string",
                                "description": "Expected outcome"
                            }
                        }
                    },
                    "description": "Acceptance scenarios in Given/When/Then format"
                },
                "diagrams": {
                    "type": "array",
                    "description": "Structured diagram definitions using Mermaid tool schemas. Preferred over flow_diagram.",
                    "items": {
                        "type": "object",
                        "required": ["type", "title", "input"],
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["flowchart", "sequence", "class", "state", "erd", "mindmap", "requirement", "journey"],
                                "description": "Diagram type (matches generate_mermaid_* tool)"
                            },
                            "title": {
                                "type": "string",
                                "description": "Human-readable title for the diagram"
                            },
                            "input": {
                                "type": "object",
                                "description": "Input matching the corresponding generate_mermaid_* tool schema"
                            }
                        }
                    }
                },
                "flow_diagram": {
                    "type": "string",
                    "description": "DEPRECATED: Use 'diagrams' field instead. Raw Mermaid diagram code."
                },
                "data_model": {
                    "type": "object",
                    "description": "Optional JSON Schema for data model"
                },
                "api_spec": {
                    "type": "object",
                    "description": "API specification (OpenAPI 3.1, AsyncAPI 2.6, JSON Schema, OpenRPC 1.3, or Serverless Workflow 0.8) for code generation",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["openapi-3.1", "asyncapi-2.6", "json-schema", "openrpc-1.3", "serverless-workflow-0.8"],
                            "description": "API specification format"
                        },
                        "spec": {
                            "type": "object",
                            "description": "Full API specification object"
                        }
                    },
                    "required": ["type", "spec"]
                },
                "spec_group": {
                    "type": "string",
                    "pattern": "^[a-z][a-z0-9-]*$",
                    "description": "Spec group for organizing specs (e.g., 'genesis', 'lens', 'auth'). Creates spec in specs/{spec_group}/ subdirectory. Omit for cross-cutting specs."
                },
                "group_id": {
                    "type": "string",
                    "description": "Change group ID for multi-group changes. When set, spec is written to groups/{group_id}/specs/ instead of specs/. Takes priority over spec_group."
                },
                "main_spec_ref": {
                    "type": "string",
                    "description": "Reference to existing main spec that this change spec extends/modifies. Used for traceability during merge. Example: 'auth-flow' means this extends .aw/tech-design/{spec_group}/auth-flow.md"
                },
                "merge_strategy": {
                    "type": "string",
                    "enum": ["new", "extend", "replace", "patch"],
                    "description": "Strategy for merging this spec back to main specs: new (create new), extend (add to existing), replace (overwrite), patch (partial update)"
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Explicit tags (merged with auto-tags from spec_type). Values: api, http, rpc, events, async, data, logic, state, external"
                },
                "depends": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Dependency spec IDs (for topological ordering during creation)"
                },
                "changes": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["file", "action"],
                        "properties": {
                            "file": { "type": "string", "description": "File path relative to project root" },
                            "action": { "type": "string", "enum": ["CREATE", "MODIFY", "DELETE"] },
                            "context_ref": { "type": "string", "description": "Reference to context artifact section" },
                            "description": { "type": "string", "description": "What changes in this file" }
                        }
                    },
                    "description": "File changes associated with this spec"
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
  - path: projects/agentic-workflow/src/tools/spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - definition
    description: "Create-spec tool definition schema."
```
