---
name: impl-for
description: Legacy AW implementation or maintenance phase for an explicitly selected queue head. Controller records AW state; Dev owns scoped code and unit tests.
---

# Legacy AW Implementation for Scope

Use this skill only when the human explicitly selects legacy AW delivery or invokes `$impl-for`. For ordinary behavior work, use `product-deliver`.

## Goal

Complete the selected legacy AW implementation or maintenance leg with scoped tests and no worker-controlled Git or tracker mutation.

## How

1. The controller resolves the current queue head and selects behavior impl or maintenance. It alone runs AW phase-state, commit, lifecycle, and close commands.
2. For behavior, after the permission readiness check, delegate to Dev. Dev adds a colocated unit test that fails before the implementation, implements the change, and runs the required tests. Dev cannot weaken or replace QA's e2e test.
3. For maintenance, delegate only the scoped maintenance edit and its declared tests. Do not manufacture behavior red evidence.
4. Dev returns changed paths, commands, exit codes, and short failure details. The controller records legacy AW evidence and decides whether to advance.

## Never

- Never invoke this skill implicitly.
- Never let Dev commit, push, alter tracker state, or close an issue.
- Never write e2e cases in this leg.
- Never add a mandatory design document, receipt, or approval checkpoint.
- Never claim hard sandbox isolation. Writable Dev work starts only after the controller's permission readiness check.
