---
name: aw:ask-user
description: Turn every question the current session has left unasked — a stated assumption, a route it picked between two on its own, an `Open:` line in a document it read past, a decision it deferred with "can be reversed later" — into AskUserQuestion rounds, and hand the answers back as a decision table. Use when a session has been deciding on the human's behalf and the human wants those decisions put to them. Asks only; writes nothing.
version: 0.1.0
user-invocable: true
---

# /aw-ask-user

Ask the human what the session has been answering for them. This skill owns
exactly one thing: finding every question that is pending in the current
context and putting each one to the human through **AskUserQuestion**. It
writes no file, edits no tracker, and applies no answer — the session that
invoked it does that, with the answers in hand.

The two grills — `/aw-grill-me-to-meta` and `/aw-grill-meta-to-wis` —
interview against a fixed schema: a section set a script prints, or a
document's fixed parts. This skill is for the questions that have no schema:
the ones a session raised, or should have raised, while doing something else,
and then settled by itself.

## What counts as a pending question

Walk the current context — the conversation, and any plan file, staged
work-item body, or product document it names — and collect every place where
one of these holds:

| Source | What it looks like |
|---|---|
| A stated assumption | "I assume", "assuming", "defaulting to", "I'll take X to mean" — a value the session supplied so it could continue |
| A reversible decision | "can be reversed", "可翻", "待拍板", a `(Recommended)` the human never picked, a table of decisions the session made in the human's place |
| A route chosen alone | two or more approaches were named and the session took one without the human saying which |
| An `Open:` line | a `- Open:` bullet in a `docs/product/` section, or a question left in an HTML comment of a staged body, that no later text settles |
| An interrupted interview | a grill whose remaining sections were never asked because the session stopped, was redirected, or the context was summarised |
| A question answered with a plan | the human asked something and the session replied with what it would do instead of an answer |

An optional argument narrows the walk to one topic — a file, a work item, a
word — and everything outside it is left alone.

Then drop every candidate the context already answers. A question the human
settled three messages ago is not pending; asking it again is the failure
this skill exists to prevent, not a safety margin.

## How to ask

1. List the candidates first, in the order they arose, each with where it
   came from, so the human sees the whole set before answering any of it.
2. Ask with **AskUserQuestion**, in rounds of at most four questions. Give
   each question 2-4 concrete options drawn from the repository or the
   context — the actual paths, commands, names, and values in play — never
   generic placeholders. Where the session had a preference, that option goes
   first and is labelled `(Recommended)`; the human can always answer
   "Other".
3. Ask in dependency order: a question whose answer decides whether another
   question exists goes in an earlier round.
4. Stop when the candidate list is empty. Never invent a question to fill a
   round.

## Hand back

Print one table and nothing else:

```
| # | Question | Answer | Applies to |
|---|---|---|---|
```

`Applies to` names the file, work item, plan step, or section the answer
changes. The session applies it; this skill does not.

## Never

This addresses the agent running the round, not the human answering it.

- Never answer a question yourself, and never treat the option the session
  preferred as the answer because the human did not object to it earlier.
  Silence is not an answer.
- Never ask what the context already answers.
- Never write a file, stage a body, run a tracker verb, or edit a document.
  The table is the whole output.
- Never widen the round past the questions that are actually pending. A
  round padded with questions the session could answer from the repository
  spends the human's attention on nothing.
