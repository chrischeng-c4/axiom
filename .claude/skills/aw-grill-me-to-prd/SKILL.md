---
name: aw:grill-me-to-prd
description: Interview the human through AskUserQuestion until a product promise is stated as observable behaviour that a STATUS row or a ROADMAP outcome owns, then create, modify, or delete one `## <title>` section — or a whole capability area — of the owning project's `docs/product/`. Use before any epic exists for the promise; `/aw-grill-me-to-epic` carves the epic from the section afterwards. Writes only under `docs/product/`; never source, tests, STATUS, ROADMAP, or the tracker. Requires a clean `docs/` tree, and lands the run through `prd.py check` and `prd.py commit` so the commit is searchable afterwards.
version: 0.1.0
user-invocable: true
---

# /aw-grill-me-to-prd

Grill the human, then write the promise down before anything tracks it. This
skill owns exactly one thing: the product requirements of one project under
`<project>/docs/product/` — a section per promise, a file per capability
area, an index that says who reads it and in what order the promises land.
It writes no product source, no test, no STATUS or ROADMAP row, and no work
item. The work item comes *after*: `/aw-grill-me-to-epic` reads an unbound
section here, opens the epic under the section's title, and stamps the
heading with the number. Until then a section carries no `(#<iid>)`, and
this skill never writes one.

## No STATUS and ROADMAP, no PRD

You need one project — `apps/<name>` or `libs/<name>`. If the invocation
names one, that is the project. If it does not, read the conversation for
candidates — the projects whose files were opened, whose names were said,
whose promise the request is plainly about — and put them to the human as
the first **AskUserQuestion**, the likeliest first, filled out from the
`apps/*` and `libs/*` directories that carry both a `STATUS.md` and a
`ROADMAP.md`; "Other" lets the human type one the conversation never
mentioned. What the context decides is which options are offered and in what
order, never which one is true: a project inferred and not confirmed is a
guess, and this skill writes nothing on a guess.
Before asking anything else, check that both `<project>/STATUS.md` and
`<project>/ROADMAP.md` are tracked. A promise is owned by exactly one of
them: a shipped promise by the STATUS rows that measure it, a future promise
by the ROADMAP outcome that will. A project with neither has nothing for a
section to stand on. Stop and say which file is missing; the repository
`CONTRIBUTING.md` chapter "Meta-doc content contract" — its adopted-set
table and the `STATUS.md` / `ROADMAP.md` shapes below it — says what each
must contain, and `/project-readme-check` is the gate on both. Writing them
is not this skill's job.

