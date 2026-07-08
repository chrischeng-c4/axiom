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

# sdd_read_implementation_summary: Git Diff Summary for Review

Tool that produces a markdown-formatted implementation summary by running git commands against the current repository. It reports the current branch (with validation against the expected `cclab/{change_id}` convention), commits ahead of the base branch, changed files with name-status, diff statistics, and a one-line commit log. Designed to give code reviewers a quick overview before diving into individual files.

## OpenRPC Method Definition
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_read_implementation_summary
summary: Get git diff summary and commit log for implementation review
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
      description: The change ID to get implementation summary for
  - name: base_branch
    required: false
    schema:
      type: string
      description: "Base branch to compare against (default: 'main')"
      default: main
result:
  name: result
  schema:
    type: string
    description: Markdown-formatted implementation summary including branch info, changed files, diff stats, and commit log
```

## Behavior
<!-- type: doc lang: markdown -->

### Preconditions

1. **Git repository required** -- the tool calls `git rev-parse --git-dir` to verify it is running inside a git repository. If not, it returns an error.
2. **change_id validation** -- the `change_id` is validated against a strict character set (lowercase ASCII alphanumeric + hyphens only). Directory traversal patterns (`..`, leading `/` or `\`) are rejected.

### Output Sections

The tool assembles a single markdown string with the following sections:

| Section | Git command | Content |
|---------|-------------|---------|
| **Header** | -- | `# Implementation Summary for: {change_id}` |
| **Branch validation** | `git rev-parse --abbrev-ref HEAD` | Current branch name; warning if it does not match `cclab/{change_id}` |
| **Commits ahead** | `git rev-list --count {base_branch}..HEAD` | Integer count of commits ahead of base |
| **Changed Files** | `git diff --name-status {base_branch}` | File-level change list (Added/Modified/Deleted) in a code block |
| **Diff Statistics** | `git diff --stat {base_branch}` | Per-file insertion/deletion summary in a code block |
| **Commit Log** | `git log --oneline {base_branch}..HEAD` | One-line commit messages in a code block |

Each section gracefully handles the empty case (e.g., `*No changes detected*`, `*No commits*`).

### Error Handling

- If a git sub-command fails, the output includes a warning line (`Git command failed: ...`) instead of aborting the entire tool call.
- Missing or invalid `change_id` causes an early error return before any git commands run.
- The `project_root` parameter is accepted but not used for file-system reads; all data comes from git commands executed in the current working directory.

### Branch Naming Convention

The tool expects implementation branches to follow the pattern `cclab/{change_id}`. If the current branch does not match, a warning is emitted but execution continues. This is advisory only -- no branch switching occurs.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

```