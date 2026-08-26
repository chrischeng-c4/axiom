---
name: aw:grill-change-to-td
description: Interview the human through AskUserQuestion until a change work item's technical design — verified premises, change points, interfaces, the e2e case that will judge it, and the decisions behind it — is written as one `## <title> (#<iid>)` section of the owning project's `docs/technical/<subsystem>.md`, with an ADR under `docs/technical/adr/` for each decision that outlives the change. Use after the change is on the tracker and before its ladder starts. Writes only under `docs/technical/`; never source, tests, or the tracker.
version: 0.1.0
user-invocable: true
---

# /aw-grill-change-to-td

Grill the human, then write one section, and one ADR per decision that
constrains more than this change. This skill owns exactly one thing: the
technical design of a change — what it stands on, what it may touch, what a
caller sees change, and which black-box case will refuse HEAD and accept the
result — filed under the project that owns it, in a document organised by
subsystem rather than by ticket. It never writes product source, never writes
the e2e case itself, never writes the tracker, and never invents an answer the
human did not give.

## No work item, no section

You need one `<iid>`, and it has to be a change. Read it:

```
python3 ".claude/aw/scripts/change.py" show <iid> --json
```

The script accepts only `type:change`, and its refusal names the type it found.
That refusal is the answer, not an error to route around: an epic's technical design is
`/aw-grill-epic-to-td`'s job, and an intent that has no work item yet is `/aw-grill-me-to-change`'s.
Nothing here starts without a tracker body — the section this skill writes is
*about* a work item, and a design for nothing is one no ladder can refuse.

From the payload take three things:

- the owning project, from the `app:<name>` or `lib:<name>` label. The document
  goes under `apps/<name>/docs/technical/` or `libs/<name>/docs/technical/`. A
  work item with no such label has no home; stop and say so rather than infer
  one from the paths its body mentions.
- `### Verified premises` and `### Change points` under `## How`, which are
  the design's footing and its write allowlist. The section restates them for
  a reader of the subsystem, and it may not widen either.
- `### Frozen decisions` and the `## Acceptance` table, which fix what was
  already decided and how the result is measured. A decision frozen there is
  restated, not reopened.

## Placement

Every technical document is a topic, and its name is a subsystem — a module or
directory under the project's `src/`, in the words the source uses — not an
issue number, not a date. One work item contributes one `## <title> (#<iid>)`
section to the topic it belongs to, so a reader of
`docs/technical/<subsystem>.md` sees every design decision made in that
subsystem and which work item made each.

ADRs are the exception to one-section-per-item: each is its own file,
`docs/technical/adr/NNNN-<slug>.md`, where `NNNN` is one more than the highest
number already in that project's `adr/` directory, four digits, starting at
`0001`. Numbers are per project and never reused, and a superseding decision
is a new file whose `Status:` line names the one it supersedes.

List what is already in the project's `docs/technical/` and ask with
**AskUserQuestion** which file this section joins, offering each existing topic
and "a new topic" as the options. A new topic takes its name from `src/`; if
the change touches no subsystem that has a name there, that is a finding about
the design, and the answer is to say so, not to file it under a name the source
does not use.

## The interview

Ask with **AskUserQuestion**, one round per gap, and stop asking as soon as the
section's own parts are answered. Each part has a consumer that can refuse it,
so an answer that would satisfy no consumer is not an answer yet.

| Part | The question | Refused when |
|---|---|---|
| Premises | which of `### Verified premises` still hold — open each file | a coordinate you did not open the file to confirm |
| Change points | anything the implementation must touch that `### Change points` does not list | a path outside `src/` and `e2e/`, or one the body does not allow |
| Interfaces | what a caller sees change — a type, a wire shape, a CLI flag, a config key, a label | "internal only" while the work item names an observable difference |
| The e2e case | the name of the case under `e2e/` that will refuse HEAD and accept the change | a case that is already green against HEAD |
| Decisions | each point where more than one design was viable, and why this one | an alternative nobody could have taken |

For every decision, ask one more question: does it constrain changes beyond
this one? If it does, it is an ADR; if it only explains this change, it is a
line in the section. Decisions the body already froze in `### Frozen decisions` are
not re-decided here; they are restated, and one becomes an ADR only if it
constrains changes after this one.

Do not ask what the body already answers. `### Verified premises`, `### Change points` and `### Frozen
decisions` are the design's shape and `## Acceptance` is its measurement;
the interview supplies the interfaces, the case, and the decisions still
open.

## Write

One file, one section, under `docs/technical/` of the owning project — and one
ADR file per decision the interview marked as one:

- an existing topic gains a `## <title> (#<iid>)` section at its end;
- a new topic is created with an H1 naming the subsystem, then the section;
- an ADR is `docs/technical/adr/NNNN-<slug>.md`, in the shape the repository
  already uses: `# ADR NNNN — <title> (#<iid>)`, a `Status:` line, then
  `## Context`, `## Decision`, `## Consequences`, `## Status of work`.

The section carries the five parts above as prose, in that order — premises as
`file:line` coordinates, change points as paths, the e2e case by name — and
links each ADR it produced. Then print every path and section title to the
human.

## Never

This addresses the agent running the interview, not the author of the change.

- Never write outside `docs/technical/` of the owning project. Not `src/**`,
  not `e2e/**` — the case this section names is `/aw-go-tdd-for-change`'s
  e2e phase to write, red first — and not `docs/product/**`.
- Never write into `tech-design/` or `external-contracts/`. Those trees are
  retired, and `docs/technical/` is a different path, not their revival.
- Never name a file after the work item. `docs/technical/<iid>.md` is a ticket
  mirror, and the tracker already is one.
- Never open, edit, close, or comment on a work item here. The tracker is
  `/aw-grill-me-to-change`'s to write.
- Never record a premise you did not observe, and never edit or renumber an
  ADR that already exists.
- Never widen `### Change points`. A path the design needs that the body does
  not allow is a finding for `/aw-grill-me-to-change`, which updates the body;
  a section that lists it anyway is one the ladder's scope gate will
  contradict.
