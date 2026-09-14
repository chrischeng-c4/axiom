---
name: e2e-for
description: Legacy AW e2e phase for an explicitly selected behavior scope. Controller records AW state; QA owns the red black-box case.
---

# Legacy AW E2E for Scope

Use this skill only when the human explicitly selects legacy AW behavior flow or invokes `$e2e-for`. For ordinary work, use `product-deliver`.

## Goal

Obtain one red black-box behavior case for the selected legacy AW queue head.

## How

1. The controller resolves one eligible behavior queue head and performs every AW phase-state, commit, and lifecycle action.
2. After the controller's permission readiness check, delegate the e2e work to the owning QA worker. QA may write only the e2e case and its exact test registration. QA runs the case against the current tree and returns changed paths, command, exit code, and a short failure summary.
3. QA has no Git, tracker, lifecycle, or close authority. QA must not write implementation files or make a green case appear red.
4. The controller records the legacy AW e2e evidence only after it verifies the worker's scoped summary. A behavior head then waits for `impl-for`.

## Never

- Never invoke this skill implicitly.
- Never select order from issue number, API order, or a priority label.
- Never use it for maintenance or intake work.
- Never claim hard sandbox isolation. Writable QA work starts only after the controller's permission readiness check.
