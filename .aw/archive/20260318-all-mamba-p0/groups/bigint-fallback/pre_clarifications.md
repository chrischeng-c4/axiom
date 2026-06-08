---
change: all-mamba-p0
group: bigint-fallback
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: BigInt library
- **Question**: Use num-bigint crate, or implement a minimal BigInt in-house for tighter NaN-boxing integration?
- **Answer**: Use num-bigint crate — battle-tested and full-featured.

### Q2: Demotion
- **Question**: Should BigInt results that fit in 48 bits be automatically demoted back to inline int (like V8 does), or stay as BigInt once promoted?
- **Answer**: Auto-demote — if result fits in 48 bits, demote back to inline int (V8 Smi strategy). Better performance for temporary overflows.

