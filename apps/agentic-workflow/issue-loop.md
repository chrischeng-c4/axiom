---
project: agentic-workflow
branch: app/aw
label: "app:agentic-workflow"
repo: chrischeng-c4/axiom
pick_order: priority
build: skip
verify:
  test: cargo test -p agentic-workflow
  perf: not-required
done_gates:
  - test_passes
pr:
  base: main
  merge_strategy: squash
  rebase_after_merge: true
---

# Agentic Workflow issue loop — per-issue rules

## Branch policy
- All work happens on `app/aw`. If current branch is anything else, merge it
  into `app/aw` first (don't escape to a feature branch — `app/aw` IS the
  working branch).
- One issue → one PR → squash-merge to `main` → `git pull --rebase origin main`
  back onto `app/aw`. Push.

## Self-AW policy (overrides the generic loop where they conflict)
- Do NOT run the full AW lifecycle (`aw wi run` / `aw capability run`) against
  the aw repo itself — self-deadlock rule. Changes land as direct commits with
  `Refs #<issue>` trailers plus capability work-root registration in
  `apps/agentic-workflow/CAPABILITIES.md`.
- Chain conformance is the removal gate: any verb removal/rename must keep
  `validate_aw_command_string` + EMIT_REGISTRY green (`cargo test -p
  agentic-workflow --test cli_tests chain`) — no emitted next-command may
  dangle.
- SPEC-MANAGED mirror discipline: when a change touches surfaces mirrored into
  generated docs (cb.md/td.md/issues.md class), resync the mirror in the same
  commit or regen will revert it (#848 class).

## Build policy: skip
- Do not run a separate `cargo build` per issue; `cargo test -p
  agentic-workflow` is the compile gate. Only rebuild the release binary when
  an issue explicitly changes the installed CLI surface a later issue depends
  on at runtime.

## When to dispatch a subagent
- Code implementation (multi-file, new modules, test authoring) → dispatch
  `aw-dev` (sonnet) with the issue number; mainthread does planning, WI
  bodies, lifecycle commands, and PR/merge mechanics.
- Triage, labels, comments, single-file few-line fixes → mainthread inline.

## Definition of done
1. **Test gate** — `cargo test -p agentic-workflow` passes from a clean state;
   plus any targeted test the issue's acceptance criteria name.
2. **Perf** — not required (CLI tool); note "perf N/A (CLI surface)" in the PR
   body.

## PR body must include
- `Closes #<n>`.
- Test gate output (the passing `cargo test` summary line).
- `perf N/A (CLI surface)` line.
- If a verb was removed/renamed: one line confirming chain conformance green.

## Loop exit
- This loop is scoped to epic #1270's child WIs (tracked in-session), not the
  whole `app:agentic-workflow` backlog. Stop when the epic's children are all
  closed and #1270 itself can close.
- If a single issue is blocked (needs human decision, fails gates in an
  unexpected way), comment on it explaining the block and move to the next.
  Don't loop on a stuck issue.
