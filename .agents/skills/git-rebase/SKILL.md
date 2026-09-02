---
name: git:rebase
description: "Rebase the current branch onto a named branch, or onto origin/main when no branch is given. Use when the user asks to git rebase, rebase on <branch>, sync with main, or catch this branch up."
user-invocable: true
---

# /git:rebase

Rebase the current branch onto one target ref, then stop. This skill does not
commit, push, stash, or open a PR.

## Rules

- The argument is the target branch. With no argument the target is
  `origin/main`, fetched first.
- Run every git command through `git -c core.fsmonitor=false`. This checkout
  enables `core.fsmonitor`, and a stalled daemon blocks any command that reads
  the index — indefinitely, with no error.
- Never run bare `git stash` / `git stash pop`. The stash stack is shared
  across worktrees and other sessions. A dirty tree stops this skill instead.
- Do not push. Report the post-rebase divergence and leave publishing to the
  user (or `/git:land`).
- Preserve persistent refs: `main`, `app/*`, `lib/*`, `project-mamba`,
  `project-lumen`, `examples`. Rebasing the current branch is fine; never
  delete or force-move any other ref.
- If conflicts occur, resolve them correctly, stage the resolved files, and
  continue the rebase. Ask only when the right resolution cannot be
  determined. Abort (`git rebase --abort`) rather than guess.

## Instructions

### Step 0: Preflight

```bash
git -c core.fsmonitor=false status --short
git -c core.fsmonitor=false branch --show-current
```

Stop and report — without starting the rebase — when any of these holds:

- the working tree or index is dirty (the user must commit or set the work
  aside first; do not commit or stash for them);
- a merge, rebase, cherry-pick, or revert is already in progress;
- the current branch is the target itself.

### Step 1: Resolve the target

No argument:

```bash
git -c core.fsmonitor=false fetch origin main
```

The target is `origin/main`.

With an argument `<branch>`: verify it resolves with
`git -c core.fsmonitor=false rev-parse --verify --quiet <branch>`. If it does
not resolve locally, fetch and try `origin/<branch>`. If neither resolves,
stop and report.

### Step 2: Rebase

```bash
git -c core.fsmonitor=false rebase <target>
```

On conflict:

```bash
git -c core.fsmonitor=false diff --name-only --diff-filter=U
```

Read each conflicted file, resolve the conflict, then:

```bash
git -c core.fsmonitor=false add <resolved-file>
git -c core.fsmonitor=false rebase --continue
```

Repeat until the rebase completes.

### Step 3: Report

```bash
git -c core.fsmonitor=false status --short
git -c core.fsmonitor=false rev-list --left-right --count HEAD...<target>
git -c core.fsmonitor=false log --oneline -5
```

The clean finish is an empty status and `HEAD...<target>` reading `N 0`
(local commits ahead, nothing behind). Report the exact counts, and — if the
branch has an upstream the rebase diverged from — say that the next push
needs `--force-with-lease`, without running it.
