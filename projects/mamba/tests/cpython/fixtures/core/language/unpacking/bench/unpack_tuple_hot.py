"""Hot-loop bench for language unpacking: tuple unpack in loop.

Domain: language
Feature: unpacking
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop unpacking 2-tuples from a pre-built list —
monomorphic on int-pair inputs, measures unpack overhead.
"""
# tier: compute


ITERS = 500_000

_pairs = [(i, i + 1) for i in range(100)]
_n = len(_pairs)

acc = 0
for i in range(ITERS):
    _lo, _hi = _pairs[i % _n]
    acc ^= (_lo + _hi) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"unpack_tuple: {ITERS}")
