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

Almost every Codex process started in this checkout is a **reviewer**. The two
skills `/aw:codex-e2e-review` and `/aw:codex-code-review` pipe a built prompt
into `codex exec -` and parse what comes back. Assume that is you unless the
prompt you were given says otherwise.

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

- Never edit a file you are reviewing. The verdict is bound to a `sha256` over
  the reviewed bytes; an edit does not fail loudly, it silently makes the record
  describe bytes that no longer exist.
- Never run `git add`, `git commit`, or any other command that writes to the
  index or to a ref.
- Never run `.claude/aw/scripts/*.py`. Those verbs advance and record the
  lifecycle. A reviewer that advances the thing it is judging has removed the
  gate it exists to be.
- Run any git you do need through `git -c core.fsmonitor=false`. This checkout
  enables `core.fsmonitor` and a stalled daemon blocks every command that reads
  the index, indefinitely and with no error.

## The limit on this file

This file enters your context automatically, and it is **outside the digest the
verdict binds to**. A change here alters what every future reviewer is told
without invalidating a single recorded approval.

So it may constrain what you do to the checkout, and it may not bear on what you
conclude. Nothing about what makes a change good, what "done" looks like, or how
much benefit of the doubt to extend belongs here — that standard lives in the
prompt, where its bytes travel with the verdict that cites them. Keep this file
short for the same reason.

## If you were not given a review prompt

A human is driving you directly. Read `README.md` for repository inventory,
`<project>/README.md` for that project's product promises, work roots, and
gates, and `<project>/CONTRIBUTING.md` for project-local edit and verification
rules. There is no `CAPABILITIES.md`; it was deleted on 2026-08-17.

Then read `CLAUDE.md`. It holds the work-item lifecycle, the per-phase write
roots, and the rules that refuse against them, and it is the single copy of
those rules — they are deliberately not repeated here.

One thing that will otherwise waste your time: `aw` names the scripts at
`.claude/aw/scripts/` and the skills at `.claude/skills/aw:*/`, and nothing
else. The Rust application that used to carry the name
is deleted and its binary is uninstalled, so an `aw` verb you reach for fails
with "command not found" — correct, but it tells you nothing about what to
reach for instead.

Repo-wide `CONTRIBUTING.md` has been reconciled against that deletion, and the
result is that several of its chapters now say plainly that a rule is policy
with nothing enforcing it. Read those sentences literally. A checklist there
that no longer has a checker behind it is not a gate you may cite as evidence.
