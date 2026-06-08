"""Hot-loop bench for PEP 526 variable annotations: annotated class attr access.

Domain: pep
Feature: variable_annotations
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop reading annotated class instance attributes —
monomorphic int regime, verifies annotations add zero access overhead.
"""
# tier: compute


class _Pt:
    x: int
    y: int
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

_obj = _Pt(0, 0)
acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    _obj.x = v
    _obj.y = v + 1
    acc ^= (_obj.x + _obj.y) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"varanno_access: {ITERS}")
