# Task: Begin Implementation for Change 'mamba-native-stdlib'

All cclab MCP tools require `project_path="/Users/chrischeng/projects/cclab"`

## Instructions

1. List all change specs via `sdd_read_artifact(scope="read_path:changes/mamba-native-stdlib/specs")`
2. Read spec **mamba-native-stdlib-spec** to understand requirements
3. Implement code for each change spec in order, starting with **mamba-native-stdlib-spec**
4. Run tests to verify
5. When done with mamba-native-stdlib-spec, call `sdd_workflow_create_change_implementation()` to advance

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chrischeng/projects/cclab", scope="read_path:changes/mamba-native-stdlib/specs/mamba-native-stdlib-spec.md")
mcp__cclab-mcp__sdd_workflow_create_change_implementation(project_path="/Users/chrischeng/projects/cclab", change_id="mamba-native-stdlib")
```