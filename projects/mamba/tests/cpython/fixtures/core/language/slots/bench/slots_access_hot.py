"""Hot-loop bench for language slots: slotted attribute get/set.

Domain: language
Feature: slots
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop reading and writing slotted attributes —
monomorphic int regime, measures slot vs dict attribute access speed.
"""
# tier: compute


class _Vec2:
    __slots__ = ("x", "y")
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

_obj = _Vec2(0, 0)
acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    _obj.x = v
    _obj.y = v + 1
    acc ^= (_obj.x + _obj.y) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"slots_access: {ITERS}")
