---
name: wi-tdd
description: Drive one work item through the e2e → unit → logic ladder, where each phase must produce a named red before the next phase is allowed to turn it green. Takes a change work item, or an epic — whose children are ordered by the dependency graph its own verification inventory declares. Each phase prints the next command; this skill exists to start the ladder and to say what it refuses.
version: 0.1.0
user-invocable: true
---

# /aw:wi-tdd

Three phases, in this order, on one change work item:

| Phase | Writes | Leaves behind |
|---|---|---|
| `e2e` | the black-box cases under the project's e2e root | cases that fail, and the digest of the bytes that failed |
| `unit` | colocated tests in `src/**/tests.rs`, plus a skeleton that compiles | a **named** failing test list, recorded on the commit |
| `logic` | the implementation, outside `tests.rs` | both oracles green in the tree and still red at HEAD |

The ordering is not a style preference. Each phase's green is only evidence if
something red was measured immediately before it, by name, in the same tree —
and the phase that would have to measure that red is the one before it. Writing
the implementation first means every test that passes afterwards passes for an
unknown reason.

## Run it

You were given one `<iid>`. Find out which kind it is by asking for its order:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/epic.py" order <iid> --open-only
```

On a change work item `epic.py order` refuses, and the refusal names the type
it actually found. That message is the answer, not an error to route around —
read it, and go to **One change** below. Do not branch on the exit code: a
non-epic and an epic with no computable order both exit 1, and only the text
tells them apart.

### One epic

`order` prints the children in the sequence they have to run, one rank per
line, each with the requirements it covers and the children it waits on. It
composes two sections that were already in the body: `## Verification
Inventory` partially orders the requirements through its `Depends On` column,
and `## Child Work Items` maps each child to the requirements it covers. A
child inherits the position of the *deepest* requirement it covers, and equal
positions break by `priority:` then by issue number.

Run each child through **One change**, in that order, to completion, before
starting the next.

Three lines mean there is no order to follow, and each one stops you:

| Line | What it means |
|---|---|
| `! no order: the requirement graph closes a cycle` | no sequence satisfies the graph; nothing is printed to run |
| `! R<n> depends on <cell>, which names no requirement` | a dependency was declared that the order could not read, so the order below it is missing an edge |
| `? #<n> has no position` | an owned child the child table maps to no requirement |

Report these to the user and stop. Do not pick an order yourself. An epic body
you edited to make the graph parse is an epic whose dependencies you decided,
and the sequence would then be yours rather than the author's.

### One change

Every command goes through the pinned interpreter:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <app> start <iid>
uv run --python 3.13 --no-project ".claude/aw/scripts/unit.py" --project <app> start <iid>
uv run --python 3.13 --no-project ".claude/aw/scripts/logic.py" --project <app> start <iid>
```

`--project` is required and must come before the verb — it is on the top-level
parser, so after the verb it exits 2. There is no default: the phases used to
assume one project, and a default is how a phase writes into a tree nobody
named. `<app>` is a directory under `apps/`, and every command a phase prints
afterwards carries the same flag, so paste those verbatim rather than
retyping them.

The pin is load-bearing. `e2e.py` and `unit.py` read TOML, `tomllib` is 3.11+,
and a bare `python3` is 3.9 on at least one machine this runs on — where the
failure is a `ModuleNotFoundError` traceback that reads like a broken script
rather than a wrong interpreter. The scripts are in the checkout at
`.claude/aw/scripts/e2e.py`, `.claude/aw/scripts/unit.py`, and
`.claude/aw/scripts/logic.py`, and those paths are relative to the repository
root — run them from there. For readability the table below writes the short
form; each entry means the same pinned launcher with that script and verb.

Twelve commands, in this order, and each one prints the next:

| Phase | The four verbs, in order |
|---|---|
| contract | `e2e.py start`, `e2e.py verify`, `e2e.py test`, `e2e.py commit` |
| invariant | `unit.py start`, `unit.py verify`, `unit.py test`, `unit.py commit` |
| implementation | `logic.py start`, `logic.py verify`, `logic.py test`, `logic.py commit` |

`start` prints what the phase is for and refuses a dirty tree. `verify` runs
the mechanical list without running anything under test. `test` runs the
oracles. `commit` re-runs everything and only then writes the commit.

Do not skip `verify` because `test` would catch it anyway. They fail for
different reasons and name different remediations, and a phase that goes
straight to `test` reports a red about the tree when the defect is in the
change's shape.

## What each phase will not let you do

Read these before writing anything, because each one is a gate you will
otherwise hit after doing the work.

`e2e` requires every case to be **red** before it will commit, and red for the
declared reason. A case that errors on an import, a typo, or a missing fixture
is not the same event as a case whose assertion refused the current behaviour,
and the phase separates them.

`unit` requires two commands to be distinguishable: the build, and the tests. A
`cargo test` that exits non-zero because nothing compiles and one that exits
non-zero because an assertion failed are the same exit code, so the phase
declares both and refuses the first. It records failing test **names**, not a
count and not an exit code — a selector that matches nothing exits 0 with 0
tests, and a count cannot tell a new red from a red that was already there. The
names are set-subtracted against HEAD, so a test that was already failing
cannot be claimed as the one this phase produced.

The skeleton `unit` writes — the `todo!()` that makes the tests compile — goes
**outside** `tests.rs`. That is what leaves `logic` free to replace it, and it
is why the split between the two phases is enforceable by filename instead of
by parsing `#[cfg(test)]` spans, which has been got wrong here before.

`logic` requires the recorded tests to be **present** as well as passing. A
deleted test and a passing test are the same absence in a summary line. It then
requires the rest of the suite to be whole — nothing else newly red, and
nothing else silently unwired — and requires the e2e cases to accept the tree
while still refusing HEAD.

Every row that could not be run prints as `PENDING`, never as nothing. A row
that vanished from a report and a row that passed read identically otherwise.

## Report

| Outcome | What to say |
|---|---|
| the ladder closed | name the three commits and the recorded red the middle one carries |
| a phase went red | quote the FAIL rows verbatim, then fix the cause — not the gate |
| an epic had no order | quote the `!` or `?` lines and stop |

A phase that refuses is doing its job. Say what it refused and why, in its own
words.

## Never

This addresses the agent running the ladder, not the author of the work item.

- Never write implementation before `unit` has recorded a named red, and never
  write a test before `e2e` has committed cases that failed. A phase run out of
  order produces a green whose cause nothing measured.
- Never edit a file the previous phase wrote. Not to fix a test, not to rename
  one, not to run a formatter over it — including when the diff is whitespace
  only.
- Never amend a phase commit. The commit is the only record of what was
  measured, and amending it rewrites the measurement to match the outcome.
- Never widen a selector, delete a case, or relax an assertion to make a phase
  pass. If a case is wrong, say so to the user with the evidence and stop.
- Never decide an epic's order yourself when the graph does not yield one, and
  never edit the epic body to make it parse.
- Never report the ladder as complete on the strength of a green `test`.
  `commit` re-runs everything, and only the commit it writes is the record.
