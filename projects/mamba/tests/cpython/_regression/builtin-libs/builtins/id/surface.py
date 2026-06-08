"""Surface contract for builtins.id.

# type-regime: monomorphic

Probes: name presence, callable, returns int, uniqueness within lifetime,
None singleton id stability.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "id"), "builtins.id missing"
assert builtins.id is id, "builtins.id is id divergence"
assert callable(builtins.id), "builtins.id not callable"

# id returns int
assert isinstance(id(42), int), "id(42) not int"
assert isinstance(id("hello"), int), "id('hello') not int"
assert isinstance(id(None), int), "id(None) not int"
assert isinstance(id([]), int), "id([]) not int"

# id of None is stable across calls (None is a singleton)
assert id(None) == id(None), "id(None) not stable"

# Two distinct list objects have different ids
a = []
b = []
assert id(a) != id(b), "distinct lists should have different ids"

# Same object has same id
x = [1, 2, 3]
assert id(x) == id(x), "same object same id"

print("surface OK")
