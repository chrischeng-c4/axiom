---
change: mamba-thread-safe
group: mamba-thread-safe-refactor
date: 2026-03-07
status: clarified
---

# Post-Clarifications

## Questions

### Q1: Async scope override
- **Question**: Should we update the thread-safe-runtime spec to remove R5 (deferred async) and include async migration in this change?
- **Answer**: Yes, update spec. Remove R5. Async thread_local migration + Tokio integration are in scope.
- **Rationale**: The previous change (mamba-thread-safe-and-no-gil) deferred async work. This change is specifically to complete that deferred work along with remaining thread-safety improvements.

## Contradictions

### C1: thread-safe-runtime vs requirement
- **Spec**: thread-safe-runtime
- **Requirement**: R5: Deferred Async Scheduling Changes
- **Conflict**: R5 states async scheduling changes are out of scope and deferred. However, pre-clarifications explicitly put Tokio integration and thread_local async state migration IN scope for this change.
- **Resolution**: User confirmed: Remove R5 from thread-safe-runtime spec. Async thread_local migration (COROUTINES/TASKS/WAKERS/TIMERS to global shared state) and Tokio multi-threaded executor integration are in scope for this change.

