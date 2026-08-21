"""Hot-loop bench for PEP 570 positional-only: calling posonly function.

Domain: pep
Feature: positional_only
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a function with positional-only params —
monomorphic int regime, measures pos-only dispatch overhead.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

def _compute(a: int, b: int, /) -> int:
    return (a * b + a) & 0xFFFF

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= _compute(v, v + 1)

# Stdout sink — byte-equal across runtimes.
print(f"posonly_call: {ITERS}")
