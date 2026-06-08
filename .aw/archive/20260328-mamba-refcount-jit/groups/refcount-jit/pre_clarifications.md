---
change: mamba-refcount-jit
group: refcount-jit
date: 2026-03-27
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Scope confirmation
- **Answer**: All 6 phases as described in the requirements. Start with Phase 1-2 (symbols + reassignment release), verify with ASan, then proceed to Phase 3-6. Immortal refcount for compile-time constants (Phase 4) preferred over tracking (Phase 5).

