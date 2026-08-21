"""Hot-loop bench for stdlib enum: enum member access.

Domain: stdlib
Feature: enum
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop accessing enum member by value lookup —
monomorphic int regime, measures Enum(value) dispatch overhead.
"""
# tier: compute

from enum import IntEnum

class _Status(IntEnum):
    A = 0
    B = 1
    C = 2
    D = 3
    E = 4

ITERS = 500_000

_inputs = list(range(5)) * 20  # 100 inputs, values 0-4
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= int(_Status(v)) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"enum_lookup: {ITERS}")
