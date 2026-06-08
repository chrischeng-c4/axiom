"""Surface contract for builtins.int.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, __doc__, class membership,
subclass of object, zero-arg default, instance membership.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "int"), "builtins.int missing"
assert builtins.int is int, f"builtins.int is int divergence"
assert callable(builtins.int), "builtins.int not callable"

# int is a class (type)
assert type(builtins.int).__name__ == "type", \
    f"type(builtins.int).__name__ = {type(builtins.int).__name__!r}"
assert issubclass(builtins.int, object), "int is not a subclass of object"

assert builtins.int.__name__ == "int", \
    f"builtins.int.__name__ = {builtins.int.__name__!r}"

# int instances
assert isinstance(0, int), "isinstance(0, int) failed"
assert isinstance(-1, int), "isinstance(-1, int) failed"
assert isinstance(42, int), "isinstance(42, int) failed"

# bool is a subclass of int
assert issubclass(bool, int), "bool is not a subclass of int"
assert isinstance(True, int), "isinstance(True, int) failed"
assert isinstance(False, int), "isinstance(False, int) failed"

# int.__doc__ exists and is non-empty
assert isinstance(builtins.int.__doc__, str) and len(builtins.int.__doc__) > 0, \
    "builtins.int.__doc__ missing or empty"

print("surface OK")
