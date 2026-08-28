---
name: aw:impl-for-wi
description: Drive the impl phase — start, red, verify, test, commit — for one work item. A change runs the five verbs on that iid; an epic runs `epic.py order --open-only` first and runs the five verbs for each child, in that order, after every child's e2e has landed. `red` is the load-bearing verb — it is the only moment a failing test can be attributed to the change, and it refuses if the implementation was written first.
version: 0.1.0
user-invocable: true
---

# /aw-impl-for-wi

The implementation phase, for a change or for every child of an epic. The
e2e cases landed and are red. This phase writes the colocated unit tests —
the ones that can see what the product does not expose — and the
implementation that satisfies them, both in `apps/<app>/src/`.

There used to be two phases here, `unit` and `logic`, kept apart by
filename. That split bought one thing: a named red, measured before
anything could satisfy it. In Rust a colocated test and the code under it
are the same tree, written together, so the split is gone and what it
bought moved off the filename and onto a verb — `red` — which is what this
skill exists to get right.

## Which iid, and which type

You were given one `<iid>`. Ask `epic.py order` what it is before doing
anything else:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/epic.py" order <iid> --open-only
```

On a change, this refuses and names the type it actually found — run the
five verbs below on `<iid>` directly. On an epic, it prints the children in
the sequence they have to run. Do not branch on the exit code — a non-epic
and an epic whose graph has no solution are both exit 1, and only the text
tells them apart. A line beginning `!` or `?` means there is no order to
follow; quote it verbatim to the human and stop:

| Line | What it means |
|---|---|
| `! no order: the requirement graph closes a cycle` | no sequence satisfies the graph |
| `! R<n> depends on <cell>, which names no requirement` | an edge the order could not read |
| `? #<n> has no position` | an owned child no requirement covers |

Every child's e2e phase has to have already landed before this phase starts
on any of them — `/aw-e2e-for-wi` runs first, for the whole epic, in the same
order. With that done, run the five verbs for the first child to a landed
commit, then the next. Never start a second child's impl phase before the
one ahead of it has committed.

## The five verbs

```
uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <app> start <iid>
uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <app> red <iid>
uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <app> verify <iid>
uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <app> test <iid>
uv run --python 3.13 --no-project ".claude/aw/scripts/impl.py" --project <app> commit <iid>
```

`--project` is required and sits on the top-level parser, ahead of the verb
— there is no default project, and a printed command after the verb exits 2.
`<app>` is a directory under `apps/`. Each verb prints the next one; paste it
rather than retyping it. The pin is load-bearing for the same reason it is
in `e2e.py`: `tomllib` needs 3.11+, and a bare `python3` here is 3.9.

`impl.py start` is the one verb that runs before anything has been written,
and it is the only one positioned to refuse a tree that was already dirty:
the four after it measure the working tree against HEAD, so uncommitted work
from another writer would be counted as this phase's. It also checks that
the e2e phase landed its commit — this phase's predecessor — which is why
entering the phase by running `red` first is not a shortcut but a skipped
check.

## The order that makes `red` pass, and the order that makes it refuse

`red` is the only moment in this phase at which a failing test can be
attributed to the change. It passes only at a moment when a named test
fails that was not failing at HEAD — which is a moment the implementation is
not finished. There is exactly one order that produces that moment:

1. Write the colocated unit test(s), wired in with `#[cfg(test)] mod
   tests;`, in `tests.rs` or `tests/mod.rs`. Write enough of a signature —
   `todo!()` is enough — for the tree to compile.
2. Run `impl.py red`. It builds, and if the build is green it runs the test
   command, subtracts what was already failing at HEAD, and records the
   *names* that are newly red into `.aw/impl-red/<iid>.json`, along with the
   HEAD sha and a sha256 of every test file. The record accumulates across
   runs, so an ordinary TDD loop — test, red, satisfy, next test — keeps
   every red it measured, not only the last.
3. Only now write the implementation, until the recorded tests and the e2e
   cases are green. Re-run `red` whenever a new test is added that fails —
   each one is unioned into the record.

