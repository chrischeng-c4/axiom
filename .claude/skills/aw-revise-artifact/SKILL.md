---
name: aw:revise-artifact
description: Revise change-spec and re-implement — fix design issues after review
user-invocable: true
auto-invoke: false
---

# /aw:revise-artifact

Revises the change-spec and re-runs implementation when design issues are found after implementation review. Resets the workflow phase to re-enter the spec → implementation cycle.

**This skill must be invoked by the user.** Do NOT invoke it automatically.

## Change-ID Resolution

If a change-id is provided, use it. Otherwise:

- Check `.aw/changes/` for existing change directories whose `STATE.yaml` has `branch` matching the current git branch AND `phase` is not terminal (`archived` or `rejected`).
  - Found → use that change's `change_id`.
  - Not found → ask the user.

## Instructions

1. Resolve change-id
2. Run `aw td revise <slug>` with the requested design changes.
3. Continue with `aw td review <slug>` and `aw cb gen <slug>` as directed by the CLI envelope.

## Usage

```
/aw:revise-artifact <change-id> "<description of design changes needed>"
/aw:revise-artifact "<description of design changes needed>"
```
