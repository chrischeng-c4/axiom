---
name: aw-grill-milestone-to-issue
description: Start in Plan mode, interview the human about one release Milestone's work set, then create its typed delivery issues and their global queue order. The Milestone's Development Order owns the sequence.
---

# AW Grill Milestone to Issue

## Goal

Make one release Milestone's assigned issue set, issue order, and issue types
describe the work the human actually wants.

## How

1. Enter Plan mode before reading files, inspecting the tracker, asking scope
   questions, or writing. Use the runtime's native Plan mode control. Stop if
   the runtime cannot confirm Plan mode.
2. Accept one `<project>` or one `milestone:<number>` reference. For a
   project, list its open release Milestones and confirm the target with the
   human:

   ```bash
   uv run --project apps/aw aw milestone versions --project <project> --state open --json
   ```

   Run the read-only gap table with `aw wis gap <project>`. This skill answers
   G2, G6, and G7: an open release Milestone or unmilestoned delivery issue no
   promise reaches, an e2e case the crate manifest does not run, and a README
   gate that names no cargo target. G1 and G3-G5 route to
   `/aw-grill-meta-to-milestone`.
3. Read the Milestone with `aw milestone show milestone:<number> --json`.
   Interview the human to resolve the issue boundaries,
   issue order, and the type of every issue.
4. Create or update delivery issues through `aw change`. Assign each delivery
   issue with `--milestone milestone:<number>` or the exact Milestone title.
   Each has exactly one of `type:feat`, `type:fix`, `type:refactor`,
   `type:perf`, `type:test`, `type:docs`, or `type:chore`, plus the owning
   `app:*` or `lib:*` label. `type:spike` and `type:report` are intake. Reject
   legacy `type:change` and every other legacy type.
5. Replace the description's `## Development Order` draft line with a
   contiguous numbered list. Each assigned delivery issue appears exactly
   once as `1. #<iid>`. The list is the global queue order.
   Only its first open row is executable.
6. Finalize the description without `--draft`, then reconcile it:

   ```bash
   uv run --project apps/aw aw milestone update milestone:<number> --description-file <path>
   uv run --project apps/aw aw milestone reconcile milestone:<number>
   uv run --project apps/aw aw milestone order milestone:<number> --open-only
   ```

7. Re-run `aw wis gap <project>`. Report the named G2, G6, and G7 rows with
   their populations.

## Acceptance

- `aw milestone reconcile` reports no structural error.
- `aw milestone order` lists every assigned delivery issue exactly once.
- Every delivery issue has exactly one supported delivery type.
- Each named G2, G6, and G7 row is `0` over a population greater than zero.
- No new `type:epic` issue or `epic:<iid>` label exists.

## Never

- Never use a GitHub issue as an epic.
- Never add, repair, or infer an `epic:<iid>` ownership label.
- Never use a bare number for a Milestone. Use `milestone:<number>` or its
  exact title.
- Never infer issue order from creation time, issue number, or API return
  order.
- Never create the Milestone or choose its version in this skill. That
  belongs to `/aw-grill-meta-to-milestone`.
- Never create, assign, or execute legacy `type:change` work.
- Never treat intake issues as delivery issues.
- Never mutate legacy issue-based epics during this flow. Report them as
  migration work unless the human separately authorizes that migration.
- Never change product source, tests, Git refs, tags, or releases.
