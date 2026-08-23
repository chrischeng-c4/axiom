---
name: agy:dispatch
description: Safely send one frozen, bounded task from Claude to headless AGY through the repository's shared dispatcher core. Use for AGY audits, measurements, investigations, transcription, or tightly scoped implementation; the Claude controller keeps verification and acceptance authority.
user-invocable: true
---

# AGY Dispatch

Use AGY as a bounded worker. Do not make it an owner.

Codex and Claude use one repository-owned dispatcher entry point:

```text
scripts/agy_dispatch.py
```

Read the shared
[lifecycle](../../../.agents/skills/agy-dispatch/references/lifecycle.md)
before the first dispatch or a takeover. That file is the normative state
machine. This skill keeps local reference material for Claude discovery. Never
use a skill-local, installed, or legacy dispatcher copy.

## Authority split

The Claude controller freezes the task contract, oracle, profile, and payload
authorization. It also chooses the Project and task worktree.

After the user authorizes the exact headless AGY payload, create one fresh
`Haiku` dispatch-operator subagent with medium reasoning. The subagent must
directly inherit that user turn. Do not reuse an older subagent. Do not forward
the authorization through a later controller message.

The dispatch-operator may only translate the frozen request into shared-core
commands and run the adapter. The controller must supply the exact action and
`snapshot_mode=create|reuse|refresh`. The operator runs only the matching
`doctor`, `snapshot`, `dispatch` or `resume`, and `status` sequence. It must
return the complete adapter output to the controller.

The controller alone runs independent `verify`. The controller alone accepts
or rejects the result. The controller also keeps all Git, tracker, publication,
cleanup, and closure authority.

The dispatch-operator must stop on a missing permission, observation, snapshot,
or authorization. It must not guess a value, widen the task, change policy, or
work around a refusal.

## Fixed adapter contract

Keep these settings unchanged:

- `model=gemini-3.7-flash-high`
- `worktree_layout=in-project`
- `launch_cwd=task-worktree`

Use one persistent AGY Project for the repository's persistent worktree. Use a
distinct clean linked task worktree below that Project root. Both worktrees
must use the same Git common directory. Launch AGY with the task worktree as
the exact process current directory.

Require a current `project_policy_observation`. A human must record it from the
official Project selector, Project Settings, and `/permissions` Project scope.
Set Outside of Folder File Access to Always Deny.

Do not infer Project identity or policy from private `~/.gemini` registry or
cache files. Do not edit those files. Do not create one Project per task. Do
not point the persistent Project at a task worktree. Do not use an external
sibling worktree or `--add-dir`.

The task profile owns the exact command and repository-write allowlists.
`measure-only` is the default. `bounded-write` requires explicit write paths,
a frozen implementation contract, and caller-selected frozen design inputs.
The AGY report remains an unverified claim until the controller accepts its
evidence.

## Active flow

Change to the repository root before every adapter verb.

```bash
python3 scripts/agy_dispatch.py doctor profile.json
python3 scripts/agy_dispatch.py snapshot profile.json TASK_KEY
python3 scripts/agy_dispatch.py dispatch profile.json TASK_KEY
python3 scripts/agy_dispatch.py status profile.json
```

For a ticketed retry, the controller must first verify the failed attempt and
approve the exact retry contract. Use `snapshot_mode=reuse` when the existing
snapshot is still valid. This includes the shared core's explicit ticketed
prompt-only correction case. The fresh dispatch-operator may then run:

```bash
python3 scripts/agy_dispatch.py resume profile.json TASK_KEY
```

Use `snapshot_mode=refresh` only after the latest attempt has a current
controller verification marker. The operator then runs `doctor`, `snapshot`,
and `resume` in that order.

Never resume a one-shot task. Use a new run id, oracle, and snapshot.

After the dispatch-operator returns, the Claude controller runs:

```bash
python3 scripts/agy_dispatch.py verify profile.json TASK_KEY
```

Read the shared
[report verification checklist](../../../.agents/skills/agy-dispatch/references/report-verification.md)
before semantic acceptance. Read the shared
[inventory verification checklist](../../../.agents/skills/agy-dispatch/references/inventory-verification.md)
for inventory or denominator work.

## Legacy boundary

The retired Claude v1 files are archived at
`.claude/dispatch/legacy/agy-dispatch-v1/`. They include the old `worktree`,
`grant`, Project-root repoint, `accept`, and `discard` flow.

Never execute that archive. It exists only for manual migration review. It does
not grant compatibility or acceptance authority.
