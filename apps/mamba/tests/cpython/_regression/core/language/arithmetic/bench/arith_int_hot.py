"""Hot-loop bench for language arithmetic: mixed int ops.

Domain: language
Feature: arithmetic
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop computing (a * b + c) % mod on integers —
monomorphic on int, exercises mul/add/mod chain.
"""
# tier: compute


ITERS = 500_000

_a = list(range(1, 11))
_b = list(range(2, 12))
_c = list(range(0, 10))
_n = len(_a)
_MOD = 1_000_007

acc = 0
for i in range(ITERS):
    idx = i % _n
    acc = (_a[idx] * _b[idx] + _c[idx] + acc) % _MOD

# Stdout sink — byte-equal across runtimes.
print(f"arith_int: {ITERS}")
