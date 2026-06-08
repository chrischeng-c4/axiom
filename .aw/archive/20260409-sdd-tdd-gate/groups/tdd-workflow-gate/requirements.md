---
change: sdd-tdd-gate
group: tdd-workflow-gate
date: 2026-04-08
---

# Requirements

PR2 — TDD workflow gate enforcement + test infrastructure (depends on test-config from PR1):

1. Gate 1 — Requirement coverage check: Parse Mermaid requirementDiagram blocks from spec files, extract requirement IDs, scan test files for matching requirement markers (e.g. comments or test names referencing the requirement ID). Reject advancement to review if any requirement lacks a corresponding test marker.
2. Gate 2 — Test execution gate: Match changed files (from git diff) against `changes` glob patterns in TestScope entries. For each matched scope, run `setup` (if any), then `test_cmd`, then `teardown` (if any). Reject advancement if any test command fails (non-zero exit).
3. Integrate both gates into the SDD workflow state machine — insert a test phase between implementation-complete and review. Implementation without tests is rejected; failing tests are rejected; only passing tests with full requirement coverage advance to review.
4. Update `sdd-change-implementation.md` agent prompt with TDD instructions: agent must write tests alongside implementation, tests must reference requirement IDs from the spec.
5. Create `projects/conductor/scripts/test-env.sh` — setup/teardown script for conductor test environment (start mock server, seed data, etc.).
6. Acceptance: implementation without tests is rejected at the gate, failing tests are rejected, passing tests with full requirement coverage advance to review.
