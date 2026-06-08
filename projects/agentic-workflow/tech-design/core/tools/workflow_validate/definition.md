---
id: sdd-tools-workflow-validate-definition
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools workflow validate definition

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/workflow_validate.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/workflow_validate.rs | function | pub | 24 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/workflow_validate.rs | function | pub | 57 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
/// @spec projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_validate".to_string(),
        description: "Validate artifact output of a score-* subagent and advance phase on pass."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "agent_type"],
            "properties": {
                "project_path": { "type": "string" },
                "change_id": { "type": "string" },
                "agent_type": {
                    "type": "string",
                    "enum": [
                        "score-issue-author",
                        "score-change-spec",
                        "score-change-implementation",
                        "score-review",
                    ]
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
  - path: projects/agentic-workflow/src/tools/workflow_validate.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:missing-generator:sdd-workflow-validate-tool-definition-json-schema>"
    description: "Workflow validation MCP tool definition."
```
