---
name: review
description: Legacy AW read-only project review when explicitly requested. An independent worker reports concise findings; it does not change project state.
---

# Legacy AW Review

Use this skill only when the human explicitly requests a legacy AW project review or invokes `$review`. It is not part of ordinary delivery.

## Goal

Return a concise independent findings report without controller inspection of product source, documents, or raw logs.

## How

1. Delegate the read-only project review to a fresh QA instance or other appropriately scoped worker.
2. The worker may inspect the project scope, declared gates, and legacy AW evidence. It returns only findings, affected paths or commands, observed result, and expected result.
3. The controller reads the summary and keeps scope, Git, tracker, and final acceptance authority.

## Never

- Never invoke this skill implicitly.
- Never edit, commit, push, or mutate tracker or release state.
- Never make a clean finding without running the relevant check in that worker.
- Never require a new reviewer role, durable receipt, or full up-front plan.
