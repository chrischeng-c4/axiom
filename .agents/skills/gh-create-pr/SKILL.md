---
name: gh:create-pr
description: "Open or reuse a GitHub pull request from the current branch to main, then report its mergeability and check status. Use when the user asks to create a PR, open a pull request, or PR this branch."
user-invocable: true
---

# /gh:create-pr

Open — or reuse — one pull request from the current branch to a base branch,
report its state, then stop. The mechanics live in `scripts/gh/create-pr.sh`;
one PR per head is its invariant, so it reuses an existing open PR instead of
creating a second.

## Rules

- The argument is the base branch; no argument means `main`. The head is
  always the current branch — never open a PR for another ref.
- The script refuses an unpushed or out-of-sync branch; the fix is
  `/git:push`, not pushing from here.
- Do not merge, close, or edit the PR beyond creation, and do not wait for
  pending checks — merging is `/gh:merge-pr`'s job.

## Instructions

1. Run the script:

```bash
scripts/gh/create-pr.sh [base]
```

2. Read its exit:
   - `0` — it said whether the PR was created or reused, then printed the
     PR's JSON state. Report the number, URL, created-vs-reused,
     mergeability, and each check's status.
   - `2` — refused (on the base branch itself, no upstream, or not `0 0`
     against the upstream). Report the printed reason; if it points at
     `scripts/git/push.sh`, run `/git:push` first when the user wants the PR.
