---
name: aw:grill-change-to-prd
description: Interview the human through AskUserQuestion until a change work item's product promise is stated as observable behaviour, then write it as one `## <title> (#<iid>)` section of the owning project's `docs/product/<capability-area>.md`. Use after the change is on the tracker and before any of its ladder runs. Writes only under `docs/product/`; never source, tests, or the tracker.
version: 0.1.0
user-invocable: true
---

# /aw-grill-change-to-prd

Grill the human, then write one section. This skill owns exactly one thing:
the product-facing statement of what a change promises, filed under the project
that owns it, in a document organised by capability rather than by ticket. It
never writes product source, never writes the tracker, and never invents an
answer the human did not give.

## No work item, no section

You need one `<iid>`, and it has to be a change. Read it:

```
python3 ".claude/aw/scripts/change.py" show <iid> --json
```

The script accepts only `type:change`, and its refusal names the type it found.
That refusal is the answer, not an error to route around: an epic's product document is
`/aw-grill-epic-to-prd`'s job, and an intent that has no work item yet is `/aw-grill-me-to-change`'s.
Nothing here starts without a tracker body — the section this skill writes is
*about* a work item, and a section about nothing is one no reader can hold
anyone to.

From the payload take three things:

- the owning project, from the `app:<name>` or `lib:<name>` label. The document
  goes under `apps/<name>/docs/product/` or `libs/<name>/docs/product/`. A work
  item with no such label has no home; stop and say so rather than infer one
  from the paths its body mentions.
- the `## Goal` sentence, which already names the trigger, the observation
  point, the current value and the target value. The product section restates
  that difference for a reader who will never open the ticket.
- the `## Acceptance` gate commands, which are the only promises the section
  may make. A promise no gate measures is a promise the ladder cannot refuse.

## Placement

Every product document is a topic, and its name is a capability area from the
project's README `## Capabilities` — not an issue number, not a date. One work
item contributes one `## <title> (#<iid>)` section to the topic it belongs to,
so a reader of `docs/product/<capability-area>.md` sees every promise made in
that area and which work item made each.

List what is already in the project's `docs/product/` and ask with
**AskUserQuestion** which file this section joins, offering each existing topic
and "a new topic" as the options. A new topic needs a name from the README's
capability areas; if none fits, that is a finding about the README, and the
answer is to say so, not to mint an area the README does not have.

## The interview

Ask with **AskUserQuestion**, one round per gap, and stop asking as soon as the
section's own parts are answered. Each part has a consumer that can refuse it,
so an answer that would satisfy no consumer is not an answer yet.

| Part | The question | Refused when |
|---|---|---|
| Problem | what a user of this project cannot do, or gets wrong, today | it restates the title |
| Who | which caller, operator, or client sees the difference | "users" |
| Promise | the observable behaviour, in the words a README capability row would use | no `## Acceptance` gate measures it |
| Non-goals | what a reader would assume is included and is not | empty |
| Neighbours | which sections already in the topic this one extends, narrows, or supersedes | the topic has sections and the answer is "none" |

Do not ask what the body already answers. `## Goal` is the promise's shape and `## Acceptance` is its
measurement; the interview supplies who it is for and what it is not.

## Write

One file, one section, under `docs/product/` of the owning project:

- an existing topic gains a `## <title> (#<iid>)` section at its end;
- a new topic is created with an H1 naming the capability area, then the
  section.

The section carries the five parts above as prose, in that order, and quotes
the `## Acceptance` gate command verbatim under the promise, so a reader can run the thing the
promise claims. Then print the path and the section title to the human.

## Never

This addresses the agent running the interview, not the author of the change.

- Never write outside `docs/product/` of the owning project. Not `src/**`, not
  `e2e/**`, not `docs/technical/**`, not the README whose capability areas the
  topics are named after.
- Never name a file after the work item. `docs/product/<iid>.md` is a ticket
  mirror, and the tracker already is one.
- Never open, edit, close, or comment on a work item here. The tracker is
  `/aw-grill-me-to-change`'s to write.
- Never promise what no gate measures, and never soften a gate the body
  declares so that the prose reads better.
- Never write a section for a work item whose type the script refused, and
  never write one from a body the human pasted in place of the tracker's.
