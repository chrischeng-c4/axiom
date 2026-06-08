"""Surface contract for builtins.bool.

# type-regime: monomorphic

Probes: name presence, is a type (subclass of int), __name__, __doc__,
bool singleton values True and False, class membership. CPython 3.12 is
the oracle.
"""

import builtins

assert hasattr(builtins, "bool"), "builtins.bool missing"
assert builtins.bool is bool, f"builtins.bool is bool divergence"
assert callable(builtins.bool), f"builtins.bool not callable"

# bool is a class (type), subclass of int
assert type(builtins.bool).__name__ == "type", \
    f"type(builtins.bool).__name__ = {type(builtins.bool).__name__!r}"
assert issubclass(builtins.bool, int), "bool is not a subclass of int"

assert builtins.bool.__name__ == "bool", \
    f"builtins.bool.__name__ = {builtins.bool.__name__!r}"

# True and False are instances of bool
assert type(True) is bool, f"type(True) = {type(True).__name__!r}"
assert type(False) is bool, f"type(False) = {type(False).__name__!r}"

# True == 1, False == 0 (int subtype)
assert True == 1, "True == 1"
assert False == 0, "False == 0"

assert isinstance(builtins.bool.__doc__, str) and len(builtins.bool.__doc__) > 0

print("surface OK")
