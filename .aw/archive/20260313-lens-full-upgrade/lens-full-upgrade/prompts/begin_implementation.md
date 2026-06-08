# Task: Begin Implementation for Change 'lens-full-upgrade'

All cclab MCP tools require `project_path="/Users/chris.cheng/cclab/cclab-lens"`

## Instructions

1. List all change specs via `sdd_read_artifact(scope="read_path:changes/lens-full-upgrade/specs")`
2. Read spec **lens-full-upgrade-spec** to understand requirements
3. Implement code for each change spec in order, starting with **lens-full-upgrade-spec**
4. Run tests to verify
5. When done with lens-full-upgrade-spec, call `sdd_workflow_create_change_implementation()` to advance

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-lens", scope="read_path:changes/lens-full-upgrade/specs/lens-full-upgrade-spec.md")
mcp__cclab-mcp__sdd_workflow_create_change_implementation(project_path="/Users/chris.cheng/cclab/cclab-lens", change_id="lens-full-upgrade")
```