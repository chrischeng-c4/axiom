"""Hot-loop bench for stdlib math: math.sqrt + math.floor in tight loop.

Domain: stdlib
Feature: math
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop computing sqrt then floor on float inputs —
monomorphic float regime, measures math transcendental throughput.
"""
# tier: compute

import math

ITERS = 500_000

_inputs = [float(i + 1) for i in range(100)]
_n = len(_inputs)

_sqrt = math.sqrt
_floor = math.floor

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= _floor(_sqrt(v)) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"math_sqrt: {ITERS}")
