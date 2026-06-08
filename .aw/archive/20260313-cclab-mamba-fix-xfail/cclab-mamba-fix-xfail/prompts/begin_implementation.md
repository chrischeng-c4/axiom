# Task: Begin Implementation for Change 'cclab-mamba-fix-xfail'

All cclab MCP tools require `project_path="/Users/chris.cheng/cclab/cclab-mamba"`

## Instructions

1. List all change specs via `sdd_read_artifact(scope="read_path:changes/cclab-mamba-fix-xfail/specs")`
2. Read spec **cclab-mamba-fix-xfail-spec** to understand requirements
3. Implement code for each change spec in order, starting with **cclab-mamba-fix-xfail-spec**
4. Run tests to verify
5. When done with cclab-mamba-fix-xfail-spec, call `sdd_workflow_create_change_implementation()` to advance

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-mamba", scope="read_path:changes/cclab-mamba-fix-xfail/specs/cclab-mamba-fix-xfail-spec.md")
mcp__cclab-mcp__sdd_workflow_create_change_implementation(project_path="/Users/chris.cheng/cclab/cclab-mamba", change_id="cclab-mamba-fix-xfail")
```