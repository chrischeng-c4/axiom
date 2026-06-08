"""Hot-loop bench for stdlib sys: sys.getsizeof in tight loop.

Domain: stdlib
Feature: sys
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling sys.getsizeof on integer values —
monomorphic int regime, measures sys.getsizeof call overhead.
"""
# tier: compute

import sys

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)
_getsizeof = sys.getsizeof

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= _getsizeof(v) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"sys_getsizeof: {ITERS}")
