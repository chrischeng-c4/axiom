---
name: wi-epic-reconcile
description: Reconcile an epic's declared scope against its actual child work items — missing children get opened, duplicates and misfiled children get resolved. Structural findings are applied automatically; semantic findings are confirmed with the human through AskUserQuestion first. Use before driving an epic to its terminal state, or whenever an epic's child set is suspect.
version: 0.1.0
user-invocable: true
---

# /aw:wi-epic-reconcile

An epic is terminal when every owned child is terminal — which is only
meaningful if the child set is actually right. This skill answers four
questions about one epic and repairs what it finds:

1. Is anything the epic promises **not opened as a child at all**?
2. Is any child **missing** from the epic's ownership (opened, but not
   owned)?
3. Are two children **duplicates** of the same promise?
4. Is any child **not this epic's work** at all?

The epic surface is the prototype bundled at this plugin's root. Invoke it as

```
python3 "plugins/aw/scripts/epic.py" <verb> [args]
```

Note the path: **one** copy of that script exists, and it sits beside the
skills rather than inside any one of them, because all three run it. Two skills
in this tree already carry same-named copies of a shared helper whose contents
have since diverged, and reconciling an epic against a stale second copy of the
epic schema is exactly the failure this skill exists to catch. If that path
does not exist the plugin is not loaded; the same script is in the checkout at
`plugins/aw/scripts/epic.py`.

The script finds the repository from your **working directory**, walking up to
the outermost `aw.toml` — never from its own location, which may or may not sit
inside a checkout — so run it from inside the checkout you mean to write
against. For readability this skill writes the short form `epic.py <verb>` from
here on.

## Gather

Never reconcile from the body alone, and never from the computed findings
alone — the gaps live exactly where the two disagree.

| Source | Command | What it gives |
|---|---|---|
| declared scope | `epic.py show <iid> --json` | the body: `## Requirements` rows and the `## Verification Inventory` table |
| owned children | `epic.py children <iid> --json` | the canonical ownership set, with each child's state and type |
| decided findings | `epic.py reconcile <iid> --json` | `structural` and `semantic` arrays, already split by who may decide them |

Build one table keyed by `R<n>`: declared requirement → the children that
claim it → their state and type. Every row with no child, and every child in
no row, is a finding the script could not compute for you.

## Two-tier write authority

Findings split by who can decide them, and `epic.py reconcile --json` has
already done the split. Apply the tiers in order; never promote a semantic
finding to structural because it looked obvious.

### Structural — apply, then report

Structural findings are applied without asking. These are the findings the
script itself already decided; the agent is transcribing a computed answer,
not making a judgement.

- A structural finding carrying a `command` is applied by running exactly
  that string, unedited. It is a complete tracker command, already carrying
  the right repository and label.
- A structural finding whose `command` is `null` is a report, not a repair —
  `not-terminal` and `non-executable-child` name a condition the human or a
  child work item has to clear, and there is nothing to run.

Report every structural repair with the command that made it.

### Semantic — confirm, then apply

Semantic findings are confirmed with the human before any work-item is
written. Anything that opens, closes, or re-parents a work item, or that
judges two children equivalent or one child out of scope, is semantic —
regardless of how confident the evidence looks. The script emits these
without a command precisely because it cannot decide them:

- **`possible-coverage-gap`** — the requirement count and the child count
  disagree. *Which* requirements have no child is the judgement. Propose the
  missing child's title, its one-sentence Goal, and which `R<n>` it
  discharges.
- **`no-children`** — the epic declares requirements and owns nothing at all.
- **`possible-duplicate`** — two children whose titles overlap. Propose which
  one survives and what happens to the other (close as duplicate, or narrow
  one of the two).

A child whose work no requirement covers is the same class of finding: it is
misfiled only if a human says the requirement set is complete. Propose
re-parenting to a named epic, or detaching it.

Ask with **AskUserQuestion** — one decision per question, at most four
questions per round, each with 2-4 concrete options plus the human's own
"Other". Never bundle a create and a re-parent into one option: a single
answer must not commit two different writes.

### Opening the children

`epic.py` exposes no child-creating verb — it owns the epic, not the axis — and
this skill does not open one either. A new child is a `type=change` work item,
and authoring one is `/aw:wi-change-grill`'s whole job: it interviews for the
body and writes it through its own facade. Reconcile decides **which** children
should exist; the grill decides what each one says.

Two rounds, in this order, and the order carries the reasoning:

1. **Settle the whole set first, in one AskUserQuestion round.** Propose every
   missing child at once — title, one-sentence Goal, and the `R<n>` it
   discharges — and let the human accept, reject, or merge them together.
   Coverage is a judgement about the *set*: whether it is complete, and whether
   two proposals are one child wearing two names. Asking child-by-child hides
   exactly that, because each answer is given without the others in view.

2. **Then grill and land them one at a time.** For each accepted child, hand
   `/aw:wi-change-grill` the epic's iid, the title, the Goal sentence, the
   `R<n>` it discharges, and the project. Let that child land before starting
   the next one. A batch of half-authored bodies is a set of work items nobody
   can validate, and an interrupted reconcile should leave whole children
   behind rather than fragments.

The grill's `change.py create --epic <iid>` attaches the ownership label, which
is the same link `epic.py children <iid>` reads back. Run that after each child
lands and confirm it appears: the label is what makes a child a child, and an
unlabelled issue is not one no matter what its body says.

What the child's body must contain is not restated here. `change.py skeleton`
emits it and `change.py validate` refuses it; a summary in this file would be a
second reading — correct on the day it is written and silently wrong
afterwards.

## Report

Close with the reconciled picture: every requirement, its child, and its
state; then the structural repairs applied, the semantic decisions the human
made, and anything still unresolved. If `epic.py close <iid>` is now expected
to pass, say so — but run it only if the user asked for it. That verb refuses
while any owned child is open, and naming the blocking children is its answer,
not an error.

## Never

This addresses the agent reconciling the epic, not the human answering it.

- Never open, close, or re-parent a work item on a semantic finding without
  an explicit human answer; silence is not approval, and a declined option is
  not a mandate to pick the next one.
- Never treat `epic.py reconcile` reporting zero findings as "the epic is
  reconciled" — the script only decides what is computable from labels and
  counts. It cannot see a requirement nobody ever opened a child for.
  Coverage is decided against the epic body, by a human.
- Never edit a structural finding's command before running it. If it looks
  wrong, that is a defect in the script and a reason to stop, not a reason to
  improvise a better command.
- Never author a child's body here, and never open one by calling the tracker
  CLI directly. Both are how a work item gets written that no validator has
  ever seen; the grill exists so that every change body passes
  `change.py validate` before it is reported.
- Never change a work item's type in place, and never edit any `src/**` path
  by hand.
- Never rewrite the epic's own body here; a thin or unvalidatable epic goes
  back to `/aw:wi-epic-grill`.
