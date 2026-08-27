---
name: aw:grill-meta-to-wis
description: Measure the gap between a project's META-docs and its open work items with `wis.py gap`, then close what the table shows through `epic.py create|update` and `change.py create|update` — opening a missing epic for an unbound future promise, binding the section to it, reconciling an epic's declared requirements against its actual children, and grilling a change's GHAN body. Interviews the human through AskUserQuestion for every judgement the script cannot make. Use after `/aw-grill-me-to-meta` has landed a promise, or whenever the work-item set is suspect. Writes nothing to `docs/**` except the four-line bind.
version: 0.1.0
user-invocable: true
---

# /aw-grill-meta-to-wis

`wis.py gap` measures the arithmetic: which promise carries no issue number,
which issue no promise reaches, which binding points at an issue that cannot
carry it. Everything past that table is judgement — whether a promise
deserves an epic of its own, whether two open changes are the same change,
whether a child belongs to this epic at all — and this skill is where that
judgement gets made, through **AskUserQuestion**, before `epic.py` or
`change.py` writes anything.

This is the one skill folding together what were three: opening an epic from
an unbound promise, reconciling an epic's children against its declared
scope, and grilling a change body. They shared one write surface — the
tracker, through `epic.py` and `change.py` — and splitting them meant a
promise could sit bound-but-childless because the second skill never ran.
Folding them means the run that finds the gap is the run that closes it.

## The one refusable step

```
uv run --python 3.13 --no-project ".claude/aw/scripts/wis.py" gap <project>
```

Run this before asking anything else, and read every row. This is the only
part of the whole skill a script can refuse — the model's own sense that a
backlog "looks about right" is not a measurement and never becomes one; only
this table is. Seven rows, each printed as `<count> / <population>` beside
the size of what it read, so `0 / 0` (nothing to check) reads differently
from `0 / 12` (checked twelve, all clean):

| Row | Names |
|---|---|
| G1 | a future promise that no epic is opened for |
| G2 | an open work item that no promise reaches |
| G3 | a promise bound to an issue that cannot carry it |
| G4 | a ROADMAP outcome no promise claims |
| G5 | a STATUS row no promise claims |
| G6 | an e2e case the crate manifest does not run |
| G7 | a README gate that names no cargo target |

A row can also print `? / ?  UNMEASURED` with a reason instead of a count —
`gh` failing outside a git directory, a project with no `docs/` tree at all,
a STATUS.md the row has nothing to read. **A run with any UNMEASURED row has
not measured the backlog.** Do not treat its zero-looking rows as clean and
do not close anything on the strength of it; report the reason verbatim and,
if it names a fixable cause (wrong `--repo`, a missing STATUS.md this skill
does not create), fix that and re-run `gap` rather than working around the
gap in your head. Only `=> ALIGNED` — every row measured, every row zero —
means there is nothing here to do.

G1, G2 and G3 are this skill's own write surface: an epic or a change gets
opened or repaired, and a section gets bound. **G4 and G5 are not** — they
name a ROADMAP outcome or a STATUS row no section claims, and the fix is a
section, which is `docs/**` prose outside the one bind edit this skill is
allowed to make. Report them and point at `/aw-grill-me-to-meta`. **G6 and
G7 are not either** — an unregistered `e2e/` case or a README gate naming no
cargo target is a `src/**`, `Cargo.toml`, or `README.md` fix, none of which
this skill touches. Report them by path and stop there.

## Resolve the project

One project, `apps/<name>` or `libs/<name>`. If the invocation names one,
that is the project; otherwise read the conversation for candidates and put
them to the human as the first **AskUserQuestion**, "Other" always open. Do
not guess and proceed — a project inferred and not confirmed is exactly the
mistake `wis.py gap` cannot catch, because it only checks the project it was
told to.

