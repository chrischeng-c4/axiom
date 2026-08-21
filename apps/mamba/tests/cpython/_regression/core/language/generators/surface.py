"""Surface contract for language generators.

# type-regime: monomorphic

Probes: yield, generator function returns generator, next(), StopIteration,
for-loop over generator, generator is iterator (iter(gen) is gen),
send(), throw(), close().
CPython 3.12 is the oracle.
"""

import types

# Generator function returns a generator object
def _gen():
    yield 1
    yield 2
    yield 3

g = _gen()
assert isinstance(g, types.GeneratorType), f"type = {type(g).__name__!r}"

# Generator is its own iterator
assert iter(g) is g, "iter(gen) is gen failed"

# next() advances the generator
g2 = _gen()
assert next(g2) == 1
assert next(g2) == 2
assert next(g2) == 3

# StopIteration at exhaustion
_raised = False
try:
    next(g2)
except StopIteration:
    _raised = True
assert _raised, "StopIteration not raised"

# for-loop over generator
vals = list(_gen())
assert vals == [1, 2, 3], f"list(gen) = {vals!r}"

# Generator has send, throw, close methods
g3 = _gen()
assert hasattr(g3, "send"), "generator.send missing"
assert hasattr(g3, "throw"), "generator.throw missing"
assert hasattr(g3, "close"), "generator.close missing"

print("surface OK")
