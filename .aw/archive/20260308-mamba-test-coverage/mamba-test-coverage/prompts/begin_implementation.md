# Task: Begin Implementation for Change 'mamba-test-coverage'

All cclab MCP tools require `project_path="/Users/chrischeng/projects/cclab"`

## Instructions

1. List all change specs via `sdd_read_artifact(scope="read_path:changes/mamba-test-coverage/specs")`
2. Read spec **mamba-test-coverage-spec** to understand requirements
3. Implement code for each change spec in order, starting with **mamba-test-coverage-spec**
4. Run tests to verify
5. When done with mamba-test-coverage-spec, call `sdd_workflow_create_change_implementation()` to advance

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chrischeng/projects/cclab", scope="read_path:changes/mamba-test-coverage/specs/mamba-test-coverage-spec.md")
mcp__cclab-mcp__sdd_workflow_create_change_implementation(project_path="/Users/chrischeng/projects/cclab", change_id="mamba-test-coverage")
```