---
name: git:push
description: "Push the current branch to origin, setting the upstream on first push and using --force-with-lease only when a rebase rewrote already-pushed history. Use when the user asks to git push, push this branch, or publish the branch to the remote."
user-invocable: true
---

# /git:push

Push the current branch to `origin`, then stop. This skill does not commit,
rebase, or open a PR.

## Rules

- Push only the current branch. Never push, delete, or force-move any other
  ref. Persistent refs — `main`, `app/*`, `lib/*`, `project-mamba`,
  `project-lumen`, `examples` — are never force-overwritten without explicit
  user confirmation.
- Run every git command through `git -c core.fsmonitor=false`. This checkout
  enables `core.fsmonitor`, and a stalled daemon blocks any command that reads
  the index — indefinitely, with no error.
- Use `--force-with-lease` only when the branch has an upstream and local
  history was rewritten (the branch is both ahead and behind its upstream).
  Never use bare `--force`.
- If the push is rejected — including a lease failure — the remote moved:
  fetch, report the divergence, and stop. Do not override it.

## Instructions

### Step 0: Preflight

```bash
git -c core.fsmonitor=false branch --show-current
git -c core.fsmonitor=false rev-parse --abbrev-ref --symbolic-full-name @{u}
```

If the branch has an upstream, measure the divergence:

```bash
git -c core.fsmonitor=false rev-list --left-right --count HEAD...@{u}
```

### Step 1: Push

No upstream — first push, set it:

```bash
git -c core.fsmonitor=false push -u origin HEAD
```

Upstream exists and the divergence is `N 0` (only ahead) — plain push:

```bash
git -c core.fsmonitor=false push
```

Upstream exists and the divergence is `N M` with `M > 0` (rewritten
history) — push with lease protection:

```bash
git -c core.fsmonitor=false push --force-with-lease
```

`0 0` means nothing to push; report and stop.

### Step 2: Report

```bash
git -c core.fsmonitor=false rev-list --left-right --count HEAD...@{u}
```

The clean finish is `0 0`. Report the pushed ref and the final divergence; on
rejection, report the exact error and what the remote holds.
