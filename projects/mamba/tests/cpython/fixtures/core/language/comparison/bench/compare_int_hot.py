"""Hot-loop bench for language comparison: < on integers.

Domain: language
Feature: comparison
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop comparing pairs of integers —
monomorphic on int, result accumulated as XOR sink.
"""
# tier: compute


ITERS = 500_000

_a = list(range(100))
_b = list(range(1, 101))
_n = len(_a)

acc = 0
for i in range(ITERS):
    acc ^= int(_a[i % _n] < _b[i % _n])

# Stdout sink — byte-equal across runtimes.
print(f"compare_int: {ITERS}")
