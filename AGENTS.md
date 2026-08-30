---
project:
  name: axiom
  owner: chrischeng-c4
  url: https://github.com/chrischeng-c4/axiom
  ssh: git@github.com:chrischeng-c4/axiom.git
  default_branch: main
---

# AGENTS.md - Codex bootstrap

This file addresses a Codex process running in the axiom checkout. It is not
shared with any other runtime: Claude Code does not load it, and this file no
longer carries repository facts on any other runtime's behalf. `CLAUDE.md` is
Claude Code's bootstrap and holds the authoring rules for anyone driving work
here.

Until 2026-08-26 almost every Codex process started in this checkout was a
**reviewer**: two skills piped a built prompt into `codex exec -` and parsed
what came back. Those skills are deleted and no phase script builds a review
prompt any more. A human can still hand you one, so the section below stays;
absent one, a human is driving you directly.

## If you were given a review prompt

Everything you need is in it. The prompt carries the standard, the work item,
every path the change touches, the full source of each artifact under review,
and the output contract that your answer is parsed against. It is assembled by
the phase script so that scope is guaranteed by construction rather than by
asking you to stay in bounds.

- Answer that prompt and nothing else. Do not widen the review by reading the
  rest of the repository, and do not import a standard from anywhere but the
  prompt.
- Emit the output contract exactly as the prompt states it. The transcript is
  parsed mechanically; a well-meaning summary in place of the required lines is
  a refused review, not a lenient one.

## What you must not do to this checkout

`.codex/config.toml` sets `sandbox_mode = "workspace-write"`. Nothing mechanical
stops you from editing, staging, or committing, so the restraint has to be
yours.

- Never edit a file you are reviewing. Each phase commit carries a
  `*-Change-Digest:` trailer over the bytes it measured; an edit does not fail
  loudly, it silently makes the trailer describe bytes that no longer exist.
- Never run `git add`, `git commit`, or any other command that writes to the
  index or to a ref.
- Never run `.claude/aw/scripts/*.py`. Those verbs advance and record the
  lifecycle. A reviewer that advances the thing it is judging has removed the
  gate it exists to be.
- Run any git you do need through `git -c core.fsmonitor=false`. This checkout
  enables `core.fsmonitor` and a stalled daemon blocks every command that reads
  the index, indefinitely and with no error.

## The limit on this file

This file enters your context automatically, and it is **outside every digest a
phase commit records**. A change here alters what every future Codex session is
told without invalidating a single measured red.

So it may constrain what you do to the checkout, and it may not bear on what you
conclude. Nothing about what makes a change good, what "done" looks like, or how
much benefit of the doubt to extend belongs here — that standard lives in the
prompt, where its bytes travel with whatever cites them. Keep this file short
for the same reason.

## If you were not given a review prompt

A human is driving you directly. Read `README.md` for repository inventory,
`<project>/README.md` for that project's product promises, work roots, and
gates, and `<project>/CONTRIBUTING.md` for project-local edit and verification
rules. There is no `CAPABILITIES.md`; it was deleted on 2026-08-17.

Then read `CLAUDE.md`. It holds the work-item lifecycle, the per-phase write
roots, and the rules that refuse against them, and it is the single copy of
those rules — they are deliberately not repeated here.

### Use Codex subagents and select effort per task

Before starting non-trivial work, split it into bounded workstreams where this
can reduce elapsed time or improve independent review. Prefer the matching
`<project>-dev` role under `.codex/agents/` for implementation, investigation,
test design, review, and verification in one owned app or library. Keep the
main thread as the controller. It freezes scope and ownership, integrates the
results, reproduces the evidence, and owns final acceptance.

Use several subagents when the workstreams are independent. Run read-only work
in parallel. Run write work in parallel only when path ownership cannot
overlap. Tell every worker that it is not alone in the checkout, that it must
preserve unrelated changes, and that it must not undo another worker's work.
Reuse a current subagent for a related follow-up. Keep a tiny task in the main
thread when delegation would cost more than the work. For a cross-project task,
assign one matching subagent per owned project and keep integration in the main
thread.

Every project `<project>-dev` role uses `gpt-5.6-terra`. Select its reasoning
effort when dispatching the task:

- `low` for narrow mechanical work with no public behavior change.
- `medium` for contained behavior in one owner with focused tests.
- `high` for material public behavior or several modules and consumers.
- `xhigh` for cross-project, concurrency, durability, security,
  compatibility, release, or supply-chain work.
- `max` for the hardest quality-first work when failure would be costly and
  deeper verification has measured value.

Pass the selected value as `reasoning_effort` when calling `spawn_agent`. Use
`fork_turns="none"` or a positive turn count when an explicit effort override
is required, and include all context that the worker needs. Do not pin one
universal effort in a role TOML.

Choose the lowest effort that fits the task. Raise it when ambiguity, risk, or
integration cost increases. A higher effort changes reasoning depth only. It
never expands scope, write access, authority, or acceptance rights.

### Use AGY only for authorized external delegation

Codex project subagents are the default bounded workers. Use one fresh
`agy-operator` subagent only when the user authorizes the exact headless AGY
payload for an external task. Its model is `GPT-5.6 Luna` at medium reasoning.
Make it directly inherit that user turn. Do not reuse an older operator. Do not
forward authorization through a controller message.

For more than one task in the same round: any number of `measure-only` tasks
may run concurrently, and `bounded-write` tasks may run concurrently only
across distinct persistent AGY Projects. AGY has not proven per-conversation
worktree confinement for two concurrent bounded writes in one Project, so
same-Project bounded-write tasks queue one at a time regardless of how
disjoint their write ownership looks. The Claude-side controller has a
`/dispatch-to-agy` skill that enforces this; a Codex-driven controller applies
the same rule by hand.

The controller freezes the profile, task key, action, snapshot mode, and all
input digests before dispatch. The snapshot mode is `create`, `reuse`, or
`refresh`. The operator only checks those inputs and runs the exact matching
`doctor` / `snapshot` / `dispatch` / `resume` / status sequence. It must return
`HANDOFF_INCOMPLETE` if the authorization or frozen handoff is incomplete.

The operator never authors the contract, oracle, injection, or prompt. It does
not verify or accept the result. It does not create or change a permission or
worktree. It does not run Git, tracker, publication, or cleanup actions.

The controller owns the profile, task contract, oracle, injection, prompt,
worktree creation, independent verification, semantic acceptance, Git, tracker
changes, publication, and cleanup. Follow the repo-local
`.agents/skills/agy-dispatch/SKILL.md` as the source of truth for the AGY model,
Project, permission, snapshot, command, and write rules. Run every adapter verb
from the repository root as
`python3 scripts/agy_dispatch.py ...`. Do not use an installed, skill-local, or
legacy dispatcher copy.

One thing that will otherwise waste your time: `aw` names the scripts at
`.claude/aw/scripts/` and the skills at `.claude/skills/aw-*/`, and nothing
else. The Rust application that used to carry the name
is deleted and its binary is uninstalled, so an `aw` verb you reach for fails
with "command not found" — correct, but it tells you nothing about what to
reach for instead.

Repo-wide `CONTRIBUTING.md` has been reconciled against that deletion, and the
result is that several of its chapters now say plainly that a rule is policy
with nothing enforcing it. Read those sentences literally. A checklist there
that no longer has a checker behind it is not a gate you may cite as evidence.
