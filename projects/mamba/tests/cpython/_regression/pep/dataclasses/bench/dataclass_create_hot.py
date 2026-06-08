# mamba-xfail: goal-end sweep; mamba diverges (TIMEOUT)
"""Hot-loop bench for PEP 557 dataclasses: instance creation + field access.

Domain: pep
Feature: dataclasses
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop creating dataclass instances and reading fields —
monomorphic int regime, measures dataclass __init__ + attribute access overhead.
"""
# tier: compute

from dataclasses import dataclass

@dataclass
class _Pt:
    x: int
    y: int

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    _p = _Pt(v, v + 1)
    acc ^= (_p.x + _p.y) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"dataclass_create: {ITERS}")
