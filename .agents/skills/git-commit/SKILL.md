---
name: git:commit
description: "Commit the current working-tree changes with a meaningful conventional commit message. Use when the user asks to git commit, commit this, or save the current changes as a commit."
user-invocable: true
---

# /git:commit

Turn the current working-tree changes into one well-formed commit, then stop.
This skill does not rebase, push, or open a PR.

## Rules

- Run every git command through `git -c core.fsmonitor=false`. This checkout
  enables `core.fsmonitor`, and a stalled daemon blocks any command that reads
  the index — indefinitely, with no error.
- Inspect the complete diff before staging anything.
- Write a conventional commit message `<type>(<scope>): <summary>`, with a
  body when the change is non-trivial. Do not add generated-with or
  co-authored trailers unless the user explicitly asks.
- Never run bare `git stash` / `git stash pop`. The stash stack is shared
  across worktrees and other sessions.
- Do not commit on top of an unresolved merge, rebase, cherry-pick, or
  revert. Stop and report it instead.
- When the user scoped the commit to certain paths, stage only those paths.

## Instructions

### Step 0: Preflight

```bash
git -c core.fsmonitor=false status --short
git -c core.fsmonitor=false branch --show-current
```

If a merge, rebase, cherry-pick, or revert is in progress, stop and report.
If there are no changes at all, report that there is nothing to commit and
stop.

### Step 1: Inspect

Read the complete diff before staging:

```bash
git -c core.fsmonitor=false diff --stat
git -c core.fsmonitor=false diff
git -c core.fsmonitor=false diff --cached --stat
git -c core.fsmonitor=false diff --cached
git -c core.fsmonitor=false status --short
```

### Step 2: Stage and commit

Stage all intended changes — everything, unless the user scoped the commit:

```bash
git -c core.fsmonitor=false add -A
git -c core.fsmonitor=false commit -m "<type>(<scope>): <summary>"
```

### Step 3: Report

```bash
git -c core.fsmonitor=false log -1 --stat
git -c core.fsmonitor=false status --short
```

Report the commit subject, the file count, and anything deliberately left
uncommitted.
