"""Hot-loop bench for language closures: calling a closure.

Domain: language
Feature: closures
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a closure that captures one int
free variable — monomorphic on int inputs, measures closure-call overhead.
"""
# tier: compute


def _make_adder(n: int):
    def _add(x: int) -> int:
        return x + n
    return _add

_add100 = _make_adder(100)

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    acc ^= _add100(_inputs[i % _n]) & 0xFFFF

# Stdout sinks make the accumulator observable and preserve every loop iteration.
print(f"closure_call: {ITERS}")
print(f"closure_acc: {acc}")
