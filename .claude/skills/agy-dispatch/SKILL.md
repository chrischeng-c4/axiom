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

**What each verb refuses is not here either.** Every refusal names its own
recovery at the moment it fires. Do what it says — except for the three places
it misleads, which are under *What not to do*.

## How to do it

```
worktree → grant → doctor → scaffold → (fill, capture) → lint → snapshot → dispatch
        → verify → review → prove ×2 → sweep → accept | revise | discard
```

```bash
S=.claude/skills/agy-dispatch/scripts/agy_dispatch.py
python3 .claude/skills/agy-dispatch/scripts/make_profile.py \
    --root /abs/controller-checkout --repo owner/name --project-id <id> \
    --scope libs/thing --issue 1234 --design-input libs/thing/CAPABILITIES.md \
    --write libs/thing/src/a.py:40 --out /abs/profile.json   # --run-id + --intent if one-shot

python3 $S worktree profile.json KEY     # derive the worker's checkout, cut from HEAD
python3 $S grant    profile.json         # install the Project permission set
python3 $S doctor   profile.json         # read-only preflight; must print dispatch_ready
python3 $S scaffold profile.json KEY     # both documents, as blank forms
python3 $S capture  profile.json KEY CMD # run CMD; its output is the only quotable transcript
python3 $S lint     profile.json KEY     # structure of both documents
python3 $S snapshot profile.json KEY     # freeze contract, tree, permissions
python3 $S dispatch profile.json KEY     # long; use Bash run_in_background
python3 $S status   profile.json         # which failure a run hit, if one did
python3 $S verify   profile.json KEY     # 1 = VOID (evidence untrustworthy), 2 = findings
python3 $S review   profile.json KEY     # the diff, for you to adjudicate
python3 $S prove    profile.json KEY mutant|candidate
python3 $S sweep    profile.json KEY SCRIPT
python3 $S accept   profile.json KEY     # commits on the worker's branch, prints a cherry-pick
python3 $S discard  profile.json KEY     # always, even after a failure
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

To send a round back: ticketed → new delta at `inject_prompt_file`, re-`snapshot`
if the contract changed, `resume`. One-shot → `revise`, which mints the new id
and carries the candidate.

## How to verify

Never on the report. The worker's report is a claim *about* the diff.

| | answers |
|---|---|
| `verify` | were the rules kept — not whether the change is right |
| `review` | you read the diff against the oracle, row by row |
| `prove` ×2 | restore the product to baseline keeping the worker's tests → gate must go red; restore the candidate → green. **`prove` reverts nothing; that is yours** |
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
  "create a new one-shot run id", which reads as exactly that — but the
  candidate is uncommitted in the worktree `worktree` would re-create from
  `HEAD`, so that path silently deletes it. Use `revise`.
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
fields carry no rules (#3440), and the delta contract is dispatched
unscaffolded and unlinted (#3442).

Interactive teamwork preview is a separate mode, not a dispatch — run
`scripts/teamwork_terminal.py` only when the user asks, and never headless.
