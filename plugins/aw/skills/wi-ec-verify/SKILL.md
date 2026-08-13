---
name: wi-ec-verify
description: Run the closed mechanical admissibility list over a change work item's whole external-contract change — scope, at least one case, registration, the previously-green roster, red at a declared assertion, and no self-satisfying assertion. Use after /aw:wi-ec-start and before /aw:wi-ec-review. Writes nothing and touches git not at all.
version: 0.1.0
user-invocable: true
---

# /aw:wi-ec-verify

This skill decides nothing. It runs one verb, reads its exit code, and reports
what the rows say.

```
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/ec.py" verify <iid> --baseline <path>
```

The interpreter pin is load-bearing — `ec.py` reads TOML and `tomllib` is
3.11+, while a bare `python3` is 3.9 on at least one machine this runs on. If
`${CLAUDE_PLUGIN_ROOT}` does not resolve, the plugin is not loaded; the script
is in the checkout at `plugins/aw/scripts/ec.py`.

`--baseline` names the JSON roster of cases that must stay green. Pass it. The
suite as a whole is not green and has not been for a while, so "all cases
pass" is not a claim anyone can gate on; the roster is the version of that
claim that can hold.

## What it measures, and what it cannot

The change is **the diff against `HEAD`** — every path `git status` reports,
which is exactly what `/aw:wi-ec-start` made readable by refusing to open the
leg over a dirty tree. Nothing is remembered between verbs; this one recomputes
the whole ladder.

| row | refuses |
|---|---|
| `P1 work item` | no staged body at `.aw/workitems/changes/<iid>.md`, or one that fails validation |
| `P3 leg is open` | an `ec(...)` commit already carries `Refs #<iid>` |
| `C0 scope` | an empty diff, or any changed path outside `apps/<project>/external-contracts/` |
| `C0b contract present` | a change that touches the inventory and no case |
| `C1`–`C6`, per case | registration disagreement, a broken green roster, not red, red somewhere it never declared, or a self-satisfying assertion |

Read every row. Rows marked `PENDING` are named on purpose: a slot that is not
wired yet and a slot that passes vacuously produce the same green, so the ones
that are not wired say so out loud, here and in the eventual commit message.

What this cannot tell you is whether the change **satisfies the work item**.
Nothing in the list above reads `<iid>`'s requirements against what was
written; a change can pass all of it and pin something the work item never
asked for. That question is `/aw:wi-ec-review`, and admissible is not the same
as correct.

## The loop

| outcome | what you do |
|---|---|
| exit 0 | report the rows, then `/aw:wi-ec-review <iid>` |
| a `FAIL` row | go back to `/aw:wi-ec-start` and fix the cases; re-run this |

Re-running after a fix is a full re-run, not a resume. Nothing is cached, and
a check that passed on the previous attempt is measured again against the
current bytes.

## Never

This addresses the agent running the verification, not the human who asked for it.

- Never edit a case, the inventory, or the work-item body in this skill. This
  verb exists to produce a reading, and an author who edits between the reading
  and the report has produced a reading of something else.
- Never run `git add`, `git commit`, or any other git write. This leg touches
  git not at all, and its exit code means nothing about what is committed.
- Never present a `FAIL` row as a limitation of the checker. Every row names
  the shape it refuses; if you believe the refusal is wrong, say which row and
  why, and stop — do not proceed to review.
- Never report this leg as passed from an earlier attempt's output, and never
  re-run a single case with `ec.py check --case` to get a second, more
  favourable reading of a row this verb marked `FAIL`.
