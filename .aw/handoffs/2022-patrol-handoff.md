---
slug: 2022
branch: td-2022
issue: chrischeng-c4/cclab#2022
timestamp: 2026-05-12T08:11:00+00:00
patrol: project:mamba
phase_at_stall: td_created (post --apply, pre validate)
---

# Patrol handoff — score td validate slug-lookup bug

## Problem

`score td validate <numeric-slug>` (here: `2022`) looks up the issue
file at `.score/issues/open/<numeric-slug>.md` but for this issue the
LocalBackend stores it under the long descriptive slug
`bug-mamba-run-does-not-forward-argv-to-sys-ar.md`. Validate cannot
read the cache file and aborts before advancing the phase.

```
$ score td validate 2022 --spec-path .score/tech_design/projects/mamba/specs/mamba-bug-run-argv-forward.md
[score issues] no [sdd.issue_platform] in .score/config.toml; using [sdd.repo_platform] (github)
Error: read cache file /Users/chris.cheng/cclab/project-mamba/.score/issues/open/2022.md

Caused by:
    No such file or directory (os error 2)
```

## State at stall

- Branch `td-2022` is at commit `3bfd75a41` plus an uncommitted
  `--apply` modification to the issue file (phase
  `td_inited` → `td_created`, updated timestamp).
- Spec file `.score/tech_design/projects/mamba/specs/mamba-bug-run-argv-forward.md`
  exists on disk and is fully drafted (4 sections: dependency, logic,
  changes, test-plan), committed in `f45d5976b`.
- The `--apply` step succeeded and emitted the dispatch envelope to
  validate, but validate's slug-lookup is wrong.

## Why patrol stops

`score td validate` is the sole commit point for the `Td-Create`
lifecycle stage. Without it, the phase cannot legitimately advance
and the next dispatch (review) cannot be emitted. Patrol cannot
work around this without manually mimicking validate's commit, which
risks drifting the state machine.

## What the operator needs to do

Pick ONE of:

1. **Patch `score td validate` slug-lookup** (preferred — root cause).
   Make the verb resolve `<slug>` against the actual filename in
   `.score/issues/open/`, e.g. by scanning frontmatter `github_id`
   matches when the literal `<slug>.md` is absent.

2. **Rename the issue file** to `2022.md` (workaround). All
   subsequent CRRR verbs already accept either slug form on the
   create/fill side; only validate's lookup is brittle.

3. **Manually commit the apply state** with a `Lifecycle-Stage: Td-Create`
   trailer and bypass validate. Loses the validate audit (section-format
   check, etc.) but unblocks the CRRR loop.

## Reproduction

```bash
cd /Users/chris.cheng/cclab/project-mamba
git checkout td-2022
score td create 2022 --apply --spec-path .score/tech_design/projects/mamba/specs/mamba-bug-run-argv-forward.md
score td validate 2022 --spec-path .score/tech_design/projects/mamba/specs/mamba-bug-run-argv-forward.md
# ^ Error: read cache file .../2022.md (No such file)
```

## Other in-flight branches in this worktree

```
issue-2027                                            (project:agentkit — main worktree)
issue-2028                                            (project:agentkit — checked out at project-agentkit/)
issue-bug-int-int-rejected-as-type-error-true-divis   (project:mamba — checked out at main/)
issue-bug-mamba-run-does-not-forward-argv-to-sys-ar   (project:mamba — same issue as #2022, stale)
issue-bug-tuple-swap-unpack-a-b-b-a-b-produces-garb   (project:mamba — free)
td-2022                                               (project:mamba — me)
td-2027                                               (project:agentkit — free)
```
