---
name: wi-ec-review
description: Route a change work item's external-contract change to the semantic reviewer named by apps/<project>/aw.toml, which answers the one question no script can — does this change satisfy the work item. Use after /aw:wi-ec-verify passes and before /aw:wi-ec-commit. Never forms or restates the verdict itself.
version: 0.1.0
user-invocable: true
---

# /aw:wi-ec-review

`/aw:wi-ec-verify` answers "is this change admissible". It cannot answer "does
it satisfy work item `<iid>`", because nothing in that list reads the work
item's requirements against what was written. A change can pass every
mechanical row and pin something the work item never asked for.

This skill routes that second question. It does not answer it, and neither do
you.

## Confirm admissibility first

```
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/ec.py" verify <iid> --baseline <path>
```

If any row is `FAIL`, stop and go back to `/aw:wi-ec-start`. Spending a
reviewer on an inadmissible change buys a semantic opinion about a change that
is going to be rejected mechanically anyway — and `ec.py review-prompt`
refuses it for the same reason, so this is the early version of a refusal you
would hit regardless.

## Route it

The reviewer is not yours to pick. `apps/<project>/aw.toml` names it:

```toml
[review]
ec = "skill:codex-review"
```

Invoke exactly what that key names, passing `<iid>`. If the value is
`skill:<name>`, run `/aw:<name> <iid>`. If the key is absent, stop and say so:
an unrouted project has no reviewer, and choosing one here would make this
skill the authority the TOML was supposed to be.

The `[review]` table is per stage — `ec`, and later `td` and `cb` — so the
reviewer skill is shared across stages and this skill owns only the EC one.

## What the reviewer is given

`ec.py review-prompt <iid>` builds the whole prompt: the rubric, the work
item's body, every path the change touches, and the full source of every case
with its inventory `promise` and `oracle`. Scope is guaranteed by construction
rather than by asking the reviewer to stay in bounds.

The primary question in that rubric is **Q0: does this change satisfy the work
item** — every promised observable pinned, nothing pinned that was not asked
for, and the specific commands and values asserted on exactly. `Q1`–`Q7`
follow as the vacuity screen. The two catch different failures: a case can be
a perfectly discriminating verifier of the wrong thing.

## Report

| verdict | what it means | next |
|---|---|---|
| `accepted` | the reviewer found nothing disqualifying | `/aw:wi-ec-commit <iid>` |
| `rejected` | at least one `FINDING:` line names a defect | quote every finding verbatim, then `/aw:wi-ec-start` |

The verdict lands at `.aw/ec-review/wi-<iid>.json`, bound to a `sha256` over
the work item's body **and** every byte of the change. Editing either
invalidates it — including editing the work item, because the question was a
comparison and changing one side changes the answer.

## Never

This addresses the agent running the review, not the reviewer.

- Never produce, transcribe, summarise, or paraphrase the verdict. It is
  parsed out of the raw transcript by `ec.py verdict` and is not something you
  may be persuaded to restate.
- Never pick, substitute, or improvise the reviewer. Read the route out of
  `aw.toml` and invoke it; a project with no route is a blocker to report, not
  a default to supply.
- Never edit a case, the inventory, or the work-item body in this skill. An
  edit here is an edit the reviewer never saw, which the digest binding refuses
  one step later and with more of your time already spent.
- Never re-run the reviewer because the verdict was `rejected`. A second sample
  is not a correction; if a finding is factually wrong, say so with the
  evidence and let the user decide.
- Never rank the findings, decide which ones matter, or offer your own view on
  whether the reviewer was right. A second opinion from the agent that wanted
  the change to pass is not a second opinion.
