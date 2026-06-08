---
change: mamba-thread-safe-and-no-gil
group: mamba-thread-safe-no-gil-runtime
date: 2026-03-06
status: clarified
---

# Post-Clarifications

## Questions

### Q1: Async scheduling with no-GIL
- **Question**: The async spec R3 relies on GIL acquire/release for coroutine scheduling. With no-GIL, how should async scheduling work?
- **Answer**: Defer async changes. Keep async as-is (single-threaded) for now. Focus this change on RC/GC/collections thread-safety only. Async no-GIL redesign in a follow-up.
- **Rationale**: Scope control — this change already has large blast radius. Get foundational thread-safety solid and tested first. Async can layer on top incrementally.

## Contradictions

### C1: async vs requirement
- **Spec**: async
- **Requirement**: No-GIL runtime with thread-safe objects
- **Conflict**: Async spec R3 explicitly relies on GIL acquire/release for coroutine scheduling (release GIL on suspend, re-acquire on resume). Removing GIL breaks this contract.
- **Resolution**: Defer async scheduling changes to a follow-up. This change focuses on foundational thread-safety (atomic RC, global GC, per-object locks on collections). Async runtime remains single-threaded for now.

