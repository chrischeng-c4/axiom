---
name: aw:go-tdd-for-epic
description: Drive an epic's children through the e2e → unit → logic ladder in the order the epic's own verification inventory declares, one child at a time through go-tdd-for-change. This skill computes nothing itself — it asks `epic.py order` for the sequence, and refuses to run when the script prints no sequence.
version: 0.1.0
user-invocable: true
---

# /aw-go-tdd-for-epic

An epic is terminal when every owned child is terminal, and the children are
not independent: the epic's `## Verification Inventory` partially orders the
requirements through its `Depends On` column, and `## Child Work Items` maps
each child to the requirements it covers. The ladder runs on one change at a
time, so the only decision left is which child goes next — and this skill does
not make that decision either. The script does.

## Run it

You were given one `<iid>`. Ask for its order:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/epic.py" order <iid> --open-only
```

On a change work item `epic.py order` refuses, and the refusal names the type
it actually found. That message is the answer, not an error to route around —
a change is `/aw-go-tdd-for-change`'s job; run it there directly. Do not
branch on the exit code: a non-epic and an epic with no computable order both
exit 1, and only the text tells them apart.

`order` prints the children in the sequence they have to run, one rank per
line, each with the requirements it covers and the children it waits on. A
child inherits the position of the *deepest* requirement it covers, and equal
positions break by `priority:` then by issue number.

Run each child through `/aw-go-tdd-for-change`, in that order, to completion —
its three phase commits landed — before starting the next. A child whose
ladder went red stops the epic where it stands: the children after it wait on
it by the epic's own declaration, and running them anyway runs them against a
tree the graph said is not ready.

Three lines mean there is no order to follow, and each one stops you:

| Line | What it means |
|---|---|
| `! no order: the requirement graph closes a cycle` | no sequence satisfies the graph; nothing is printed to run |
| `! R<n> depends on <cell>, which names no requirement` | a dependency was declared that the order could not read, so the order below it is missing an edge |
| `? #<n> has no position` | an owned child the child table maps to no requirement |

Report these to the user and stop. Do not pick an order yourself. An epic body
you edited to make the graph parse is an epic whose dependencies you decided,
and the sequence would then be yours rather than the author's.

## Report

| Outcome | What to say |
|---|---|
| every child's ladder closed | name each child and its three commits, in the order they ran |
| a child's ladder went red | name the child, quote its FAIL rows verbatim, and list the children that were still waiting on it |
| the epic had no order | quote the `!` or `?` lines and stop |

A refusal — the script's or a phase's — is doing its job. Say what it refused
and why, in its own words.

## Never

This addresses the agent driving the epic, not the author of the epic.

- Never decide an epic's order yourself when the graph does not yield one, and
  never edit the epic body to make it parse.
- Never start a child before the one ahead of it has landed its `logic`
  commit. "To completion" is the commit, not a green `test`.
- Never run two children's ladders at once in one worktree. Each phase
  measures its red against HEAD, and a second writer moves HEAD under the
  first.
- Never open, close, or re-parent a child here. The child set is
  `/aw-grill-epic-to-changes`'s to repair; this skill runs the set it was
  given.
