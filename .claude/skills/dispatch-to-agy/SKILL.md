---
name: dispatch-to-agy
description: "Fan out more than one bounded AGY task in a single round, safely. Classifies each task measure-only or bounded-write per the agy-dispatch lifecycle, groups tasks by persistent AGY Project (app/*, lib/*, project-mamba, project-lumen, main), and spawns one fresh agy-operator subagent per task that may start now: unlimited measure-only concurrency, at most one active bounded-write per Project, and no limit across distinct Projects. Use when the user asks to dispatch to agy, hand multiple tasks to agy-operator, or run several AGY workers in parallel. Does not replace the per-task agy-dispatch protocol — this only decides which tasks may start together."
user-invocable: true
---

# /dispatch-to-agy

Schedule a round of AGY delegation across more than one task. This skill
decides *when* a task may start; it never changes *how* one task runs. Every
dispatched task still goes through `.claude/skills/agy-dispatch/SKILL.md` and
`references/lifecycle.md` in full — profile freeze, per-task consent,
snapshot, one fresh `agy-operator`, independent verification, and
controller-only acceptance stay exactly as that skill defines them.

## Rules

- One persistent AGY Project is one persistent worktree: `main`, `app/<name>`,
  `lib/<name>`, `project-mamba`, or `project-lumen`
  (`.claude/rules/operations/persistent-branches.md`). Resolve every task's
  target Project before scheduling it.
- Classify every task exactly as `references/lifecycle.md` phase 0 does:
  implementation is `bounded-write` and needs a frozen design; measurement,
  investigation, review, or audit is `measure-only` and needs an oracle;
  planning or HITL is neither and is never dispatched as worker
  implementation.
- At most one `bounded-write` task may be active per persistent Project at a
  time — never two, no matter how disjoint their declared write paths look.
  AGY has not proven per-conversation worktree confinement for two concurrent
  bounded writes in one Project.
- `measure-only` tasks in the same Project may run concurrently with each
  other, but never concurrently with an active `bounded-write` task in that
  same Project: a concurrent write can move the baseline a measure-only task
  is reading and invalidate its oracle mid-run.
- Tasks in different Projects never block each other, in any combination of
  `measure-only` and `bounded-write`.
- Confirm the user stated the exact headless AGY payload (task kind, target
  Project, scope) for every task before scheduling it. Do not infer consent
  from a general instruction to "use AGY" — same rule as
  `.claude/skills/agy-dispatch/SKILL.md`'s Setup section.
- Spawn every operator cleared to start in one round together, in a single
  message with one Agent tool call per task, so each inherits this same
  authorized user turn directly. Never forward authorization through a
  controller message, and never reuse an operator across tasks.
- Integrate one task at a time even when several ran concurrently: stage,
  verify, commit, and publish strictly serially, in an explicit order, and
  rerun affected gates after each merge before starting the next integration.
- A task queued behind a same-Project `bounded-write` task takes a fresh
  snapshot only after that task's change has landed — never reuse a snapshot
  taken before the Project's baseline moved.
- This is a Claude Code mechanism: it schedules `agy-operator` subagents
  through the Agent tool. A Codex-driven controller applies the same
  classify/group/queue rule by hand, per `AGENTS.md`'s "Prefer AGY for
  bounded delegation" section — there is no Codex-side twin of this skill.

## Instructions

### Step 1: Collect and authorize the round

List every task the user described in this turn. Refuse any task that lacks
an exact stated payload (task kind, target Project, scope); do not schedule
it and tell the user what is missing.

### Step 2: Classify and resolve Project

For each authorized task, record:
- `project` — the persistent worktree it targets (`app/<name>`, `lib/<name>`,
  `project-mamba`, `project-lumen`, or `main`).
- `mode` — `bounded-write` or `measure-only`, per the Rules above.

### Step 3: Build the concurrency plan

Group tasks by `project`. Within each group:
- If the group has no `bounded-write` task, every `measure-only` task in it
  may start now, concurrently.
- If the group has one or more `bounded-write` tasks, exactly one of them may
  start now (the user's stated priority, or declaration order); every other
  task in that group — `bounded-write` or `measure-only` — queues.

Tasks in different groups never affect each other's plan.

### Step 4: Dispatch the cleared tasks

Spawn one fresh `agy-operator` subagent per task cleared in Step 3, all in one
message using multiple Agent tool calls. For each spawn, follow
`.claude/skills/agy-dispatch/SKILL.md` in full: disclose and resolve consent,
freeze the profile, take the snapshot, and run doctor/dispatch through that
skill's own sequence.

### Step 5: Verify, integrate, and unblock the queue

As each operator reports:
1. Independently verify per `references/lifecycle.md`'s state machine
   (`REPORTED` -> `ISOLATION_VERIFIED` -> ... -> `CONTROLLER_ACCEPTED`).
2. Integrate one task at a time only: stage its exact accepted paths, run the
   project's gates, commit, publish. Never integrate two tasks concurrently,
   even if their verification finished at the same time.
3. Once a `bounded-write` task's change lands, any task queued behind it in
   the same Project may take a fresh snapshot and start — repeat Step 4 for
   that task alone, or fold it into the next round.
4. Report to the user per task: what ran, its current state (`REPORTED`
   through `PUBLISHED`), and anything still queued and why.
