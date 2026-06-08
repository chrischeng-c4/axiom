"""Hot-loop bench for builtins.getattr: getattr() on object instances.

Domain: builtins
Feature: getattr
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling getattr(obj, name) on a fixed
object and name — monomorphic on instance attribute lookup.
"""
# tier: compute


ITERS = 500_000

class _Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

_attrs = ["x", "y"]
_n = len(_attrs)
_obj = _Point(3, 7)

acc = 0
for i in range(ITERS):
    v = getattr(_obj, _attrs[i % _n])
    acc ^= v

# Stdout sink — byte-equal across runtimes.
print(f"getattr: {ITERS}")
