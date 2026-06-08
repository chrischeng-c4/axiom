---
number: 833
title: "BigInt fallback for NaN-boxing 48-bit integer overflow"
state: open
labels: [enhancement, P0, crate:mamba]
group: "bigint-fallback"
---

# #833 — BigInt fallback for NaN-boxing 48-bit integer overflow

## Summary

Mamba uses NaN-boxing to encode integers in 48 bits (range: ±2^47). CPython supports arbitrary-precision integers. When a Mamba program exceeds 48-bit range, it silently overflows — this is a correctness bug.

## Proposed Solution

Adopt V8's Smi (Small Integer) strategy:
1. **Fast path**: 48-bit inline int via NaN-boxing (current behavior)
2. **Slow path**: When overflow detected, promote to heap-allocated BigInt
3. **Arithmetic ops**: Check overflow flag after each op; promote if needed

## Scope

- **MbValue**: Add a BigInt variant (heap pointer via NaN-boxing tag)
- **Arithmetic codegen**: Emit overflow check after add/sub/mul; branch to promotion
- **Runtime**: BigInt type backed by `num-bigint` crate or similar
- **Comparison/hashing**: BigInt must interop with inline int seamlessly

## Why P0

Python's arbitrary-precision integers are a language guarantee. Code like `2**100` or `factorial(50)` will silently produce wrong results without this fix. This is not an edge case — it affects cryptography, combinatorics, and any large number computation.
