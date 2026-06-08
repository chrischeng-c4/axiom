---
number: 1136
title: "feat(sdd): platform config restructure + merge auto-commit/PR"
state: open
labels: [type:enhancement, priority:p1, crate:sdd]
group: "platform-config-and-merge-git"
---

# #1136 — feat(sdd): platform config restructure + merge auto-commit/PR

## Summary

Restructure `cclab/config.toml` to separate platform concerns and add git operations to the SDD merge workflow.

### Config restructure

Split `[sdd]` into distinct platform sections:

- `[sdd.issue_platform]` — issue tracking (existing, unchanged)
- `[sdd.repo_platform]` — codebase + PR + commit operations (new)
- `[sdd.spec_platform]` — spec storage (new, `type = "local"` for now)
- `[sdd.docs_platform]` — user-facing docs (future, commented out)

### Merge workflow enhancements

After `ChangeArchived`, automatically:

1. **git commit** — stage affected paths (`cclab/specs/`, `cclab/changes/`, `cclab/archive/`) and commit with conventional message. Controlled by `repo_platform.auto_commit`.
2. **Open PR** (optional) — create PR to `repo_platform.default_branch`. Controlled by `repo_platform.auto_pr`.

### Config shape

```toml
[sdd.issue_platform]
type = "github"
repo = "chrischeng-c4/cclab"
auth_method = "cli"

[sdd.repo_platform]
type = "github"
repo = "chrischeng-c4/cclab"
default_branch = "main"
auto_commit = true
auto_pr = false

[sdd.spec_platform]
type = "local"
path = "cclab/specs"

# [sdd.docs_platform]
# type = "github_pages"
```

