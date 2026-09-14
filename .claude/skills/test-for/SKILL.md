---
name: test-for
description: Legacy AW read-only final verification for an explicitly selected scope. A fresh QA worker runs declared full gates and returns a concise summary.
---

# Legacy AW Test for Scope

Use this skill only when the human explicitly requests legacy AW verification or invokes `$test-for`. For ordinary delivery verification, use `product-deliver` in test-only mode.

## Goal

Verify the selected legacy scope without a controller reading product files or raw logs.

## How

1. Delegate verification to a fresh QA instance that did not author the case or implementation.
2. QA checks the required legacy evidence and runs the owning project's declared complete, unfiltered gate. It returns command, exit code, commit identifiers, changed-path scope, and a short failure summary.
3. The controller reads the summary and makes no Git, tracker, lifecycle, or close change during this skill.

## Never

- Never invoke this skill implicitly.
- Never replace a full gate with a filtered test command.
- Never treat a worker summary as an acceptance decision.
- Never ask the controller to inspect product source, documents, or raw logs.
