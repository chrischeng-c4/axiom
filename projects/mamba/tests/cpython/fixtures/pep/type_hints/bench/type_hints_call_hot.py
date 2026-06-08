"""Hot-loop bench for PEP 484 type hints: annotated function call.

Domain: pep
Feature: type_hints
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a fully type-annotated function —
monomorphic int regime, verifies hint annotations add zero runtime overhead.
"""
# tier: compute

from typing import List

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

def _compute(a: int, b: int) -> int:
    return (a * b) & 0xFFFF

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= _compute(v, v + 1)

# Stdout sink — byte-equal across runtimes.
print(f"type_hints_call: {ITERS}")
