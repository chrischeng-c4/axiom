"""Hot-loop bench for language type annotations: calling annotated function.

Domain: language
Feature: type_annotations
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a fully-annotated function —
monomorphic int regime, verifies annotations add zero runtime cost.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

def _compute(a: int, b: int, c: int) -> int:
    return (a * b + c) & 0xFFFF

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= _compute(v, v + 1, v + 2)

# Stdout sink — byte-equal across runtimes.
print(f"annotated_call: {ITERS}")
