---
name: aw-ask-user
description: Surface every unresolved product or workflow decision that the current session has been making implicitly, ask the human for each answer, and return one decision table. Read-only; it does not edit files or tracker state.
---

# AW Ask User

## Goal

Turn assumptions, silently selected routes, deferred choices, and unresolved
`Open:` items into explicit human decisions.

## How

1. Review the current conversation and the files already read in this session.
2. Collect only decisions that can change scope, behavior, order, version, or
   acceptance. Treat the default minor Milestone bump returned by
   `milestone.py next-version` as a readable policy, not a decision. A missing
   initial version and any major, patch, or exact version override are
   decisions. Do not ask for other facts that can be read safely.
3. Group dependent questions together. Ask no more than three short questions
   in one round.
4. Use the runtime's native question interface when it is available. Otherwise
   ask one concise plain-text question and wait.
5. Give two or three mutually exclusive choices when the real options are
   known. Put the recommended choice first and state its effect.
6. Continue until every collected decision has an answer or the human declines
   to decide it.
7. Return a table with `Decision`, `Answer`, and `Effect`.

## Acceptance

- Every material decision found in the session appears in the table.
- Each answer is the human's answer. No answer is inferred from silence.
- The skill reports declined or deferred decisions as unresolved.
- No file, Git ref, issue, milestone, or release changes.

## Never

- Never turn a readable fact into a question.
- Never choose a version, milestone order, scope boundary, or acceptance gate
  for the human. This includes an initial version and a major, patch, or exact
  version override.
- Never continue a write workflow while a material decision remains unresolved.
- Never edit the artifact that contains the question.
