---
name: grill-release
description: Legacy AW release-plan preparation and application for an explicitly requested plan. Controller-only; use product-plan for ordinary product planning.
---

# Legacy AW Grill Release

Use this skill only when the human explicitly requests the legacy AW release plan flow or invokes `$grill-release`.

## Goal

Prepare or apply one explicitly authorized legacy AW release plan without inventing product decisions.

## How

1. Delegate product-document and technical reading to the owning PM and TL. They return short summaries and proposed bytes; the controller does not inspect product source, documents, or raw logs.
2. The controller may validate a plan read-only. It may apply only the exact plan digest the human explicitly approved.
3. The controller alone runs AW commands that write Git, tracker, or release plan state. Workers have no Git or tracker authority.
4. Report the applied scope, durable result, and any unresolved drift. Do not start another project or resume a partial operation without explicit scope.

## Never

- Never invoke this skill implicitly.
- Never treat plan validation as write approval.
- Never add a receipt, approval step, or design checkpoint beyond legacy AW.
- Never commit, push, publish, close, or change a tracker item without the controller's explicit authority and the human's required authorization.
