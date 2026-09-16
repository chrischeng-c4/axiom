---
name: prepare-goal
description: Prepare and track one explicitly requested legacy AW project, issue, or Milestone as a Codex goal. The goal must require actual use of its project subagents. Controller-only; use product-plan for ordinary new work.
---

# Legacy AW Prepare Goal

Use this skill only when the human explicitly asks for legacy AW routing or invokes `$prepare-goal`. It is not the default product workflow.

## Goal

Prepare one already-defined legacy AW scope as the active Codex goal. The goal must require actual use of the project subagents that will do the work. Route all project implementation and e2e work through those subagents. Do not make product decisions or change project state.

## How

1. The controller resolves the scope and keeps all AW state authority.
2. Read the current Codex goal. If no active goal exists, call `create_goal` with a concrete objective that names the requested scope, observable completion condition, and the required project subagents. The objective must say that the controller will actually dispatch and use `<project>-qa` for e2e behavior contracts and `<project>-dev` for source plus colocated unit tests. If the active goal already covers the scope, keep it. Do not replace an unrelated active goal.
3. Dispatch and use the named subagents for project work. `<project>-qa` owns e2e behavior contracts and `<project>-dev` owns source plus colocated unit tests. A name in the goal is not enough. The controller owns scope, integration, verification, and acceptance. Keep each worker's paths disjoint and preserve unrelated work.
4. When product facts are needed, delegate their reading to the owning PM or TL. The controller reads their short summary, not product files or raw logs.
5. Route behavior work to the legacy e2e then impl legs through those subagents. Route maintenance to the owning `<project>-dev` maintenance leg. Stop on an invalid order or unknown type.
6. State the active goal, named subagent, next controller action, and every unresolved human decision.

## Never

- Never invoke this skill implicitly.
- Never create, replace, complete, or block a goal without the human's explicit `$prepare-goal` or `/goal` request and the corresponding Codex goal-tool condition.
- Never create a goal that omits a required actual dispatch of the project subagents, their ownership, or the controller's integration role.
- Never treat naming a subagent in the goal as proof that the subagent was used.
- Never edit product files or make tracker changes.
- Never have the controller implement project e2e or source work when the owning project subagent is available.
- Never let a worker run AW phase state, commit, lifecycle, or close commands.
- Never infer scope, order, authorization, or acceptance from a summary.
