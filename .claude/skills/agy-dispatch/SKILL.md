---
name: agy:dispatch
description: Safely dispatch one bounded ticketed or one-shot task to headless AGY using persistent AGY Project permissions plus a task-local command/write contract, then independently verify its report. Use whenever Claude delegates audits, measurements, investigation, transcription, or tightly scoped implementation to AGY; ticketed tasks reuse one conversation per live issue, unticketed tasks run once without resume, implementation requires frozen design inputs, and acceptance remains controller-only.
user-invocable: true
---

# AGY Dispatch

## What you do

Three things. Everything else is a verb.

1. **Set the question** — the oracle: what this round claims, and the rows that
   decide it.
2. **Brief the worker** — the injection.
3. **Accept** — only you can.

**How to write those two documents is not here.** `scaffold` emits them as
forms whose every slot carries its own rule as a `<!-- fill -->` comment, and
`lint` refuses a form still holding one. Read the form when you author; the
comments are stripped before the worker sees anything.

**Write the work item first and most of both documents is already yours.** A
`type=change` body in Goal / How / Acceptance / Never carries the round: the
change points *are* the write allowlist, the acceptance table *is* the
measurement table, the premises *are* the reference list and the quote under
`## Current behavior`. `from_wi.py` projects them, and refuses rather than
half-project. Four slots have no source in a work item and stay forms —
`## Fabrication tells`, `## Required change`, `## Shape to follow`,
`## Definition of done`. Those four are the authoring; the rest was
transcription, which is where a write allowlist quietly stops matching the
change points it was copied from.

**What each verb refuses is not here either.** Every refusal names its own
recovery at the moment it fires. Do what it says — except for the three places
it misleads, which are under *What not to do*.

## How to do it

```
from_wi | (make_profile + scaffold) → worktree → grant → doctor → (fill, capture)
        → lint → snapshot → dispatch
        → verify → review → adjudicate → prove ×2 → sweep → accept | revise | discard
```

```bash
S=.claude/skills/agy-dispatch/scripts/agy_dispatch.py

# From a `type=change` work item: profile and both documents in one step.
python3 .claude/skills/agy-dispatch/scripts/from_wi.py 1234
    # --print-only first, to read what it derived before it writes anything
    # --body-file to project a body that is not on the tracker yet
    # --scope overrides the scope derived from the change points
    # --root defaults to this checkout; commit first, since `worktree` cuts HEAD
    # never overwrites a document that already exists

# Or by hand, when there is no such work item.
python3 .claude/skills/agy-dispatch/scripts/make_profile.py \
    --scope libs/thing --issue 1234 --design-input libs/thing/CAPABILITIES.md \
    --write libs/thing/src/a.py:40 --gate "uv run pytest libs/thing"
    # --run-id + --intent instead of --issue if one-shot
    # prints the profile path; --root --repo --project-id --out --inject override

python3 $S worktree   profile.json KEY     # derive the worker's checkout, cut from HEAD
python3 $S grant      profile.json         # install the Project permission set
python3 $S doctor     profile.json         # preflight; 2 on a blocker, and the round's own unfinished steps
python3 $S scaffold   profile.json KEY     # both documents, as blank forms
python3 $S capture    profile.json KEY CMD # run CMD; its output is the only quotable transcript
python3 $S lint       profile.json KEY     # structure of both documents
python3 $S snapshot   profile.json KEY     # freeze contract, tree, permissions
python3 $S dispatch   profile.json KEY     # long; use Bash run_in_background
python3 $S status     profile.json         # which failure a run hit, if one did
python3 $S verify     profile.json KEY     # 1 = VOID (evidence untrustworthy), 2 = findings
python3 $S review     profile.json KEY     # the diff, for you to adjudicate
python3 $S adjudicate profile.json KEY admit|reject FINDING # record decision on a scope finding
python3 $S prove      profile.json KEY mutant|candidate
python3 $S sweep      profile.json KEY SCRIPT
python3 $S accept     profile.json KEY     # commits on the worker's branch, prints a cherry-pick
python3 $S discard    profile.json KEY     # always, even after a failure; refuses while work sits in no commit, or in a commit the controller holds in no form
```

`grant`, `doctor`, and `status` take the profile alone; the rest also take the
task key — the issue id, or the `run_id` for a one-shot. Exit `0` clean, `1`
VOID, `2` findings. Never read `2` as `1`: a VOID discards the round, a finding
is a question about a candidate that still exists.

Three things the scripts will not tell you:

- **Commit before `worktree`.** It cuts from `HEAD`, so anything uncommitted —
  the design input, the fixture, the file being extended — is invisible to the
  worker.
- **`capture` one single-line command per fence.** `lint` reads each `$ ` line
  as a command, so a command wrapped across lines never matches its record.
- **Run one already-passing gate in the worktree before dispatching.** It
  proves the base is green and leaves a warm `target/`, so the round's budget
  buys reasoning instead of a cold build.

