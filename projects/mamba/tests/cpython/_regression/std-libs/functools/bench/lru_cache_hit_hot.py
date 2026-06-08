"""Hot-loop bench for stdlib functools: lru_cache repeated hits.

Domain: stdlib
Feature: functools
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop with majority cache hits on lru_cache —
monomorphic int regime, measures lru_cache cache-hit throughput.
"""
# tier: compute

import functools

@functools.lru_cache(maxsize=128)
def _compute(n: int) -> int:
    return n * n + n

ITERS = 500_000

_inputs = list(range(50))  # all fit in cache (maxsize=128)
_n = len(_inputs)

# Warm cache
for i in _inputs:
    _compute(i)

acc = 0
for i in range(ITERS):
    acc ^= _compute(_inputs[i % _n]) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"lru_cache_hit: {ITERS}")
