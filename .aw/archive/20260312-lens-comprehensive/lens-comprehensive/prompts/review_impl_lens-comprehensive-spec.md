# Task: Review Implementation of Spec 'lens-comprehensive-spec' for Change 'lens-comprehensive'

All cclab MCP tools require `project_path="/Users/chris.cheng/cclab/cclab-lens"`

## Instructions

1. Read spec 'lens-comprehensive-spec' via `sdd_read_artifact(scope="read_path:changes/lens-comprehensive/specs/lens-comprehensive-spec.md")`
2. Read implementation diff via `sdd_read_artifact(scope="read_path:changes/lens-comprehensive/implementation.md")`
3. List changed files via `sdd_list_changed_files`
4. Review code changes against spec requirements
5. Write review via `sdd_artifact_review_change_implementation`

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-lens", scope="read_path:changes/lens-comprehensive/specs/lens-comprehensive-spec.md")
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-lens", scope="read_path:changes/lens-comprehensive/implementation.md")
mcp__cclab-mcp__sdd_list_changed_files(project_path="/Users/chris.cheng/cclab/cclab-lens", change_id="lens-comprehensive")
mcp__cclab-mcp__sdd_artifact_review_change_implementation(project_path="/Users/chris.cheng/cclab/cclab-lens", change_id="lens-comprehensive", spec_id="lens-comprehensive-spec", verdict="...", summary="...", checklist_results=[...], issues=[...])
```