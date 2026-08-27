---
name: aw:e2e-for-wi
description: Drive the e2e phase — start, verify, test, commit — for one work item. A change runs the four verbs on that iid; an epic runs `epic.py order --open-only` first and runs the four verbs for each child, in that order, before touching the next child's e2e. Refuses a case that is not RED against the current tree, and refuses one that was never run. Writes nothing under src/.
version: 0.1.0
user-invocable: true
---

# /aw-e2e-for-wi

The contract phase, for a change or for every child of an epic. Nothing
under `apps/<name>/src/` may be written here: the only thing this phase can
produce is a case that is red, and a case that is red before the
implementation exists is the only kind that later proves anything by going
green.

## Which iid, and which type

You were given one `<iid>`. Ask `epic.py order` what it is before doing
anything else:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/epic.py" order <iid> --open-only
```

On a change, this refuses and names the type it actually found — that
refusal is the answer, not an error to route around. Stage the change's body
and run the four verbs below on `<iid>` directly:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/change.py" fetch <iid>
```

On an epic, `order` prints the children in the sequence they have to run,
one rank per line. Do not branch on the exit code — a non-epic and an epic
whose graph has no solution are both exit 1, and only the text tells them
apart. A line beginning `!` or `?` means there is no order to follow; quote
it verbatim to the human and stop:

| Line | What it means |
|---|---|
| `! no order: the requirement graph closes a cycle` | no sequence satisfies the graph |
| `! R<n> depends on <cell>, which names no requirement` | an edge the order could not read |
| `? #<n> has no position` | an owned child no requirement covers |

With an order in hand, run the four verbs for the first child to a landed
commit, then the next, in that order. Never start a second child's e2e
before the one ahead of it has committed.

## The four verbs

```
uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <app> start <iid>
uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <app> verify <iid>
uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <app> test <iid>
uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <app> commit <iid>
```

`--project` is required and sits on the top-level parser, ahead of the verb
— there is no default project, and a printed command after the verb exits 2.
`<app>` is a directory under `apps/`. Each verb prints the next one; paste it
rather than retyping it.

The pin is load-bearing: `e2e.py` reads TOML, `tomllib` is 3.11+, and a bare
`python3` is 3.9 on at least one machine this runs on, where the failure is a
`ModuleNotFoundError` that reads like a broken script rather than a wrong
interpreter.

`e2e.py start` prints what to write and refuses a dirty tree. `verify` runs the
mechanical list — every case inventoried in `Cargo.toml` and self-declaring
— without running anything under test. `test` runs every case and requires
each one red. `commit` re-runs everything and only then writes the commit.
Do not skip `verify` because `test` would catch it anyway: they fail for
different reasons and name different remediations.

## The non-negotiable

Write the case, then **run it**, before moving on — do not write it, glance
at it, and assume it fails. `test` is what observes the failure, and a case
never run is a case whose "it should fail here" is a guess. A contract that
was green before the change was written proves nothing about the change: it
either observes something other than the behaviour it names, or that
behaviour is already there and this work item has nothing to do.

`e2e.py test`'s own row for this is `E1 cases are red`, over every case the
change wrote. It reads red *in the working tree* as a HEAD measurement: `C0`
refused every changed path outside the e2e root, so the product in the tree
is the product at HEAD, byte for byte.

## The write root, and the one path outside it

Cases go directly under `apps/<app>/e2e/`, one file per case — never in a
subdirectory, which holds fixtures for a harness rather than cases. Every
case is declared in the crate manifest one level up:

```
[[test]]
name = "<stem>"
path = "e2e/<stem>.rs"
```

with `autotests = false` set under `[package]`. That manifest edit is the
one path outside `e2e/` this phase may write — `C0` allows it by exact
filename because registering a case and writing it are one act: with
autodiscovery off, a file nobody declared does not run, and a declaration
whose file is gone is ignored. Writing the `.rs` file without the `[[test]]`
stanza is a case nothing will ever run; `C1 registered` refuses it and names
the stanza it wants.

Every assertion has to read something the product produced. `C2 observes
the product` refuses an `assert_eq!` between two literals, or an `assert!`
over a literal boolean — the one shape decidable without running anything,
and green forever for reasons that have nothing to do with the change.

## The epic ordering hazard

The old order ran one child through all three phases before starting the
next. This skill's order does not: an epic's e2e phase runs child by child,
and its impl phase — `/aw-impl-for-wi` — runs after every child's e2e has
landed, also child by child. Between those two sweeps, an earlier child's
implementation can already exist by the time a later child's e2e is being
written.

That does not make this phase's own gate weaker. `E1 cases are red` still
measures the tree at the moment the case is written, and a case that comes
out green here — because a sibling's implementation, landed in an earlier
round, already satisfies it — fails exactly as it would for any other reason
a case is not observing the change. Do not write a case that cannot be
red against the current tree; if the behaviour already exists, that child's
`docs/product/` promise or its epic body has drifted from what is actually
outstanding, and that is a finding to report, not a case to write anyway.

## Report

| Outcome | What to say |
|---|---|
| the phase closed | name the commit and the case ids `E2E-Red` carries |
| a row went red | quote the FAIL rows verbatim, then fix the cause — not the gate |
| the work item was not a change or an epic | quote `epic.py order`'s refusal and stop |
| the epic had no order | quote the `!` or `?` lines and stop |

A refusal — the script's or a phase's — is doing its job. Say what it
refused and why, in its own words.

## Never

This addresses the agent running the phase, not the author of the work item.

- Never write anything under `apps/<app>/src/`. That tree belongs to
  `/aw-impl-for-wi`, and a case written beside the code it is meant to
  refuse has nothing left to refuse.
- Never edit a file a previous run of this phase already committed — not to
  fix a test, not to rename one, not to run a formatter over it, even when
  the diff is whitespace only.
- Never amend a phase commit. It is the only record of what was measured,
  and amending it rewrites the measurement to match the outcome.
- Never widen a selector, delete a case, or relax an assertion to make a
  phase pass. If a case is wrong, say so to the human with the evidence and
  stop.
- Never pick an epic's child order yourself, and never edit an epic body to
  make `epic.py order` parse. The order is the epic's own graph.
- Never start a second child's e2e phase before the one ahead of it has
  landed its commit — a second writer moves HEAD under the first, and every
  row here measures against HEAD.
- Never report the phase complete on the strength of a green `test`.
  `commit` re-runs everything, and only the commit it writes is the record.
