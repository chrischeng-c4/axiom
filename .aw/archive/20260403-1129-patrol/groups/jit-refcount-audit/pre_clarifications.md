---
change: 1129-patrol
group: jit-refcount-audit
date: 2026-04-03
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Runtime ownership audit scope for mb_* functions?
- **Answer**: Exhaustive audit of ALL mb_* runtime functions. Classify every return value as new reference (caller owns) or borrowed reference (caller must retain). This is safer and prevents future regressions.

### Q2: General
- **Question**: Should re-enabling GC be part of this change or a separate issue?
- **Answer**: Include GC re-enable in this change. After refcounting is working correctly with EMIT_REFCOUNT_CALLS=true, re-enable GC in gc.rs (set enabled: true).

