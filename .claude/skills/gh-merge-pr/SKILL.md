---
name: gh:merge-pr
description: "Merge one GitHub pull request after its checks pass, defaulting to a squash merge, and verify the merged state. Use when the user asks to merge a PR, merge pull request <n>, or land an already-open PR."
user-invocable: true
---

# /gh:merge-pr

Merge one pull request, verify it merged, then stop. The mechanics live in
`scripts/gh/merge-pr.sh`: it watches pending checks to completion, refuses
failing ones, merges with an explicit strategy, and verifies `MERGED`.

## Rules

- The argument is the PR number or URL; no argument means the one open PR
  whose head is the current branch (zero or several matches refuse).
- Strategy: pass `--strategy` from the user's request or discoverable
  repository policy; otherwise the script's `squash` default stands.
- Pass `--delete-branch` only when the user explicitly asked to delete the
  branch. Persistent refs (`main`, `app/*`, `lib/*`, `project-mamba`,
  `project-lumen`, `examples`) stay.
- A `refused: failing checks` exit is final here — report the named checks
  verbatim and stop; fixing them is separate work the user directs.
- Merging moves `origin/main`; syncing the local branch back is
  `/git:rebase` + `/git:push` (or `/git:land`), not this skill.

## Instructions

1. Run the script (it may sit in the check-watch loop for a while):

```bash
scripts/gh/merge-pr.sh [pr]
```

2. Read its exit:
   - `0` — merged and verified; it printed the merge commit and a reminder
     that the local branch now trails the merged base. Report both.
   - `2` — refused: could not resolve exactly one PR, or checks failed (the
     failing names were printed). Report verbatim and stop.
