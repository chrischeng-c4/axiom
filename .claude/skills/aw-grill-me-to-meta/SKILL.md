---
name: aw:grill-me-to-meta
description: Interview the human through AskUserQuestion until a product promise is stated as observable behaviour, then write it into the owning project's META-docs — `README.md`, `STATUS.md`, `ROADMAP.md`, and the areas under `docs/**` — in one run, so a promise and the ground it stands on land together. Use before any work item exists for the promise; `/aw-grill-meta-to-wis` measures the gap between these documents and the tracker afterwards. Writes nothing else: no source, no test, no tracker. Requires a clean tree over those four paths, and lands through `metadoc.py check`, `meta.py check`, and `metadoc.py commit`.
version: 0.1.0
user-invocable: true
---

# /aw-grill-me-to-meta

Grill the human, then write the promise down before anything tracks it. This
skill owns the product documents of one project: its `README.md`, its
`STATUS.md`, its `ROADMAP.md`, and the areas under `docs/**` — a section per
promise, a file per area, an index per directory. It writes no product
source, no test, and no work item. The work item comes *after*:
`/aw-grill-meta-to-wis` measures these documents against the codebase and
the tracker, opens what is missing, and stamps a section's heading with the
number it opened. Until then a section carries no `(#<iid>)`, and this skill
never writes one.

## Four paths, one run

The allowlist is `<project>/README.md`, `<project>/STATUS.md`,
`<project>/ROADMAP.md`, and `<project>/docs/**`. It was `docs/product/`
alone until 2026-08-27, and the other three were *forbidden* — the argument
being that an area stands on its capability id, its STATUS row, its ROADMAP
outcome, so editing both at once is standing on the ground while moving it.

That argument was right about the hazard and wrong about the remedy. Two
runs by two skills meant the second one got skipped, and a promise whose
STATUS row never arrived is a promise nothing measures. What replaces the
separation is measurement: `metadoc.py check` resolves every claimed id
against the file it is claimed from **as edited**, not as committed, so a run
that moved the ground under itself is refused in the same breath rather than
assumed not to have. The ground is checked, not fenced off.

So a section that claims a new STATUS row and the row itself land in one
commit. What still may not happen in that commit is source, tests, or a
tracker write.

## Nothing to stand on, no promise

You need one project — `apps/<name>` or `libs/<name>`. If the invocation
names one, that is the project. If it does not, read the conversation for
candidates — the projects whose files were opened, whose names were said,
whose promise the request is plainly about — and put them to the human as
the first **AskUserQuestion**, the likeliest first, filled out from the
`apps/*` and `libs/*` directories. "Other" lets the human type one the
conversation never mentioned. What the context decides is which options are
offered and in what order, never which one is true: a project inferred and
not confirmed is a guess, and this skill writes nothing on a guess.

A promise is owned by exactly one thing: a shipped promise by the STATUS
rows that measure it, a future promise by the ROADMAP outcome that will. Both
files are now yours to write, so a project missing one is not a stop — it is
the first thing the interview has to produce. Ask for the rows or the outcome
before the section that will claim them, and write them in the same run. The
repository `CONTRIBUTING.md` chapter "Meta-doc content contract" — its
adopted-set table and the `STATUS.md` / `ROADMAP.md` shapes below it — says
what each must contain, and `/project-readme-check` is the gate on both.

