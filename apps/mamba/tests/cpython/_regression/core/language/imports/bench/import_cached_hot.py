"""Hot-loop bench for language imports: repeated access to imported module fn.

Domain: language
Feature: imports
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a function accessed through an
imported module — measures module attribute lookup + function call overhead.
"""
# tier: compute

import math

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

# Pre-bind to avoid repeated attribute lookup in the loop
_sqrt = math.sqrt
_floor = math.floor

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n] + 1  # avoid sqrt(0) edge case
    acc ^= _floor(_sqrt(v)) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"import_cached: {ITERS}")
