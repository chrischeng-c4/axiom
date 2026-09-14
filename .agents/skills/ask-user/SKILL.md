---
name: ask-user
description: Legacy AW decision interview for an explicitly requested scope. It exposes material choices and changes no project state.
---

# Legacy AW Ask User

Use this skill only when the human explicitly asks for a legacy AW decision interview or invokes `$ask-user`.

## Goal

Make material product or workflow choices explicit without choosing them for the human.

## How

1. Collect only choices that change scope, behavior, order, authorization, or acceptance.
2. Ask short grouped questions. Offer known choices and a recommendation when useful.
3. Return each answer, effect, and unresolved choice. The controller stops a write flow that depends on an unresolved material decision.

## Never

- Never invoke this skill implicitly.
- Never convert a readable fact into a question.
- Never infer an answer from silence.
- Never edit files, run lifecycle actions, or mutate Git, tracker, or releases.
