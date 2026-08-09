---
name: wi:ghan
description: Rewrite one legacy six-section `type=change` work item into Goal / How / Acceptance / Never by dispatching it as a bounded codex round, judged by the product's own draft validator with no tracker side effect. Use when an open change work item still carries the body shape the GHAN flip replaced, or when a batch of them has to be converted without a human writing each one; publishing the accepted body and dispatching the change itself remain separate, controller-only steps.
user-invocable: true
---

# WI → GHAN

## What you do

`type=change` work items are validated as Goal / How / Acceptance / Never. A few
hundred open ones still carry the six-section body that shape replaced, and each
one has to be rewritten by someone who reads the checkout — because the expensive
part is not the prose, it is turning "the CLI is wrong" into a premise carrying a
`file:line` that resolves.

That is a worker's job. Yours is three things:

1. **Project the work item into a round** — one script call. It reads the
   tracker, refuses anything that is not a rewritable legacy change, and writes
   the profile, the oracle, and the injection.
2. **Judge what comes back** — the gate says the body is *well-formed*. Whether
   it still asks for what the work item asked for, and whether its premises
   resolve, is yours to check and is written into the oracle as rows 2 and 3.
3. **Publish** — `aw wi update`. Only you can, and only after the rows pass.

Everything after the projection is `codex-dispatch`, unchanged. This skill adds
no verbs; it decides what one round is.

## How to do it

```bash
python3 .claude/skills/wi-ghan/scripts/wi_to_ghan.py <issue>
```

It prints the profile path and then the exact verb sequence, which is the
ordinary codex round with one addition:

```
worktree → rules → doctor → capture → lint → snapshot → dispatch
        → verify → review → adjudicate → accept | discard
```

`capture` records `gh issue view <issue> --json body --jq .body`. It is not
optional. The injection quotes the tracker body verbatim under
`## Current behavior`, and lint accepts a quoted line only when it is in a cited
file or in a recorded capture — the tracker body is in neither until that command
runs.

Once a round is accepted, publish and close the loop:

```bash
aw wi update <issue> --body-file .aw-wi/<issue>.md --push
```

Then the rewritten work item is ordinary backlog: dispatch the change it
describes with `agy-dispatch`, whose `from_wi.py` already reads the GHAN shape.

For a batch, run the projection per issue and let the rounds proceed
independently. Each has its own worktree, its own gate, and its own accept.

## How to verify it

The gate is one command, and it is the product's own validator rather than a
restatement of it:

```bash
python3 .claude/skills/wi-ghan/scripts/wi_draft_gate.py .aw-wi/<issue>.md --project <project>
```

`aw wi draft init` writes the body under a local workspace and `aw wi draft
validate` reports `passed` plus the per-section errors — the same rules
`aw wi validate` applies to a published body, with no issue created. The gate
also refuses a body `draft init` did not store verbatim, because a silent
normalization would mean the thing judged is not the thing you are about to
publish.

What the gate cannot see, and what you check at `review`:

- **Every `file:line` under `### Verified premises`, read at that line.** The
  validator checks a coordinate is *shaped* like one, never that it resolves.
  A premise pointing at a line that says something else passes the gate.
- **Requirement survival.** A well-formed body about a different change passes
  the gate too. Read the captured tracker body beside the rewrite and confirm
  each requirement crossed over — including ones the old body kept in `## Scope`
  or `## Reference Context` rather than in `## Requirements`.
- **The negative control's sha256.** Any 64 hex characters are accepted. Confirm
  it is the current digest of the file it claims to restore.

The gate needs an `aw` on `PATH` that post-dates the GHAN flip, and it says so
itself: a binary that still answers a change with the legacy six-section
template is refused by name — `predates the GHAN flip … the round was not
judged` — rather than reported as the body failing. Rebuild and reinstall, or
pass `--aw <path-to-built-aw>`. A `cargo install` of `aw` takes about ten
minutes; a gate run started while one is in flight judges the old binary.

## What not to do

- **Do not implement the work item in this round.** It produces a work-item
  body. The change it describes is a later round, dispatched separately.
- **Do not let the round decide the work item is wrong.** A requirement you
  believe is mistaken stays, stated as the tracker states it. If it really is
  wrong, that is a tracker edit you make deliberately, not a rewrite side effect.
- **Do not run this against a `spike`, `report`, or `epic`.** Goal / How /
  Acceptance / Never is the change shape; the other types keep the six-section
  body and the script refuses them.
- **Do not accept on a green gate alone.** A green gate says the shape holds.
  Rows 2 and 3 of the oracle are the ones that say the body is true, and only
  you run them.
- **Do not publish a body you have not read.** `aw wi update --push` is the
  irreversible half of this loop; everything before it is a local file.
