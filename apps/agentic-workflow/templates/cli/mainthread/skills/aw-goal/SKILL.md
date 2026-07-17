---
name: aw:goal
description: Record a bounded, verifiable-condition goal and drive it to a machine-checked terminal state — for ad-hoc work outside the WI lifecycle (backlog drains, migration sweeps, test-pass gates).
user-invocable: true
---

# /aw:goal

Thin dispatcher over the CLI-owned `aw goal` verifiable-condition loop. Use
this for bounded work that is explicitly outside the WI/TD/EC lifecycle
(`aw wi run`/`aw capability run` own aw-managed work); this skill has no
Stop hook and no prose-judged completion — `aw goal check`'s gate commands
are the enforcement.

## Instructions

1. From the human's stated intent, derive the narrowest machine-runnable
   command that proves it (a single `cargo test`/`cargo build`/lint/grep
   invocation, not the whole workspace). Prose alone is never a gate.
2. Record the goal:

   ```bash
   aw goal set --gate "<narrowest proving command>" <prose intent>
   ```

   Repeat `--gate` for multiple conditions. Add `--budget-checks <N>` or
   `--budget-minutes <N>` for a bounded loop; every goal also self-expires
   after 24h regardless.
3. Do the work.
4. Re-run the emitted `next.command` (`aw goal check <id>`) and read the
   envelope:
   - `status: "done"` (`completion.workflow_complete = true`): report
     completion to the user.
   - `status: "blocked"`: read `gates[].output_tail` for the failing gate,
     fix it, go back to step 3.
   - `status: "gave_up"`: budget or 24h expiry exhausted; report the
     recorded intent and the current blocker, do not claim success.

Never declare the goal done without `completion.workflow_complete = true`
from `aw goal check`.
