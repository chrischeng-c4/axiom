---
change: all-mamba-p0
group: bigint-fallback
date: 2026-03-18
---

# Requirements

Add BigInt fallback for NaN-boxing 48-bit integer overflow. Fast path: keep current 48-bit inline int. Slow path: on overflow, promote to heap-allocated BigInt. Arithmetic ops emit overflow check after add/sub/mul and branch to promotion. BigInt must interop seamlessly with inline int for comparison, hashing, and mixed arithmetic. Use num-bigint crate or similar.
