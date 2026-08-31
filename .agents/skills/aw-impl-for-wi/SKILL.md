---
name: aw-impl-for-wi
description: Drive the implementation phase for the queue head of one behavior delivery issue or release Milestone. Use only for type:feat, type:fix, or type:perf after e2e evidence.
---

# AW Implementation for Work Item

## Goal

Land one attributable implementation commit for the queue head of a behavior
delivery issue.

## How

1. Accept only one `type:feat`, `type:fix`, or `type:perf` delivery issue.
   Reject maintenance, intake, `type:change`, and every other legacy type.
2. Resolve the queue head with the structural gate:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/milestone.py" next <milestone-ref> --json
   ```

   For a direct issue, inspect it with `change.py show <iid> --json`. When it
   has a Milestone, run `milestone.py next` for that Milestone and require the
   returned `iid` to match. Require `flow: behavior` and `next_phase: impl`.
   Stop on every mismatch or structural error.
3. Fetch the live issue. This refreshes its body and type receipt. Then start
   the phase with the project option before the verb:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/change.py" fetch <iid>
   uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <project> start <iid>
   ```

4. Add the implementation test or skeleton under `apps/<project>/src/` without
   adding the behavior that makes it pass.
5. Record the red result. It includes the named failure, current HEAD, and test
   file digest:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <project> red <iid>
   ```

6. Write the implementation. Do not change the measured test after `red`.
7. Verify, test, and commit:

   ```bash
   uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <project> verify <iid>
   uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <project> test <iid>
   uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <project> commit <iid>
   ```

8. Run the exact `change.py lifecycle` command printed by `impl.py commit`.
   Then run `change.py close <iid>`. For a Milestone, re-run
   `milestone.py next <milestone-ref> --json`; only this successful close may
   advance the queue.

## Acceptance

- One behavior queue head has one landed implementation commit with `Impl-Red:`
  and `Impl-Contract:` evidence.
- Its recorded red matches the test bytes and HEAD used by its green
  verification.
- A Milestone run processes only its queue head.
- The commit changes only the implementation write root and includes at least
  one implementation test file.

## Never

- Never implement before `impl.py red` records a real failing test.
- Never edit, weaken, skip, or replace the measured test after `red`.
- Never choose or infer Milestone order.
- Never continue after the Milestone order changes during the run.
- Never process more than the queue head.
- Never use impl for a maintenance or intake issue.
- Never modify tracker order, Milestone description, e2e cases, docs, tags, or
  releases during implementation.
