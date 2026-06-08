---
change: all-mamba-p0
group: match-case
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: Pattern completeness
- **Question**: Should exhaustiveness checking (warn on non-exhaustive match) be included in this iteration, or deferred?
- **Answer**: Include exhaustiveness checking in this iteration alongside pattern matching implementation.

### Q2: Optimization
- **Question**: Should the decision tree compilation optimize for common patterns (e.g., integer switch tables) or use a uniform nested-if approach first?
- **Answer**: Optimized from start — generate switch tables for int/str literals, interleave guards for better codegen performance.