Then read what is there: the README `## Capabilities` index (its capability
ids are what an area file's first paragraph names), every STATUS row id,
every ROADMAP outcome id, and everything already under `docs/**`. Never ask
for what those already answer.

## A clean tree, or there is no diff

This skill is usually run against a project that already promises things, and
what the human has to read afterwards is not the file — it is what changed in
it. Git is what separates the two, and it can only do that while the change
stands alone. Before the first question, run

```
git -c core.fsmonitor=false status --short -- <project>/README.md <project>/STATUS.md <project>/ROADMAP.md <project>/docs
```

and require it to print nothing. The `-c core.fsmonitor=false` is not
decoration: this checkout enables the file-system monitor, and a stalled
daemon hangs every command that reads the index, indefinitely and without an
error. If the listing is not empty, stop and print it. Somebody's uncommitted
META-doc edit is already there, and an interview that writes on top of it
produces one diff nobody can split back into two changes. Committing,
stashing, or declaring this run a continuation of that work is the human's
call, and this skill makes none of the three.

The same cleanliness is what the work item needs later: the ladder's first
phase refuses a dirty tree outright, and none of these four paths is inside
any phase's write root, so a META-doc written and left uncommitted blocks the
very item it exists for. Landing this run before the work starts is the
point, not a courtesy.

## Three modes

| The human wants | Mode | Precondition |
|---|---|---|
| documents where there are none | **create** | the directory has no index |
| a section added, a section's part changed, a new area, an index changed, a STATUS row or ROADMAP outcome added | **modify** | the index exists |
| a section or an area removed | **delete** | the section heading carries no ` (#` |

Resolve the mode before the first question; the request usually states it,
and the directory listing settles the rest. In **create**, the index comes
first — positioning, who reads it, the horizon order — and sections follow.
In **modify**, a bound section (one whose heading carries ` (#<iid>)`) keeps
its title: the title *is* the work item's title, so renaming it is
`/aw-grill-meta-to-wis`'s update path first and this skill second. Every
other part of a bound section may change. In **delete**, a bound section is
refused: its work item is still open on it. Say so and stop; the tracker side
ends first — closed, or not planned — and only then does the section go.
Deleting an area file removes its rows from its directory's index in the same
edit.

## Placement

- One file per area under `docs/<family>/`, named for the area — a slug a
  reader of the README would recognise — never for a work item, a date, or a
  horizon. The file's first paragraph names the README capability id or ids
  it covers, so the README and the area agree on what an area is without the
  file name having to be the id.
- **Each directory under `docs/` carries its own `README.md`, and that file
  is its index.** `metadoc.py check` reads the index beside the file it is
  checking, not one index for the whole project, so `docs/product/` and
  `docs/operating/` can each hold a family of areas without either index
  having to claim the other's sections. A section indexed in the wrong
  directory's README is refused as unindexed.
- The index carries `## How this directory is organised`, `## Positioning`,
  a reader table under `## Who <project> is for`, `## Horizons`
  (`| Horizon | Outcome | Section |`, ordered by dependency and saying why),
  and `## Section index` (`| Section | File | Kind | Owner |`, where Kind is
  `shipped`, `shipped, limited`, `shipped, leaving`, or `outcome`, and Owner
  is `STATUS <ids>` or `ROADMAP <id>`).
- A new area is a change to its index first, then the file. Ask with
  **AskUserQuestion** which existing area a new section joins, offering each
  area and "a new area" as the options. A new area needs a README capability
  to stand on; unlike before, minting that capability is now inside this
  skill's scope — but it is a question for the human, not a line you add so
  the check passes.
- Non-goals are not sections. Each area file ends with
  `## Non-goals in this area`, listing what a reader of that area would
  otherwise assume and pointing at the ROADMAP entry that gives the reason.
- The three top-level documents are in the allowlist and are **not** areas.
  `README.md`, `STATUS.md` and `ROADMAP.md` are not measured against the
  section schema below — their own shapes are owned by the product-document
  contract and by `meta.py check`, both of which run before the commit.

## The interview

Ask with **AskUserQuestion**, in rounds of at most four questions, with 2-4
concrete options per question drawn from STATUS, ROADMAP, and the README —
the human can always answer "Other". Stop as soon as the section's own parts
are answered. Each part has a consumer that can refuse it — the skill that
will read the section, or the reader the index names — so an answer that
satisfies no consumer is not an answer yet.

| Part | The question | Refused when |
|---|---|---|
| Problem | what a user of this project cannot do, or gets wrong, today | it restates the title, or is under twenty words — `/aw-grill-meta-to-wis` drafts a work item's `## Problem` from this |
| Who | which publisher, subscriber, operator, caller, or client sees the difference | "users" |
| Promise | the observable behaviour, one sentence per thing a gate could refuse | it names behaviour no STATUS row or ROADMAP outcome owns — including one this run is adding |
| Limits today | *(shipped only)* what the promise does not do yet, in the terms the next section will call its `Problem:` | it repeats a non-goal — a limit is something the project intends to close, a non-goal is not |
| Non-goals | what a reader would assume is included and is not | empty |
| Open | *(future only)* the decisions the work item will have to settle, each as a question | a default dressed as a question, or an answer the human already gave |
| Neighbours | which sections this one extends, narrows, repairs, or supersedes, in this file or another | the area has sections and the answer is "none" |
| Owner | the STATUS row ids (shipped) or the one ROADMAP outcome id (future) | `grep` does not find the id in the file it is claimed from, *after* this run's edits |

`Open` is where this skill is most tempted to help, and it may not: an
`Open:` line is a question the work item or the human answers later, and
writing the answer here is answering for them.

The widened scope adds one temptation the old one could not have: an owner
id that does not resolve is now fixable by adding the row. Ask before you do.
A STATUS row is a claim that something is measured today, and inventing one
to satisfy a check is how a promise reaches the README with nothing behind
it.

## Write

A section is one of two kinds, and its last bullet says which. A **future**
promise names the one ROADMAP outcome that will measure it, and carries the
decisions its work item still has to settle. A **shipped** promise names the
STATUS rows that already measure it, and cannot carry those: shipping settled
them. What it carries instead is what it does not do yet.

```
## <title>                                   (future)

- Problem: ...
- Who: ...
- Promise: ...
- Non-goals: ...
- Open: ...                                  (or: Open: none; <why>)
- Neighbours: ...
- Outcome: `<id>`. Tracking: not assigned.   (one line, never wrapped)
```

```
## <title>                                   (shipped)

- Problem: ...
- Who: ...
- Promise: ...                               (or: Promise, for now: — leaving)
- Limits today: ...                          (omit only when there are none)
- Non-goals: ...
- Neighbours: ...
- Status rows: `<id>`, `<id>`
```

Never borrow across the two. An `Open:` on a shipped section is a question
nobody will answer, because no work item is coming for it; a `Limits today:`
on a future one is a limit of something that does not exist. `metadoc.py
check` refuses both by name.

`Promise, for now:` is for a surface that is public today and leaving — the
promise is scoped to the present tense rather than withdrawn, so no new
caller builds on it while the callers it already has keep their contract.

`Tracking: not assigned.` stays on one line because `/aw-grill-meta-to-wis`
finds it with `grep` when it binds the section, and a soft wrap hides it. A
new section is appended at the end of its area, before
`## Non-goals in this area`. Then:

- the directory's own index gains or loses the section's row in
  `## Section index`, and a future section's row in `## Horizons`;
- a new area file starts with an H1 and the paragraph naming its README
  capability id, and ends with `## Non-goals in this area`;
- a `.gitkeep` in the directory goes when the first real file lands.

Then show the change instead of describing it:

```
git -c core.fsmonitor=false diff --stat -- <project>/README.md <project>/STATUS.md <project>/ROADMAP.md <project>/docs
git -c core.fsmonitor=false diff -- <project>/README.md <project>/STATUS.md <project>/ROADMAP.md <project>/docs
```

Hand the human the diff itself. The section is new prose, but the file around
it is a promise somebody already made, and only the diff says which is which.
A deletion has no other evidence at all — the section is gone, and the diff is
the sole record of what it said. Name the path and the section title beside
it, and for a future section say that `/aw-grill-meta-to-wis` opens its work
item next.

## Land it

Three commands, in this order, and none of them is yours to skip. The first
two measure what the run did; the third is the only thing here allowed to
write a commit.

```
uv run --python 3.13 --no-project ".claude/aw/scripts/metadoc.py" check <project>
```

`metadoc.py check` reads and changes nothing. It measures the working tree
against HEAD: every changed path against the four-path allowlist, every
touched section under `docs/**` against its own kind's bullets and their
order, every STATUS row id and ROADMAP outcome id against the file it is
claimed from *as edited*, each touched directory's section index against its
own area files in both directions, and the whole project against the
product-document contract. It also refuses a heading or a `Tracking:` line
that gained an issue number, which is where the `## Never` list below stops
being advice you could read past. Each finding names its rule and its path.

```
uv run --python 3.13 --no-project ".claude/aw/scripts/meta.py" check <project>
```

`meta.py check` is the second half, and it exists in this sequence because
the scope widened. It was reached through a skill of its own, `aw-check-meta`,
until 2026-08-27; that skill is deleted and folding its verb in here was not
tidying. `M1`–`M7` are the rules that
refuse a META-doc fact whose owner is gone — a path that does not resolve, a
gate command naming a cargo target that is not in the checkout, a
self-graded status field — and until this run the three documents they cover
were outside the allowlist, so nothing this skill wrote could break them. Now
everything it writes can. A separate skill somebody remembers to run is not a
gate; a step in the landing sequence is.

Both read and neither writes, so a refusal from either leaves everything
where it is. The file you were not supposed to write is still written, still
uncommitted, and still the human's to decide about — undoing it is not this
skill's call any more than making it was.

Once both are clean, write the commit message to a file: first line the
subject, which has to read `docs(<name>): <what changed>`, then a blank line,
then why the promise changed, in the terms the human used. Then hand it to
the only writer here:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/metadoc.py" commit <project> --why <path>
```

`metadoc.py commit` re-runs every check, stages exactly the allowlist, and
appends the trailer block that makes the commit findable — `Meta-Project:`,
one `Meta-Top: <name>` per top-level document this run touched, one
`Meta-Index: <path>` per directory index it touched, one
`Meta-Section: added|modified|removed <path>#<title>` per section, and
`Meta-Unbound:`, how many of those sections still carry no work item. The
trailers are derived from the diff, not from anything you assert, so they
cannot describe some other commit. `--dry-run` prints the message and stages
nothing.

That block is what `/aw-grill-meta-to-wis` searches for. A META-doc commit
made by hand carries no trailers, so it is a promise the ladder cannot find.

## Never

This addresses the agent running the interview, not the human answering it.

- Never write ` (#<iid>)` into a heading or a `[#<iid>](...)` into a
  `Tracking:` line. Binding is `/aw-grill-meta-to-wis`'s, done in the same
  run that opens the work item, so a section and its number cannot drift
  apart.
- Never write outside the four allowlisted paths of the named project. Not
  `src/**`, not `e2e/**`, not `Cargo.toml`, and not another project's
  documents — the allowlist widened within one project, it did not widen
  across projects.
- Never read or write the tracker here. A section describes a promise, not an
  issue; the work item that will carry the promise does not exist yet, and
  this skill is not where it starts existing.
- Never invent a STATUS row, a ROADMAP outcome, or a README capability so
  that a section's owner bullet resolves. Adding one is now inside the
  allowlist, which makes this the easiest false green available: a row that
  measures nothing turns a refusal into a pass without changing what the
  project actually promises. Every id added is one the human asked for.
- Never soften a ROADMAP completion evidence so the prose reads better.
- Never answer an `Open:` line yourself, and never drop one: an open question
  the section stops carrying is a decision somebody made without saying so.
- Never commit, stash, or branch to reach the clean tree the run needs. A
  tree this skill tidied is a change the human never got to read.
- Never skip `meta.py check` because `metadoc.py check` printed `=> CLEAN`.
  They refuse different things, and the second one is the half that was a
  separate skill precisely because nobody was running it.
- Never run `git add` or `git commit` on the run itself, and never work around
  a refusal by committing the allowed paths by hand. `metadoc.py commit` is
  the writer; a commit made beside it carries none of the trailers the next
  skill reads, and reports a run that was never measured.
- Never delete a bound section, and never rename one.