Write the implementation first and this collapses: the build is green, the
test the record was supposed to name is already passing, and `red` refuses
with nothing to name — there is no failing test to record, and no red can
be manufactured after the fact. That refusal is `red`'s row `R2 named red`.
Its sibling row, `R1 build is green`, refuses first and separately when the
tests do not even compile — a test naming a function nobody wrote fails to
*compile*, not to *run*, and one exit code cannot tell those apart, so the
row does. A third row, `R3 contract still red`, only runs on the run that
opens the record and refuses if the e2e cases are already green — a tree
where the implementation was written alongside the test can build and fail
the test correctly while still having satisfied the contract, and that is
not a moment `R1`/`R2` alone can see.

## Editing a test after `red` measured it

This is detected, not refused. `impl.py verify` and `impl.py test` both run
`C2 the recorded red`, which holds the sha256 of every test file at the
moment `red` ran. Change a test file afterwards and `C2` refuses until
`red` runs again — and `red` over a test edited into passing finds nothing
newly failing, because the implementation is already there. That is the
retired `logic`-may-not-touch-`tests.rs` rule, restated as a measurement
instead of a filename: colocated tests and the implementation share
`src/**` deliberately, but an impl commit touching no test file at all is
still refused outright by `C0` (the write-root check `e2e.py` also carries),
because a phase that wrote no test has nothing for `red` to have measured
in the first place.

`verify` runs the mechanical list — the write root, the test-file
existence, and `C2` — without building or running anything. `test` runs the
recorded names, the whole suite, and both sides of the e2e contract:
`T1`/`T2`/`T3` are the build, the recorded tests, and the suite being whole;
`T4`/`T5` are the contract accepting the tree and still refusing HEAD.
`commit` re-runs everything and only then writes the commit, deleting the
`.aw/` record once its names are copied onto the commit's `Impl-Red:`
trailer.

## The epic ordering hazard, and the row that catches it

The old order ran one child through all three phases before starting the
next. The new order does not: `/aw-e2e-for-wi` lands e2e for every child of
the epic first, then this skill runs impl for every child, in the same
order. Between those two sweeps, an earlier child's implementation can turn
a later child's e2e cases green before that later child's impl phase ever
starts — leaving the later child with no e2e case still refusing HEAD to
attribute a red to.

`impl.py test`'s row **`T5 the contract refused HEAD`** is what catches
this, and it names the shape when it fires: cases already green at HEAD,
before this implementation exists. Its own message says what that means
driving an epic — the ordinary shape of a child whose sibling already
delivered the behaviour, not a broken gate. When `T5` fires this way:

- do not weaken, delete, or route around the case;
- report it to the human as what it is — this child's e2e cases do not
  discriminate any more, because an earlier sibling's implementation already
  satisfies them;
- the fix belongs to `/aw-grill-meta-to-wis`, not to this phase: the child
  set or its requirement mapping is what drifted, and only that skill
  reorganises children.

## Report

| Outcome | What to say |
|---|---|
| the phase closed | name the commit, the `Impl-Red:` names, and the `Impl-Contract:` case ids |
| `red` refused | quote `R1`/`R2`/`R3` verbatim — most often the implementation was written before the test |
| a later row went red | quote the FAIL rows verbatim, then fix the cause — not the gate |
| `T5` fired on a sibling-satisfied case | quote it and say which sibling child already delivered the behaviour |
| the work item was not a change or an epic | quote `epic.py order`'s refusal and stop |
| the epic had no order | quote the `!` or `?` lines and stop |

## Never

This addresses the agent running the phase, not the author of the work item.

- Never write the implementation before `red` has recorded a named failure.
  A phase run out of order produces a green whose cause nothing measured.
- Never edit a test file to make it pass instead of writing the
  implementation, and never edit one after `red` ran without expecting to
  re-run `red` — `C2` will send you back regardless.
- Never amend a phase commit. It is the only record of what was measured.
- Never widen a test selector, delete a test, or relax an assertion to make
  a phase pass. If a test is wrong, say so to the human with the evidence
  and stop.
- Never route around a `T5` firing on a sibling-satisfied case by editing
  the case or the child's requirement mapping yourself. That belongs to
  `/aw-grill-meta-to-wis`.
- Never pick an epic's child order yourself, and never edit an epic body to
  make `epic.py order` parse.
- Never start a second child's impl phase before the one ahead of it has
  landed its commit, and never start any child's impl phase before every
  child's e2e has landed.
- Never report the phase complete on the strength of a green `test`.
  `commit` re-runs everything, and only the commit it writes is the record.
