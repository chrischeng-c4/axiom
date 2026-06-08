"""Hot-loop bench for language descriptors: property get in tight loop.

Domain: language
Feature: descriptors
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop reading a property descriptor on an object —
monomorphic int regime, measures descriptor __get__ dispatch overhead.
"""
# tier: compute


class _Vec:
    def __init__(self, x: int, y: int):
        self._x = x
        self._y = y

    @property
    def magnitude_sq(self) -> int:
        return self._x * self._x + self._y * self._y

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

_obj = _Vec(0, 1)
acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    _obj._x = v
    _obj._y = v + 1
    acc ^= _obj.magnitude_sq & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"descriptor_get: {ITERS}")
