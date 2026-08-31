---
name: aw-grill-meta-to-wis
description: Start in Plan mode and reconcile one product promise with a release Milestone and typed delivery or intake issues. The Milestone owns epic grouping, version, and global queue order.
---

# AW Grill Meta to Work Items

## Goal

Make one product promise, its release Milestone, and its typed issue queue
describe the same work.

One GitHub Milestone is one versioned epic. Its title is
`<project>@<major>.<minor>.<patch>`. GitHub's native milestone field is the only
parent relation for a delivery issue.

## How

1. Enter Plan mode before reading files, inspecting the tracker, asking release
   questions, or writing. Use the runtime's native Plan mode control. Stop if
   the runtime cannot confirm Plan mode.
2. Run the read-only gap table:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/wis.py" gap <project>
   ```

3. Resolve each semantic choice with the human. This includes the target
   version, whether a promise deserves its own Milestone, issue boundaries,
   issue order, and the type of every issue.
4. Create a Milestone description with:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/milestone.py" skeleton
   ```

   Fill `## Goal` and `## Acceptance`. During initial child creation, keep the
   exact `## Development Order` draft line printed by the skeleton.
5. Create the Milestone in draft form:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/milestone.py" create --title <project>@<version> --description-file <path> --draft
   ```

6. Create or update delivery issues through `change.py`. Assign each delivery
   issue with `--milestone milestone:<number>` or the exact Milestone title.
   Each has exactly one of `type:feat`, `type:fix`, `type:refactor`,
   `type:perf`, `type:test`, `type:docs`, or `type:chore`, plus the owning
   `app:*` or `lib:*` label. `type:spike` and `type:report` are intake. Reject
   legacy `type:change` and every other legacy type.
7. Replace the draft line with a contiguous numbered list. Each assigned
   delivery issue appears exactly once as `1. #<iid>`. The list is the global
   queue. Only its first open row is executable.
8. Finalize the description without `--draft`, then reconcile it:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/milestone.py" update milestone:<number> --description-file <path>
   uv run --python 3.13 --no-project ".claude/aw/scripts/milestone.py" reconcile milestone:<number>
   uv run --python 3.13 --no-project ".claude/aw/scripts/milestone.py" order milestone:<number> --open-only
   ```

9. Bind the product section as `## <title> (Milestone #<number>)`. Set its
   tracking link to `[Milestone #<number>](<milestone-url>)`.
10. Re-run `wis.py gap <project>`. Report the named G1-G3 rows and their
   populations.

## Acceptance

- The Milestone title is the release identity and passes the repository's
  0..63 minor and patch rule.
- `milestone.py reconcile` reports no structural error.
- `milestone.py order` lists every assigned delivery issue exactly once.
- Every delivery issue has exactly one supported delivery type.
- Each named G1-G3 row is `0` over a population greater than zero.
- No new `type:epic` issue or `epic:<iid>` label exists.

## Never

- Never use a GitHub issue as an epic.
- Never add, repair, or infer an `epic:<iid>` ownership label.
- Never use a bare number for a Milestone. Use `milestone:<number>` or its
  exact title.
- Never infer issue order from creation time, issue number, or API return order.
- Never create, assign, or execute legacy `type:change` work.
- Never treat intake issues as delivery issues.
- Never mutate legacy issue-based epics during this flow. Report them as
  migration work unless the human separately authorizes that migration.
- Never change product source, tests, Git refs, tags, or releases.
