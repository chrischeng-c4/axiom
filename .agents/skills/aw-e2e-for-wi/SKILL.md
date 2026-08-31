---
name: aw-e2e-for-wi
description: Drive the e2e phase for the queue head of one behavior delivery issue or release Milestone. Use only for type:feat, type:fix, or type:perf.
---

# AW E2E for Work Item

## Goal

Land one black-box contract commit for the queue head of a behavior delivery
issue.

## How

1. Accept only one `type:feat`, `type:fix`, or `type:perf` delivery issue.
   Reject `type:refactor`, `type:test`, `type:docs`, `type:chore`, `type:spike`,
   `type:report`, legacy `type:change`, and every other legacy type.
2. Resolve the queue head with the structural gate:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/milestone.py" next <milestone-ref> --json
   ```

   For a direct issue, inspect it with `change.py show <iid> --json`. When it
   has a Milestone, run `milestone.py next` for that Milestone and require the
   returned `iid` to match. Require `flow: behavior` and `next_phase: e2e`.
   Stop on every mismatch or structural error.
3. Fetch the live issue. This stages its body and type receipt. Then start the
   phase with the project option before the verb:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/change.py" fetch <iid>
   uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <project> start <iid>
   ```

4. Write only the black-box case under `apps/<project>/e2e/`. Declare the case
   in the project manifest when required by the project contract.
5. Verify the case. The new case must fail against the current product tree:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <project> verify <iid>
   ```

6. Run the red test, then commit only after the phase gate accepts it:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <project> test <iid>
   uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <project> commit <iid>
   ```

7. Run the exact `change.py lifecycle` command printed by `e2e.py commit`.
   For a Milestone, re-run `milestone.py next <milestone-ref> --json`. Require
   the same `iid`, `flow: behavior`, and `next_phase: impl`.

## Acceptance

- One behavior queue head has one landed e2e commit with `E2E-Red:` evidence.
- A Milestone run processes only its queue head.
- The final Milestone order read remains valid after the last commit.
- No `src/**` file changes in this phase.

## Never

- Never choose a Milestone's issue order yourself.
- Never treat GitHub API return order, issue number, or priority label as the
  Milestone order.
- Never edit product implementation during e2e.
- Never write a case that was already green and present it as a red contract.
- Never continue past an order error, missing issue, wrong project label, or
  unsupported issue type.
- Never process more than the queue head.
- Never use e2e for a maintenance or intake issue.
