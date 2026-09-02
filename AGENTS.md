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

The two subsections below apply only when you received a review prompt. They do
not apply when a human is driving you directly.

- Answer that prompt and nothing else. Do not widen the review by reading the
  rest of the repository, and do not import a standard from anywhere but the
  prompt.
- Emit the output contract exactly as the prompt states it. The transcript is
  parsed mechanically; a well-meaning summary in place of the required lines is
  a refused review, not a lenient one.

### What a reviewer must not do to this checkout

`.codex/config.toml` sets `sandbox_mode = "workspace-write"`. Nothing mechanical
stops you from editing, staging, or committing, so the restraint has to be
yours.

- Never edit a file you are reviewing. Each phase commit carries a
  `*-Change-Digest:` trailer over the bytes it measured; an edit does not fail
  loudly, it silently makes the trailer describe bytes that no longer exist.
- Never run `git add`, `git commit`, or any other command that writes to the
  index or to a ref.
- Never run the `aw` CLI (`uv run --project apps/aw aw ...`) or any
  script under `apps/aw/src/aw/scripts/`. Those verbs advance and record the
  lifecycle. A reviewer that advances the thing it is judging has removed the
  gate it exists to be.
- Run any git you do need through `git -c core.fsmonitor=false`. This checkout
  enables `core.fsmonitor` and a stalled daemon blocks every command that reads
  the index, indefinitely and with no error.

### The limit on the reviewer bootstrap

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

### Use Codex subagents; effort is pinned per role

The fleet under `.codex/agents/` is the Codex projection of
`.claude/agents/` — 91 roles, generated from the Claude markdown, which is
the source of truth. Two agents per project (22 apps and 22 libs) plus
`aw-dev` and two operators:

- `<p>-e2e-dev` (effort `max`) owns the e2e contract — black-box cases
  written to fail before the implementation exists; never writes `src/`.
- `<p>-dev` (effort `medium`) owns source plus colocated unit tests; never
  writes `e2e/`.
- `aw-dev` (`medium`) owns bounded changes to the `apps/aw` Python CLI.
- `agy-operator` (`low`) runs one frozen AGY dispatch round.
- `gke-operator` (`medium`) babysits the paid GKE acceptance harness.

Keep the main thread as the controller. It freezes scope and ownership,
integrates the results, reproduces the evidence, and owns final acceptance.
Use several subagents when the workstreams are independent. Run read-only
work in parallel. Run write work in parallel only when path ownership cannot
overlap. Tell every worker that it is not alone in the checkout, that it must
preserve unrelated changes, and that it must not undo another worker's work.
Keep a tiny task in the main thread when delegation would cost more than the
work.

Every role fixes `gpt-5.6-terra` and pins its `model_reasoning_effort` to the
same value the Claude frontmatter pins. Dispatch does not select an effort;
it passes `agent_type` naming a registered role and the matching value as
`reasoning_effort`, plus `fork_turns="none"` or a positive turn count.
`.codex/hooks/require_spawn_agent_effort.py` refuses a spawn whose
`agent_type` is unregistered or whose `reasoning_effort` differs from the
role's pinned value, mirroring `.claude/hooks/require_agent_effort.py`. A
hard case may be raised at dispatch time by overriding `model` in the spawn
call — phase ownership does not move with the model, and effort stays the
pinned value.

### Use AGY only for authorized external delegation

Codex project subagents are the default bounded workers. Use one fresh
`agy-operator` subagent only when the user authorizes the exact headless AGY
payload for an external task. Its role TOML pins `gpt-5.6-terra` at `low`
reasoning, matching the Claude definition.
Make it directly inherit that user turn. Do not reuse an older operator. Do not
forward authorization through a controller message.

For more than one task in the same round: any number of `measure-only` tasks
may run concurrently, and `bounded-write` tasks may run concurrently only
across distinct persistent AGY Projects. AGY has not proven per-conversation
worktree confinement for two concurrent bounded writes in one Project, so
same-Project bounded-write tasks queue one at a time regardless of how
disjoint their write ownership looks. No skill enforces this any more — the
Claude-side `/dispatch-to-agy` skill was deleted on 2026-09-02 — so every
controller applies the rule by hand.

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
changes, publication, and cleanup. The `agy-dispatch` skill that carried the
AGY model, Project, permission, snapshot, command, and write rules was deleted
on 2026-09-02 with its reference material; the adapter itself is what remains.
Run every adapter verb
from the repository root as
`python3 scripts/agy_dispatch.py ...`. Do not use an installed or
legacy dispatcher copy.

One thing that will otherwise waste your time: `aw` names the Typer CLI at
`apps/aw`, whose engine scripts live under `apps/aw/src/aw/scripts/`; it runs
as `uv run --project apps/aw aw <group> ...` from the repository root and is
not on `PATH`. Codex skills live at `.agents/skills/aw-*/`. Their
byte-identical Claude Code mirrors live at `.claude/skills/aw-*/`. Release
Milestones own epic grouping, development order, and version identity.
`epic.py` is a read-compatible legacy facade and refuses issue-epic writes.
The Rust application that used to carry the name
is deleted and its binary is uninstalled, so a bare `aw` on `PATH` fails with
"command not found" — reach for the `uv run --project apps/aw aw ...` form
instead.

Repo-wide `CONTRIBUTING.md` has been reconciled against that deletion, and the
result is that several of its chapters now say plainly that a rule is policy
with nothing enforcing it. Read those sentences literally. A checklist there
that no longer has a checker behind it is not a gate you may cite as evidence.
