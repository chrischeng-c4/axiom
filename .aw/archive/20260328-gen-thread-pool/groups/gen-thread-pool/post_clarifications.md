---
change: gen-thread-pool
group: gen-thread-pool
date: 2026-03-26
status: skipped
---

# Post-Clarifications

## Scope Summary

### Problem
requirements.md: after ~130 sequential generator thread::spawn calls on macOS aarch64, JIT code execution crashes with EXC_BAD_ACCESS code=257 (PAC failure). Thread counts stay at 2 — threads ARE joined — but cumulative pthread lifecycle corrupts process state.

### Success Criteria
(not provided)

### Boundary
(not provided)

