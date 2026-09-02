---
name: gh:create-pr
description: "Open or reuse a GitHub pull request from the current branch to main, then report its mergeability and check status. Use when the user asks to create a PR, open a pull request, or PR this branch."
user-invocable: true
---

# /gh:create-pr

Open or reuse one pull request from the current branch to GitHub base `main`,
report its state, then stop. This skill does not commit, rebase, push, or
merge.

## Rules

- The argument is the base branch. With no argument the base is `main`.
- Run every git command through `git -c core.fsmonitor=false`. This checkout
  enables `core.fsmonitor`, and a stalled daemon blocks any command that reads
  the index — indefinitely, with no error.
- The head is always the current branch. Never open a PR for another ref.
- The branch must already be pushed and in sync with its upstream. If it is
  not, stop and point at `/git:push`; do not push from this skill.
- One PR per head: if an open PR from the current branch to the base already
  exists, reuse it instead of creating a second one.
- Do not merge, close, or edit the PR beyond creation. Report and stop.

## Instructions

### Step 0: Preflight

```bash
git -c core.fsmonitor=false branch --show-current
git -c core.fsmonitor=false rev-list --left-right --count HEAD...@{u}
```

Stop and report — without touching GitHub — when any of these holds:

- the current branch is the base itself;
- the branch has no upstream, or the divergence is not `0 0` (run
  `/git:push` first).

### Step 1: Reuse or create

```bash
gh pr list --head "$(git -c core.fsmonitor=false branch --show-current)" --base <base> --state open --json number,url
```

If an open PR exists, reuse it. Otherwise create one:

```bash
gh pr create --base <base> --head "$(git -c core.fsmonitor=false branch --show-current)" --fill
```

### Step 2: Report

```bash
gh pr view <pr> --json number,url,state,mergeable,mergeStateStatus,statusCheckRollup
```

Report the PR number, URL, whether it was created or reused, its
mergeability, and each check's status. Do not wait for pending checks and do
not merge; that is `/gh:merge-pr`'s job.
