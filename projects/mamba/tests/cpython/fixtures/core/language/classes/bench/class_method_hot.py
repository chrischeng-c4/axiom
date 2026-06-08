"""Hot-loop bench for language classes: instance method dispatch.

Domain: language
Feature: classes
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a method on a fixed instance —
monomorphic on user-defined class instance.
"""
# tier: compute


ITERS = 500_000

class _Acc:
    def __init__(self) -> None:
        self.v = 0
    def add(self, x: int) -> int:
        self.v += x
        return self.v

_obj = _Acc()
_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    acc ^= _obj.add(_inputs[i % _n]) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"class_method: {ITERS}")
