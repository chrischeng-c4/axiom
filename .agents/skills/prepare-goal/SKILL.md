---
name: prepare-goal
description: Legacy AW routing for an explicitly requested project, issue, or Milestone. Controller-only; use product-plan for ordinary new work.
---

# Legacy AW Prepare Goal

Use this skill only when the human explicitly asks for legacy AW routing or invokes `$prepare-goal`. It is not the default product workflow.

## Goal

Route one already-defined legacy AW scope without making product decisions or changing project state.

## How

1. The controller resolves the scope and keeps all AW state authority.
2. When product facts are needed, delegate their reading to the owning PM or TL. The controller reads their short summary, not product files or raw logs.
3. Route behavior work to the legacy e2e then impl legs. Route maintenance to the legacy impl-for maintenance leg. Stop on an invalid order or unknown type.
4. State the next controller action and every unresolved human decision.

## Never

- Never invoke this skill implicitly.
- Never create a goal, edit product files, or make tracker changes.
- Never let a worker run AW phase state, commit, lifecycle, or close commands.
- Never infer scope, order, authorization, or acceptance from a summary.
