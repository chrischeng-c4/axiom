# Task: Revise Implementation of Spec 'mamba-thread-safe-spec' for Change 'mamba-thread-safe'

All cclab MCP tools require `project_path="/Users/chrischeng/projects/cclab"`

## Instructions

1. Read `implementation.md` for the inline `## Review: mamba-thread-safe-spec` section
2. Fix all identified issues in the code
3. Re-run tests to verify
4. When done, call `sdd_run_change(advance_to="change_implementation_revised")` to advance

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chrischeng/projects/cclab", scope="read_path:changes/mamba-thread-safe/implementation.md")
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chrischeng/projects/cclab", scope="read_path:changes/mamba-thread-safe/specs/mamba-thread-safe-spec.md")
mcp__cclab-mcp__sdd_run_change(project_path="/Users/chrischeng/projects/cclab", change_id="mamba-thread-safe", advance_to="change_implementation_revised")
```