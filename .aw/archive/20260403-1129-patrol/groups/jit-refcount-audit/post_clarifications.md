---
change: 1129-patrol
group: jit-refcount-audit
date: 2026-04-03
status: skipped
---

# Post-Clarifications

## Scope Summary

### Problem
mb_* runtime functions have mixed return ownership semantics (new vs borrowed refs). JIT releases all locals at return, causing use-after-free for borrowed refs → requirements.md, issue #1129 'Blocked: Ownership Audit'

### Success Criteria
(not provided)

### Boundary
(not provided)

