---
name: approve-next-step
description: Interpret a clear current assent as approval for one latest pending, well-scoped action and no more.
---

# Approve Next Step

## Goal

Recognize a bounded approval without expanding it.

## How

1. Treat an explicit user invocation or clear current assent as approval only for the latest pending, well-scoped request. A model selecting this skill is not user approval.
2. Keep the target, limit, purpose, and completion condition from that request. An explicit release, paid run, or deletion remains limited to the stated target and limit.
3. Recheck only safety- or correctness-relevant current state. Then execute the exact approved action through the proper route without asking for the same consent again. Stop after that one action.
4. Stop and ask when there is no clear pending item, the assent is ambiguous, or the action has material scope or state change beyond the pending request. Explain the difference before asking.
5. Honor Plan mode, read-only mandates, the blocked-writer policy, and all existing environment and specialist gates.

## Acceptance

The approval authorizes one identified action and leaves all other authority unchanged.

## Never

- Never treat model skill selection alone as authorization.
- Never grant blanket future authority, an environment bypass, or another gate bypass.
- Never create an AW binding, receipt, plan file, schema, or approval hook.
