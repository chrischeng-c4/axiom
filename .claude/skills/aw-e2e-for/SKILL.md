---
name: aw-e2e-for
description: Drive the e2e phase for a scope reference — one behavior delivery issue, one release Milestone's queue head, or every open release Milestone of a project. Behavior types only; maintenance heads are reported and skipped.
---

# AW E2E for Scope

## Goal

Land one black-box contract commit per executable behavior queue head inside
the given scope.

## How

1. Resolve the scope from the one argument:
   - `#<iid>` is one delivery issue;
   - `milestone:<number>` or an exact `<project>@<version>` title is one
     release Milestone;
   - a `<project>` name means every open release Milestone of that project,
     listed and processed in version order:

     ```bash
     uv run --project apps/aw aw milestone versions --project <project> --state open --json
     ```

   A bare number never means a Milestone.
2. Accept only `type:feat`, `type:fix`, or `type:perf` for the phase itself.
   Reject intake, legacy `type:change`, and every other legacy type. A
   maintenance head (`type:refactor`, `type:test`, `type:docs`, `type:chore`)
   has no e2e phase: report it as `/aw-impl-for` work and continue with the
   next Milestone in scope.
3. Resolve each Milestone's queue head with the structural gate:

   ```bash
   uv run --project apps/aw aw milestone next <milestone-ref> --json
   ```

   For a direct issue, inspect it with `aw change show <iid> --json`. When it
   has a Milestone, run `aw milestone next` for that Milestone and require the
   returned `iid` to match. Require `flow: behavior` and `next_phase: e2e`.
   A head whose `next_phase` is already `impl` needs `/aw-impl-for`, not this
   skill. Stop the whole run on a structural error; skip to the next Milestone
   in scope on a head this phase cannot serve.
4. Fetch the live issue. This stages its body and type receipt. Then start the
   phase with the project option before the verb:

   ```bash
   uv run --project apps/aw aw change fetch <iid>
   uv run --project apps/aw aw e2e --project <project> start <iid>
   ```

5. Write only the black-box case under `apps/<project>/e2e/`. Declare the case
   in the project manifest when required by the project contract.
6. Verify the case. The new case must fail against the current product tree:

   ```bash
   uv run --project apps/aw aw e2e --project <project> verify <iid>
   ```

7. Run the red test, then commit only after the phase gate accepts it:

   ```bash
   uv run --project apps/aw aw e2e --project <project> test <iid>
   uv run --project apps/aw aw e2e --project <project> commit <iid>
   ```

8. Run the exact `aw change lifecycle` command printed by `aw e2e commit`.
   Re-run `aw milestone next <milestone-ref> --json`. Require the same `iid`,
   `flow: behavior`, and `next_phase: impl`. The e2e phase closes nothing, so
   the queue head does not advance: one Milestone yields at most one e2e
   commit per run. Continue with the next Milestone in scope, and report
   every Milestone's outcome at the end.

## Acceptance

- Every executable behavior queue head in scope has one landed e2e commit
  with `E2E-Red:` evidence.
- A Milestone run processes only its queue head.
- Each final Milestone order read remains valid after the last commit.
- No `src/**` file changes in this phase.
- Skipped Milestones are reported with the reason (maintenance head, head
  already past e2e, or queue empty).

## Never

- Never choose a Milestone's issue order yourself.
- Never treat GitHub API return order, issue number, or priority label as the
  Milestone order.
- Never edit product implementation during e2e.
- Never write a case that was already green and present it as a red contract.
- Never continue past an order error, missing issue, wrong project label, or
  unsupported issue type.
- Never process more than the queue head of any one Milestone.
- Never use e2e for a maintenance or intake issue.
