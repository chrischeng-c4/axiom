---
files:
  - tools/implementation.rs
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd_list_changed_files: List Changed Files with Statistics

Tool that lists files changed between the current branch and a base branch, with detailed per-file addition/deletion statistics. Designed for implementation review workflows where agents need a structured view of what changed.

**Key design**: Uses `git diff --numstat` to produce a markdown table with file path, status classification (Added/Modified/Deleted/Binary), and line-level statistics. Supports optional path filtering.

## OpenRPC Method Definition
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_list_changed_files
summary: List changed files with detailed statistics (additions/deletions)
params:
  - name: project_path
    required: true
    schema:
      type: string
      description: Project root path (use $PWD for current directory)
  - name: change_id
    required: true
    schema:
      type: string
      description: The change ID to list files for
  - name: base_branch
    required: false
    schema:
      type: string
      default: main
      description: "Base branch to compare against (default: 'main')"
  - name: filter
    required: false
    schema:
      type: string
      description: Optional filter pattern (simple string match on file path)
result:
  name: result
  schema:
    type: string
    description: Markdown-formatted table of changed files with per-file and aggregate statistics
```

## Behavior
<!-- type: doc lang: markdown -->

### Input Validation

1. **change_id**: Must be lowercase alphanumeric with hyphens only. Directory traversal patterns (`..`, leading `/` or `\`) are rejected.
2. **Git repository check**: Fails with error if the current working directory is not inside a git repository.

### Execution Flow

1. Runs `git diff --numstat {base_branch}` to collect per-file addition/deletion counts.
2. If `filter` is provided, only files whose path contains the filter string (simple substring match) are included.
3. Parses the tab-separated numstat output into structured records.
4. Classifies each file's status:
   - **Binary**: both added and removed fields are `-`
   - **Added**: removed count is `0`
   - **Deleted**: added count is `0`
   - **Modified**: all other cases
5. Accumulates total added/removed line counts (binary files excluded from totals).

### Output Format

Returns a markdown string with:

- A heading: `# Changed Files for: {change_id}`
- If filtered: `**Filter**: \`{filter}\``
- A markdown table:

```
| File | Status | +Lines | -Lines |
|------|--------|--------|--------|
| path/to/file.rs | Modified | 42 | 10 |
```

- A totals summary: `**Totals**: N files, +X lines, -Y lines`

### Edge Cases

| Condition | Result |
|-----------|--------|
| No changes detected (empty numstat) | Returns `*No changes detected*` |
| Numstat starts with warning marker | Returns `*No changes detected*` |
| Filter matches no files | Returns `*No matching files found*` |
| Binary files | Status shown as "Binary", lines shown as `-`, excluded from totals |

### Workflow Context

This tool is registered in the `implement` and `all` tool stages. It is typically called by review agents to understand the scope of changes before performing detailed code review.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

```