Then read what is there: the README `## Capabilities` index (its capability
ids are what an area file's first paragraph names), every STATUS row id,
every ROADMAP outcome id, and everything already under `docs/product/`.
Never ask for what those already answer.

## A clean tree, or there is no diff

This skill is usually run against a project that already promises things, and
what the human has to read afterwards is not the file — it is what changed in
it. Git is what separates the two, and it can only do that while the change
stands alone. Before the first question, run

```
git -c core.fsmonitor=false status --short -- <project>/docs
```

and require it to print nothing. The `-c core.fsmonitor=false` is not
decoration: this checkout enables the file-system monitor, and a stalled
daemon hangs every command that reads the index, indefinitely and without an
error. If the listing is not empty, stop and print it. Somebody's uncommitted
PRD or TD edit is already there, and an interview that writes on top of it
produces one diff nobody can split back into two changes. Committing,
stashing, or declaring this run a continuation of that work is the human's
call, and this skill makes none of the three.

The same cleanliness is what the work item needs later: the ladder's first
phase refuses a dirty tree outright, and `docs/` is outside every phase's
write root, so a PRD written and left uncommitted blocks the very item it
exists for. Landing this run before the work starts is the point, not a
courtesy.

## Three modes

| The human wants | Mode | Precondition |
|---|---|---|
| a PRD where there is none | **create** | `docs/product/README.md` does not exist |
| a section added, a section's part changed, a new area, the index changed | **modify** | the index exists |
| a section or an area removed | **delete** | the section heading carries no ` (#` |

Resolve the mode before the first question; the request usually states it,
and the directory listing settles the rest. In **create**, the index comes
first — positioning, who reads it, the horizon order — and sections follow.
In **modify**, a bound section (one whose heading carries ` (#<iid>)`) keeps
its title: the title *is* the epic's title, so renaming it is
`/aw-grill-me-to-epic`'s update path first and this skill second. Every other
part of a bound section may change. In **delete**, a bound section is
refused: its epic is still open on it. Say so and stop; the tracker side
ends first — closed, or not planned — and only then does the section go.
Deleting an area file removes its rows from the index in the same edit.

## Placement

- One file per capability area, named for the area — a slug a reader of the
  README would recognise — never for a work item, a date, or a horizon. The
  file's first paragraph names the README capability id or ids it covers, so
  the README and the PRD agree on what an area is without the file name
  having to be the id.
- `docs/product/README.md` is the index: `## How this directory is
  organised`, `## Positioning`, a reader table under `## Who <project> is
  for`, `## Horizons` (`| Horizon | Outcome | Section |`, ordered by
  dependency and saying why), and `## Section index`
  (`| Section | File | Kind | Owner |`, where Kind is `shipped`,
  `shipped, limited`, `shipped, leaving`, or `outcome`, and Owner is
  `STATUS <ids>` or `ROADMAP <id>`).
- A new area is a change to the index first, then the file. Ask with
  **AskUserQuestion** which existing area a new section joins, offering each
  area and "a new area" as the options. A new area needs a README capability
  to stand on; if none fits, that is a finding about the README to report,
  not an area to mint.
- Non-goals are not sections. Each area file ends with
  `## Non-goals in this area`, listing what a reader of that area would
  otherwise assume and pointing at the ROADMAP entry that gives the reason.

## The interview

Ask with **AskUserQuestion**, in rounds of at most four questions, with 2-4
concrete options per question drawn from STATUS, ROADMAP, and the README —
the human can always answer "Other". Stop as soon as the section's own parts
are answered. Each part has a consumer that can refuse it — the epic grill
that will read the section, or the reader the index names — so an answer
that satisfies no consumer is not an answer yet.

| Part | The question | Refused when |
|---|---|---|
| Problem | what a user of this project cannot do, or gets wrong, today | it restates the title, or is under twenty words — `/aw-grill-me-to-epic` drafts its `## Problem` from this |
| Who | which publisher, subscriber, operator, caller, or client sees the difference | "users" |
| Promise | the observable behaviour, one sentence per thing a gate could refuse | it names behaviour no STATUS row or ROADMAP outcome owns |
| Limits today | *(shipped only)* what the promise does not do yet, in the terms the next section will call its `Problem:` | it repeats a non-goal — a limit is something the project intends to close, a non-goal is not |
| Non-goals | what a reader would assume is included and is not | empty |
| Open | *(future only)* the decisions the epic will have to settle, each as a question | a default dressed as a question, or an answer the human already gave |
| Neighbours | which sections this one extends, narrows, repairs, or supersedes, in this file or another | the area has sections and the answer is "none" |
| Owner | the STATUS row ids (shipped) or the one ROADMAP outcome id (future) | `grep` does not find the id in the file it is claimed from |

`Open` is where this skill is most tempted to help, and it may not: an
`Open:` line is a question the epic body or the human answers later, and
writing the answer here is answering for them.

## Write

A section is one of two kinds, and its last bullet says which. A **future**
promise names the one ROADMAP outcome that will measure it, and carries the
decisions its epic still has to settle. A **shipped** promise names the
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
nobody will answer, because no epic is coming for it; a `Limits today:` on a
future one is a limit of something that does not exist. `prd.py check`
refuses both by name.

`Promise, for now:` is for a surface that is public today and leaving — the
promise is scoped to the present tense rather than withdrawn, so no new
caller builds on it while the callers it already has keep their contract.

`Tracking: not assigned.` stays on one line because `/aw-grill-me-to-epic`
finds it with `grep` when it binds the section, and a soft wrap hides it. A
new section is appended at the end of its area, before
`## Non-goals in this area`. Then:

- the index gains or loses the section's row in `## Section index`, and a
  future section's row in `## Horizons`;
- a new area file starts with an H1 and the paragraph naming its README
  capability id, and ends with `## Non-goals in this area`;
- the `.gitkeep` under `docs/product/` goes when the first real file lands.

Then show the change instead of describing it:

```
git -c core.fsmonitor=false diff --stat -- <project>/docs/product
git -c core.fsmonitor=false diff -- <project>/docs/product
```

Hand the human the diff itself. The section is new prose, but the file around
it is a promise somebody already made, and only the diff says which is which.
A deletion has no other evidence at all — the section is gone, and the diff is
the sole record of what it said. Name the path and the section title beside
it, and for a future section say that `/aw-grill-me-to-epic` opens its epic
next.

## Land it

Two verbs, and neither is yours to skip. The first measures what the run did;
the second is the only thing here allowed to write a commit.

```
uv run --python 3.13 --no-project ".claude/aw/scripts/prd.py" check <project>
```

`prd.py check` reads and changes nothing. It measures the working tree against
HEAD: every changed path against the single allowlist `<project>/docs/product/`,
every touched section against its own kind's bullets and their order, every STATUS
row id and ROADMAP outcome id against the file it is claimed from, the section
index against the area files in both directions, and the whole project against
the product-document contract. It also refuses a heading or a `Tracking:` line
that gained an issue number, which is where the `## Never` list below stops
being advice you could read past. Each finding names its rule and its path.

A refusal leaves everything where it is. The file you were not supposed to
write is still written, still uncommitted, and still the human's to decide
about — undoing it is not this skill's call any more than making it was.

Once it prints `=> CLEAN`, write the commit message to a file: first line the
subject, which has to read `docs(<name>): <what changed>`, then a blank line,
then why the promise changed, in the terms the human used. Then hand it to the
only writer here:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/prd.py" commit <project> --why <path>
```

`prd.py commit` re-runs every check, stages exactly the allowlist, and appends
the trailer block that makes the commit findable — `PRD-Project:`, one
`PRD-Section: added|modified|removed <path>#<title>` line per section this run
touched, and `PRD-Unbound:`, how many of those still carry no work item. The
trailers are derived from the diff, not from anything you assert, so they
cannot describe some other commit. `--dry-run` prints the message and stages
nothing.

That block is what the skill after this one — `aw-grill-prd-to-wi`, which
turns written promises into an epic or a change — searches for. A PRD commit
made by hand carries no trailers, so it is a promise the ladder cannot find.

## Never

This addresses the agent running the interview, not the human answering it.

- Never write ` (#<iid>)` into a heading or a `[#<iid>](...)` into a
  `Tracking:` line. Binding is `/aw-grill-me-to-epic`'s, done in the same run
  that opens the epic, so a section and its number cannot drift apart.
- Never write outside `docs/product/` of the named project. Not `src/**`,
  not `e2e/**`, not `docs/technical/**`, not `STATUS.md`, not `ROADMAP.md`,
  not the README whose capability ids the areas stand on.
- Never read or write the tracker here. A section describes a promise, not an
  issue; the work item that will carry the promise does not exist yet, and
  this skill is not where it starts existing.
- Never promise what no STATUS row or ROADMAP outcome owns, and never soften
  a ROADMAP completion evidence so the prose reads better.
- Never answer an `Open:` line yourself, and never drop one: an open question
  the section stops carrying is a decision somebody made without saying so.
- Never commit, stash, or branch to reach the clean tree the run needs. A
  tree this skill tidied is a change the human never got to read.
- Never run `git add` or `git commit` on the run itself, and never work around
  a refusal by committing the allowed paths by hand. `prd.py commit` is the
  writer; a commit made beside it carries none of the trailers the next skill
  reads, and reports a run that was never measured.
- Never delete a bound section, and never rename one.
