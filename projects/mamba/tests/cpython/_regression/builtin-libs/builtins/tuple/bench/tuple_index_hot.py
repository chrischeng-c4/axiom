"""Hot-loop bench for builtins.tuple: indexing into a fixed tuple.

Domain: builtins
Feature: tuple
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop indexing a fixed tuple of ints —
monomorphic on tuple[int], JIT-specializable.
"""
# tier: compute


ITERS = 500_000

_t = tuple(range(10))  # (0, 1, ..., 9)
_n = len(_t)

acc = 0
for i in range(ITERS):
    acc ^= _t[i % _n]

# Stdout sink — byte-equal across runtimes.
print(f"tuple_index: {ITERS}")
