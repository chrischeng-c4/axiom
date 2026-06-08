---
name: git:rebase:main
description: Rebase current branch onto main and force push
user-invocable: true
---

# /git:rebase:main

Rebase the current feature branch onto `main` and force push.

## Instructions

### Step 1: Check current branch

```bash
git branch --show-current
```

If the current branch is `main`, tell the user **"Already on main, nothing to rebase."** and stop.

### Step 2: Fetch latest main

```bash
git fetch origin main
```

### Step 3: Rebase onto main

```bash
git rebase origin/main
```

If the rebase succeeds with no conflicts, skip to Step 4.

If there are **conflicts**:
1. List conflicted files with `git diff --name-only --diff-filter=U`.
2. For each conflicted file, read it, understand the conflict markers, and resolve by keeping the correct combination of both sides.
3. After resolving each file, stage it with `git add <file>`.
4. Continue the rebase with `git rebase --continue`.
5. Repeat until the rebase completes.

### Step 4: Force push

Ask the user for confirmation before force pushing, then run:

```bash
git push --force-with-lease
```

Report the result to the user.