To send a round back: `revise`, ticketed or not. It mints the new id, carries
the candidate, and copies the oracle unchanged, so the delta says what was
wrong rather than what to do. Name a delta file and it is held to the same
seven sections before it is copied; omit the argument and you are handed the
blank form, as `scaffold` hands out the round form. A ticketed round comes back as a one-shot
revision — the ticket id is its identity and is spent — with `revision_of`
keeping the descent in the prompt, the sealed task state, and the `Refs #`
trailer on the accepted commit. `resume` is the other thing: same injection,
continuation framing, for a round that stopped rather than went wrong.

## How to verify

Never on the report. The worker's report is a claim *about* the diff.

| | answers |
|---|---|
| `verify` | were the rules kept — not whether the change is right |
| `review` | you read the diff against the oracle, row by row |
| `prove` ×2 | restore the product to baseline keeping the worker's tests → gate must go red; restore the candidate → green. **`prove` reverts nothing; that is yours** |
| | the `mutant` half may break a frozen file — a round asserting *about* a design input has its only real falsifier there, and the proof records which paths it perturbed. The `candidate` half refuses until you restore them, and so do `verify` and `accept` |
| `sweep` | one single-defect mutant per rule the oracle claims |

A gate nobody has seen fail proves nothing — a test written against the
implementation just produced passes by construction. So the pair is a floor and
the sweep is the evidence. Writing that sweep is the part with no verb behind
it:

- **A mutant that will not build is a badly written mutant, not a kill.** Make
  the same defect representable; never swap in a different one that compiles.
- **Apply and restore with `write_text`.** A copy preserving the old mtime lets
  cargo skip the rebuild — false kill, then false green.
- **Classify by whether the harness ran**, not by grepping for "error". Cargo
  prints `error: test failed` for an ordinary assertion failure.
- **A survivor is a finding against the round's evidence, not the product.**
  Close it by authoring the missing row, never by editing the test yourself.
- **A round that only adds evidence has nothing to revert.** Mutate the
  *unchanged* product instead: break the rule the new rows are meant to pin.

Two things no gate can see, so check them at `review`:

- a `SPEC-MANAGED` block edited in the `.rs` and not in its tech-design mirror —
  it compiles, tests green, and is reverted at the next generation
- a fixture name that also exists in this checkout, which makes a leak into the
  ambient repository satisfy the assertion

For a round with no shell (empty allowlists), acceptance is a script *you*
write that imports the symbols, calls them on the oracle's inputs, and exits
non-zero — kept where the worker cannot read it. Any test result or
per-criterion verdict in such a report is fabricated by construction.

## What not to do

- **Never derive a profile from the last round's** — regenerate. Editing
  `--write` leaves the frozen complement describing the old round, so the new
  target arrives already frozen: writing it is a finding and not writing it is
  a finding. `doctor` has a check for this and it cannot fire (#3439).
- **Never author a fresh round after a one-shot id is spent.** The refusal says
  "create a new one-shot run id", which reads as exactly that — but a new key
  derives a new branch and path, so `worktree` builds a *second* checkout from
  `HEAD` and strands the uncommitted candidate in the first, where no verb of
  the new round can see it. Use `revise`. When the contract itself must widen,
  `revise` is not enough either: it copies the write set and the oracle
  unchanged. Generate a fresh profile over the full widened write set — so
  `protected_artifacts` is recomputed rather than left stale — and graft the
  spent round's `worktree` block into it. `worktree` reuses a path whose branch
  matches, so the candidate carries.
- **Never treat a timeout as a denial.** The worker was cut off with its work on
  disk. Read the diff and run the gate before deciding anything.
- **Never `abandon` a result you dislike** — it releases a run id, and it is for
  runs that provably produced nothing.
- **Never widen permissions to clear a denial.** Tighten the prompt, or add one
  narrow Project rule and re-`doctor`.
- **Never edit the worker's checkout while its round runs.** Your own checkout
  is free — the worker has its own.
- **Never `nohup ... &`.** Use the Bash tool's `run_in_background`.
- **Never leave a round undiscarded.** The Project binding stays pointed at the
  worktree and the next `agy` session opens there.
- **Never fabricate a HITL approval, and never let AGY close a ticket.**

## Reference

[references/lifecycle.md](references/lifecycle.md) is the normative state
machine — read it fully before a first dispatch or a takeover.
[references/report-verification.md](references/report-verification.md) and
[references/inventory-verification.md](references/inventory-verification.md)
are acceptance checklists;
[references/report-review-vocabulary.md](references/report-review-vocabulary.md)
holds distinctions worth copying into a round's `## Fabrication tells`.
[references/profile-template.json](references/profile-template.json) and
[references/one-shot-profile-template.json](references/one-shot-profile-template.json)
document fields the generator does not emit.

Two rules live in prose only because they have no slot yet: the profile's
fields carry no rules (#3440), and the delta contract carries the round form's
slots but none of its own — that a revision report is a self-contained
replacement, that a list edit says `append` or `replace <entry>`, and that a
delta correcting the controller quotes the line it supersedes (#3442).

Interactive teamwork preview is a separate mode, not a dispatch — run
`scripts/teamwork_terminal.py` only when the user asks, and never headless.
