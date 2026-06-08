"""Hot-loop bench for stdlib itertools: chain + islice iteration.

Domain: stdlib
Feature: itertools
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop consuming itertools.chain of two lists via islice —
monomorphic int regime, measures iterator chaining throughput.
"""
# tier: compute

import itertools

ITERS = 500_000

_a_list = list(range(50))
_b_list = list(range(50, 100))

acc = 0
for i in range(ITERS):
    _it = itertools.islice(itertools.chain(_a_list, _b_list), 10)
    for v in _it:
        acc ^= v & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"chain_islice: {ITERS}")
