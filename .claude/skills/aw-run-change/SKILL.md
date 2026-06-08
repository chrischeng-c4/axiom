---
name: aw:run-change
description: DEPRECATED — use /aw:td:create instead
user-invocable: true
---

# /aw:run-change — DEPRECATED

The legacy workflow command family and `/aw:run-change` have been retired. All tech-design
work runs through `aw td` (CRRR envelope protocol, same shape as
`aw wi`).

## Migration

| Old                                   | New                                   |
|---------------------------------------|---------------------------------------|
| `/aw:run-change <slug>`            | `/aw:td:create <slug>`             |
| legacy create-change-spec command          | `aw td create <slug>`              |
| legacy review-change-spec command          | `aw td review <slug>`              |
| legacy create-change-impl command          | `aw cb gen <slug>`                 |
| legacy create-change-merge command         | `aw td merge <slug>`               |

## What to do

1. Read the user's request — if they invoked this skill, translate to `/aw:td:create <slug>`.
2. Run that skill instead.
3. Surface this deprecation notice to the user once per session so they learn the new entry point.

See `CLAUDE.md § SDD: Spec-Driven Development` for the full `aw td` flow.
