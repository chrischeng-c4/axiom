"""Hot-loop bench for language decorators: wrapped function call.

Domain: language
Feature: decorators
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a functools.wraps-wrapped function —
monomorphic on int inputs, measures decorator wrapper overhead.
"""
# tier: compute

import functools

def _double(fn):
    @functools.wraps(fn)
    def _w(x: int) -> int:
        return fn(x) * 2
    return _w

@_double
def _square(x: int) -> int:
    return x * x

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    acc ^= _square(_inputs[i % _n]) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"decorator_call: {ITERS}")
