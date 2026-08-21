"""Hot-loop bench for builtins.divmod: divmod() on integers.

Domain: builtins
Feature: divmod
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling divmod(a, b) on positive integer
pairs — monomorphic on int inputs.
"""
# tier: compute


ITERS = 500_000

_pairs = [(a, b) for a in range(1, 11) for b in range(1, 11)]
_n = len(_pairs)

acc = 0
for i in range(ITERS):
    a, b = _pairs[i % _n]
    q, r = divmod(a, b)
    acc ^= q ^ r

# Stdout sink — byte-equal across runtimes.
print(f"divmod_int: {ITERS}")
