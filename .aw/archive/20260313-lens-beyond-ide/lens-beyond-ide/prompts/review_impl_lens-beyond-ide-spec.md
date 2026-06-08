# Task: Review Implementation of Spec 'lens-beyond-ide-spec' for Change 'lens-beyond-ide'

All cclab MCP tools require `project_path="/Users/chris.cheng/cclab/cclab-lens"`

## Instructions

1. Read spec 'lens-beyond-ide-spec' via `sdd_read_artifact(scope="read_path:changes/lens-beyond-ide/specs/lens-beyond-ide-spec.md")`
2. Read implementation diff via `sdd_read_artifact(scope="read_path:changes/lens-beyond-ide/implementation.md")`
3. List changed files via `sdd_list_changed_files`
4. Review code changes against spec requirements
5. Write review via `sdd_artifact_review_change_implementation`

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-lens", scope="read_path:changes/lens-beyond-ide/specs/lens-beyond-ide-spec.md")
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-lens", scope="read_path:changes/lens-beyond-ide/implementation.md")
mcp__cclab-mcp__sdd_list_changed_files(project_path="/Users/chris.cheng/cclab/cclab-lens", change_id="lens-beyond-ide")
mcp__cclab-mcp__sdd_artifact_review_change_implementation(project_path="/Users/chris.cheng/cclab/cclab-lens", change_id="lens-beyond-ide", spec_id="lens-beyond-ide-spec", verdict="...", summary="...", checklist_results=[...], issues=[...])
```