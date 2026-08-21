"""Hot-loop bench for language metaclasses: calling metaclass-created method.

Domain: language
Feature: metaclasses
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a method on an instance of a
metaclass-augmented class — monomorphic int regime.
"""
# tier: compute


class _MultMeta(type):
    def __new__(mcs, name, bases, namespace):
        def _compute(self, v: int) -> int:
            return v * self._factor
        namespace["compute"] = _compute
        return super().__new__(mcs, name, bases, namespace)

class _Processor(metaclass=_MultMeta):
    def __init__(self, factor: int):
        self._factor = factor

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

_proc = _Processor(7)
acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= _proc.compute(v) & 0xFFFF  # type: ignore[attr-defined]

# Stdout sink — byte-equal across runtimes.
print(f"metaclass_create: {ITERS}")
