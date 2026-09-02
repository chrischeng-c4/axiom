---
name: gh:merge-pr
description: "Merge one GitHub pull request after its checks pass, defaulting to a squash merge, and verify the merged state. Use when the user asks to merge a PR, merge pull request <n>, or land an already-open PR."
user-invocable: true
---

# /gh:merge-pr

Merge one pull request, verify it merged, then stop. This skill does not
commit, rebase, push, create a PR, or sync the local branch afterwards.

## Rules

- The argument is the PR (number or URL). With no argument, resolve the one
  open PR whose head is the current branch; zero or several matches stop the
  skill.
- Run every git command through `git -c core.fsmonitor=false`. This checkout
  enables `core.fsmonitor`, and a stalled daemon blocks any command that reads
  the index — indefinitely, with no error.
- Do not merge while required checks are failing. Report the failing checks
  verbatim and stop. Pending checks are waited on, not skipped.
- Use the user-requested merge strategy when provided; otherwise follow
  discoverable repository policy; otherwise squash. Always pass the strategy
  explicitly.
- Never pass `--delete-branch` unless the user explicitly asked to delete the
  branch. Preserve persistent refs: `main`, `app/*`, `lib/*`,
  `project-mamba`, `project-lumen`, `examples`.
- Merging rewrites what `origin/main` holds; syncing the working branch back
  is `/git:rebase` + `/git:push` (or `/git:land`), not this skill.

## Instructions

### Step 0: Resolve the PR

With no argument:

```bash
gh pr list --head "$(git -c core.fsmonitor=false branch --show-current)" --base main --state open --json number,url
```

Exactly one row is the PR; zero or several, stop and report.

### Step 1: Gate on checks

```bash
gh pr view <pr> --json number,url,state,mergeable,mergeStateStatus,statusCheckRollup
```

If checks are pending, watch them to completion:

```bash
gh pr checks <pr> --watch --interval 15
```

If any required check fails, report the failing checks and stop.

### Step 2: Merge

```bash
gh pr merge <pr> --squash
```

Substitute the explicit user-requested or repository-policy strategy when one
applies.

### Step 3: Verify

```bash
gh pr view <pr> --json state,mergedAt,mergeCommit,url
```

The clean finish is `state` `MERGED` with a merge commit. Report the merge
commit and remind that the local branch now trails the merged base until it
is rebased and pushed.
