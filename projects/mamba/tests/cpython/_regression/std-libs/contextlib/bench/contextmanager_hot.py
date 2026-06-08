"""Hot-loop bench for stdlib contextlib: contextmanager in tight loop.

Domain: stdlib
Feature: contextlib
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop entering a @contextmanager context —
monomorphic int regime, measures context manager overhead.
"""
# tier: compute

import contextlib

@contextlib.contextmanager
def _ctx(v: int):
    yield v * 2

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    with _ctx(v) as _r:
        acc ^= _r & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"contextmanager: {ITERS}")
