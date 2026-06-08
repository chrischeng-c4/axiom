# Task: Review Implementation of Spec 'cclab-mamba-fix-xfail-spec' for Change 'cclab-mamba-fix-xfail'

All cclab MCP tools require `project_path="/Users/chris.cheng/cclab/cclab-mamba"`

## Instructions

1. Read spec 'cclab-mamba-fix-xfail-spec' via `sdd_read_artifact(scope="read_path:changes/cclab-mamba-fix-xfail/specs/cclab-mamba-fix-xfail-spec.md")`
2. Read implementation diff via `sdd_read_artifact(scope="read_path:changes/cclab-mamba-fix-xfail/implementation.md")`
3. List changed files via `sdd_list_changed_files`
4. Review code changes against spec requirements
5. Write review via `sdd_artifact_review_change_implementation`

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-mamba", scope="read_path:changes/cclab-mamba-fix-xfail/specs/cclab-mamba-fix-xfail-spec.md")
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-mamba", scope="read_path:changes/cclab-mamba-fix-xfail/implementation.md")
mcp__cclab-mcp__sdd_list_changed_files(project_path="/Users/chris.cheng/cclab/cclab-mamba", change_id="cclab-mamba-fix-xfail")
mcp__cclab-mcp__sdd_artifact_review_change_implementation(project_path="/Users/chris.cheng/cclab/cclab-mamba", change_id="cclab-mamba-fix-xfail", spec_id="cclab-mamba-fix-xfail-spec", verdict="...", summary="...", checklist_results=[...], issues=[...])
```