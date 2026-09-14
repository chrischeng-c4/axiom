---
name: product-deliver
description: Deliver a scoped product change through QA, Dev, and fresh integration QA. Supports e2e-only, impl-only, test-only, and read-only-review modes.
---

# Product Deliver

## Goal

Deliver one authorized scope with red-first tests, scoped edits, independent final verification, and controller-owned acceptance.

## Modes

- `e2e-only`: QA writes and runs the red black-box case and its exact test registration.
- `impl-only`: Dev adds a red colocated unit test, implements the scoped change, and runs required tests. Dev cannot weaken QA tests.
- `test-only`: a fresh `<p>-qa` instance runs a single project's declared complete, unfiltered gate. Use Integration QA only when the scope crosses projects.
- `read-only-review`: a fresh `<p>-qa` instance reports single-project findings without edits. Use Integration QA only for a cross-project review.
- Default behavior delivery runs e2e-only, then impl-only, then test-only.

## How

1. The controller fixes one authorized scope and mode. For one project, it delegates to the owning QA, then Dev, then a fresh `<p>-qa` instance. Use Integration QA only for a cross-project scope. The final QA instance must not be the QA instance that authored the e2e case.
2. QA owns e2e cases and the exact manifest registration needed to run them. Dev owns source, colocated unit tests, and their wiring. PM owns product docs. TL owns task drafts, not source.
3. Workers return only changed paths, commands, exit codes, and short failure details. The controller reads summaries, not product source, documents, or raw logs.
4. Writable workers may start only after the controller's permission readiness check. This is not a claim of hard sandbox isolation.
5. The controller owns Git, tracker changes, scope changes, and final acceptance. It does not auto-commit, push, release, close work, or expand a partial scope.

## Never

- Never make e2e, unit-test, implementation, or final-test stages mandatory when the selected mode excludes them.
- Never use a filtered gate as final verification.
- Never require a receipt, a new team hierarchy, arbitrary approval, or a full design document.
- Never let QA or Dev commit, push, mutate tracker state, or make the final acceptance decision.
