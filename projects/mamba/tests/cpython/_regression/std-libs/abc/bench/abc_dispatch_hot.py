"""Hot-loop bench for stdlib abc: calling abstract-method-implementing class.

Domain: stdlib
Feature: abc
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a method on a concrete ABC subclass —
monomorphic int regime, measures virtual dispatch overhead via ABC.
"""
# tier: compute

from abc import ABC, abstractmethod

class _Processor(ABC):
    @abstractmethod
    def process(self, v: int) -> int: ...

class _DoubleProc(_Processor):
    def process(self, v: int) -> int:
        return v * 2

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

_proc = _DoubleProc()
acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= _proc.process(v) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"abc_dispatch: {ITERS}")
