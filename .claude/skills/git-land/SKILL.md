---
name: git:land
description: "Land the current branch end-to-end: commit changes, rebase on origin/main, push, open or reuse a PR to main, merge it, then rebase and push the working branch back onto origin/main. Use when the user asks to git land, land this branch, publish, PR to main and merge, or gives the sequence commit/rebase/push/PR/merge/rebase."
user-invocable: true
---

# /git:land

Land the current branch through GitHub, then sync the working branch back to the
merged `origin/main` state.

## Rules

- Execute the phases in this order: commit, rebase `origin/main`, push, PR to
  `main`, merge, rebase `origin/main`.
- Treat `origin/main` as the remote branch backing GitHub PR base `main`; `gh pr
  create` uses `--base main`.
- Request escalation for sandboxed commands that write Git refs/indexes or use
  the network, including `git fetch`, `git push`, and `gh`.
- Do not delete the current branch unless the user explicitly asks. Preserve
  persistent branches such as `project-*` and `lib-*`.
- Do not merge if required checks fail. Report the failing checks and stop.
- If conflicts occur, resolve them correctly, stage the resolved files, and
  continue the rebase. Ask only when the right resolution cannot be determined.

## Instructions

### Step 0: Preflight

Inspect repository state:

```bash
git status --short
git branch --show-current
git remote -v
git rev-parse --abbrev-ref --symbolic-full-name @{u}
```

If the current branch is `main`, stop and tell the user that landing requires a
non-main source branch.

If a merge, rebase, cherry-pick, or revert is already in progress, inspect it
and finish or abort only with clear evidence. Do not start a new land flow on
top of an unresolved operation.

### Step 1: Commit

Inspect the complete diff before staging:

```bash
git diff --stat
git diff
git diff --cached --stat
git diff --cached
git status --short
```

If there are no local changes, skip the commit and continue only if the branch
already has commits to land.

Otherwise stage all intended repo changes and commit with a meaningful
conventional commit message:

```bash
git add -A
git commit -m "<type>(<scope>): <summary>"
```

Use a body when the change is non-trivial. Do not add generated-with or
co-authored trailers unless the user explicitly asks.

### Step 2: Rebase on origin/main

Fetch and rebase onto the current remote main:

```bash
git fetch origin main
git rebase origin/main
```

On conflict:

```bash
git diff --name-only --diff-filter=U
```

Read each conflicted file, resolve the conflict, then run:

```bash
git add <resolved-file>
git rebase --continue
```

Repeat until the rebase completes.

### Step 3: Push

Push the current branch. If there is no upstream, set it:

```bash
git push -u origin HEAD
```

If the branch already has an upstream and the rebase rewrote commits, push with
lease protection:

```bash
git push --force-with-lease
```

### Step 4: PR to origin/main

Open or reuse a PR from the current branch to GitHub base `main`:

```bash
gh pr list --head "$(git branch --show-current)" --base main --state open --json number,url
gh pr create --base main --head "$(git branch --show-current)" --fill
```

If a PR already exists, reuse it. After creating or finding the PR, inspect its
state:

```bash
gh pr view <pr> --json number,url,state,mergeable,mergeStateStatus,statusCheckRollup
```

If checks are pending, wait or poll when appropriate. If checks fail, stop and
report the failing checks.

### Step 5: Merge

Use the user-requested merge strategy when provided. If none is provided, follow
the repository's local policy when one is discoverable; otherwise use a
non-interactive GitHub merge command with an explicit allowed strategy, usually:

```bash
gh pr merge <pr> --squash
```

If squash merge is unavailable or inappropriate for the repository, use the
allowed explicit alternative, such as `--merge`. Do not pass `--delete-branch`
unless the user asked to delete the branch.

After merging, verify the PR is merged:

```bash
gh pr view <pr> --json state,mergedAt,mergeCommit,url
```

### Step 6: Rebase on origin/main again

Sync the still-current working branch back to the merged main:

```bash
git fetch origin main
git rebase origin/main
```

Then push the synchronized branch:

```bash
git push --force-with-lease
```

Verify the final state:

```bash
git status --short
git rev-list --left-right --count HEAD...origin/main
git rev-list --left-right --count HEAD...@{u}
```

The clean finish is an empty status, `HEAD...origin/main` equal to `0 0`, and
`HEAD...@{u}` equal to `0 0`. If any verification differs, report the exact
divergence and what remains.
