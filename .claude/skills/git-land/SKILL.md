---
name: git:land
description: "Land the current branch end-to-end: commit changes, rebase on origin/main, push, open or reuse a PR to origin/main, merge it, then rebase and push the working branch back onto origin/main. Use when the user asks to git land, land this branch, publish, PR to main and merge, or gives the sequence commit/rebase/push/PR/merge/rebase."
user-invocable: true
---

# /git:land

Land the current branch through GitHub, then sync the working branch back to the
merged `origin/main` state.

## Rules

- Execute the phases in this order: commit, rebase `origin/main`, push, PR to
  `origin/main`, merge, rebase `origin/main`.
- The commit phase delegates to `/git:commit`, both rebase phases to
  `/git:rebase` (no argument = `origin/main`), both push phases to
  `/git:push`, the PR phase to `/gh:create-pr`, and the merge phase to
  `/gh:merge-pr`; do not inline a different sequence for any of them.
- Treat `origin/main` as the remote branch backing GitHub PR base `main`; `gh pr
  create` uses `--base main`.
- Request escalation for sandboxed commands that write Git refs/indexes or use
  the network, including `git fetch`, `git push`, and `gh`.
- Do not delete the current branch unless the user explicitly asks. Preserve
  persistent branches such as `app/*`, `lib/*`, `project-mamba`, and
  `project-lumen`.
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

If there are no local changes, skip the commit and continue only if the branch
already has commits to land.

Otherwise run the `/git:commit` skill. It inspects the complete diff, stages
the intended changes, and writes one conventional commit; it does not push,
so continue with Step 2.

### Step 2: Rebase on origin/main

Run the `/git:rebase` skill with no argument. It fetches `origin/main`,
rebases the current branch onto it, and carries the conflict-resolution loop.
Its preflight passes here because Step 1 left the tree clean; it does not
push, so continue with Step 3.

### Step 3: Push

Run the `/git:push` skill. It sets the upstream on a first push and uses
`--force-with-lease` when the Step 2 rebase rewrote already-pushed commits.

### Step 4: PR to origin/main

Run the `/gh:create-pr` skill with no argument. It opens — or reuses — the
one open PR from the current branch to `origin/main` (GitHub base name
`main`) and reports its
mergeability and check status. Its preflight passes here because Step 3 left
the branch in sync with its upstream.

### Step 5: Merge

Run the `/gh:merge-pr` skill with the PR from Step 4. It waits out pending
checks, refuses failing ones, merges with the explicit strategy (squash
unless the user or repository policy says otherwise, never `--delete-branch`
unasked), and verifies the PR reached `MERGED`.

### Step 6: Rebase on origin/main again

Run the `/git:rebase` skill with no argument again, syncing the still-current
working branch back to the merged main.

Then run the `/git:push` skill to push the synchronized branch — after the
merge and rebase it detects the rewritten history and pushes with lease
protection.

Verify the final state:

```bash
git status --short
git rev-list --left-right --count HEAD...origin/main
git rev-list --left-right --count HEAD...@{u}
```

The clean finish is an empty status, `HEAD...origin/main` equal to `0 0`, and
`HEAD...@{u}` equal to `0 0`. If any verification differs, report the exact
divergence and what remains.
