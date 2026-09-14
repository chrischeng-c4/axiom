---
name: follow-next-step
description: Continue the latest clear unfinished recommendation after the user asks to do it, while preserving current limits and routing.
---

# Follow Next Step

## Goal

Continue exactly the latest unfinished, clear main recommendation.

## How

1. Accept a clear user request to do the latest recommendation, including plain prose that did not come from `next-step`.
2. Choose the clear main recommendation when an earlier response also listed an alternative. Do not invent a target or replay completed work.
3. Recheck only the targeted current state. Route any real operation to an available specialist skill or agent.
4. Honor Plan mode, a read-only mandate, and the current blocked-writer policy. Stop and ask when there is no clear pending item, the request is ambiguous, or the action has material scope or state change beyond the stated recommendation. Explain the difference before asking.
5. After the operation or targeted check, return one result or one next recommendation. Do not create an endless chain of new recommendations.

## Acceptance

The continuation remains within the one stated target, authority, and completion condition.

## Never

- Never treat skill selection as approval.
- Never bypass an existing gate, permission limit, or specialist boundary.
- Never bind this continuation to AW, a receipt, a plan file, or a new approval framework.
