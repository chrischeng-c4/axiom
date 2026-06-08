# Task: Review Implementation of Spec 'mamba-test-coverage-spec' for Change 'mamba-test-coverage'

All cclab MCP tools require `project_path="/Users/chrischeng/projects/cclab"`

## Instructions

1. Read spec 'mamba-test-coverage-spec' via `sdd_read_artifact(scope="read_path:changes/mamba-test-coverage/specs/mamba-test-coverage-spec.md")`
2. Read implementation diff via `sdd_read_artifact(scope="read_path:changes/mamba-test-coverage/implementation.md")`
3. List changed files via `sdd_list_changed_files`
4. Review code changes against spec requirements
5. Write review via `sdd_artifact_review_change_implementation`

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chrischeng/projects/cclab", scope="read_path:changes/mamba-test-coverage/specs/mamba-test-coverage-spec.md")
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chrischeng/projects/cclab", scope="read_path:changes/mamba-test-coverage/implementation.md")
mcp__cclab-mcp__sdd_list_changed_files(project_path="/Users/chrischeng/projects/cclab", change_id="mamba-test-coverage")
mcp__cclab-mcp__sdd_artifact_review_change_implementation(project_path="/Users/chrischeng/projects/cclab", change_id="mamba-test-coverage", spec_id="mamba-test-coverage-spec", verdict="...", summary="...", checklist_results=[...], issues=[...])
```