Every write below goes through `epic.py` or `change.py`:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/epic.py" <verb> [args]
uv run --python 3.13 --no-project ".claude/aw/scripts/change.py" <verb> [args]
```

Both walk up from the working directory to the outermost `aw.toml`; run them
from inside the checkout you mean to write against. This file writes the
short forms `epic.py <verb>` and `change.py <verb>` from here on.

## G1 — opening a missing epic

`wis.py gap` names a `docs/**` heading with no ` (#<iid>)` whose section is a
future promise (an `Outcome:` bullet, not a `Status rows:` one — a shipped
section owes nothing here). For each:

**Hard stop first.** An epic is never opened for a promise no `docs/**`
section makes. If the title `wis.py gap` printed does not resolve to a real
`## <title>` heading in the project's `docs/**`, stop and report it as a
defect in the measurement, not a reason to open one anyway.

**Draft, do not decide.** Read the section and seed the epic body from it —
`Problem:` into `## Problem`, the area file's capability id into
`Capability:`, each sentence of `Promise:` into one `R<n>`, `Non-goals:` into
`### Out of Scope`, `Neighbours:` into `### Related Specs`, the ROADMAP
outcome's completion evidence into `## Acceptance Criteria`. Confirm the
draft with the human in one round — drafted from the section is not the same
as answered by it. Every `Open:` line becomes a question in the next round,
never a default.

**Grill the rest.** Run `epic.py skeleton` and treat its output as the
authoritative section set — do not keep a second copy of it here, because a
copy drifts the moment the schema moves. Two sections decide whether the
epic is verifiable at all: every `R<n>` in `## Requirements` must be
observable (reject one no command can ever disagree with), and every
`## Verification Inventory` row must name a command that exists and an
oracle stating what its output looks like when the requirement holds.
`validate` refuses a requirement with no inventory row by name, so grill a
requirement and its gate together; one row may cover several `R<n>` where one
gate genuinely does. Ground every gate in `CLAUDE.md` and the project's
`CONTRIBUTING.md` — never offer a gate the repository does not already run.
Ask with AskUserQuestion, at most four questions a round, 2-4 concrete
options plus "Other". Stop once the skeleton's sections are answered; do not
extend into sequencing or implementation, which is the children's job.

**Write.** Resolve identity before drafting, the same way the change grill
below does: an existing epic whose body is thin or stale means
`epic.py fetch <iid>`, edit the staged copy, `epic.py update <iid>`; nothing on
the tracker yet means author fresh and `epic.py create`. Run `fetch` **before**
writing anything — it overwrites the local staged copy unconditionally, so a
fetch run after you've drafted eats the draft.

Stage the body under `epic.py bodydir` (created if missing; ask
the script rather than rebuilding the path, since `.aw/` is gitignored and
`--body-file` resolves against the current directory). Check it first —
`epic.py validate --body-file <path>` — then whichever the identity was:

```
epic.py create --title "<title>" --project <project> --priority <p0|p1|p2|p3> --body-file <path>
epic.py update <iid> --body-file <path>
```

Title is the section heading, verbatim. `create` fixes the type from the
axis; there is no `--type` flag. Then run `epic.py validate <iid>` against
the live issue. A failure names the offending sections — one more
AskUserQuestion round on exactly those, then update again. Report the epic
only once `validate` passes on the live issue, never on the create exit code
alone: one code says the write landed, the other says the body is
admissible, and they answer different questions.

## Bind the section

Once `validate` passes, and only then — for a G1 epic just opened, or any
other unbound section a human points you at — bind it. This is the one place
this skill edits `docs/**` prose by hand, and it writes exactly these:

1. The heading `## <title>` becomes `## <title> (#<iid>)`. Match the title
   exactly — stripped, case-sensitive, punctuation as written. Already
   carrying this iid: nothing to do. Carrying a different one: stop and
   report both numbers rather than choose.
2. `Tracking: not assigned.` becomes `Tracking: [#<iid>](<url>)`, the url
   from `epic.py show <iid> --json`. Match case-insensitively, tolerate a
   soft wrap, write the result on one line — `wis.py gap` and this skill both
   find it with a plain match, and a wrap hides it.
3. An `Open:` line the body now answers becomes
   `Open: none; settled by #<iid>`. One it does not answer stays, verbatim.
4. In `<project>/ROADMAP.md`, under the `### ` block whose `- ID:` is the
   section's `Outcome:`, `- Tracking: Not assigned.` becomes
   `- Tracking: [#<iid>](<url>)`. A shipped section (ends in
   `Status rows:`) has no ROADMAP block and no outcome to bind — skip it.

Then read-only:

```
uv run --python 3.13 --no-project scripts/meta/project_docs_contract.py check <apps|libs>/<project> --format json
```

and read `ok`. A red here is a bind that landed somewhere the validator did
not expect; the fix is the bind, not the validator.

## G3 — repairing a stale binding

`wis.py gap` also finds the opposite defect: a heading carrying `(#<iid>)`
where `<iid>` does not resolve to an open, live, correctly-typed work item —
closed as something other than what the section claims, wrong type, or
simply gone. Confirm the mismatch with the human — this is a semantic
finding, never applied on the script's say-so alone — then either repoint
the binding at the right iid (steps 1-2 above, against the corrected number)
or, if nothing on the tracker can carry it, treat the section as unbound and
run it through G1.

## G2 and epic-level reconciliation

`wis.py gap` finds an open work item no promise reaches. Two shapes:

- **A change carrying no epic label at all.** Ownership is what
  `epic.py children <iid>` reads, and the label is what makes a child a
  child — an unlabelled issue is not one, whatever its body claims. Read
  what it already carries with `change.py show <iid> --json` first: a change
  the table called orphaned because its title matched no section may still
  carry an epic label, and adding a second one makes it a child of two epics
  rather than moving it. Confirm with the human which epic it belongs to (or
  that it belongs to none), and
  `change.py update <iid> --add-label <epic label>` if one is chosen.
- **An epic whose own children may not match what it declares.** This needs
  its own pass, one level down from the project-wide table: for the epic in
  question, gather

  | Source | Command | What it gives |
  |---|---|---|
  | declared scope | `epic.py show <iid> --json` | `## Requirements` rows and `## Verification Inventory` |
  | owned children | `epic.py children <iid> --json` | the canonical ownership set, state and type |
  | decided findings | `epic.py reconcile <iid> --json` | `structural` and `semantic` arrays, already split |

  Build one table keyed by `R<n>`: requirement → claiming children → their
  state and type. A row with no child, and a child in no row, are findings
  the script could not compute for you.

**Two-tier write authority**, never blurred:

- **Structural — apply, then report.** These are already decided.
  `command` non-null: run it exactly as printed, unedited — it is a complete
  tracker command. `command` null (`not-terminal`, `non-executable-child`):
  a report, not a repair; there is nothing to run. Report every repair with
  the command that made it. Never edit a structural command before running
  it — if it looks wrong, that is a defect in the script and a reason to
  stop, not a reason to improvise a better one.
- **Semantic — confirm, then apply.** Opening, closing, or re-parenting a
  work item, or judging two children equivalent or one out of scope, is
  semantic regardless of how confident the evidence looks:
  - **`possible-coverage-gap`** / **`no-children`** — which requirements have
    no child is the judgement. Propose each missing child's title,
    one-sentence Goal, and the `R<n>` it discharges, all in **one round**
    covering the whole set — coverage is a judgement about the set, and
    asking child-by-child hides whether two proposals are one child wearing
    two names.
  - **`possible-duplicate`** — propose which survives and what happens to
    the other (close as duplicate, or narrow one of the two).
  - A child no requirement covers is misfiled only if the requirement set is
    complete — a human call. Propose re-parenting to a named epic, or
    detaching it.

  - **An epic whose children are all terminal.** That is the epic's own
    terminal state, and `epic.py close <iid>` is the only write that records
    it — the verb refuses and names every child still open, so a premature
    close is refused rather than argued about. Closing is still semantic:
    confirm with the human that the requirement set was complete before
    accepting "every child closed" as "the promise is kept."

  Never bundle a create and a re-parent into one option: one answer must not
  commit two different writes. Never treat `epic.py reconcile` printing zero
  findings as "reconciled" — it only sees what labels and counts can show;
  it cannot see a requirement nobody ever opened a child for.

Land each accepted child one at a time through the change grill below,
confirming `epic.py children <iid>` shows it before starting the next —
`change.py create --epic <iid>` is what attaches the label, and an
interrupted run should leave whole children behind, never fragments.

## Grilling a change body

Whether opened fresh from a reconcile round or repaired because
`change.py validate` refuses it, a change is grilled the same way. The GHAN
schema — `## Goal / ## How / ## Acceptance / ## Never` — predates this
skill; `change.py validate` is its only judge, and a refusal means fix the
body, not reshape it until the validator stops complaining, and never edit
the rule to fit an answer.

Resolve identity first: an iid given and existing means
`change.py fetch <iid>`, edit, `change.py update <iid>`; no iid means author
fresh and `change.py create`. On update, `fetch` **before** writing anything
— it overwrites the local staged copy unconditionally, so a fetch run after
you've drafted eats the draft.

A change's promise is the `docs/**` section its epic is bound to (the
heading carrying `(#<epic iid>)`), or, for a change with no epic, the
shipped section it narrows or repairs. Cite it as the first premise of
`## How`, as `docs/<family>/<file>.md § <title>`.

Run `change.py skeleton` for the authoritative section set. What it checks
is shape; four things only the interview can establish:

1. **The coordinates are real.** A premise cites `file:line` — open it and
   read that line before accepting it.
2. **`current` was measured, not remembered.** Run each gate command now,
   against this checkout, and write down what it printed. Copying a value
   from an issue description or a previous round is the single most
   productive source of false work.
3. **The gate can fail.** `current` and `target` must differ by something
   the named command can actually observe — a filter matching nothing exits
   0 and looks identical to "all passed." Prefer a command with no selector.
4. **The negative control is real.** Name the mutation, require verbatim
   failure output, require a byte-for-byte restore verified by sha256. Ask
   what specifically would be mutated and what red looks like — a control
   nobody can describe concretely is one nobody will run.

Ground every gate in `CLAUDE.md` and the project's `CONTRIBUTING.md`. Run
candidate gate commands yourself during the interview — the point is values
you observed, not values you were told. Stop once the skeleton's sections
are answered; do not extend into implementation.

Stage the body under `change.py bodydir`, check with
`change.py validate --body-file <path>`, then:

```
change.py create --title "<title>" --epic <iid> --project <project> --priority <p0|p1|p2|p3> --body-file <path>
change.py update <iid> --body-file <path>
```

`--epic` attaches the ownership label — the same link `epic.py children`
reads back; omit it only for a change genuinely owned by no epic. Then
`change.py validate <iid>` against the live issue; a failure names the
offending sections for one more AskUserQuestion round. Report the change
only once `validate` passes on the live issue.

## Never

This addresses the agent running this skill, not the human answering it.

- Never open an epic for a promise no `docs/**` section makes, and never
  write that section here — `/aw-grill-me-to-meta` does, before this skill
  starts.
- Never write `docs/**` prose beyond the four-line bind above. G4 and G5 are
  reported, not fixed here.
- Never write `src/**`, `Cargo.toml`, or `README.md`. G6 and G7 are
  reported, not fixed here.
- Never fabricate an answer, a gate command, an oracle, a baseline value, or
  a negative control the human did not supply or confirm.
- Never write a tracker body by hand or reach past `epic.py` / `change.py` to
  the tracker's own CLI; they are the only writers here.
- Never open, close, or re-parent a work item on a semantic finding without
  an explicit human answer; silence is not approval.
- Never promote a semantic finding to structural because it looks obvious,
  and never edit a structural command before running it.
- Never change a work item's type in place — the closed enum converges by
  spawn-and-link.
- Never report an epic or a change as authored on a create/update exit code
  alone; `validate` passing on the live issue is the signal.
- Never treat an UNMEASURED row, or `epic.py reconcile` printing zero
  findings, as clean. Both mean "not measured," not "nothing to do."
