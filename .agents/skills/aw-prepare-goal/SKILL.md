---
name: aw-prepare-goal
description: Turn a project, typed issue, release Milestone, or interviewed intent into decidable conditions and route the next step without creating a goal by default.
---

# AW Prepare Goal

## Goal

Prepare conditions whose completion can be decided from evidence shown in the
conversation.

This skill does not set a goal by default. It may use a runtime goal mechanism
only when the human explicitly asks it to create or set the goal.

## How

1. Resolve the input:
   - a project routes to `aw-grill-release plan`;
   - `#<iid>` routes by type: behavior to e2e then impl, maintenance to maint,
     and intake to intake handling;
   - `milestone:<number>` or `<project>@<version>` routes to its queue head;
   - no input starts a short interview.
2. For a Milestone, run:

   ```bash
   uv run --project apps/aw aw milestone next <milestone-ref> --json
   ```

   Emit one condition only for the returned queue head and route it by type.
   Reject `type:change` and every other legacy type. Stop if the order is
   invalid.
3. Give every condition four parts:
   - one observable end state;
   - one exact check whose output must appear in the conversation;
   - the paths or behavior that must not change;
   - one stop clause for an unreachable result.
4. Use the fixed gate for the selected flow:
   - product documents: `aw metadoc check`;
   - tracker alignment: `aw wis gap` plus `aw milestone reconcile`;
   - behavior e2e: `aw e2e commit` with `E2E-Red:`;
   - behavior implementation: `aw impl commit` with `Impl-Red:`;
   - maintenance: `aw maint commit` with `Maint-Contract:`.
5. Present the conditions as an ordered queue. Say that only one condition is
   active at a time when the runtime has a single-goal limit.

## Acceptance

- Each condition is true or false from evidence in the conversation.
- A Milestone condition names only its queue head.
- The next route matches the issue type.
- Each check is an existing repository command, not a command invented for the
  condition.
- No goal is reported as created unless the human explicitly requested it and
  the runtime confirmed creation.

## Never

- Never use a bare number as a Milestone reference.
- Never infer Milestone order from API output or issue numbers.
- Never route maintenance work through e2e or impl.
- Never route intake work into execution.
- Never write a condition such as "the tests pass" without requiring the exact
  command output in the conversation.
- Never create, replace, clear, or mark a goal complete without explicit human
  authority for that goal action.
- Never edit documents, issues, Milestones, source, tests, or gates to make a
  condition easier to satisfy.
