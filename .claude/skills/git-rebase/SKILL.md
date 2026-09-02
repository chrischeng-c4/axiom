---
name: git:rebase
description: "Rebase the current branch onto a named branch, or onto origin/main when no branch is given. Use when the user asks to git rebase, rebase on <branch>, sync with main, or catch this branch up."
user-invocable: true
---

# /git:rebase

Rebase the current branch onto one target ref, then stop. The mechanics live
in `scripts/git/rebase.sh`; your job is the one thing it cannot do — resolve
conflicts correctly.

## Rules

- The argument is the target branch; no argument means a freshly fetched
  `origin/main`. The script resolves a named branch locally first, then as
  `origin/<branch>`.
- Do not push afterwards. Report the divergence the script prints and leave
  publishing to the user (or `/git:push`, `/git:land`).
- A `refused:` exit is the user's to fix — a dirty tree means they commit or
  set work aside; never commit or stash for them, and never bare
  `git stash` / `git stash pop`.
- On conflicts, resolve each file correctly from both sides' intent. Ask
  only when the right resolution cannot be determined; abort
  (`git -c core.fsmonitor=false rebase --abort`) rather than guess.

## Instructions

1. Run the script:

```bash
scripts/git/rebase.sh [branch]
```

2. Read its exit:
   - `0` — rebased (or already up to date); it printed the status, the
     `HEAD...<target>` and `HEAD...@{u}` counts, and the last five commits.
     Clean is an empty status and `N 0` against the target. If the upstream
     count shows `N M` with `M > 0`, say the next push needs
     `--force-with-lease` — without running it.
   - `2` — refused (dirty tree, in-progress operation, unresolvable target,
     or target is the current branch). Report the printed reason.
   - `3` — conflicts. It listed the conflicted files: read each one, resolve
     it, stage it, run `git -c core.fsmonitor=false rebase --continue`,
     repeat until the rebase completes, then rerun the script — a completed
     rebase makes the rerun a no-op that just prints the report.
