# Task: Begin Implementation for Change 'lens-comprehensive'

All cclab MCP tools require `project_path="/Users/chris.cheng/cclab/cclab-lens"`

## Instructions

1. List all change specs via `sdd_read_artifact(scope="read_path:changes/lens-comprehensive/specs")`
2. Read spec **lens-comprehensive-spec** to understand requirements
3. Implement code for each change spec in order, starting with **lens-comprehensive-spec**
4. Run tests to verify
5. When done with lens-comprehensive-spec, call `sdd_workflow_create_change_implementation()` to advance

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-lens", scope="read_path:changes/lens-comprehensive/specs/lens-comprehensive-spec.md")
mcp__cclab-mcp__sdd_workflow_create_change_implementation(project_path="/Users/chris.cheng/cclab/cclab-lens", change_id="lens-comprehensive")
```