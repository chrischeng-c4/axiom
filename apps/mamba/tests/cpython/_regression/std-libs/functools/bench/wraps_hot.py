"""Hot-loop bench for `functools.wraps` decorator application (#1451).

End-user scenario: a framework that wraps user functions with logging,
metrics, or retry decorators applies `@functools.wraps(fn)` to copy
`__name__`, `__doc__`, `__wrapped__`, and `__dict__` from the original
to the wrapper. Real-world examples: Flask view decorators, Django's
`method_decorator`, `tenacity.retry`, `cached_property`-adjacent
plumbing. Decoration runs at module-import time and at every dynamic
re-decoration (e.g. instance-bound method binding inside a factory).

Tier: `decorator-plumbing-light` (target mamba/cpython <= 1.0x —
`functools.wraps` is pure attribute-copy in CPython implemented in
Python; mamba's edge is that the underlying `__name__`/`__doc__`
attribute writes lower to direct slot writes in Rust while CPython
goes through `__setattr__` + dict resize per attribute).

Workload: 10_000 fresh decorator applications. Each iteration
constructs a new wrapper closure and invokes `wraps(base)` on it,
then reads back `__name__` to verify the copy landed. The
construct-wrapper-then-decorate pattern matches the lifecycle of
runtime-built decorators (factory functions, parametrized adapters).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
and reports the ratio. Floor is 1.0x per #1265 Goal 2. Today's
measured ratio is ~0.48x on the reference machine (mamba ~6.3 ms vs
CPython ~13.1 ms across 5 samples).
"""

import functools

# Hoist module attrs to dodge per-iter attribute lookup overhead (#2097).
wraps = functools.wraps


def base(x):
    """base docstring"""
    return x + 1


def make_wrapper(fn):
    @wraps(fn)
    def wrapper(*a, **kw):
        return fn(*a, **kw)
    return wrapper


ITERS = 10_000

acc = 0
for i in range(ITERS):
    w = make_wrapper(base)
    # Accumulator readback prevents DCE of the decoration result.
    if w.__name__ == "base":
        acc = acc + 1

# Correctness: every iter must have copied the source __name__.
assert acc == ITERS, f"wraps drift: acc={acc} expected={ITERS}"
print("wraps_hot:", acc)
