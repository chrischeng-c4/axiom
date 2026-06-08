---
name: aw:merge
description: DEPRECATED — use `aw td merge <slug>` instead
user-invocable: true
auto-invoke: false
---

# /aw:merge — DEPRECATED

The legacy create-change-merge command and `/aw:merge` have been retired.
All tech-design merges run through `aw td merge`.

## Migration

| Old                    | New                       |
|------------------------|---------------------------|
| `/aw:merge <id>`    | `aw td merge <slug>`   |

`aw td merge` merges the approved `td-<slug>` branch into the resolved target
branch, writes `state: closed` + `phase: td_merged` to the issue, and deletes
the local TD branch when possible.

## What to do

1. If the user invoked this skill, translate the change-id / branch to the
   corresponding issue slug (they are the same string under the new flow).
2. Run `aw td merge <slug>` via Bash.
3. Surface this deprecation notice once per session.

See `CLAUDE.md § SDD: Spec-Driven Development` for the full `aw td` flow.
